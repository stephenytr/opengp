use std::sync::Arc;

use color_eyre::eyre::eyre;
use opengp_config::{load_practice_config, Config};
use opengp_domain::domain::appointment::{
    AppointmentCalendarQuery, AppointmentRepository, AppointmentService, AvailabilityService,
};
use opengp_domain::domain::audit::{AuditEmitter, AuditRepository, AuditService};
use opengp_domain::domain::billing::{BillingRepository, BillingService};
use opengp_domain::domain::clinical::{
    ClinicalRepositories, ClinicalService, ConsultationRepository,
};
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
use opengp_ui::ui::app::{
    AppCommand, AppError, AppEvent, AppState, DialogContent, GlobalState, RetryOperation,
};
use opengp_ui::ui::components::appointment::{
    AppointmentDetailModalAction, AppointmentForm, AppointmentFormAction, AppointmentState,
    AppointmentView, CalendarAction, ScheduleAction,
};
use opengp_ui::ui::components::help::HelpOverlay;
use opengp_ui::ui::components::patient::{
    PatientForm, PatientFormAction, PatientList, PatientListAction,
};
use opengp_ui::ui::components::status_bar::StatusBar;
use opengp_ui::ui::components::tabs::{Tab, TabBar};
use opengp_ui::ui::components::workspace::WorkspaceManager;
use opengp_ui::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use opengp_ui::ui::services::{
    AppointmentUiService, BillingUiService, ClinicalUiService, PatientUiService,
};
use opengp_ui::ui::theme::{ColorPalette, Theme};
use opengp_ui::ui::widgets::FormNavigation;
use rat_event::{HandleEvent, Outcome, Regular};
use rat_focus::{Focus, FocusBuilder, HasFocus};
use rat_salsa::poll::{PollCrossterm, PollTasks, PollTokio};
use rat_salsa::{run_tui, Control, RunConfig, SalsaAppContext, SalsaContext};
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

    let (mut global, mut state) = rt.block_on(async { bootstrap(config).await })?;

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

    let (billing_service, clinical_ui_service, appointment_ui_service, patient_ui_service) =
        build_services(&config.app.api_server.database, &config.encryption_key).await?;

    let keybinds = KeybindRegistry::global();
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel::<AppCommand>();

    let mut state = AppState {
        tab_bar: TabBar::new(theme.clone()),
        previous_tab: Tab::Schedule,
        status_bar: StatusBar::schedule(theme.clone()),
        login_screen: opengp_ui::ui::screens::LoginScreen::new(theme.clone()),
        authenticated: has_session_token,
        current_context: KeyContext::Global,
        should_quit: false,
        current_user_id: uuid::Uuid::nil(),
        terminal_size: Rect::new(0, 0, 80, 24),
        patient_list: PatientList::new(theme.clone()),
        appointment_state: AppointmentState::new(theme.clone(), config.app.calendar.clone()),
        workspace_manager: WorkspaceManager::new(theme.clone(), config.patient.max_open_patients),
        patient_page_limit: 100,
        appointment_page_limit: 100,
        consultation_page_limit: 100,
        active_login_attempt: None,
        active_appointment_refresh_date: None,
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
        dialogs: Default::default(),
        api_client: Some(api_client),
        billing_ui_service: billing_service,
        clinical_ui_service,
        appointment_ui_service,
        patient_ui_service,
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
        Option<Arc<PatientUiService>>,
    ),
    AppError,
> {
    let db_url = database_config.url.clone();
    let database_pool = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        create_pool(database_config),
    )
    .await
    .map_err(|_| {
        eyre!(
            "Database connection timed out after 5 seconds — is PostgreSQL running?\n  URL: {}",
            db_url
        )
    })
    .and_then(|r| {
        r.map_err(|e| {
            eyre!(
                "Failed to connect to database — is PostgreSQL running?\n  URL: {}\n  Cause: {}",
                db_url,
                e
            )
        })
    })?;

    let pool = database_pool.as_postgres().clone();
    let encryption_service = Arc::new(
        EncryptionService::new_with_key(encryption_key).map_err(|err| eyre!(err.to_string()))?,
    );

    let billing_repo: Arc<dyn BillingRepository> =
        Arc::new(SqlxBillingRepository::new(pool.clone()));
    let consultation_repo: Arc<dyn ConsultationRepository> = Arc::new(SqlxClinicalRepository::new(
        pool.clone(),
        Arc::clone(&encryption_service),
    ));

    let billing_domain_service =
        BillingService::new(Arc::clone(&billing_repo), Arc::clone(&consultation_repo));
    let billing_service = Some(Arc::new(BillingUiService::new(Arc::new(
        billing_domain_service,
    ))));

    let clinical_repos = ClinicalRepositories {
        consultation: Arc::clone(&consultation_repo),
        allergy: Arc::new(SqlxAllergyRepository::new(
            pool.clone(),
            Arc::clone(&encryption_service),
        )),
        medical_history: Arc::new(SqlxMedicalHistoryRepository::new(
            pool.clone(),
            Arc::clone(&encryption_service),
        )),
        vital_signs: Arc::new(SqlxVitalSignsRepository::new(
            pool.clone(),
            Arc::clone(&encryption_service),
        )),
        social_history: Arc::new(SqlxSocialHistoryRepository::new(
            pool.clone(),
            Arc::clone(&encryption_service),
        )),
        family_history: Arc::new(SqlxFamilyHistoryRepository::new(
            pool.clone(),
            Arc::clone(&encryption_service),
        )),
    };

    let patient_repo: Arc<dyn PatientRepository> = Arc::new(SqlxPatientRepository::new(
        pool.clone(),
        Arc::clone(&encryption_service),
    ));
    let patient_service = Arc::new(opengp_domain::domain::patient::PatientService::new(
        patient_repo,
    ));
    let patient_ui_service = Some(Arc::new(PatientUiService::new(Arc::clone(
        &patient_service,
    ))));

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
    let appointment_repo: Arc<dyn AppointmentRepository> =
        Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let appointment_calendar_query: Arc<dyn AppointmentCalendarQuery> =
        Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let working_hours_repo: Arc<dyn WorkingHoursRepository> =
        Arc::new(SqlxWorkingHoursRepository::new(pool.clone()));

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

    Ok((
        billing_service,
        clinical_ui_service,
        appointment_ui_service,
        patient_ui_service,
    ))
}

fn init(state: &mut AppState, _ctx: &mut GlobalState) -> Result<(), AppError> {
    if state.authenticated {
        spawn_patient_list_refresh(_ctx);
    }
    Ok(())
}

fn spawn_login_request(ctx: &GlobalState, username: String, password: String) {
    let client = ctx.api_client.clone();
    ctx.spawn_async(async move {
        let result = if let Some(c) = client {
            c.login(username, password).await.map_err(|e| e.to_string())
        } else {
            Err("No API client".to_string())
        };
        Ok(Control::Event(AppEvent::LoginResult(result)))
    });
}

fn spawn_patient_list_refresh(ctx: &GlobalState) {
    let service = ctx.patient_ui_service.clone();
    ctx.spawn_async(async move {
        let result = if let Some(svc) = service {
            svc.list_patients_as_view_items()
                .await
                .map_err(|e| e.to_string())
        } else {
            Err("No patient service".to_string())
        };
        Ok(Control::Event(AppEvent::PatientListLoaded(result)))
    });
}

fn spawn_patient_save(ctx: &GlobalState, pending: opengp_ui::ui::app::PendingPatientData) {
    let service = ctx.patient_ui_service.clone();
    ctx.spawn_async(async move {
        let result = if let Some(svc) = service {
            let save_result = match pending {
                opengp_ui::ui::app::PendingPatientData::New(data) => {
                    svc.create_patient(data).await.map(|_| ())
                }
                opengp_ui::ui::app::PendingPatientData::Update { id, data } => {
                    svc.update_patient(id, data).await.map(|_| ())
                }
            };

            match save_result {
                Ok(()) => svc
                    .list_patients_as_view_items()
                    .await
                    .map_err(|e| e.to_string()),
                Err(err) => Err(err.to_string()),
            }
        } else {
            Err("No patient service".to_string())
        };
        Ok(Control::Event(AppEvent::PatientListLoaded(result)))
    });
}

fn spawn_load_patient_for_edit(ctx: &GlobalState, patient_id: uuid::Uuid) {
    let service = ctx.patient_ui_service.clone();
    ctx.spawn_async(async move {
        let result = if let Some(svc) = service {
            svc.get_patient(patient_id).await.map_err(|e| e.to_string())
        } else {
            Err("No patient service".to_string())
        };
        Ok(Control::Event(AppEvent::PatientEditLoaded(result)))
    });
}

fn spawn_refresh_appointments(ctx: &GlobalState, date: chrono::NaiveDate) {
    let service = ctx.appointment_ui_service.clone();
    ctx.spawn_async(async move {
        let result = if let Some(svc) = service {
            svc.get_schedule(date).await.map_err(|e| e.to_string())
        } else {
            Err("No appointment service".to_string())
        };
        Ok(Control::Event(AppEvent::AppointmentsRefreshed(result)))
    });
}

fn spawn_load_practitioners(ctx: &GlobalState) {
    let service = ctx.appointment_ui_service.clone();
    ctx.spawn_async(async move {
        let result = if let Some(svc) = service {
            svc.get_practitioners()
                .await
                .map(|items| items.into_iter().map(|p| p.into()).collect())
                .map_err(|e| e.to_string())
        } else {
            Err("No appointment service".to_string())
        };
        Ok(Control::Event(AppEvent::PractitionersLoaded(result)))
    });
}

fn spawn_load_booked_slots(
    ctx: &GlobalState,
    practitioner_id: uuid::Uuid,
    date: chrono::NaiveDate,
    duration: u32,
) {
    let service = ctx.appointment_ui_service.clone();
    ctx.spawn_async(async move {
        let result = if let Some(svc) = service {
            svc.get_available_slots(practitioner_id, date, duration)
                .await
                .map_err(|e| e.to_string())
        } else {
            Err("No appointment service".to_string())
        };
        Ok(Control::Event(AppEvent::AvailableSlotsLoaded(result)))
    });
}

fn spawn_save_appointment(
    ctx: &GlobalState,
    data: opengp_domain::domain::appointment::NewAppointmentData,
    user_id: uuid::Uuid,
) {
    let service = ctx.appointment_ui_service.clone();
    ctx.spawn_async(async move {
        let result = if let Some(svc) = service {
            svc.create_appointment(data, user_id)
                .await
                .map_err(|e| e.to_string())
        } else {
            Err("No appointment service".to_string())
        };
        Ok(Control::Event(AppEvent::AppointmentSaved(result)))
    });
}

fn spawn_update_appointment_status(
    ctx: &GlobalState,
    appointment_id: uuid::Uuid,
    transition: opengp_ui::ui::app::AppointmentStatusTransition,
    user_id: uuid::Uuid,
    refresh_date: chrono::NaiveDate,
) {
    let service = ctx.appointment_ui_service.clone();
    ctx.spawn_async(async move {
        let result = if let Some(svc) = service {
            match transition {
                opengp_ui::ui::app::AppointmentStatusTransition::SetStatus(status) => {
                    let status_result = match status {
                        opengp_domain::domain::appointment::AppointmentStatus::Arrived => {
                            svc.mark_arrived(appointment_id, user_id).await
                        }
                        opengp_domain::domain::appointment::AppointmentStatus::InProgress => {
                            svc.mark_in_progress(appointment_id, user_id).await
                        }
                        opengp_domain::domain::appointment::AppointmentStatus::Completed => {
                            svc.mark_completed(appointment_id, user_id).await
                        }
                        opengp_domain::domain::appointment::AppointmentStatus::NoShow => {
                            svc.mark_no_show(appointment_id, user_id).await
                        }
                        opengp_domain::domain::appointment::AppointmentStatus::Billing => {
                            svc.mark_billing(appointment_id, user_id).await
                        }
                        opengp_domain::domain::appointment::AppointmentStatus::Scheduled
                        | opengp_domain::domain::appointment::AppointmentStatus::Confirmed
                        | opengp_domain::domain::appointment::AppointmentStatus::Rescheduled
                        | opengp_domain::domain::appointment::AppointmentStatus::Cancelled => {
                            Ok(())
                        }
                    };
                    status_result
                        .map(|_| (appointment_id, refresh_date))
                        .map_err(|e| e.to_string())
                }
            }
        } else {
            Err("No appointment service".to_string())
        };
        Ok(Control::Event(AppEvent::AppointmentStatusUpdated(result)))
    });
}

fn spawn_reschedule_appointment(
    ctx: &GlobalState,
    data: opengp_ui::ui::app::PendingRescheduleData,
    user_id: uuid::Uuid,
) {
    let service = ctx.appointment_ui_service.clone();
    ctx.spawn_async(async move {
        let result = if let Some(svc) = service {
            match (data.new_date, data.new_time) {
                (Some(new_date), Some(new_time)) => {
                    let new_start = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                        new_date.and_time(new_time),
                        chrono::Utc,
                    );
                    svc.reschedule_appointment(
                        data.appointment_id,
                        new_start,
                        data.duration_minutes,
                        user_id,
                    )
                    .await
                    .map(|_| (data.appointment_id, new_date))
                    .map_err(|e| e.to_string())
                }
                _ => Err("Missing date/time for reschedule".to_string()),
            }
        } else {
            Err("No appointment service".to_string())
        };
        Ok(Control::Event(AppEvent::AppointmentRescheduled(result)))
    });
}

fn spawn_refresh_consultations(ctx: &GlobalState, patient_id: uuid::Uuid) {
    let service = ctx.clinical_ui_service.clone();
    ctx.spawn_async(async move {
        let result = if let Some(svc) = service {
            svc.list_consultations(patient_id)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        } else {
            Err("No clinical service".to_string())
        };
        Ok(Control::Event(AppEvent::ConsultationsRefreshed(result)))
    });
}

fn dialog_render(
    area: Rect,
    buf: &mut Buffer,
    dialog: &mut dyn std::any::Any,
    _state: &mut AppState,
) {
    let Some(dialog) = dialog.downcast_mut::<DialogContent>() else {
        return;
    };

    match dialog {
        DialogContent::HelpOverlay(help) => help.clone().render(area, buf),
        DialogContent::PatientForm(form) => form.clone().render(area, buf),
        DialogContent::AppointmentForm(form) => form.clone().render(area, buf),
        DialogContent::AppointmentDetailModal(modal) => modal.clone().render(area, buf),
        DialogContent::ContextMenu(_) => {}
        DialogContent::ServerUnavailable { .. } => {}
    }
}

fn push_dialog(ctx: &GlobalState, dialog: DialogContent) {
    ctx.dialogs.push(
        dialog_render,
        |_event, _dialog, _state| Ok(Outcome::Continue.into()),
        dialog,
    );
}

fn dialog_index(ctx: &GlobalState, predicate: impl Fn(&DialogContent) -> bool) -> Option<usize> {
    for index in (0..ctx.dialogs.len()).rev() {
        let Some(dialog) = ctx.dialogs.get::<DialogContent>(index) else {
            continue;
        };
        if predicate(&dialog) {
            return Some(index);
        }
    }
    None
}

fn close_dialog(ctx: &GlobalState, predicate: impl Fn(&DialogContent) -> bool) -> bool {
    if let Some(index) = dialog_index(ctx, predicate) {
        ctx.dialogs.remove(index);
        return true;
    }
    false
}

fn toggle_help_dialog(state: &mut AppState, ctx: &GlobalState) {
    if close_dialog(ctx, |dialog| {
        matches!(dialog, DialogContent::HelpOverlay(_))
    }) {
        return;
    }

    let mut help_overlay = HelpOverlay::new(ctx.theme.clone());
    help_overlay.set_context(state.current_context);
    help_overlay.show();
    push_dialog(ctx, DialogContent::HelpOverlay(help_overlay));
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut AppState,
    ctx: &mut GlobalState,
) -> Result<(), AppError> {
    if !state.authenticated {
        state.login_screen.clone().render(area, buf);
    } else {
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
    }

    if let Some(index) = ctx.dialogs.top::<DialogContent>() {
        if let Some(mut dialog) = ctx.dialogs.get_mut::<DialogContent>(index) {
            match &mut *dialog {
                DialogContent::HelpOverlay(help) => help.clone().render(area, buf),
                DialogContent::PatientForm(form) => form.clone().render(area, buf),
                DialogContent::AppointmentForm(form) => form.clone().render(area, buf),
                DialogContent::AppointmentDetailModal(modal) => modal.clone().render(area, buf),
                DialogContent::ContextMenu(menu) => menu.clone().render(area, buf),
                DialogContent::ServerUnavailable { error, retry: _ } => {
                    use ratatui::widgets::Clear;
                    let error_area = centered_rect(60, 20, area);
                    Clear.render(error_area, buf);
                    let border = ratatui::widgets::Block::default()
                        .title("Server Unavailable")
                        .borders(ratatui::widgets::Borders::ALL)
                        .style(ratatui::style::Style::default().red());
                    border.clone().render(error_area, buf);

                    let inner = border.inner(error_area);
                    Paragraph::new(format!(
                        "{}\n\n(Press 'r' to retry or Esc to dismiss)",
                        error
                    ))
                    .render(inner, buf);
                }
            }
        }
    }

    Ok(())
}

fn build_focus(state: &AppState) -> Focus {
    let mut builder = FocusBuilder::new(None);
    builder
        .widget(&state.tab_bar)
        .widget(&state.login_screen)
        .widget(&state.patient_list)
        .widget(&state.appointment_state);

    builder.build()
}

fn refresh_status_and_context(state: &mut AppState, ctx: &GlobalState) {
    state.status_bar = match state.tab_bar.selected() {
        Tab::PatientSearch | Tab::PatientWorkspace => {
            if let Some(workspace) = state.workspace_manager.active() {
                StatusBar::patient_workspace(
                    ctx.theme.clone(),
                    &workspace.patient_snapshot.full_name,
                )
            } else {
                StatusBar::patient_list(ctx.theme.clone())
            }
        }
        Tab::Schedule => StatusBar::schedule(ctx.theme.clone()),
    };

    state.current_context = if !state.authenticated {
        KeyContext::Global
    } else if state.tab_bar.is_focused() {
        KeyContext::TabBar
    } else {
        let mut dialog_context = None;
        if let Some(index) = ctx.dialogs.top::<DialogContent>() {
            if let Some(dialog) = ctx.dialogs.get::<DialogContent>(index) {
                dialog_context = match &*dialog {
                    DialogContent::HelpOverlay(_) => Some(KeyContext::Global),
                    DialogContent::PatientForm(form) if form.is_focused() => {
                        Some(KeyContext::PatientForm)
                    }
                    DialogContent::AppointmentForm(form) if form.is_focused() => {
                        Some(KeyContext::Schedule)
                    }
                    DialogContent::AppointmentDetailModal(modal) if modal.is_focused() => {
                        Some(KeyContext::Schedule)
                    }
                    _ => None,
                };
            }
        }

        if let Some(context) = dialog_context {
            context
        } else {
            match state.tab_bar.selected() {
                Tab::PatientSearch | Tab::PatientWorkspace => {
                    if state.patient_list.is_searching() {
                        KeyContext::Search
                    } else if state.workspace_manager.active().is_some() {
                        KeyContext::PatientWorkspace
                    } else {
                        KeyContext::PatientList
                    }
                }
                Tab::Schedule => match state.appointment_state.current_view {
                    AppointmentView::Calendar => KeyContext::Calendar,
                    AppointmentView::Schedule => KeyContext::Schedule,
                },
            }
        }
    };
}

enum DialogKeyResult {
    NotHandled,
    HandledKeep,
    HandledClose,
}

fn handle_patient_form_key(
    _state: &mut AppState,
    ctx: &GlobalState,
    dialog: &mut DialogContent,
    key: crossterm::event::KeyEvent,
) -> DialogKeyResult {
    let mut consumed = false;
    let mut close_form = false;

    if let DialogContent::PatientForm(form) = dialog {
        if let Some(action) = form.handle_key(key) {
            consumed = true;
            match action {
                PatientFormAction::FocusChanged | PatientFormAction::ValueChanged => {}
                PatientFormAction::Submit => {
                    if !form.validate() || form.has_errors() {
                        form.focus_first_error();
                    } else if form.is_edit_mode() {
                        if let Some((id, data)) = form.to_update_patient_data() {
                            spawn_patient_save(
                                ctx,
                                opengp_ui::ui::app::PendingPatientData::Update { id, data },
                            );
                            close_form = true;
                        } else {
                            form.focus_first_error();
                        }
                    } else if let Some(data) = form.to_new_patient_data() {
                        spawn_patient_save(ctx, opengp_ui::ui::app::PendingPatientData::New(data));
                        close_form = true;
                    } else {
                        form.focus_first_error();
                    }
                }
                PatientFormAction::Cancel => {
                    close_form = true;
                }
                PatientFormAction::SaveComplete => {
                    spawn_patient_list_refresh(ctx);
                }
            }
        }
    }

    if close_form {
        DialogKeyResult::HandledClose
    } else if consumed {
        DialogKeyResult::HandledKeep
    } else {
        DialogKeyResult::NotHandled
    }
}

fn handle_appointment_form_key(
    state: &mut AppState,
    ctx: &GlobalState,
    dialog: &mut DialogContent,
    key: crossterm::event::KeyEvent,
) -> DialogKeyResult {
    if let DialogContent::AppointmentForm(form) = dialog {
        if let Some(action) = form.handle_key(key) {
            match action {
                AppointmentFormAction::FocusChanged | AppointmentFormAction::ValueChanged => {}
                AppointmentFormAction::Submit => {
                    if let Some(data) = form.to_new_appointment_data() {
                        form.set_saving(true);
                        spawn_save_appointment(ctx, data, state.current_user_id);
                    } else {
                        let msg = form
                            .first_error()
                            .unwrap_or_else(|| "Check required fields".to_string());
                        state.status_bar.set_error(Some(msg));
                    }
                }
                AppointmentFormAction::Cancel => {
                    state.status_bar.clear_error();
                    return DialogKeyResult::HandledClose;
                }
                AppointmentFormAction::SaveComplete => {
                    state.status_bar.clear_error();
                    spawn_refresh_appointments(ctx, chrono::Utc::now().date_naive());
                    return DialogKeyResult::HandledClose;
                }
                AppointmentFormAction::OpenTimePicker {
                    practitioner_id,
                    date,
                    duration,
                } => {
                    let practitioner_id_i64 = practitioner_id.as_u128() as i64;
                    form.open_time_picker(practitioner_id_i64, date, duration);
                    spawn_load_booked_slots(ctx, practitioner_id, date, duration);
                }
            }
            return DialogKeyResult::HandledKeep;
        }
    }
    DialogKeyResult::NotHandled
}

fn handle_detail_modal_key(
    state: &mut AppState,
    ctx: &GlobalState,
    dialog: &mut DialogContent,
    key: crossterm::event::KeyEvent,
) -> DialogKeyResult {
    if let DialogContent::AppointmentDetailModal(modal) = dialog {
        if let Some(action) = modal.handle_key(key) {
            match action {
                AppointmentDetailModalAction::Close => {
                    return DialogKeyResult::HandledClose;
                }
                AppointmentDetailModalAction::MarkStatus(status) => {
                    let appointment_id = modal.appointment_id();
                    let refresh_date = modal.appointment().start_time.date_naive();
                    spawn_update_appointment_status(
                        ctx,
                        appointment_id,
                        opengp_ui::ui::app::AppointmentStatusTransition::SetStatus(status),
                        state.current_user_id,
                        refresh_date,
                    );
                    return DialogKeyResult::HandledClose;
                }
                AppointmentDetailModalAction::OpenTimePicker {
                    practitioner_id,
                    date,
                    duration,
                } => {
                    spawn_load_booked_slots(ctx, practitioner_id, date, duration);
                }
                AppointmentDetailModalAction::RescheduleTime => {
                    if let (Some(new_date), Some(new_time)) = (
                        modal.pending_reschedule_date(),
                        modal.pending_reschedule_time(),
                    ) {
                        spawn_reschedule_appointment(
                            ctx,
                            opengp_ui::ui::app::PendingRescheduleData {
                                appointment_id: modal.appointment_id(),
                                new_date: Some(new_date),
                                new_time: Some(new_time),
                                practitioner_id: modal.appointment().practitioner_id,
                                duration_minutes: modal.appointment().duration_minutes() as i64,
                            },
                            state.current_user_id,
                        );
                        return DialogKeyResult::HandledClose;
                    }
                }
                AppointmentDetailModalAction::ViewClinicalNotes
                | AppointmentDetailModalAction::StartConsultation
                | AppointmentDetailModalAction::RescheduleDate => {}
            }
            return DialogKeyResult::HandledKeep;
        }
    }
    DialogKeyResult::NotHandled
}

fn event_fn(
    event: &AppEvent,
    state: &mut AppState,
    ctx: &mut GlobalState,
) -> Result<Control<AppEvent>, AppError> {
    use crossterm::event::{Event, KeyCode, KeyModifiers};

    match event {
        AppEvent::Term(term_event) => match term_event {
            Event::Key(key) => {
                let mut focus = build_focus(state);

                if focus.focused().is_none() {
                    if !state.authenticated {
                        focus.focus(&state.login_screen);
                    } else {
                        match state.tab_bar.selected() {
                            Tab::Schedule => focus.focus(&state.appointment_state),
                            Tab::PatientSearch | Tab::PatientWorkspace => {
                                focus.focus(&state.patient_list)
                            }
                        }
                    }
                }

                if matches!(focus.handle(term_event, Regular), Outcome::Changed) {
                    refresh_status_and_context(state, ctx);
                    return Ok(Control::Changed);
                }

                refresh_status_and_context(state, ctx);

                if let Some(index) = ctx.dialogs.top::<DialogContent>() {
                    let dialog_result = if let Some(mut dialog) =
                        ctx.dialogs.get_mut::<DialogContent>(index)
                    {
                        match &mut *dialog {
                            DialogContent::HelpOverlay(_) => {
                                if key.code == KeyCode::Esc || key.code == KeyCode::F(1) {
                                    DialogKeyResult::HandledClose
                                } else {
                                    DialogKeyResult::HandledKeep
                                }
                            }
                            DialogContent::PatientForm(_) => {
                                handle_patient_form_key(state, ctx, &mut dialog, *key)
                            }
                            DialogContent::AppointmentForm(_) => {
                                handle_appointment_form_key(state, ctx, &mut dialog, *key)
                            }
                            DialogContent::AppointmentDetailModal(_) => {
                                handle_detail_modal_key(state, ctx, &mut dialog, *key)
                            }
                            DialogContent::ContextMenu(_) => DialogKeyResult::NotHandled,
                            DialogContent::ServerUnavailable { .. } => DialogKeyResult::NotHandled,
                        }
                    } else {
                        DialogKeyResult::NotHandled
                    };

                    match dialog_result {
                        DialogKeyResult::HandledClose => {
                            _ = ctx.dialogs.pop();
                            refresh_status_and_context(state, ctx);
                            return Ok(Control::Changed);
                        }
                        DialogKeyResult::HandledKeep | DialogKeyResult::NotHandled => {
                            refresh_status_and_context(state, ctx);
                            return Ok(Control::Continue);
                        }
                    }
                }

                if let Some(index) = ctx.dialogs.top::<DialogContent>() {
                    if let Some(dialog) = ctx.dialogs.get::<DialogContent>(index) {
                        match &*dialog {
                            DialogContent::ServerUnavailable { error: _, retry } => {
                                match key.code {
                                    KeyCode::Char('r') | KeyCode::Char('R') => {
                                        if let Some(operation) = retry.clone() {
                                            match operation {
                                                RetryOperation::Login { username, password } => {
                                                    spawn_login_request(ctx, username, password);
                                                }
                                                RetryOperation::RefreshPatients => {
                                                    spawn_patient_list_refresh(ctx);
                                                }
                                                RetryOperation::RefreshAppointments { date } => {
                                                    spawn_refresh_appointments(ctx, date);
                                                }
                                                RetryOperation::RefreshConsultations {
                                                    patient_id,
                                                } => {
                                                    spawn_refresh_consultations(ctx, patient_id);
                                                }
                                            }
                                            _ = ctx.dialogs.pop();
                                        }
                                        return Ok(Control::Changed);
                                    }
                                    KeyCode::Esc => {
                                        _ = ctx.dialogs.pop();
                                        return Ok(Control::Changed);
                                    }
                                    _ => {}
                                }
                            }
                            DialogContent::ContextMenu(menu) => {
                                if menu.is_visible() {
                                    if let Some(mut menu_mut) =
                                        ctx.dialogs.get_mut::<DialogContent>(index)
                                    {
                                        if let DialogContent::ContextMenu(ref mut context_menu) =
                                            &mut *menu_mut
                                        {
                                            if let Some(action) = context_menu.handle_key(*key) {
                                                match action {
                                                    opengp_ui::ui::widgets::ContextMenuAction::Selected(app_action) => {
                                                        _ = ctx.dialogs.pop();
                                                        match app_action {
                                                            opengp_ui::ui::app::AppContextMenuAction::PatientEdit(id) => {
                                                                spawn_load_patient_for_edit(ctx, id);
                                                            }
                                                            opengp_ui::ui::app::AppContextMenuAction::PatientDelete(_)
                                                            | opengp_ui::ui::app::AppContextMenuAction::PatientViewHistory(_)
                                                            | opengp_ui::ui::app::AppContextMenuAction::AppointmentEdit(_)
                                                            | opengp_ui::ui::app::AppContextMenuAction::AppointmentCancel(_)
                                                            | opengp_ui::ui::app::AppContextMenuAction::AppointmentReschedule(_)
                                                            | opengp_ui::ui::app::AppContextMenuAction::ClinicalEdit(_)
                                                            | opengp_ui::ui::app::AppContextMenuAction::ClinicalDelete(_)
                                                            | opengp_ui::ui::app::AppContextMenuAction::BillingEdit(_)
                                                            | opengp_ui::ui::app::AppContextMenuAction::BillingViewInvoice(_) => {}
                                                        }
                                                    }
                                                    opengp_ui::ui::widgets::ContextMenuAction::Dismissed => {
                                                        _ = ctx.dialogs.pop();
                                                    }
                                                    opengp_ui::ui::widgets::ContextMenuAction::FocusChanged => {}
                                                }
                                                return Ok(Control::Changed);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if !state.authenticated {
                    if let Some(opengp_ui::ui::screens::LoginAction::Submit {
                        username,
                        password,
                    }) = state.login_screen.handle_key(*key)
                    {
                        spawn_login_request(ctx, username, password);
                        return Ok(Control::Changed);
                    }

                    if let Some(action) = ctx
                        .keybinds
                        .lookup(*key, KeyContext::Global)
                        .map(|kb| kb.action.clone())
                    {
                        match action {
                            Action::OpenHelp => {
                                toggle_help_dialog(state, ctx);
                                return Ok(Control::Changed);
                            }
                            Action::Quit => {
                                state.should_quit = true;
                                return Ok(Control::Quit);
                            }
                            _ => {}
                        }
                    }

                    return Ok(Control::Continue);
                }

                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
                    state.should_quit = true;
                    return Ok(Control::Quit);
                }

                if state.patient_list.is_focused() {
                    if let Some(action) = state.patient_list.handle_key(*key) {
                        match action {
                            PatientListAction::SelectionChanged
                            | PatientListAction::FocusSearch
                            | PatientListAction::SearchChanged => {
                                return Ok(Control::Changed);
                            }
                            PatientListAction::OpenPatient(_id) => {
                                return Ok(Control::Changed);
                            }
                            PatientListAction::ContextMenu { .. } => {
                                return Ok(Control::Changed);
                            }
                        }
                    }
                }

                if state.appointment_state.is_focused() {
                    if state.appointment_state.current_view == AppointmentView::Calendar {
                        if let Some(action) = state.appointment_state.calendar.handle_key(*key) {
                            match action {
                                CalendarAction::SelectDate(date) => {
                                    state.appointment_state.selected_date = Some(date);
                                    state.appointment_state.current_view =
                                        AppointmentView::Schedule;
                                    spawn_refresh_appointments(ctx, date);
                                }
                                CalendarAction::FocusDate(_)
                                | CalendarAction::MonthChanged(_)
                                | CalendarAction::GoToToday => {}
                            }
                            return Ok(Control::Changed);
                        }
                    } else if let Some(action) = state.appointment_state.handle_key(*key) {
                        match action {
                            ScheduleAction::SelectPractitioner(id) => {
                                state.appointment_state.selected_practitioner = Some(id);
                            }
                            ScheduleAction::SelectAppointment(id) => {
                                state.appointment_state.selected_appointment = Some(id);
                            }
                            ScheduleAction::CreateAtSlot {
                                practitioner_id,
                                date,
                                time,
                            } => {
                                let mut form = AppointmentForm::new(
                                    ctx.theme.clone(),
                                    ctx.healthcare_config.clone(),
                                );
                                if let Some(schedule_data) =
                                    state.appointment_state.schedule_data.as_ref()
                                {
                                    if let Some(practitioner) = schedule_data
                                        .practitioners
                                        .iter()
                                        .find(|p| p.practitioner_id == practitioner_id)
                                    {
                                        form.set_practitioner(
                                            practitioner_id,
                                            practitioner.practitioner_name.clone(),
                                        );
                                    }
                                }
                                form.set_value(
                                    opengp_ui::ui::components::appointment::AppointmentFormField::Date,
                                    opengp_ui::ui::widgets::format_date(date),
                                );
                                form.set_value(
                                    opengp_ui::ui::components::appointment::AppointmentFormField::StartTime,
                                    time,
                                );
                                push_dialog(ctx, DialogContent::AppointmentForm(form));
                                spawn_load_practitioners(ctx);
                            }
                            ScheduleAction::NavigateTimeSlot(_)
                            | ScheduleAction::NavigatePractitioner(_)
                            | ScheduleAction::ToggleColumn => {}
                        }
                        return Ok(Control::Changed);
                    }
                }

                match key.code {
                    KeyCode::F(1) => {
                        toggle_help_dialog(state, ctx);
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
                            .lookup(*key, state.current_context)
                            .map(|k| k.action.clone());

                        if let Some(action) = action {
                            match action {
                                Action::SwitchToSchedule => {
                                    state.tab_bar.select(Tab::Schedule);
                                    state.previous_tab = Tab::Schedule;
                                    refresh_status_and_context(state, ctx);
                                    return Ok(Control::Changed);
                                }
                                Action::SwitchToPatientSearch => {
                                    if state.workspace_manager.active().is_some() {
                                        state.workspace_manager.active_index = None;
                                    }
                                    if state.previous_tab != Tab::PatientSearch {
                                        state.patient_list.reset_search();
                                        spawn_patient_list_refresh(ctx);
                                    }
                                    state.tab_bar.select(Tab::PatientSearch);
                                    state.previous_tab = Tab::PatientSearch;
                                    refresh_status_and_context(state, ctx);
                                    return Ok(Control::Changed);
                                }
                                Action::OpenHelp => {
                                    toggle_help_dialog(state, ctx);
                                    return Ok(Control::Changed);
                                }
                                Action::Quit => {
                                    state.should_quit = true;
                                    return Ok(Control::Quit);
                                }
                                Action::New => {
                                    if state.tab_bar.selected() == Tab::PatientSearch
                                        && dialog_index(ctx, |dialog| {
                                            matches!(dialog, DialogContent::PatientForm(_))
                                        })
                                        .is_none()
                                    {
                                        push_dialog(
                                            ctx,
                                            DialogContent::PatientForm(PatientForm::new(
                                                ctx.theme.clone(),
                                                &ctx.patient_config,
                                            )),
                                        );
                                        refresh_status_and_context(state, ctx);
                                        return Ok(Control::Changed);
                                    }
                                }
                                Action::Edit => {
                                    if state.tab_bar.selected() == Tab::PatientSearch {
                                        if let Some(patient_id) =
                                            state.patient_list.selected_patient_id()
                                        {
                                            spawn_load_patient_for_edit(ctx, patient_id);
                                            return Ok(Control::Changed);
                                        }
                                    }
                                }
                                Action::Escape => {
                                    let mut changed = false;
                                    if close_dialog(ctx, |dialog| {
                                        matches!(
                                            dialog,
                                            DialogContent::PatientForm(_)
                                                | DialogContent::AppointmentForm(_)
                                                | DialogContent::AppointmentDetailModal(_)
                                        )
                                    }) {
                                        changed = true;
                                    }

                                    if state.tab_bar.selected() == Tab::Schedule
                                        && state.appointment_state.current_view
                                            == AppointmentView::Schedule
                                        && dialog_index(ctx, |dialog| {
                                            matches!(dialog, DialogContent::AppointmentForm(_))
                                        })
                                        .is_none()
                                    {
                                        state.appointment_state.current_view =
                                            AppointmentView::Calendar;
                                        state.appointment_state.calendar.focused = true;
                                        state.appointment_state.focused = false;
                                        changed = true;
                                    }
                                    if changed {
                                        refresh_status_and_context(state, ctx);
                                        return Ok(Control::Changed);
                                    }
                                }
                                Action::Refresh => {
                                    match state.tab_bar.selected() {
                                        Tab::PatientSearch | Tab::PatientWorkspace => {
                                            spawn_patient_list_refresh(ctx);
                                        }
                                        Tab::Schedule => {
                                            let date = state
                                                .appointment_state
                                                .selected_date
                                                .unwrap_or_else(|| chrono::Utc::now().date_naive());
                                            spawn_refresh_appointments(ctx, date);
                                        }
                                    }
                                    return Ok(Control::Changed);
                                }
                                Action::NewAppointment => {
                                    if state.tab_bar.selected() == Tab::Schedule {
                                        push_dialog(
                                            ctx,
                                            DialogContent::AppointmentForm(AppointmentForm::new(
                                                ctx.theme.clone(),
                                                ctx.healthcare_config.clone(),
                                            )),
                                        );
                                        spawn_load_practitioners(ctx);
                                        return Ok(Control::Changed);
                                    }
                                }
                                _ => {}
                            }
                        }

                        Ok(Control::Continue)
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
                    spawn_patient_list_refresh(ctx);
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
        AppEvent::PatientEditLoaded(result) => {
            match result {
                Ok(patient) => {
                    push_dialog(
                        ctx,
                        DialogContent::PatientForm(PatientForm::from_patient(
                            patient.clone(),
                            ctx.theme.clone(),
                            &ctx.patient_config,
                        )),
                    );
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::AppointmentSaved(result) => {
            match result {
                Ok(()) => {
                    close_dialog(ctx, |dialog| {
                        matches!(
                            dialog,
                            DialogContent::AppointmentDetailModal(_)
                                | DialogContent::AppointmentForm(_)
                        )
                    });
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::AppointmentsRefreshed(result) => {
            match result {
                Ok(schedule) => {
                    state.appointment_state.schedule_data = Some(schedule.clone());
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::AppointmentStatusUpdated(result) => {
            match result {
                Ok((_, date)) => {
                    close_dialog(ctx, |dialog| {
                        matches!(dialog, DialogContent::AppointmentDetailModal(_))
                    });
                    spawn_refresh_appointments(ctx, *date);
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::AppointmentRescheduled(result) => {
            match result {
                Ok((_, date)) => {
                    close_dialog(ctx, |dialog| {
                        matches!(dialog, DialogContent::AppointmentDetailModal(_))
                    });
                    spawn_refresh_appointments(ctx, *date);
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::AppointmentCancelled(result) => {
            match result {
                Ok(()) => {
                    close_dialog(ctx, |dialog| {
                        matches!(dialog, DialogContent::AppointmentDetailModal(_))
                    });
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::PractitionersLoaded(result) => {
            match result {
                Ok(view_items) => {
                    if let Some(index) = ctx.dialogs.top::<DialogContent>() {
                        if let Some(mut dialog) = ctx.dialogs.get_mut::<DialogContent>(index) {
                            if let DialogContent::AppointmentForm(form) = &mut *dialog {
                                form.set_practitioners(view_items.clone());
                            }
                        }
                    }
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::AvailableSlotsLoaded(result) => {
            match result {
                Ok(slots) => {
                    if let Some(index) = ctx.dialogs.top::<DialogContent>() {
                        if let Some(mut dialog) = ctx.dialogs.get_mut::<DialogContent>(index) {
                            match &mut *dialog {
                                DialogContent::AppointmentForm(form) => {
                                    form.set_booked_slots(slots.clone());
                                }
                                DialogContent::AppointmentDetailModal(modal) => {
                                    modal.set_booked_slots(slots.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::ConsultationsRefreshed(result) => {
            match result {
                Ok(()) => state.status_bar.clear_error(),
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::ClinicalDataSaved(result) => {
            match result {
                Ok(_) => {
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::BillingDataSaved(result) => {
            match result {
                Ok(_) => {
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::PatientWorkspaceDataLoaded {
            patient_id: _,
            subtab: _,
            result,
        } => {
            match result {
                Ok(_load_result) => {
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
        AppEvent::BillingDataLoaded(result) => {
            match result {
                Ok(_) => {
                    state.status_bar.clear_error();
                }
                Err(err) => state.status_bar.set_error(Some(err.clone())),
            }
            Ok(Control::Changed)
        }
    }
}

fn error_fn(
    err: AppError,
    state: &mut AppState,
    _ctx: &mut GlobalState,
) -> Result<Control<AppEvent>, AppError> {
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
