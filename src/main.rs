use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use opengp_domain::domain::patient::PatientRepository;
use opengp_domain::domain::user::{PractitionerRepository, UserRepository};
use opengp_domain::domain::appointment::AppointmentCalendarQuery;
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use opengp_infrastructure::infrastructure::database::{create_pool, run_migrations};
use opengp_infrastructure::infrastructure::database::repositories::patient::SqlxPatientRepository;
use opengp_infrastructure::infrastructure::database::repositories::practitioner::SqlxPractitionerRepository;
use opengp_infrastructure::infrastructure::database::repositories::appointment::SqlxAppointmentRepository;
use opengp_infrastructure::infrastructure::database::repositories::user::SqlxUserRepository;
use opengp_ui::ui::app::App;
use opengp_ui::ui::services::AppointmentUiService;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use opengp_config::Config;
use opengp_config::CalendarConfig;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = Config::from_env()?;

    init_logging(&config.log_level);

    tracing::info!("Starting OpenGP");
    tracing::info!("Database URL: {}", config.database.url);

    let db_pool = create_pool(&config.database).await?;

    run_migrations(&db_pool).await?;

    tracing::info!("Database pool created with {} connection(s)", db_pool.size());

    let crypto = Arc::new(EncryptionService::new()?);
    let patient_repo = Arc::new(SqlxPatientRepository::new(db_pool.clone(), crypto.clone()));
    
    // Create appointment-related repositories and service
    let practitioner_repo: Arc<dyn PractitionerRepository> = Arc::new(SqlxPractitionerRepository::new(db_pool.clone()));
    let appointment_repo_impl = Arc::new(SqlxAppointmentRepository::new(db_pool.clone()));
    let appointment_repo_for_create: Arc<dyn opengp_domain::domain::appointment::AppointmentRepository> = appointment_repo_impl.clone();
    let appointment_repo: Arc<dyn AppointmentCalendarQuery> = appointment_repo_impl.clone();
    
    // Create domain appointment service for status transitions
    let audit_service: std::sync::Arc<dyn opengp_domain::domain::audit::AuditEmitter> = Arc::new(opengp_domain::domain::audit::AuditService::new(
        Arc::new(opengp_infrastructure::infrastructure::database::repositories::audit::SqlxAuditRepository::new(db_pool.clone()))
    ));
    let domain_appointment_service = Arc::new(opengp_domain::domain::appointment::AppointmentService::new(
        appointment_repo_for_create.clone(),
        audit_service.clone(),
        appointment_repo.clone(),
    ));
    
    let appointment_service = Arc::new(AppointmentUiService::new(
        practitioner_repo,
        appointment_repo,
        appointment_repo_for_create,
        domain_appointment_service,
    ));

    // Create patient service
    let patient_service = Arc::new(opengp_ui::ui::services::PatientUiService::new(
        Arc::new(opengp_domain::domain::patient::PatientService::new(patient_repo.clone()))
    ));

    // Create clinical service repositories
    let consultation_repo: Arc<dyn opengp_domain::domain::clinical::ConsultationRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxClinicalRepository::new(db_pool.clone(), crypto.clone()));
    let allergy_repo: Arc<dyn opengp_domain::domain::clinical::AllergyRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxAllergyRepository::new(db_pool.clone(), crypto.clone()));
    let medical_history_repo: Arc<dyn opengp_domain::domain::clinical::MedicalHistoryRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxMedicalHistoryRepository::new(db_pool.clone(), crypto.clone()));
    let vital_signs_repo: Arc<dyn opengp_domain::domain::clinical::VitalSignsRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxVitalSignsRepository::new(db_pool.clone(), crypto.clone()));
    let social_history_repo: Arc<dyn opengp_domain::domain::clinical::SocialHistoryRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxSocialHistoryRepository::new(db_pool.clone(), crypto.clone()));
    let family_history_repo: Arc<dyn opengp_domain::domain::clinical::FamilyHistoryRepository> = Arc::new(opengp_infrastructure::infrastructure::database::repositories::clinical::SqlxFamilyHistoryRepository::new(db_pool.clone(), crypto.clone()));
    
    let clinical_repos = opengp_domain::domain::clinical::ClinicalRepositories {
        consultation: consultation_repo,
        allergy: allergy_repo,
        medical_history: medical_history_repo,
        vital_signs: vital_signs_repo,
        social_history: social_history_repo,
        family_history: family_history_repo,
    };
    let clinical_service = Arc::new(opengp_ui::ui::services::ClinicalUiService::new(
        Arc::new(opengp_domain::domain::clinical::ClinicalService::new(
            clinical_repos,
            Arc::new(opengp_domain::domain::patient::PatientService::new(patient_repo.clone())),
            audit_service,
        ))
    ));

    let patients: Vec<opengp_domain::domain::patient::Patient> = patient_repo.list_active().await?;
    tracing::info!("Loaded {} patients from database", patients.len());

    let user_repo = SqlxUserRepository::new(db_pool.clone());
    let system_user_id = match user_repo.find_all().await {
        Ok(users) => {
            if let Some(first_user) = users.first() {
                tracing::info!("Using system_user_id from user: {} ({})", first_user.username, first_user.id);
                first_user.id
            } else {
                tracing::warn!("No users found in database, using nil UUID for system_user_id");
                uuid::Uuid::nil()
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load users, using nil UUID for system_user_id: {}", e);
            uuid::Uuid::nil()
        }
    };

    run_tui(patients, patient_repo.clone(), appointment_service, patient_service, clinical_service, system_user_id, config.calendar).await?;

    tracing::info!("OpenGP shutdown complete");

    Ok(())
}

async fn run_tui(
    patients: Vec<opengp_domain::domain::patient::Patient>,
    patient_repo: Arc<SqlxPatientRepository>,
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

    let mut app = App::new(Some(appointment_service.clone()), Some(patient_service.clone()), Some(clinical_service.clone()), calendar_config);
    app.load_patients(patients);

    loop {
        terminal.draw(|frame| {
            app.render(frame);
        })?;

        // Check if there's pending patient data to save
        if let Some(pending) = app.take_pending_patient_data() {
            use opengp_domain::domain::patient::Patient;
            match pending {
                opengp_ui::ui::app::PendingPatientData::New(data) => {
                    let patient = Patient::from_dto(data)?;
                    patient_repo.create(patient).await?;
                    tracing::info!("Created new patient in database");
                }
                opengp_ui::ui::app::PendingPatientData::Update { id, data } => {
                    let mut patient = patient_repo.find_by_id(id).await?.ok_or_else(|| color_eyre::eyre::eyre!("Patient not found"))?;
                    patient.update(data)?;
                    patient_repo.update(patient).await?;
                    tracing::info!("Updated patient in database");
                }
            }
            
            // Reload patients to update the list
            let patients = patient_repo.list_active().await?;
            app.load_patients(patients);
        }

        // Check if there's a pending patient to load for editing
        if let Some(patient_id) = app.take_pending_edit_patient_id() {
            match patient_repo.find_by_id(patient_id).await {
                Ok(Some(patient)) => {
                    app.open_patient_form(patient);
                    tracing::info!("Loaded patient for editing: {}", patient_id);
                }
                Ok(None) => {
                    tracing::error!("Patient not found for editing: {}", patient_id);
                }
                Err(e) => {
                    tracing::error!("Failed to load patient for editing: {}", e);
                }
            }
        }

        // Check if there's a pending appointment date to load practitioners and schedule for
        if let Some(date) = app.take_pending_appointment_date() {
            match appointment_service.get_practitioners().await {
                Ok(practitioners) => {
                    app.appointment_state_mut().practitioners = practitioners;
                    tracing::info!("Loaded practitioners for schedule view");
                }
                Err(e) => {
                    tracing::error!("Failed to load practitioners: {}", e);
                }
            }

            // Load the schedule for the selected date
            match appointment_service.get_schedule(date).await {
                Ok(schedule) => {
                    app.appointment_state_mut().schedule_data = Some(schedule);
                    tracing::info!("Loaded schedule for date: {}", date);
                }
                Err(e) => {
                    tracing::error!("Failed to load schedule: {}", e);
                }
            }
        }

        // Check if practitioners need to be loaded for appointment form picker
        if app.take_pending_load_practitioners() {
            let needs_load = app.appointment_state_mut().practitioners.is_empty();
            if needs_load {
                match appointment_service.get_practitioners().await {
                    Ok(practitioners) => {
                        app.appointment_state_mut().practitioners = practitioners;
                        tracing::info!("Loaded practitioners for appointment form");
                    }
                    Err(e) => {
                        tracing::error!("Failed to load practitioners for form: {}", e);
                    }
                }
            }
            let practitioner_items: Vec<opengp_ui::ui::view_models::PractitionerViewItem> = 
                app.practitioners().iter()
                    .map(|p: &opengp_domain::domain::user::Practitioner| opengp_ui::ui::view_models::PractitionerViewItem::from(p.clone()))
                    .collect();
            app.appointment_form_set_practitioners(practitioner_items);
        }

        // Pass patients to appointment form if it exists
        if app.has_appointment_form() {
            let patient_items: Vec<opengp_ui::ui::view_models::PatientListItem> = 
                app.patient_list_patients().to_vec();
            app.appointment_form_set_patients(patient_items);
        }

        if let Some(data) = app.take_pending_appointment_save() {
            let appointment_date = data.start_time.date_naive();
            match appointment_service.create_appointment(data, system_user_id).await {
                Ok(()) => {
                    tracing::info!("Created new appointment in database");
                    let date = app.appointment_state_mut().selected_date.unwrap_or(appointment_date);
                    match appointment_service.get_schedule(date).await {
                        Ok(schedule) => {
                            app.appointment_state_mut().schedule_data = Some(schedule);
                        }
                        Err(e) => tracing::error!("Failed to reload schedule: {}", e),
                    }
                }
                Err(e) => tracing::error!("Failed to create appointment: {}", e),
            }
        }

        if let Some((appointment_id, transition)) = app.take_pending_appointment_status_transition() {
            let result = match transition {
                opengp_ui::ui::app::AppointmentStatusTransition::MarkArrived => {
                    appointment_service.mark_arrived(appointment_id, system_user_id).await
                }
                opengp_ui::ui::app::AppointmentStatusTransition::MarkInProgress => {
                    appointment_service.mark_in_progress(appointment_id, system_user_id).await
                }
                opengp_ui::ui::app::AppointmentStatusTransition::MarkCompleted => {
                    appointment_service.mark_completed(appointment_id, system_user_id).await
                }
            };
            match result {
                Ok(()) => {
                    tracing::info!("Updated appointment status: {:?}", transition);
                    if let Some(date) = app.appointment_state_mut().selected_date {
                        match appointment_service.get_schedule(date).await {
                            Ok(schedule) => {
                                app.appointment_state_mut().schedule_data = Some(schedule);
                            }
                            Err(e) => tracing::error!("Failed to reload schedule: {}", e),
                        }
                    }
                }
                Err(e) => tracing::error!("Failed to update appointment status: {}", e),
            }
        }

        if let Some(pending) = app.take_pending_clinical_save_data() {
            match pending {
                opengp_ui::ui::app::PendingClinicalSaveData::Allergy { patient_id, allergy } => {
                    match clinical_service.add_allergy(
                        patient_id,
                        allergy.allergen,
                        allergy.allergy_type,
                        allergy.severity,
                        allergy.reaction,
                        allergy.notes,
                        system_user_id,
                    ).await {
                        Ok(_) => {
                            tracing::info!("Saved allergy for patient {}", patient_id);
                            match clinical_service.list_allergies(patient_id, false).await {
                                Ok(allergies) => app.clinical_state_mut().allergies = allergies,
                                Err(e) => tracing::error!("Failed to reload allergies: {}", e),
                            }
                        }
                        Err(e) => tracing::error!("Failed to save allergy: {}", e),
                    }
                }
                opengp_ui::ui::app::PendingClinicalSaveData::MedicalHistory { patient_id, history } => {
                    match clinical_service.add_medical_history(
                        patient_id,
                        history.condition,
                        history.status,
                        history.severity,
                        history.notes,
                        system_user_id,
                    ).await {
                        Ok(_) => {
                            tracing::info!("Saved medical history for patient {}", patient_id);
                            match clinical_service.list_medical_history(patient_id, false).await {
                                Ok(conditions) => app.clinical_state_mut().medical_history = conditions,
                                Err(e) => tracing::error!("Failed to reload medical history: {}", e),
                            }
                        }
                        Err(e) => tracing::error!("Failed to save medical history: {}", e),
                    }
                }
                opengp_ui::ui::app::PendingClinicalSaveData::VitalSigns { patient_id, vitals } => {
                    match clinical_service.record_vitals(
                        patient_id,
                        vitals.systolic_bp,
                        vitals.diastolic_bp,
                        vitals.heart_rate,
                        vitals.respiratory_rate,
                        vitals.temperature,
                        vitals.oxygen_saturation,
                        vitals.height_cm,
                        vitals.weight_kg,
                        vitals.notes,
                        system_user_id,
                    ).await {
                        Ok(_) => {
                            tracing::info!("Saved vital signs for patient {}", patient_id);
                            match clinical_service.list_vitals_history(patient_id, 50).await {
                                Ok(v) => app.clinical_state_mut().vital_signs = v,
                                Err(e) => tracing::error!("Failed to reload vital signs: {}", e),
                            }
                        }
                        Err(e) => tracing::error!("Failed to save vital signs: {}", e),
                    }
                }
                opengp_ui::ui::app::PendingClinicalSaveData::FamilyHistory { patient_id, entry } => {
                    match clinical_service.add_family_history(
                        patient_id,
                        entry.relative_relationship,
                        entry.condition,
                        entry.age_at_diagnosis,
                        entry.notes,
                        system_user_id,
                    ).await {
                        Ok(_) => {
                            tracing::info!("Saved family history for patient {}", patient_id);
                            match clinical_service.list_family_history(patient_id).await {
                                Ok(entries) => app.clinical_state_mut().family_history = entries,
                                Err(e) => tracing::error!("Failed to reload family history: {}", e),
                            }
                        }
                        Err(e) => tracing::error!("Failed to save family history: {}", e),
                    }
                }
                opengp_ui::ui::app::PendingClinicalSaveData::Consultation {
                    patient_id,
                    practitioner_id,
                    appointment_id: _,
                    reason,
                    clinical_notes,
                } => {
                    match clinical_service.create_consultation(
                        patient_id,
                        practitioner_id,
                        system_user_id,
                        reason,
                        clinical_notes,
                    ).await {
                        Ok(consultation) => {
                            tracing::info!("Created consultation {} for patient {}", consultation.id, patient_id);
                            match clinical_service.list_consultations(patient_id).await {
                                Ok(consultations) => {
                                    app.clinical_state_mut().consultation_list.consultations = consultations.clone();
                                    app.clinical_state_mut().consultations = consultations;
                                }
                                Err(e) => tracing::error!("Failed to reload consultations: {}", e),
                            }
                        }
                        Err(e) => tracing::error!("Failed to create consultation: {}", e),
                    }
                }
            }
        }

        if let Some(patient_id) = app.take_pending_clinical_patient_id() {
            app.clinical_state_mut().set_loading(true);

            match clinical_service.list_consultations(patient_id).await {
                Ok(consultations) => {
                    app.clinical_state_mut().consultations = consultations;
                    tracing::info!("Loaded consultations for clinical view");
                }
                Err(e) => tracing::error!("Failed to load consultations: {}", e),
            }

            match clinical_service.list_allergies(patient_id, false).await {
                Ok(allergies) => {
                    app.clinical_state_mut().allergies = allergies;
                    tracing::info!("Loaded allergies for clinical view");
                }
                Err(e) => tracing::error!("Failed to load allergies: {}", e),
            }

            match clinical_service.list_medical_history(patient_id, false).await {
                Ok(conditions) => {
                    app.clinical_state_mut().medical_history = conditions;
                    tracing::info!("Loaded medical history for clinical view");
                }
                Err(e) => tracing::error!("Failed to load medical history: {}", e),
            }

            match clinical_service.list_vitals_history(patient_id, 50).await {
                Ok(vitals) => {
                    app.clinical_state_mut().vital_signs = vitals;
                    tracing::info!("Loaded vital signs for clinical view");
                }
                Err(e) => tracing::error!("Failed to load vital signs: {}", e),
            }

            match clinical_service.get_social_history(patient_id).await {
                Ok(history) => {
                    app.clinical_state_mut().social_history = history;
                    tracing::info!("Loaded social history for clinical view");
                }
                Err(e) => tracing::error!("Failed to load social history: {}", e),
            }

            match clinical_service.list_family_history(patient_id).await {
                Ok(entries) => {
                    app.clinical_state_mut().family_history = entries;
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
                    let terminal_rect = ratatui::layout::Rect::new(0, 0, terminal_size.width, terminal_size.height);
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
