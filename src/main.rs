use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use chrono::Utc;
use opengp_domain::domain::api::{
    AllergyRequest, AllergyResponse, AppointmentRequest, ConsultationRequest, FamilyHistoryRequest,
    FamilyHistoryResponse, MedicalHistoryRequest, MedicalHistoryResponse, PatientRequest,
    PatientResponse, SocialHistoryRequest, SocialHistoryResponse, VitalSignsRequest,
    VitalSignsResponse,
};
use opengp_domain::domain::appointment::{
    AppointmentCalendarQuery, AppointmentType, AvailabilityService,
};
use opengp_domain::domain::patient::{Gender, Patient};
use opengp_domain::domain::user::{PractitionerRepository, UserRepository};
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use opengp_infrastructure::infrastructure::database::repositories::appointment::SqlxAppointmentRepository;
use opengp_infrastructure::infrastructure::database::repositories::patient::SqlxPatientRepository;
use opengp_infrastructure::infrastructure::database::repositories::practitioner::SqlxPractitionerRepository;
use opengp_infrastructure::infrastructure::database::repositories::user::SqlxUserRepository;
use opengp_infrastructure::infrastructure::database::repositories::working_hours::SqlxWorkingHoursRepository;
use opengp_infrastructure::infrastructure::database::{create_pool, run_migrations, DatabasePool};
use opengp_infrastructure::infrastructure::fixtures::seed_working_hours;
use opengp_ui::api::ApiClient;
use opengp_ui::ui::app::App;
use opengp_ui::ui::services::AppointmentUiService;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use opengp_config::CalendarConfig;
use opengp_config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = Config::from_env()?;

    init_logging(&config.log_level);

    tracing::info!("Starting OpenGP");
    tracing::info!("Database URL: {}", config.database.url);

    let db_pool = create_pool(&config.database).await?;

    run_migrations(&db_pool).await?;

    tracing::info!(
        "Database pool created with {} connection(s)",
        db_pool.size()
    );

    let sqlite_pool = match db_pool {
        DatabasePool::Sqlite(pool) => pool,
        DatabasePool::Postgres(_) => {
            return Err(color_eyre::eyre::eyre!(
                "TUI local repositories currently require SQLite pool"
            ));
        }
    };

    seed_working_hours(&sqlite_pool).await?;
    tracing::info!("Practitioner working hours seeded");

    let crypto = Arc::new(EncryptionService::new()?);
    let patient_repo = Arc::new(SqlxPatientRepository::new(sqlite_pool.clone(), crypto.clone()));

    // Create appointment-related repositories and service
    let practitioner_repo: Arc<dyn PractitionerRepository> =
        Arc::new(SqlxPractitionerRepository::new(sqlite_pool.clone()));
    let appointment_repo_impl = Arc::new(SqlxAppointmentRepository::new(sqlite_pool.clone()));
    let appointment_repo_for_create: Arc<
        dyn opengp_domain::domain::appointment::AppointmentRepository,
    > = appointment_repo_impl.clone();
    let appointment_repo: Arc<dyn AppointmentCalendarQuery> = appointment_repo_impl.clone();

    // Create domain appointment service for status transitions
    let audit_service: std::sync::Arc<dyn opengp_domain::domain::audit::AuditEmitter> = Arc::new(opengp_domain::domain::audit::AuditService::new(
        Arc::new(opengp_infrastructure::infrastructure::database::repositories::audit::SqlxAuditRepository::new(sqlite_pool.clone()))
    ));
    let domain_appointment_service =
        Arc::new(opengp_domain::domain::appointment::AppointmentService::new(
            appointment_repo_for_create.clone(),
            audit_service.clone(),
            appointment_repo.clone(),
        ));

    // Create working hours repository and availability service
    let working_hours_repo = Arc::new(SqlxWorkingHoursRepository::new(sqlite_pool.clone()));
    let availability_service = Arc::new(AvailabilityService::new(
        appointment_repo_for_create.clone(),
        working_hours_repo,
    ));

    let appointment_service = Arc::new(AppointmentUiService::new(
        practitioner_repo,
        appointment_repo,
        appointment_repo_for_create,
        domain_appointment_service,
        availability_service,
    ));

    // Create patient service
    let patient_service = Arc::new(opengp_ui::ui::services::PatientUiService::new(Arc::new(
        opengp_domain::domain::patient::PatientService::new(patient_repo.clone()),
    )));

    // Create clinical service repositories
    let consultation_repo: Arc<dyn opengp_domain::domain::clinical::ConsultationRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxClinicalRepository::new(sqlite_pool.clone(), crypto.clone()));
    let allergy_repo: Arc<dyn opengp_domain::domain::clinical::AllergyRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxAllergyRepository::new(sqlite_pool.clone(), crypto.clone()));
    let medical_history_repo: Arc<dyn opengp_domain::domain::clinical::MedicalHistoryRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxMedicalHistoryRepository::new(sqlite_pool.clone(), crypto.clone()));
    let vital_signs_repo: Arc<dyn opengp_domain::domain::clinical::VitalSignsRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxVitalSignsRepository::new(sqlite_pool.clone(), crypto.clone()));
    let social_history_repo: Arc<dyn opengp_domain::domain::clinical::SocialHistoryRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxSocialHistoryRepository::new(sqlite_pool.clone(), crypto.clone()));
    let family_history_repo: Arc<dyn opengp_domain::domain::clinical::FamilyHistoryRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxFamilyHistoryRepository::new(sqlite_pool.clone(), crypto.clone()));

    let clinical_repos = opengp_domain::domain::clinical::ClinicalRepositories {
        consultation: consultation_repo,
        allergy: allergy_repo,
        medical_history: medical_history_repo,
        vital_signs: vital_signs_repo,
        social_history: social_history_repo,
        family_history: family_history_repo,
    };
    let clinical_service = Arc::new(opengp_ui::ui::services::ClinicalUiService::new(Arc::new(
        opengp_domain::domain::clinical::ClinicalService::new(
            clinical_repos,
            Arc::new(opengp_domain::domain::patient::PatientService::new(
                patient_repo.clone(),
            )),
            audit_service,
        ),
    )));

    let user_repo = SqlxUserRepository::new(sqlite_pool.clone());
    let system_user_id = match user_repo.find_all().await {
        Ok(users) => {
            if let Some(first_user) = users.first() {
                tracing::info!(
                    "Using system_user_id from user: {} ({})",
                    first_user.username,
                    first_user.id
                );
                first_user.id
            } else {
                tracing::warn!("No users found in database, using nil UUID for system_user_id");
                uuid::Uuid::nil()
            }
        }
        Err(e) => {
            tracing::warn!(
                "Failed to load users, using nil UUID for system_user_id: {}",
                e
            );
            uuid::Uuid::nil()
        }
    };

    let api_base_url = std::env::var("API_BASE_URL")
        .or_else(|_| std::env::var("OPENGP_API_BASE_URL"))
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let api_client = Arc::new(ApiClient::new(api_base_url));
    if let Ok(token) = std::env::var("API_SESSION_TOKEN") {
        api_client.set_session_token(Some(token)).await;
    }

    run_tui(
        api_client,
        appointment_service,
        patient_service,
        clinical_service,
        system_user_id,
        config.calendar,
    )
    .await?;

    tracing::info!("OpenGP shutdown complete");

    Ok(())
}

async fn run_tui(
    api_client: Arc<ApiClient>,
    appointment_service: Arc<AppointmentUiService>,
    patient_service: Arc<opengp_ui::ui::services::PatientUiService>,
    clinical_service: Arc<opengp_ui::ui::services::ClinicalUiService>,
    system_user_id: uuid::Uuid,
    calendar_config: CalendarConfig,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let has_session_token = api_client.current_session_token().await.is_some();

    let mut app = App::new(
        Some(api_client.clone()),
        Some(appointment_service.clone()),
        Some(patient_service.clone()),
        Some(clinical_service.clone()),
        calendar_config.clone(),
    );
    app.current_user_id = system_user_id;
    app.set_authenticated(has_session_token);
    if has_session_token {
        app.request_refresh_patients();
    }

    loop {
        app.poll_api_tasks().await;

        terminal.draw(|frame| {
            app.render(frame);
        })?;

        // Check if there's pending patient data to save
        if let Some(pending) = app.take_pending_patient_data() {
            match pending {
                opengp_ui::ui::app::PendingPatientData::New(data) => {
                    let request = patient_request_from_new(data);
                    api_client.create_patient(&request).await?;
                    tracing::info!("Created new patient via API");
                }
                opengp_ui::ui::app::PendingPatientData::Update { id, data } => {
                    let existing = api_client.get_patient(id).await?;
                    let request = patient_request_from_update(data, &existing);
                    api_client.update_patient(id, request).await?;
                    tracing::info!("Updated patient via API");
                }
            }

            app.request_refresh_patients();
        }

        // Check if there's a pending patient to load for editing
        if let Some(patient_id) = app.take_pending_edit_patient_id() {
            match api_client.get_patient(patient_id).await {
                Ok(patient) => {
                    app.open_patient_form(domain_patient_from_api_response(patient));
                    tracing::info!("Loaded patient for editing: {}", patient_id);
                }
                Err(e) => {
                    tracing::error!("Failed to load patient for editing: {}", e);
                }
            }
        }

        // Check if there's a pending appointment date to load practitioners and schedule for
        if let Some(date) = app.take_pending_appointment_date() {
            app.request_refresh_appointments(date);
        }

        // Check if practitioners need to be loaded for appointment form picker
        if app.take_pending_load_practitioners() {
            match api_client.get_practitioners().await {
                Ok(practitioners) => {
                    let practitioner_items: Vec<opengp_ui::ui::view_models::PractitionerViewItem> =
                        practitioners
                            .into_iter()
                            .map(|p| opengp_ui::ui::view_models::PractitionerViewItem {
                                id: p.id,
                                display_name: p.name,
                            })
                            .collect();
                    app.appointment_form_set_practitioners(practitioner_items);
                    tracing::info!("Loaded practitioners for appointment form");
                }
                Err(e) => {
                    tracing::error!("Failed to load practitioners for form: {}", e);
                }
            }
        }

        if let Some((practitioner_id, date, duration)) = app.take_pending_load_booked_slots() {
            match api_client
                .get_available_slots(practitioner_id, date, duration as i64)
                .await
            {
                Ok(available_slots) => {
                    let booked_slots = compute_booked_slots(&available_slots, &calendar_config);
                    app.appointment_form_set_booked_slots(booked_slots);
                    tracing::info!("Loaded booked slots for time picker");
                }
                Err(e) => {
                    tracing::error!("Failed to load available slots: {:?}", e);
                }
            }
        }

        // Pass patients to appointment form if it exists
        if app.has_appointment_form() {
            let patient_items: Vec<opengp_ui::ui::view_models::PatientListItem> =
                app.patient_list_patients().to_vec();
            app.appointment_form_set_patients(patient_items);
        }

        if let Some(data) = app.take_pending_appointment_save() {
            let appointment_date = data.start_time.date_naive();
            let request = appointment_request_from_new(data);
            match api_client.create_appointment(&request).await {
                Ok(_) => {
                    tracing::info!("Created new appointment via API");
                    let date = app
                        .appointment_state_mut()
                        .selected_date
                        .unwrap_or(appointment_date);
                    app.request_refresh_appointments(date);
                }
                Err(e) => tracing::error!("Failed to create appointment: {}", e),
            }
        }

        if let Some((appointment_id, transition)) = app.take_pending_appointment_status_transition()
        {
            let result = match transition {
                opengp_ui::ui::app::AppointmentStatusTransition::MarkArrived => {
                    api_client
                        .update_appointment_status(appointment_id, "arrived")
                        .await
                }
                opengp_ui::ui::app::AppointmentStatusTransition::MarkInProgress => {
                    api_client
                        .update_appointment_status(appointment_id, "in_progress")
                        .await
                }
                opengp_ui::ui::app::AppointmentStatusTransition::MarkCompleted => {
                    api_client
                        .update_appointment_status(appointment_id, "completed")
                        .await
                }
            };
            match result {
                Ok(_) => {
                    tracing::info!("Updated appointment status: {:?}", transition);
                    if let Some(date) = app.appointment_state_mut().selected_date {
                        app.request_refresh_appointments(date);
                    }
                }
                Err(e) => tracing::error!("Failed to update appointment status: {}", e),
            }
        }

        if let Some(pending) = app.take_pending_clinical_save_data() {
            match pending {
                opengp_ui::ui::app::PendingClinicalSaveData::Allergy {
                    patient_id,
                    allergy,
                } => {
                    let request = AllergyRequest {
                        allergen: allergy.allergen,
                        allergy_type: allergy_type_to_api_string(allergy.allergy_type).to_string(),
                        severity: severity_to_api_string(allergy.severity).to_string(),
                        reaction: allergy.reaction,
                        onset_date: allergy.onset_date,
                        notes: allergy.notes,
                    };
                    match api_client.create_allergy(patient_id, &request).await {
                        Ok(_) => {
                            tracing::info!("Saved allergy for patient {}", patient_id);
                            match api_client.get_allergies(patient_id).await {
                                Ok(allergies) => {
                                    app.clinical_state_mut().allergies = allergies
                                        .into_iter()
                                        .map(domain_allergy_from_api_response)
                                        .collect()
                                }
                                Err(e) => {
                                    tracing::error!("Failed to reload allergies: {}", e);
                                    app.set_status_error(format!(
                                        "Failed to reload allergies: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to save allergy: {}", e);
                            app.set_status_error(format!("Failed to save allergy: {}", e));
                        }
                    }
                }
                opengp_ui::ui::app::PendingClinicalSaveData::MedicalHistory {
                    patient_id,
                    history,
                } => {
                    let request = MedicalHistoryRequest {
                        condition: history.condition,
                        diagnosis_date: history.diagnosis_date,
                        status: condition_status_to_api_string(history.status).to_string(),
                        severity: history
                            .severity
                            .map(|severity| severity_to_api_string(severity).to_string()),
                        notes: history.notes,
                    };
                    match api_client.create_medical_history(patient_id, &request).await {
                        Ok(_) => {
                            tracing::info!("Saved medical history for patient {}", patient_id);
                            match api_client.get_medical_history(patient_id).await {
                                Ok(conditions) => {
                                    app.clinical_state_mut().medical_history = conditions
                                        .into_iter()
                                        .map(domain_medical_history_from_api_response)
                                        .collect()
                                }
                                Err(e) => {
                                    tracing::error!("Failed to reload medical history: {}", e);
                                    app.set_status_error(format!(
                                        "Failed to reload medical history: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to save medical history: {}", e);
                            app.set_status_error(format!("Failed to save medical history: {}", e));
                        }
                    }
                }
                opengp_ui::ui::app::PendingClinicalSaveData::VitalSigns { patient_id, vitals } => {
                    let request = VitalSignsRequest {
                        consultation_id: vitals.consultation_id,
                        systolic_bp: vitals.systolic_bp,
                        diastolic_bp: vitals.diastolic_bp,
                        heart_rate: vitals.heart_rate,
                        respiratory_rate: vitals.respiratory_rate,
                        temperature: vitals.temperature,
                        oxygen_saturation: vitals.oxygen_saturation,
                        height_cm: vitals.height_cm,
                        weight_kg: vitals.weight_kg,
                        notes: vitals.notes,
                    };
                    match api_client.create_vitals(patient_id, &request).await {
                        Ok(_) => {
                            tracing::info!("Saved vital signs for patient {}", patient_id);
                            match api_client.get_vitals(patient_id).await {
                                Ok(v) => {
                                    app.clinical_state_mut().vital_signs = v
                                        .into_iter()
                                        .map(domain_vital_signs_from_api_response)
                                        .collect()
                                }
                                Err(e) => {
                                    tracing::error!("Failed to reload vital signs: {}", e);
                                    app.set_status_error(format!(
                                        "Failed to reload vital signs: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to save vital signs: {}", e);
                            app.set_status_error(format!("Failed to save vital signs: {}", e));
                        }
                    }
                }
                opengp_ui::ui::app::PendingClinicalSaveData::FamilyHistory {
                    patient_id,
                    entry,
                } => {
                    let request = FamilyHistoryRequest {
                        relative_relationship: entry.relative_relationship,
                        condition: entry.condition,
                        age_at_diagnosis: entry.age_at_diagnosis,
                        notes: entry.notes,
                    };
                    match api_client.create_family_history(patient_id, &request).await {
                        Ok(_) => {
                            tracing::info!("Saved family history for patient {}", patient_id);
                            match api_client.get_family_history(patient_id).await {
                                Ok(entries) => {
                                    app.clinical_state_mut().family_history = entries
                                        .into_iter()
                                        .map(domain_family_history_from_api_response)
                                        .collect()
                                }
                                Err(e) => {
                                    tracing::error!("Failed to reload family history: {}", e);
                                    app.set_status_error(format!(
                                        "Failed to reload family history: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to save family history: {}", e);
                            app.set_status_error(format!("Failed to save family history: {}", e));
                        }
                    }
                }
                opengp_ui::ui::app::PendingClinicalSaveData::Consultation {
                    patient_id,
                    practitioner_id,
                    appointment_id,
                    reason,
                    clinical_notes,
                } => {
                    let effective_practitioner_id = if practitioner_id.is_nil() {
                        app.current_user_id
                    } else {
                        practitioner_id
                    };
                    let request = ConsultationRequest {
                        patient_id,
                        practitioner_id: effective_practitioner_id,
                        appointment_id,
                        reason,
                        clinical_notes,
                        version: 1,
                    };
                    match api_client.create_consultation(&request).await {
                        Ok(consultation) => {
                            tracing::info!(
                                "Created consultation {} for patient {}",
                                consultation.id,
                                patient_id
                            );
                            app.request_refresh_consultations(patient_id);
                        }
                        Err(e) => {
                            tracing::error!("Failed to create consultation: {}", e);
                            app.set_status_error(format!("Failed to create consultation: {}", e));
                        }
                    }
                }
                opengp_ui::ui::app::PendingClinicalSaveData::SocialHistory {
                    patient_id,
                    history,
                } => {
                    let request = SocialHistoryRequest {
                        smoking_status: smoking_status_to_api_string(history.smoking_status)
                            .to_string(),
                        cigarettes_per_day: history.cigarettes_per_day,
                        smoking_quit_date: history.smoking_quit_date,
                        alcohol_status: alcohol_status_to_api_string(history.alcohol_status)
                            .to_string(),
                        standard_drinks_per_week: history.standard_drinks_per_week,
                        exercise_frequency: history
                            .exercise_frequency
                            .map(|frequency| exercise_frequency_to_api_string(frequency).to_string()),
                        occupation: history.occupation,
                        living_situation: history.living_situation,
                        support_network: history.support_network,
                        notes: history.notes,
                    };
                    match api_client.update_social_history(patient_id, &request).await {
                        Ok(_) => {
                            tracing::info!("Saved social history for patient {}", patient_id);
                            match api_client.get_social_history(patient_id).await {
                                Ok(sh) => {
                                    app.clinical_state_mut().social_history =
                                        Some(domain_social_history_from_api_response(sh))
                                }
                                Err(e) => {
                                    tracing::error!("Failed to reload social history: {}", e);
                                    app.set_status_error(format!(
                                        "Failed to reload social history: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to save social history: {}", e);
                            app.set_status_error(format!("Failed to save social history: {}", e));
                        }
                    }
                }
            }
        }

        if let Some(patient_id) = app.take_pending_clinical_patient_id() {
            app.clinical_state_mut().set_loading(true);

            app.request_refresh_consultations(patient_id);

            match api_client.get_allergies(patient_id).await {
                Ok(allergies) => {
                    app.clinical_state_mut().allergies = allergies
                        .into_iter()
                        .map(domain_allergy_from_api_response)
                        .collect();
                    tracing::info!("Loaded allergies for clinical view");
                }
                Err(e) => tracing::error!("Failed to load allergies: {}", e),
            }

            match api_client.get_medical_history(patient_id).await {
                Ok(conditions) => {
                    app.clinical_state_mut().medical_history = conditions
                        .into_iter()
                        .map(domain_medical_history_from_api_response)
                        .collect();
                    tracing::info!("Loaded medical history for clinical view");
                }
                Err(e) => tracing::error!("Failed to load medical history: {}", e),
            }

            match api_client.get_vitals(patient_id).await {
                Ok(vitals) => {
                    app.clinical_state_mut().vital_signs = vitals
                        .into_iter()
                        .map(domain_vital_signs_from_api_response)
                        .collect();
                    tracing::info!("Loaded vital signs for clinical view");
                }
                Err(e) => tracing::error!("Failed to load vital signs: {}", e),
            }

            match api_client.get_social_history(patient_id).await {
                Ok(history) => {
                    app.clinical_state_mut().social_history =
                        Some(domain_social_history_from_api_response(history));
                    tracing::info!("Loaded social history for clinical view");
                }
                Err(e) => tracing::error!("Failed to load social history: {}", e),
            }

            match api_client.get_family_history(patient_id).await {
                Ok(entries) => {
                    app.clinical_state_mut().family_history = entries
                        .into_iter()
                        .map(domain_family_history_from_api_response)
                        .collect();
                    tracing::info!("Loaded family history for clinical view");
                }
                Err(e) => tracing::error!("Failed to load family history: {}", e),
            }

            app.clinical_state_mut().set_loading(false);
        }

        if let Ok(event) = crossterm::event::read() {
            match event {
                Event::Key(key) => {
                    let action = app.handle_key_event(key);

                    if action == opengp_ui::ui::keybinds::Action::Quit || app.should_quit() {
                        break;
                    }
                }
                Event::Mouse(mouse) => {
                    let terminal_size = terminal.size().unwrap_or_default();
                    let terminal_rect =
                        ratatui::layout::Rect::new(0, 0, terminal_size.width, terminal_size.height);
                    app.handle_mouse_event(mouse, terminal_rect);
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn compute_booked_slots(
    available_slots: &[chrono::NaiveTime],
    calendar_config: &CalendarConfig,
) -> Vec<chrono::NaiveTime> {
    use chrono::NaiveTime;

    let mut all_slots = Vec::new();

    // Generate all 15-minute slots from min_hour to max_hour
    for hour in calendar_config.min_hour..calendar_config.max_hour {
        for minute in [0, 15, 30, 45].iter() {
            if let Some(time) = NaiveTime::from_hms_opt(hour as u32, *minute, 0) {
                all_slots.push(time);
            }
        }
    }

    // Booked = all slots minus available slots
    all_slots
        .into_iter()
        .filter(|slot| !available_slots.contains(slot))
        .collect()
}

fn patient_request_from_new(data: opengp_domain::domain::patient::NewPatientData) -> PatientRequest {
    PatientRequest {
        first_name: data.first_name,
        last_name: data.last_name,
        date_of_birth: data.date_of_birth,
        gender: gender_to_api_string(data.gender),
        phone_mobile: data.phone_mobile,
        email: data.email,
        medicare_number: data.medicare_number,
        version: 1,
    }
}

fn appointment_request_from_new(
    data: opengp_domain::domain::appointment::NewAppointmentData,
) -> AppointmentRequest {
    AppointmentRequest {
        patient_id: data.patient_id,
        practitioner_id: data.practitioner_id,
        start_time: data.start_time,
        duration_minutes: data.duration.num_minutes(),
        appointment_type: appointment_type_to_api_string(data.appointment_type).to_string(),
        reason: data.reason,
        is_urgent: data.is_urgent,
        version: 1,
    }
}

fn patient_request_from_update(
    data: opengp_domain::domain::patient::UpdatePatientData,
    current: &PatientResponse,
) -> PatientRequest {
    PatientRequest {
        first_name: data.first_name.unwrap_or_else(|| current.first_name.clone()),
        last_name: data.last_name.unwrap_or_else(|| current.last_name.clone()),
        date_of_birth: data.date_of_birth.unwrap_or(current.date_of_birth),
        gender: data
            .gender
            .map(gender_to_api_string)
            .unwrap_or_else(|| current.gender.clone()),
        phone_mobile: data.phone_mobile.or_else(|| current.phone_mobile.clone()),
        email: data.email.or_else(|| current.email.clone()),
        medicare_number: data.medicare_number,
        version: current.version,
    }
}

fn domain_patient_from_api_response(response: PatientResponse) -> Patient {
    Patient {
        id: response.id,
        ihi: None,
        medicare_number: None,
        medicare_irn: None,
        medicare_expiry: None,
        title: None,
        first_name: response.first_name,
        middle_name: None,
        last_name: response.last_name,
        preferred_name: None,
        date_of_birth: response.date_of_birth,
        gender: parse_api_gender(&response.gender),
        address: opengp_domain::domain::patient::Address::default(),
        phone_home: None,
        phone_mobile: response.phone_mobile,
        email: response.email,
        emergency_contact: None,
        concession_type: None,
        concession_number: None,
        preferred_language: "English".to_string(),
        interpreter_required: false,
        aboriginal_torres_strait_islander: None,
        is_active: response.is_active,
        is_deceased: false,
        deceased_date: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        version: response.version,
    }
}

fn domain_allergy_from_api_response(response: AllergyResponse) -> opengp_domain::domain::clinical::Allergy {
    opengp_domain::domain::clinical::Allergy {
        id: response.id,
        patient_id: response.patient_id,
        allergen: response.allergen,
        allergy_type: parse_api_allergy_type(&response.allergy_type),
        severity: parse_api_severity(&response.severity),
        reaction: response.reaction,
        onset_date: response.onset_date,
        notes: response.notes,
        is_active: response.is_active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: uuid::Uuid::nil(),
        updated_by: None,
    }
}

fn domain_medical_history_from_api_response(
    response: MedicalHistoryResponse,
) -> opengp_domain::domain::clinical::MedicalHistory {
    opengp_domain::domain::clinical::MedicalHistory {
        id: response.id,
        patient_id: response.patient_id,
        condition: response.condition,
        diagnosis_date: response.diagnosis_date,
        status: parse_api_condition_status(&response.status),
        severity: response
            .severity
            .map(|severity| parse_api_severity(&severity)),
        notes: response.notes,
        is_active: response.is_active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: uuid::Uuid::nil(),
        updated_by: None,
    }
}

fn domain_vital_signs_from_api_response(
    response: VitalSignsResponse,
) -> opengp_domain::domain::clinical::VitalSigns {
    opengp_domain::domain::clinical::VitalSigns {
        id: response.id,
        patient_id: response.patient_id,
        consultation_id: response.consultation_id,
        measured_at: response.measured_at,
        systolic_bp: response.systolic_bp,
        diastolic_bp: response.diastolic_bp,
        heart_rate: response.heart_rate,
        respiratory_rate: response.respiratory_rate,
        temperature: response.temperature,
        oxygen_saturation: response.oxygen_saturation,
        height_cm: response.height_cm,
        weight_kg: response.weight_kg,
        bmi: response.bmi,
        notes: response.notes,
        created_at: response.measured_at,
        created_by: uuid::Uuid::nil(),
    }
}

fn domain_social_history_from_api_response(
    response: SocialHistoryResponse,
) -> opengp_domain::domain::clinical::SocialHistory {
    opengp_domain::domain::clinical::SocialHistory {
        id: response.id,
        patient_id: response.patient_id,
        smoking_status: parse_api_smoking_status(&response.smoking_status),
        cigarettes_per_day: response.cigarettes_per_day,
        smoking_quit_date: response.smoking_quit_date,
        alcohol_status: parse_api_alcohol_status(&response.alcohol_status),
        standard_drinks_per_week: response.standard_drinks_per_week,
        exercise_frequency: response
            .exercise_frequency
            .map(|frequency| parse_api_exercise_frequency(&frequency)),
        occupation: response.occupation,
        living_situation: response.living_situation,
        support_network: response.support_network,
        notes: response.notes,
        updated_at: response.updated_at,
        updated_by: response.updated_by,
    }
}

fn domain_family_history_from_api_response(
    response: FamilyHistoryResponse,
) -> opengp_domain::domain::clinical::FamilyHistory {
    opengp_domain::domain::clinical::FamilyHistory {
        id: response.id,
        patient_id: response.patient_id,
        relative_relationship: response.relative_relationship,
        condition: response.condition,
        age_at_diagnosis: response.age_at_diagnosis,
        notes: response.notes,
        created_at: response.created_at,
        created_by: response.created_by,
    }
}

fn gender_to_api_string(gender: Gender) -> String {
    match gender {
        Gender::Male => "male".to_string(),
        Gender::Female => "female".to_string(),
        Gender::Other => "other".to_string(),
        Gender::PreferNotToSay => "prefer_not_to_say".to_string(),
    }
}

fn appointment_type_to_api_string(appointment_type: AppointmentType) -> &'static str {
    match appointment_type {
        AppointmentType::Standard => "standard",
        AppointmentType::Long => "long",
        AppointmentType::Brief => "brief",
        AppointmentType::NewPatient => "new_patient",
        AppointmentType::HealthAssessment => "health_assessment",
        AppointmentType::ChronicDiseaseReview => "chronic_disease_review",
        AppointmentType::MentalHealthPlan => "mental_health_plan",
        AppointmentType::Immunisation => "immunisation",
        AppointmentType::Procedure => "procedure",
        AppointmentType::Telephone => "telephone",
        AppointmentType::Telehealth => "telehealth",
        AppointmentType::HomeVisit => "home_visit",
        AppointmentType::Emergency => "emergency",
    }
}

fn allergy_type_to_api_string(allergy_type: opengp_domain::domain::clinical::AllergyType) -> &'static str {
    match allergy_type {
        opengp_domain::domain::clinical::AllergyType::Drug => "drug",
        opengp_domain::domain::clinical::AllergyType::Food => "food",
        opengp_domain::domain::clinical::AllergyType::Environmental => "environmental",
        opengp_domain::domain::clinical::AllergyType::Other => "other",
    }
}

fn severity_to_api_string(severity: opengp_domain::domain::clinical::Severity) -> &'static str {
    match severity {
        opengp_domain::domain::clinical::Severity::Mild => "mild",
        opengp_domain::domain::clinical::Severity::Moderate => "moderate",
        opengp_domain::domain::clinical::Severity::Severe => "severe",
    }
}

fn condition_status_to_api_string(
    condition_status: opengp_domain::domain::clinical::ConditionStatus,
) -> &'static str {
    match condition_status {
        opengp_domain::domain::clinical::ConditionStatus::Active => "active",
        opengp_domain::domain::clinical::ConditionStatus::Resolved => "resolved",
        opengp_domain::domain::clinical::ConditionStatus::Chronic => "chronic",
        opengp_domain::domain::clinical::ConditionStatus::Recurring => "recurring",
        opengp_domain::domain::clinical::ConditionStatus::InRemission => "in_remission",
    }
}

fn smoking_status_to_api_string(
    smoking_status: opengp_domain::domain::clinical::SmokingStatus,
) -> &'static str {
    match smoking_status {
        opengp_domain::domain::clinical::SmokingStatus::NeverSmoked => "never_smoked",
        opengp_domain::domain::clinical::SmokingStatus::CurrentSmoker => "current_smoker",
        opengp_domain::domain::clinical::SmokingStatus::ExSmoker => "ex_smoker",
    }
}

fn alcohol_status_to_api_string(
    alcohol_status: opengp_domain::domain::clinical::AlcoholStatus,
) -> &'static str {
    match alcohol_status {
        opengp_domain::domain::clinical::AlcoholStatus::None => "none",
        opengp_domain::domain::clinical::AlcoholStatus::Occasional => "occasional",
        opengp_domain::domain::clinical::AlcoholStatus::Moderate => "moderate",
        opengp_domain::domain::clinical::AlcoholStatus::Heavy => "heavy",
    }
}

fn exercise_frequency_to_api_string(
    exercise_frequency: opengp_domain::domain::clinical::ExerciseFrequency,
) -> &'static str {
    match exercise_frequency {
        opengp_domain::domain::clinical::ExerciseFrequency::None => "none",
        opengp_domain::domain::clinical::ExerciseFrequency::Rarely => "rarely",
        opengp_domain::domain::clinical::ExerciseFrequency::OnceOrTwicePerWeek => {
            "once_or_twice_per_week"
        }
        opengp_domain::domain::clinical::ExerciseFrequency::ThreeToFiveTimes => {
            "three_to_five_times"
        }
        opengp_domain::domain::clinical::ExerciseFrequency::Daily => "daily",
    }
}

fn parse_api_gender(gender: &str) -> Gender {
    match gender.trim().to_ascii_lowercase().as_str() {
        "male" => Gender::Male,
        "female" => Gender::Female,
        "other" => Gender::Other,
        "prefer_not_to_say" | "prefer-not-to-say" => Gender::PreferNotToSay,
        _ => Gender::PreferNotToSay,
    }
}

fn parse_api_allergy_type(allergy_type: &str) -> opengp_domain::domain::clinical::AllergyType {
    match allergy_type.trim().to_ascii_lowercase().as_str() {
        "drug" => opengp_domain::domain::clinical::AllergyType::Drug,
        "food" => opengp_domain::domain::clinical::AllergyType::Food,
        "environmental" => opengp_domain::domain::clinical::AllergyType::Environmental,
        _ => opengp_domain::domain::clinical::AllergyType::Other,
    }
}

fn parse_api_severity(severity: &str) -> opengp_domain::domain::clinical::Severity {
    match severity.trim().to_ascii_lowercase().as_str() {
        "mild" => opengp_domain::domain::clinical::Severity::Mild,
        "moderate" => opengp_domain::domain::clinical::Severity::Moderate,
        _ => opengp_domain::domain::clinical::Severity::Severe,
    }
}

fn parse_api_condition_status(
    condition_status: &str,
) -> opengp_domain::domain::clinical::ConditionStatus {
    match condition_status.trim().to_ascii_lowercase().as_str() {
        "active" => opengp_domain::domain::clinical::ConditionStatus::Active,
        "resolved" => opengp_domain::domain::clinical::ConditionStatus::Resolved,
        "chronic" => opengp_domain::domain::clinical::ConditionStatus::Chronic,
        "recurring" => opengp_domain::domain::clinical::ConditionStatus::Recurring,
        "in_remission" | "in-remission" => {
            opengp_domain::domain::clinical::ConditionStatus::InRemission
        }
        _ => opengp_domain::domain::clinical::ConditionStatus::Active,
    }
}

fn parse_api_smoking_status(
    smoking_status: &str,
) -> opengp_domain::domain::clinical::SmokingStatus {
    match smoking_status.trim().to_ascii_lowercase().as_str() {
        "never_smoked" | "never-smoked" => opengp_domain::domain::clinical::SmokingStatus::NeverSmoked,
        "current_smoker" | "current-smoker" => {
            opengp_domain::domain::clinical::SmokingStatus::CurrentSmoker
        }
        "ex_smoker" | "ex-smoker" => opengp_domain::domain::clinical::SmokingStatus::ExSmoker,
        _ => opengp_domain::domain::clinical::SmokingStatus::NeverSmoked,
    }
}

fn parse_api_alcohol_status(
    alcohol_status: &str,
) -> opengp_domain::domain::clinical::AlcoholStatus {
    match alcohol_status.trim().to_ascii_lowercase().as_str() {
        "none" => opengp_domain::domain::clinical::AlcoholStatus::None,
        "occasional" => opengp_domain::domain::clinical::AlcoholStatus::Occasional,
        "moderate" => opengp_domain::domain::clinical::AlcoholStatus::Moderate,
        "heavy" => opengp_domain::domain::clinical::AlcoholStatus::Heavy,
        _ => opengp_domain::domain::clinical::AlcoholStatus::None,
    }
}

fn parse_api_exercise_frequency(
    exercise_frequency: &str,
) -> opengp_domain::domain::clinical::ExerciseFrequency {
    match exercise_frequency.trim().to_ascii_lowercase().as_str() {
        "none" => opengp_domain::domain::clinical::ExerciseFrequency::None,
        "rarely" => opengp_domain::domain::clinical::ExerciseFrequency::Rarely,
        "once_or_twice_per_week" | "once-or-twice-per-week" => {
            opengp_domain::domain::clinical::ExerciseFrequency::OnceOrTwicePerWeek
        }
        "three_to_five_times" | "three-to-five-times" => {
            opengp_domain::domain::clinical::ExerciseFrequency::ThreeToFiveTimes
        }
        "daily" => opengp_domain::domain::clinical::ExerciseFrequency::Daily,
        _ => opengp_domain::domain::clinical::ExerciseFrequency::None,
    }
}

fn init_logging(level: &str) {
    let log_level = level.parse().unwrap_or(tracing::Level::INFO);

    std::fs::create_dir_all("logs").ok();

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/opengp.log")
        .expect("Failed to open log file");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::sync::Arc::new(log_file))
                .with_target(true)
                .with_thread_ids(false)
                .with_line_number(true)
                .with_ansi(false),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("opengp", log_level)
                .with_default(tracing::Level::WARN),
        )
        .init();
}
