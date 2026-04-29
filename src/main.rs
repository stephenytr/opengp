use std::sync::Arc;

use color_eyre::eyre::eyre;
use opengp_config::{load_practice_config, CalendarConfig, Config, PracticeConfig};
use opengp_domain::domain::appointment::{
    AppointmentCalendarQuery, AppointmentRepository, AppointmentService, AvailabilityService,
};
use opengp_domain::domain::audit::{AuditEmitter, AuditRepository, AuditService};
use opengp_domain::domain::billing::{BillingRepository, BillingService};
use opengp_domain::domain::clinical::{ClinicalRepositories, ClinicalService, ConsultationRepository};
use opengp_domain::domain::patient::PatientRepository;
use opengp_domain::domain::user::WorkingHoursRepository;
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use opengp_infrastructure::infrastructure::database::repositories::{
    SqlxAllergyRepository, SqlxAppointmentRepository, SqlxAuditRepository, SqlxBillingRepository,
    SqlxClinicalRepository, SqlxFamilyHistoryRepository, SqlxMedicalHistoryRepository,
    SqlxPatientRepository, SqlxPractitionerRepository, SqlxSocialHistoryRepository,
    SqlxVitalSignsRepository, SqlxWorkingHoursRepository,
};
use opengp_infrastructure::infrastructure::database::{create_pool, DatabaseConfig};
use opengp_ui::api::ApiClient;
use opengp_ui::ui::app::{AppCommand, AppError, AppEvent, AppState, GlobalState};
use opengp_ui::ui::components::appointment::AppointmentState;
use opengp_ui::ui::components::help::HelpOverlay;
use opengp_ui::ui::components::patient::PatientList;
use opengp_ui::ui::components::status_bar::StatusBar;
use opengp_ui::ui::components::tabs::{Tab, TabBar};
use opengp_ui::ui::components::workspace::WorkspaceManager;
use opengp_ui::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use opengp_ui::ui::services::{AppointmentUiService, BillingUiService, ClinicalUiService};
use opengp_ui::ui::theme::{ColorPalette, Theme};
use rat_salsa::poll::{PollCrossterm, PollTasks, PollTokio};
use rat_salsa::{run_tui, Control, RunConfig, SalsaAppContext};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Paragraph, Widget};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod conversions;

fn main() -> Result<(), AppError> {
    color_eyre::install()?;

    let config = Config::from_env()?;
    init_logging(&config.app.logging.level, &config.app.logging.log_file);
    tracing::info!("Starting OpenGP (rat-salsa runtime)");

    let rt = tokio::runtime::Runtime::new()?;

    let (mut global, mut state) = rt.block_on(async {
        bootstrap(config).await
    })?;

    run_tui(
        init,
        render,
        event_fn,
        error_fn,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTasks::default())
            .poll(PollTokio::new(rt)),
    )?;

    tracing::info!("OpenGP shutdown complete");
    Ok(())
}

async fn bootstrap(config: Config) -> Result<(GlobalState, AppState), AppError> {
    let api_base_url = config.app.api_client.base_url.clone();
    let api_client = Arc::new(ApiClient::new(api_base_url));
    if let Ok(token) = std::env::var("API_SESSION_TOKEN") {
        api_client.set_session_token(Some(token)).await;
    }
    let has_session_token = api_client.current_session_token().await.is_some();

    let (mut theme, palette_config) = match config.app.ui.theme.as_str() {
        "light" => (Theme::light(), &config.theme.light),
        "high_contrast" => (Theme::high_contrast(), &config.theme.high_contrast),
        _ => (Theme::dark(), &config.theme.dark),
    };
    theme.colors = ColorPalette::from_config(palette_config);

    let (billing_service, clinical_ui_service, appointment_ui_service) = build_services(
        &config.app.api_server.database,
        &config.encryption_key,
    )
    .await?;

    let keybinds = KeybindRegistry::global();
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel::<AppCommand>();

    let mut state = AppState {
        tab_bar: TabBar::new(theme.clone()),
        previous_tab: Tab::Schedule,
        status_bar: StatusBar::schedule(theme.clone()),
        help_overlay: HelpOverlay::new(theme.clone()),
        login_screen: opengp_ui::ui::screens::LoginScreen::new(theme.clone()),
        authenticated: has_session_token,
        current_context: KeyContext::Global,
        should_quit: false,
        current_user_id: uuid::Uuid::nil(),
        terminal_size: Rect::new(0, 0, 80, 24),
        patient_list: PatientList::new(theme.clone()),
        patient_form: None,
        pending_patient_data: None,
        pending_edit_patient_id: None,
        appointment_state: AppointmentState::new(theme.clone(), config.app.calendar.clone()),
        appointment_form: None,
        appointment_detail_modal: None,
        pending_load_practitioners: false,
        pending_load_booked_slots: None,
        pending_appointment_save: None,
        pending_appointment_status_transition: None,
        pending_reschedule: None,
        workspace_manager: WorkspaceManager::new(theme.clone(), config.patient.max_open_patients),
        pending_clinical_save_data: None,
        patient_page_limit: 100,
        appointment_page_limit: 100,
        consultation_page_limit: 100,
        pending_patient_list_refresh: false,
        pending_appointment_list_refresh: None,
        pending_consultation_list_refresh: None,
        pending_practitioners_list_refresh: false,
        patient_list_fetch_task: None,
        appointment_list_fetch_task: None,
        practitioners_list_fetch_task: None,
        reschedule_task: None,
        status_update_task: None,
        login_task: None,
        clinical_workspace_load_task: None,
        pending_login_request: None,
        active_login_attempt: None,
        server_unavailable_error: None,
        server_unavailable_retry: None,
        active_appointment_refresh_date: None,
        context_menu_state: None,
        last_billing_render: None,
        hovered_clinical_menu: None,
        command_tx,
        command_rx: Some(command_rx),
    };

    if !has_session_token {
        state.authenticated = false;
    }

    let global = GlobalState {
        salsa_ctx: SalsaAppContext::default(),
        api_client: Some(api_client),
        billing_ui_service: billing_service,
        clinical_ui_service,
        appointment_ui_service,
        patient_ui_service: None,
        practice_config: load_practice_config()?,
        healthcare_config: config.healthcare,
        patient_config: config.patient,
        allergy_config: config.allergies,
        clinical_config: config.clinical,
        social_history_config: config.social_history,
        theme,
        keybinds,
    };

    Ok((global, state))
}

async fn build_services(
    database_config: &DatabaseConfig,
    encryption_key: &str,
) -> Result<
    (
        Option<Arc<BillingUiService>>,
        Option<Arc<ClinicalUiService>>,
        Option<Arc<AppointmentUiService>>,
    ),
    AppError,
> {
    let db_url = database_config.url.clone();
    let database_pool = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        create_pool(database_config),
    )
    .await
    .map_err(|_| eyre!("Database connection timed out after 5 seconds — is PostgreSQL running?\n  URL: {}", db_url))
    .and_then(|r| {
        r.map_err(|e| eyre!("Failed to connect to database — is PostgreSQL running?\n  URL: {}\n  Cause: {}", db_url, e))
    })?;

    let pool = database_pool.as_postgres().clone();
    let encryption_service = Arc::new(
        EncryptionService::new_with_key(encryption_key)
            .map_err(|err| eyre!(err.to_string()))?,
    );

    let billing_repo: Arc<dyn BillingRepository> = Arc::new(SqlxBillingRepository::new(pool.clone()));
    let consultation_repo: Arc<dyn ConsultationRepository> = Arc::new(
        SqlxClinicalRepository::new(pool.clone(), Arc::clone(&encryption_service)),
    );

    let billing_domain_service = BillingService::new(Arc::clone(&billing_repo), Arc::clone(&consultation_repo));
    let billing_service = Some(Arc::new(BillingUiService::new(Arc::new(billing_domain_service))));

    let clinical_repos = ClinicalRepositories {
        consultation: Arc::clone(&consultation_repo),
        allergy: Arc::new(SqlxAllergyRepository::new(pool.clone(), Arc::clone(&encryption_service))),
        medical_history: Arc::new(SqlxMedicalHistoryRepository::new(pool.clone(), Arc::clone(&encryption_service))),
        vital_signs: Arc::new(SqlxVitalSignsRepository::new(pool.clone(), Arc::clone(&encryption_service))),
        social_history: Arc::new(SqlxSocialHistoryRepository::new(pool.clone(), Arc::clone(&encryption_service))),
        family_history: Arc::new(SqlxFamilyHistoryRepository::new(pool.clone(), Arc::clone(&encryption_service))),
    };

    let patient_repo: Arc<dyn PatientRepository> = Arc::new(SqlxPatientRepository::new(
        pool.clone(),
        Arc::clone(&encryption_service),
    ));
    let patient_service = Arc::new(opengp_domain::domain::patient::PatientService::new(patient_repo));

    let audit_repo: Arc<dyn AuditRepository> = Arc::new(SqlxAuditRepository::new(pool.clone()));
    let audit_service: Arc<dyn AuditEmitter> = Arc::new(AuditService::new(audit_repo));

    let clinical_domain_service = Arc::new(ClinicalService::new(
        clinical_repos,
        patient_service,
        Arc::clone(&audit_service),
    ));
    let clinical_ui_service = Some(Arc::new(ClinicalUiService::new(clinical_domain_service)));

    let practitioner_repo: Arc<dyn opengp_domain::domain::user::PractitionerRepository> =
        Arc::new(SqlxPractitionerRepository::new(pool.clone()));
    let appointment_repo: Arc<dyn AppointmentRepository> = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let appointment_calendar_query: Arc<dyn AppointmentCalendarQuery> = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let working_hours_repo: Arc<dyn WorkingHoursRepository> = Arc::new(SqlxWorkingHoursRepository::new(pool.clone()));

    let appointment_domain_service = Arc::new(AppointmentService::new(
        Arc::clone(&appointment_repo),
        Arc::clone(&audit_service),
        Arc::clone(&appointment_calendar_query),
    ));
    let availability_service = Arc::new(AvailabilityService::new(
        Arc::clone(&appointment_repo),
        Arc::clone(&working_hours_repo),
    ));

    let appointment_ui_service = Some(Arc::new(AppointmentUiService::new(
        Arc::clone(&practitioner_repo),
        appointment_calendar_query,
        appointment_repo,
        appointment_domain_service,
        availability_service,
        working_hours_repo,
    )));

    Ok((billing_service, clinical_ui_service, appointment_ui_service))
}

fn init(state: &mut AppState, _ctx: &mut GlobalState) -> Result<(), AppError> {
    if state.authenticated {
        state.pending_patient_list_refresh = true;
    }
    Ok(())
}

fn render(area: Rect, buf: &mut Buffer, state: &mut AppState, _ctx: &mut GlobalState) -> Result<(), AppError> {
    if state.help_overlay.is_visible() {
        state.help_overlay.clone().render(area, buf);
        return Ok(());
    }

    if !state.authenticated {
        state.login_screen.clone().render(area, buf);
        return Ok(());
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    state.tab_bar.clone().render(layout[0], buf);

    match state.tab_bar.selected() {
        Tab::Schedule => Paragraph::new("Schedule").render(layout[1], buf),
        Tab::PatientSearch => state.patient_list.clone().render(layout[1], buf),
        Tab::PatientWorkspace => Paragraph::new("Patient workspace").render(layout[1], buf),
    }

    state.status_bar.clone().render(layout[2], buf);
    Ok(())
}

fn event_fn(event: &AppEvent, state: &mut AppState, _ctx: &mut GlobalState) -> Result<Control<AppEvent>, AppError> {
    use crossterm::event::{Event, KeyCode, KeyModifiers};

    match event {
        AppEvent::Term(term_event) => match term_event {
            Event::Key(key) => {
                if !state.authenticated {
                    if let Some(opengp_ui::ui::screens::LoginAction::Submit { username, password }) =
                        state.login_screen.handle_key(*key)
                    {
                        state.pending_login_request = Some((username, password));
                        return Ok(Control::Changed);
                    }
                }

                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
                    state.should_quit = true;
                    return Ok(Control::Quit);
                }

                match key.code {
                    KeyCode::F(1) => {
                        state.help_overlay.toggle();
                        Ok(Control::Changed)
                    }
                    KeyCode::F(2) => {
                        state.tab_bar.select(Tab::Schedule);
                        Ok(Control::Changed)
                    }
                    KeyCode::F(3) => {
                        state.tab_bar.select(Tab::PatientSearch);
                        Ok(Control::Changed)
                    }
                    _ => {
                        let action = KeybindRegistry::global()
                            .lookup(*key, KeyContext::Global)
                            .map(|k| k.action.clone());
                        if matches!(action, Some(Action::Quit)) {
                            state.should_quit = true;
                            Ok(Control::Quit)
                        } else {
                            Ok(Control::Continue)
                        }
                    }
                }
            }
            Event::Resize(w, h) => {
                state.terminal_size = Rect::new(0, 0, *w, *h);
                Ok(Control::Changed)
            }
            _ => Ok(Control::Continue),
        },
        AppEvent::LoginResult(result) => {
            match result {
                Ok(_) => {
                    state.authenticated = true;
                    state.pending_patient_list_refresh = true;
                    state.status_bar.clear_error();
                }
                Err(err) => {
                    state.authenticated = false;
                    state.status_bar.set_error(Some(err.clone()));
                }
            }
            Ok(Control::Changed)
        }
        AppEvent::PatientListLoaded(result) => {
            match result {
                Ok(patients) => state.patient_list.set_patients(patients.clone()),
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        _ => Ok(Control::Continue),
    }
}

fn error_fn(err: AppError, state: &mut AppState, _ctx: &mut GlobalState) -> Result<Control<AppEvent>, AppError> {
    state.status_bar.set_error(Some(err.to_string()));
    Ok(Control::Changed)
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
