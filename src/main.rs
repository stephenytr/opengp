use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use opengp_domain::domain::api::{
    AllergyRequest, ConsultationRequest, FamilyHistoryRequest, MedicalHistoryRequest,
    SocialHistoryRequest, VitalSignsRequest,
};
use opengp_ui::api::ApiClient;
use opengp_ui::ui::app::{App, AppCommand, PendingBillingSaveData};
use opengp_ui::ui::services::{BillingUiService, ClinicalUiService};
use opengp_domain::domain::billing::{BillingRepository, BillingService, BillingType};
use opengp_domain::domain::clinical::{
    Consultation, ConsultationRepository, ClinicalRepositories, ClinicalService, suggest_mbs_level,
};
use opengp_domain::domain::patient::PatientRepository;
use opengp_domain::domain::audit::{AuditEmitter, AuditRepository, AuditService};
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use opengp_infrastructure::infrastructure::database::{create_pool, DatabaseConfig};
use opengp_infrastructure::infrastructure::database::repositories::{
    SqlxAllergyRepository, SqlxAuditRepository, SqlxBillingRepository, SqlxClinicalRepository,
    SqlxFamilyHistoryRepository, SqlxMedicalHistoryRepository, SqlxPatientRepository,
    SqlxSocialHistoryRepository, SqlxVitalSignsRepository,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use opengp_config::CalendarConfig;
use opengp_config::Config;
use opengp_config::{load_practice_config, PracticeConfig};
use opengp_ui::ui::theme::{ColorPalette, Theme};

mod conversions;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = Config::from_env()?;

    init_logging(&config.app.logging.level, &config.app.logging.log_file);

    tracing::info!("Starting OpenGP");

    let api_base_url = config.app.api_client.base_url.clone();
    let api_client = Arc::new(ApiClient::new(api_base_url));
    if let Ok(token) = std::env::var("API_SESSION_TOKEN") {
        api_client.set_session_token(Some(token)).await;
    }

    run_tui(
        api_client,
        config.app.calendar,
        config.app.ui,
        config.theme,
        load_practice_config()?,
        config.app.api_server.database,
        config.encryption_key,
        config.healthcare,
        config.patient,
        config.allergies,
        config.clinical,
        config.social_history,
    )
    .await?;

    tracing::info!("OpenGP shutdown complete");

    Ok(())
}

async fn run_tui(
    api_client: Arc<ApiClient>,
    calendar_config: CalendarConfig,
    ui_config: opengp_config::UiConfig,
    theme_config: opengp_config::ThemeConfig,
    practice_config: PracticeConfig,
    database_config: DatabaseConfig,
    encryption_key: String,
    healthcare_config: opengp_config::healthcare::HealthcareConfig,
    patient_config: opengp_config::PatientConfig,
    allergy_config: opengp_config::AllergyConfig,
    clinical_config: opengp_config::ClinicalConfig,
    social_history_config: opengp_config::SocialHistoryConfig,
) -> Result<()> {
    // All setup runs before entering the alternate screen so that errors
    // (e.g. database unreachable) are printed to the normal terminal instead
    // of being swallowed by a black screen.

    let has_session_token = api_client.current_session_token().await.is_some();

    let (mut theme, palette_config) = match ui_config.theme.as_str() {
        "light" => (Theme::light(), &theme_config.light),
        "high_contrast" => (Theme::high_contrast(), &theme_config.high_contrast),
        _ => (Theme::dark(), &theme_config.dark),
    };
    theme.colors = ColorPalette::from_config(palette_config);

    let (billing_service, clinical_ui_service) = {
        let db_url = database_config.url.clone();
        let database_pool = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            create_pool(&database_config),
        )
        .await
        .map_err(|_| {
            color_eyre::eyre::eyre!(
                "Database connection timed out after 5 seconds — is PostgreSQL running?\n  URL: {}",
                db_url
            )
        })
        .and_then(|r| {
            r.map_err(|e| {
                color_eyre::eyre::eyre!(
                    "Failed to connect to database — is PostgreSQL running?\n  URL: {}\n  Cause: {}",
                    db_url,
                    e
                )
            })
        })?;
        let pool = database_pool.as_postgres().clone();

        let encryption_service = Arc::new(
            EncryptionService::new_with_key(&encryption_key)
                .map_err(|err| color_eyre::eyre::eyre!(err.to_string()))?,
        );

        let billing_repo: Arc<dyn BillingRepository> =
            Arc::new(SqlxBillingRepository::new(pool.clone()));
        let consultation_repo: Arc<dyn ConsultationRepository> = Arc::new(
            SqlxClinicalRepository::new(pool.clone(), Arc::clone(&encryption_service)),
        );

        let billing_domain_service =
            BillingService::new(Arc::clone(&billing_repo), Arc::clone(&consultation_repo));
        let billing_service = Some(Arc::new(BillingUiService::new(Arc::new(billing_domain_service))));

        // Create clinical service with all required repositories
        let clinical_repos = ClinicalRepositories {
            consultation: Arc::clone(&consultation_repo),
            allergy: Arc::new(SqlxAllergyRepository::new(pool.clone(), Arc::clone(&encryption_service))),
            medical_history: Arc::new(SqlxMedicalHistoryRepository::new(
                pool.clone(),
                Arc::clone(&encryption_service),
            )),
            vital_signs: Arc::new(SqlxVitalSignsRepository::new(pool.clone(), Arc::clone(&encryption_service))),
            social_history: Arc::new(SqlxSocialHistoryRepository::new(
                pool.clone(),
                Arc::clone(&encryption_service),
            )),
            family_history: Arc::new(SqlxFamilyHistoryRepository::new(
                pool.clone(),
                Arc::clone(&encryption_service),
            )),
        };

        let patient_repo: Arc<dyn PatientRepository> =
            Arc::new(SqlxPatientRepository::new(pool.clone(), Arc::clone(&encryption_service)));
        let patient_service = Arc::new(opengp_domain::domain::patient::PatientService::new(patient_repo));

        let audit_repo: Arc<dyn AuditRepository> =
            Arc::new(SqlxAuditRepository::new(pool.clone()));
        let audit_service: Arc<dyn AuditEmitter> = Arc::new(AuditService::new(audit_repo));

        let clinical_domain_service = Arc::new(ClinicalService::new(
            clinical_repos,
            patient_service,
            audit_service,
        ));
        let clinical_service = Some(Arc::new(ClinicalUiService::new(clinical_domain_service)));

        (billing_service, clinical_service)
    };

    let mut app = App::new(
        Some(api_client.clone()),
        calendar_config.clone(),
        theme,
        healthcare_config,
        patient_config,
        allergy_config,
        clinical_config,
        social_history_config,
        billing_service,
        practice_config,
    );
    let mut command_rx = app.take_command_rx().expect("failed to extract command_rx from app");
    app.set_authenticated(has_session_token);
    if has_session_token {
        app.request_refresh_patients();
    }

    // Only enter the alternate screen once all setup has succeeded.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        app.poll_api_tasks().await;

        terminal.draw(|frame| {
            app.render(frame);
        })?;

        // Check if there's pending patient data to save
        if let Some(pending) = app.take_pending_patient_data() {
            match pending {
                opengp_ui::ui::app::PendingPatientData::New(data) => {
                    let request = conversions::patient_request_from_new(data);
                    api_client.create_patient(&request).await?;
                    tracing::info!("Created new patient via API");
                }
                opengp_ui::ui::app::PendingPatientData::Update { id, data } => {
                    let existing = api_client.get_patient(id).await?;
                    let request = conversions::patient_request_from_update(data, &existing);
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
                    app.open_patient_form(conversions::domain_patient_from_api_response(patient));
                    tracing::info!("Loaded patient for editing: {}", patient_id);
                }
                Err(e) => {
                    tracing::error!("Failed to load patient for editing: {}", e);
                }
            }
        }

        // Drain appointment commands from the channel
        while let Ok(cmd) = command_rx.try_recv() {
            match cmd {
                AppCommand::RefreshAppointments(date) => {
                    app.request_refresh_appointments(date);
                }
                AppCommand::LoadPractitioners => {
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
                AppCommand::LoadAvailableSlots { practitioner_id, date, duration_minutes } => {
                    match api_client
                        .get_available_slots(practitioner_id, date, duration_minutes as i64)
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
                AppCommand::CreateAppointment(data) => {
                    let appointment_date = data.start_time.date_naive();
                    let request = conversions::appointment_request_from_new(data);
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
                AppCommand::UpdateAppointmentStatus { id, status } => {
                    let status_str = match status {
                        opengp_domain::domain::appointment::AppointmentStatus::Scheduled => "scheduled",
                        opengp_domain::domain::appointment::AppointmentStatus::Confirmed => "confirmed",
                        opengp_domain::domain::appointment::AppointmentStatus::Arrived => "arrived",
                        opengp_domain::domain::appointment::AppointmentStatus::InProgress => "in_progress",
                        opengp_domain::domain::appointment::AppointmentStatus::Billing => "billing",
                        opengp_domain::domain::appointment::AppointmentStatus::Completed => "completed",
                        opengp_domain::domain::appointment::AppointmentStatus::Cancelled => "cancelled",
                        opengp_domain::domain::appointment::AppointmentStatus::NoShow => "no_show",
                        opengp_domain::domain::appointment::AppointmentStatus::Rescheduled => "rescheduled",
                    };
                    match api_client.update_appointment_status(id, status_str).await {
                        Ok(_) => {
                            tracing::info!("Updated appointment status");
                        }
                        Err(e) => tracing::error!("Failed to update appointment status: {}", e),
                    }
                }
                _ => {}
            }
        }

        // Pass patients to appointment form if it exists
        if app.has_appointment_form() {
            let patient_items: Vec<opengp_ui::ui::view_models::PatientListItem> =
                app.patient_list_patients().to_vec();
            app.appointment_form_set_patients(patient_items);
        }



        if let Some(pending) = app.take_pending_clinical_save_data() {
            match pending {
                opengp_ui::ui::app::PendingClinicalSaveData::Allergy {
                    patient_id,
                    allergy,
                } => {
                    let request = AllergyRequest {
                        allergen: allergy.allergen,
                        allergy_type: conversions::allergy_type_to_api_string(allergy.allergy_type)
                            .to_string(),
                        severity: conversions::severity_to_api_string(allergy.severity).to_string(),
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
                                        .map(conversions::domain_allergy_from_api_response)
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
                        status: conversions::condition_status_to_api_string(history.status)
                            .to_string(),
                        severity: history.severity.map(|severity| {
                            conversions::severity_to_api_string(severity).to_string()
                        }),
                        notes: history.notes,
                    };
                    match api_client
                        .create_medical_history(patient_id, &request)
                        .await
                    {
                        Ok(_) => {
                            tracing::info!("Saved medical history for patient {}", patient_id);
                            match api_client.get_medical_history(patient_id).await {
                                Ok(conditions) => {
                                    app.clinical_state_mut().medical_history = conditions
                                        .into_iter()
                                        .map(conversions::domain_medical_history_from_api_response)
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
                                        .map(conversions::domain_vital_signs_from_api_response)
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
                                        .map(conversions::domain_family_history_from_api_response)
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
                            let started_at = chrono::Utc::now();
                            if let Some(ref service) = clinical_ui_service {
                                if let Err(e) = service.start_timer(consultation.id).await {
                                    tracing::error!("Failed to start timer: {}", e);
                                }
                            }
                            let domain_consultation = Consultation {
                                id: consultation.id,
                                patient_id: consultation.patient_id,
                                practitioner_id: consultation.practitioner_id,
                                appointment_id: consultation.appointment_id,
                                consultation_date: consultation.consultation_date,
                                reason: consultation.reason.clone(),
                                clinical_notes: consultation.clinical_notes.clone(),
                                is_signed: consultation.is_signed,
                                signed_at: None,
                                signed_by: None,
                                consultation_started_at: Some(started_at),
                                consultation_ended_at: None,
                                created_at: consultation.consultation_date,
                                updated_at: consultation.consultation_date,
                                version: consultation.version,
                                created_by: consultation.practitioner_id,
                                updated_by: None,
                            };
                            app.clinical_state_mut().consultations.push(domain_consultation);
                            app.clinical_state_mut().set_active_timer_started_at(started_at);
                            if consultation.is_signed {
                                app.set_pending_billing(PendingBillingSaveData::AwaitingMbsSelection {
                                    consultation_id: consultation.id,
                                    patient_id: consultation.patient_id,
                                });
                            }
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
                        smoking_status: conversions::smoking_status_to_api_string(
                            history.smoking_status,
                        )
                        .to_string(),
                        cigarettes_per_day: history.cigarettes_per_day,
                        smoking_quit_date: history.smoking_quit_date,
                        alcohol_status: conversions::alcohol_status_to_api_string(
                            history.alcohol_status,
                        )
                        .to_string(),
                        standard_drinks_per_week: history.standard_drinks_per_week,
                        exercise_frequency: history.exercise_frequency.map(|frequency| {
                            conversions::exercise_frequency_to_api_string(frequency).to_string()
                        }),
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
                                    app.clinical_state_mut().social_history = Some(
                                        conversions::domain_social_history_from_api_response(sh),
                                    )
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
                opengp_ui::ui::app::PendingClinicalSaveData::TimerStart { consultation_id } => {
                    if let Some(ref service) = clinical_ui_service {
                        if let Err(e) = service.start_timer(consultation_id).await {
                            app.set_status_error(format!("Timer start failed: {}", e));
                        }
                    }
                }
                opengp_ui::ui::app::PendingClinicalSaveData::TimerStop { consultation_id } => {
                    if let Some(ref service) = clinical_ui_service {
                        if let Err(e) = service.stop_timer(consultation_id).await {
                            app.set_status_error(format!("Timer stop failed: {}", e));
                        }
                    }
                }
            }
        }

        if let Some(pending) = app.take_pending_billing() {
            match pending {
                PendingBillingSaveData::AwaitingMbsSelection {
                    consultation_id,
                    patient_id,
                } => {
                    // Fetch the consultation to get the timer duration
                    match api_client
                        .get_consultations(patient_id, 1, 100)
                        .await
                    {
                        Ok(response) => {
                            // Find the consultation with the matching ID
                            if let Some(consultation_response) = response
                                .data
                                .iter()
                                .find(|c| c.id == consultation_id)
                            {
                                // Calculate duration from consultation timer
                                let duration_minutes = match (
                                    consultation_response.consultation_started_at,
                                    consultation_response.consultation_ended_at,
                                ) {
                                    (Some(start), Some(end)) => {
                                        let duration = end.signed_duration_since(start);
                                        duration.num_minutes()
                                    }
                                    _ => 0, // Default to 0 if timer not set
                                };

                                // Get the appropriate MBS item based on duration
                                let mbs_item = suggest_mbs_level(duration_minutes);
                                let selected_items =
                                    vec![(mbs_item.to_string(), 89.0, true)];

                                app.set_pending_billing(PendingBillingSaveData::CreatingInvoice {
                                    consultation_id,
                                    mbs_items: selected_items,
                                    billing_type: BillingType::PrivateBilling,
                                });
                                tracing::info!(
                                    "Selected MBS item {} (duration: {} minutes) for consultation {}",
                                    mbs_item,
                                    duration_minutes,
                                    consultation_id
                                );
                            } else {
                                // Consultation not found, fall back to default MBS item 23
                                let selected_items = vec![("23".to_string(), 89.0, true)];
                                app.set_pending_billing(PendingBillingSaveData::CreatingInvoice {
                                    consultation_id,
                                    mbs_items: selected_items,
                                    billing_type: BillingType::PrivateBilling,
                                });
                                tracing::warn!(
                                    "Consultation {} not found in API response; using default MBS item 23",
                                    consultation_id
                                );
                            }
                        }
                        Err(e) => {
                            // API error, fall back to default MBS item 23
                            let selected_items = vec![("23".to_string(), 89.0, true)];
                            app.set_pending_billing(PendingBillingSaveData::CreatingInvoice {
                                consultation_id,
                                mbs_items: selected_items,
                                billing_type: BillingType::PrivateBilling,
                            });
                            tracing::error!(
                                "Failed to fetch consultations for patient {}: {}; using default MBS item 23",
                                patient_id,
                                e
                            );
                        }
                    }
                }
                PendingBillingSaveData::CreatingInvoice {
                    consultation_id,
                    mbs_items,
                    billing_type,
                } => {
                    if let Some(service) = app.billing_ui_service() {
                        match service
                            .create_invoice(
                                consultation_id,
                                mbs_items,
                                billing_type,
                                app.current_user_id,
                            )
                            .await
                        {
                            Ok(invoice) => {
                                app.open_billing_invoice_detail(invoice.id);
                                tracing::info!(
                                    "Created invoice {} from consultation {}",
                                    invoice.id,
                                    consultation_id
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to create invoice from consultation {}: {}",
                                    consultation_id,
                                    e
                                );
                                app.set_status_error(format!(
                                    "Failed to create invoice from signed consultation: {}",
                                    e
                                ));
                            }
                        }
                    } else {
                        tracing::warn!(
                            "Billing service not wired; skipping invoice creation for consultation {}",
                            consultation_id
                        );
                        app.set_status_error(
                            "Billing service not yet wired; invoice creation deferred",
                        );
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
                        .map(conversions::domain_allergy_from_api_response)
                        .collect();
                    tracing::info!("Loaded allergies for clinical view");
                }
                Err(e) => tracing::error!("Failed to load allergies: {}", e),
            }

            match api_client.get_medical_history(patient_id).await {
                Ok(conditions) => {
                    app.clinical_state_mut().medical_history = conditions
                        .into_iter()
                        .map(conversions::domain_medical_history_from_api_response)
                        .collect();
                    tracing::info!("Loaded medical history for clinical view");
                }
                Err(e) => tracing::error!("Failed to load medical history: {}", e),
            }

            match api_client.get_vitals(patient_id).await {
                Ok(vitals) => {
                    app.clinical_state_mut().vital_signs = vitals
                        .into_iter()
                        .map(conversions::domain_vital_signs_from_api_response)
                        .collect();
                    tracing::info!("Loaded vital signs for clinical view");
                }
                Err(e) => tracing::error!("Failed to load vital signs: {}", e),
            }

            match api_client.get_social_history(patient_id).await {
                Ok(history) => {
                    app.clinical_state_mut().social_history = Some(
                        conversions::domain_social_history_from_api_response(history),
                    );
                    tracing::info!("Loaded social history for clinical view");
                }
                Err(e) => tracing::error!("Failed to load social history: {}", e),
            }

            match api_client.get_family_history(patient_id).await {
                Ok(entries) => {
                    app.clinical_state_mut().family_history = entries
                        .into_iter()
                        .map(conversions::domain_family_history_from_api_response)
                        .collect();
                    tracing::info!("Loaded family history for clinical view");
                }
                Err(e) => tracing::error!("Failed to load family history: {}", e),
            }

            app.clinical_state_mut().set_loading(false);
        }

        if crossterm::event::poll(std::time::Duration::from_millis(ui_config.tick_rate_ms))? {
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
                        let terminal_rect = ratatui::layout::Rect::new(
                            0,
                            0,
                            terminal_size.width,
                            terminal_size.height,
                        );
                        app.handle_global_mouse_event(mouse, terminal_rect);
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
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

fn init_logging(level: &str, log_file_path: &str) {
    let log_level = level.parse().unwrap_or(tracing::Level::INFO);

    if let Some(parent) = std::path::Path::new(log_file_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
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
