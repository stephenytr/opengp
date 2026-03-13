use chrono::NaiveDate;
use chrono::TimeZone;
use std::collections::HashMap;

use crate::ui::app::{
    ApiTaskError, App, AppointmentStatusTransition, PendingClinicalSaveData, PendingPatientData,
    RetryOperation,
};
use crate::ui::components::appointment::AppointmentState;
use crate::ui::components::clinical::ClinicalState;
use crate::ui::components::patient::PatientForm;
use crate::ui::keybinds::KeyContext;
use crate::ui::view_models::{PatientListItem, PractitionerViewItem};
use opengp_domain::domain::appointment::{
    AppointmentStatus, AppointmentType, CalendarAppointment, CalendarDayView, PractitionerSchedule,
};
use opengp_domain::domain::clinical::Consultation;
use opengp_domain::domain::patient::Gender;

impl App {
    pub fn request_refresh_patients(&mut self) {
        self.pending_patient_list_refresh = true;
    }

    pub fn request_refresh_appointments(&mut self, date: NaiveDate) {
        self.pending_appointment_list_refresh = Some(date);
    }

    pub fn request_refresh_consultations(&mut self, patient_id: uuid::Uuid) {
        self.pending_consultation_list_refresh = Some(patient_id);
    }

    pub async fn poll_api_tasks(&mut self) {
        self.start_pending_login_request();

        if self
            .login_task
            .as_ref()
            .is_some_and(tokio::task::JoinHandle::is_finished)
        {
            let handle = self.login_task.take().expect("task exists");
            let retry_login = self.active_login_attempt.clone();
            self.active_login_attempt = None;

            match handle.await {
                Ok(Ok(login_response)) => {
                    self.login_screen.set_loading(false);
                    self.login_screen.clear_error();
                    self.clear_server_unavailable_error();
                    self.authenticated = true;
                    self.current_user_id = login_response.user.id;
                    self.request_refresh_patients();
                    self.status_bar.clear_error();
                }
                Ok(Err(err)) => {
                    match err {
                        crate::api::ApiClientError::ServerUnavailable(message) => {
                            let retry_operation = retry_login.map(|(username, password)| {
                                RetryOperation::Login { username, password }
                            });
                            if let Some(retry_operation) = retry_operation {
                                self.show_server_unavailable_error(message, retry_operation);
                            }
                            self.login_screen.set_error("Cannot connect to server");
                        }
                        other => {
                            self.login_screen.set_error(other.to_string());
                        }
                    }
                }
                Err(err) => {
                    self.login_screen
                        .set_error(format!("Login task failed: {}", err));
                }
            }
        }

        self.start_pending_api_requests();

        if self
            .patient_list_fetch_task
            .as_ref()
            .is_some_and(tokio::task::JoinHandle::is_finished)
        {
            let handle = self.patient_list_fetch_task.take().expect("task exists");
            self.patient_list.set_loading(false);

            match handle.await {
                Ok(Ok(items)) => {
                    self.patient_list.set_patients(items);
                    self.clear_server_unavailable_error();
                    self.status_bar.clear_error();
                }
                Ok(Err(err)) => {
                    self.handle_api_task_error(err, Some(RetryOperation::RefreshPatients))
                        .await
                }
                Err(err) => {
                    self.handle_api_task_error(ApiTaskError::message(format!(
                        "Failed to load patients: {}",
                        err
                    )), None)
                    .await;
                }
            }
        }

        if self
            .appointment_list_fetch_task
            .as_ref()
            .is_some_and(tokio::task::JoinHandle::is_finished)
        {
            let handle = self
                .appointment_list_fetch_task
                .take()
                .expect("task exists");
            let retry_date = self.active_appointment_refresh_date;
            self.active_appointment_refresh_date = None;
            self.appointment_state.is_loading = false;

            match handle.await {
                Ok(Ok(schedule)) => {
                    self.appointment_state.schedule_data = Some(schedule);
                    self.clear_server_unavailable_error();
                    self.status_bar.clear_error();
                }
                Ok(Err(err)) => {
                    self.handle_api_task_error(
                        err,
                        retry_date.map(|date| RetryOperation::RefreshAppointments { date }),
                    )
                    .await
                }
                Err(err) => {
                    self.handle_api_task_error(ApiTaskError::message(format!(
                        "Failed to load appointments: {}",
                        err
                    )), None)
                    .await;
                }
            }
        }

        if self
            .consultation_list_fetch_task
            .as_ref()
            .is_some_and(tokio::task::JoinHandle::is_finished)
        {
            let handle = self
                .consultation_list_fetch_task
                .take()
                .expect("task exists");
            let retry_patient_id = self.active_consultation_refresh_patient_id;
            self.active_consultation_refresh_patient_id = None;
            self.clinical_state.consultation_list.set_loading(false);

            match handle.await {
                Ok(Ok(consultations)) => {
                    self.clinical_state.consultations = consultations.clone();
                    self.clinical_state.consultation_list.consultations = consultations;
                    self.clear_server_unavailable_error();
                    self.status_bar.clear_error();
                }
                Ok(Err(err)) => {
                    self.handle_api_task_error(
                        err,
                        retry_patient_id
                            .map(|patient_id| RetryOperation::RefreshConsultations { patient_id }),
                    )
                    .await
                }
                Err(err) => {
                    self.handle_api_task_error(ApiTaskError::message(format!(
                        "Failed to load consultations: {}",
                        err
                    )), None)
                    .await;
                }
            }
        }
    }

    async fn handle_api_task_error(&mut self, error: ApiTaskError, retry: Option<RetryOperation>) {
        match error {
            ApiTaskError::Unauthorized => {
                if let Some(api_client) = self.api_client.clone() {
                    api_client.set_session_token(None).await;
                }

                self.authenticated = false;
                self.current_user_id = uuid::Uuid::nil();
                self.pending_patient_list_refresh = false;
                self.pending_appointment_list_refresh = None;
                self.pending_consultation_list_refresh = None;
                self.active_login_attempt = None;
                self.active_appointment_refresh_date = None;
                self.active_consultation_refresh_patient_id = None;
                self.patient_list.set_loading(false);
                self.appointment_state.is_loading = false;
                self.clinical_state.consultation_list.set_loading(false);
                self.login_screen.set_loading(false);
                self.login_screen
                    .set_error("Session expired. Please log in again.");
            }
            ApiTaskError::ServerUnavailable(message) => {
                if let Some(retry_operation) = retry {
                    self.show_server_unavailable_error(message, retry_operation);
                    self.set_status_error("Cannot connect to server");
                } else {
                    self.set_status_error("Cannot connect to server");
                }
            }
            ApiTaskError::Message(message) => self.set_status_error(message),
        }
    }

    fn start_pending_api_requests(&mut self) {
        if !self.authenticated {
            return;
        }

        let Some(api_client) = self.api_client.clone() else {
            if self.pending_patient_list_refresh
                || self.pending_appointment_list_refresh.is_some()
                || self.pending_consultation_list_refresh.is_some()
            {
                self.set_status_error("API client is not configured");
                self.pending_patient_list_refresh = false;
                self.pending_appointment_list_refresh = None;
                self.pending_consultation_list_refresh = None;
            }
            return;
        };

        if self.pending_patient_list_refresh && self.patient_list_fetch_task.is_none() {
            let page_limit = self.patient_page_limit;
            self.pending_patient_list_refresh = false;
            self.patient_list.set_loading(true);

            self.patient_list_fetch_task = Some(tokio::spawn(async move {
                fetch_patients(api_client, page_limit).await
            }));
        }

        if let Some(date) = self.pending_appointment_list_refresh.take() {
            if self.appointment_list_fetch_task.is_none() {
                let page_limit = self.appointment_page_limit;
                let patient_names = self
                    .patient_list
                    .patients()
                    .iter()
                    .map(|p| (p.id, p.full_name.clone()))
                    .collect::<HashMap<_, _>>();
                self.appointment_state.is_loading = true;
                self.active_appointment_refresh_date = Some(date);

                let api_client = self.api_client.clone().expect("api client already checked");
                self.appointment_list_fetch_task = Some(tokio::spawn(async move {
                    fetch_appointments_for_day(api_client, date, page_limit, patient_names).await
                }));
            }
        }

        if let Some(patient_id) = self.pending_consultation_list_refresh.take() {
            if self.consultation_list_fetch_task.is_none() {
                let page_limit = self.consultation_page_limit;
                self.clinical_state.consultation_list.set_loading(true);
                self.active_consultation_refresh_patient_id = Some(patient_id);

                let api_client = self.api_client.clone().expect("api client already checked");
                self.consultation_list_fetch_task = Some(tokio::spawn(async move {
                    fetch_consultations(api_client, patient_id, page_limit).await
                }));
            }
        }
    }

    fn start_pending_login_request(&mut self) {
        let Some((username, password)) = self.pending_login_request.take() else {
            return;
        };

        if self.login_task.is_some() {
            self.pending_login_request = Some((username, password));
            return;
        }

        let Some(api_client) = self.api_client.clone() else {
            self.login_screen.set_error("API client is not configured");
            return;
        };

        self.login_screen.set_loading(true);
        self.active_login_attempt = Some((username.clone(), password.clone()));
        self.login_task = Some(tokio::spawn(async move {
            api_client.login(username, password).await
        }));
    }

    /// Load patients into the list
    pub fn load_patients(&mut self, patients: Vec<opengp_domain::domain::patient::Patient>) {
        let list_items: Vec<PatientListItem> =
            patients.into_iter().map(PatientListItem::from).collect();
        self.patient_list.set_patients(list_items);
    }

    /// Take pending patient data (for saving to database)
    pub fn take_pending_patient_data(&mut self) -> Option<PendingPatientData> {
        if !self.authenticated {
            return None;
        }
        self.pending_patient_data.take()
    }

    /// Take pending patient ID to load for editing
    pub fn take_pending_edit_patient_id(&mut self) -> Option<uuid::Uuid> {
        self.pending_edit_patient_id.take()
    }

    /// Set pending patient ID to load for editing (from UI event)
    pub fn request_edit_patient(&mut self, patient_id: uuid::Uuid) {
        self.pending_edit_patient_id = Some(patient_id);
    }

    /// Take pending appointment date (for loading practitioners in main loop)
    pub fn take_pending_appointment_date(&mut self) -> Option<NaiveDate> {
        self.pending_appointment_date.take()
    }

    /// Request loading practitioners for appointment form picker
    pub fn request_load_practitioners(&mut self) {
        self.pending_load_practitioners = true;
    }

    /// Take pending load practitioners flag
    pub fn take_pending_load_practitioners(&mut self) -> bool {
        std::mem::take(&mut self.pending_load_practitioners)
    }

    /// Take pending appointment save data (for saving to database in main loop)
    pub fn take_pending_appointment_save(
        &mut self,
    ) -> Option<opengp_domain::domain::appointment::NewAppointmentData> {
        if !self.authenticated {
            return None;
        }
        self.pending_appointment_save.take()
    }

    pub fn take_pending_appointment_status_transition(
        &mut self,
    ) -> Option<(uuid::Uuid, AppointmentStatusTransition)> {
        self.pending_appointment_status_transition.take()
    }

    pub fn take_pending_clinical_patient_id(&mut self) -> Option<uuid::Uuid> {
        self.pending_clinical_patient_id.take()
    }

    pub fn take_pending_clinical_save_data(&mut self) -> Option<PendingClinicalSaveData> {
        if !self.authenticated {
            return None;
        }
        self.pending_clinical_save_data.take()
    }

    /// Set an error message on the status bar (for use by main loop)
    pub fn set_status_error(&mut self, message: impl Into<String>) {
        self.status_bar.set_error(message);
    }

    /// Get mutable reference to appointment state (for loading practitioners)
    pub fn appointment_state_mut(&mut self) -> &mut AppointmentState {
        &mut self.appointment_state
    }

    /// Set patients in the appointment form picker
    pub fn appointment_form_set_patients(&mut self, patients: Vec<PatientListItem>) {
        if let Some(ref mut form) = self.appointment_form {
            form.set_patients(patients);
        }
    }

    /// Set practitioners in the appointment form picker
    pub fn appointment_form_set_practitioners(&mut self, practitioners: Vec<PractitionerViewItem>) {
        if let Some(ref mut form) = self.appointment_form {
            form.set_practitioners(practitioners);
        }
    }

    /// Take pending booked slots load request (practitioner_id, date, duration)
    pub fn take_pending_load_booked_slots(&mut self) -> Option<(uuid::Uuid, NaiveDate, u32)> {
        self.pending_load_booked_slots.take()
    }

    /// Set booked slots in the appointment form time picker
    pub fn appointment_form_set_booked_slots(&mut self, booked_slots: Vec<chrono::NaiveTime>) {
        if let Some(ref mut form) = self.appointment_form {
            form.set_booked_slots(booked_slots);
        }
    }

    pub fn clinical_state_mut(&mut self) -> &mut ClinicalState {
        &mut self.clinical_state
    }

    /// Open patient form for editing (called from main loop after fetching patient)
    pub fn open_patient_form(&mut self, patient: opengp_domain::domain::patient::Patient) {
        self.patient_form = Some(PatientForm::from_patient(patient, self.theme.clone()));
        self.current_context = KeyContext::PatientForm;
    }

    /// Get patients from the patient list
    pub fn patient_list_patients(&self) -> &[PatientListItem] {
        self.patient_list.patients()
    }
}

async fn fetch_patients(
    api_client: std::sync::Arc<crate::api::ApiClient>,
    limit: u32,
) -> Result<Vec<PatientListItem>, ApiTaskError> {
    let mut page = 1;
    let mut collected = Vec::new();

    loop {
        tracing::debug!(page, limit, "UI fetch_patients requesting API page");
        let response = match api_client.get_patients(page, limit).await {
            Ok(response) => {
                tracing::debug!(
                    page,
                    limit,
                    response_page = response.page,
                    response_limit = response.limit,
                    response_total = response.total,
                    response_count = response.data.len(),
                    "UI fetch_patients received API response"
                );
                response
            }
            Err(error) => {
                tracing::error!(
                    page,
                    limit,
                    error = %error,
                    "UI fetch_patients API request failed"
                );
                return Err(ApiTaskError::from_client_error(
                    error,
                    "Failed to fetch patients",
                ));
            }
        };
        let page_count = response.data.len();
        let response_total = response.total;

        for patient in response.data {
            collected.push(PatientListItem {
                id: patient.id,
                full_name: format!("{} {}", patient.first_name, patient.last_name),
                date_of_birth: patient.date_of_birth,
                gender: parse_gender(&patient.gender),
                medicare_number: None,
                medicare_irn: None,
                ihi: None,
                phone_mobile: patient.phone_mobile,
            });
        }

        tracing::debug!(
            page,
            limit,
            collected_count = collected.len(),
            response_total,
            page_count,
            "UI fetch_patients accumulated patient results"
        );

        if collected.len() as u64 >= response_total || page_count == 0 {
            break;
        }

        page += 1;
    }

    tracing::debug!(total_collected = collected.len(), "UI fetch_patients completed");

    Ok(collected)
}

async fn fetch_appointments_for_day(
    api_client: std::sync::Arc<crate::api::ApiClient>,
    date: NaiveDate,
    limit: u32,
    patient_names: HashMap<uuid::Uuid, String>,
) -> Result<CalendarDayView, ApiTaskError> {
    let start = chrono::Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).expect("valid start"));
    let end = chrono::Utc.from_utc_datetime(&date.and_hms_opt(23, 59, 59).expect("valid end"));

    let mut page = 1;
    let mut all = Vec::new();

    loop {
        let response = api_client
            .get_appointments(page, limit, Some(start), Some(end), None)
            .await
            .map_err(|e| {
                ApiTaskError::from_client_error(e, "Failed to fetch appointments")
            })?;
        let page_count = response.data.len();

        all.extend(response.data);

        if all.len() as u64 >= response.total || page_count == 0 {
            break;
        }

        page += 1;
    }

    let mut grouped: HashMap<uuid::Uuid, Vec<CalendarAppointment>> = HashMap::new();

    for appointment in all {
        let appointment_type = parse_appointment_type(&appointment.appointment_type);
        let start_time = appointment.start_time;
        let end_time = appointment.end_time;
        let duration = (end_time - start_time).num_minutes().max(1);
        let slot_span = ((duration + 14) / 15) as u8;

        grouped
            .entry(appointment.practitioner_id)
            .or_default()
            .push(CalendarAppointment {
                id: appointment.id,
                patient_id: appointment.patient_id,
                patient_name: patient_names
                    .get(&appointment.patient_id)
                    .cloned()
                    .unwrap_or_else(|| {
                        format!("Patient {}", &appointment.patient_id.to_string()[..8])
                    }),
                practitioner_id: appointment.practitioner_id,
                start_time,
                end_time,
                appointment_type,
                status: parse_appointment_status(&appointment.status),
                is_urgent: appointment.is_urgent,
                slot_span,
                reason: appointment.reason,
                notes: None,
            });
    }

    let mut practitioners: Vec<PractitionerSchedule> = grouped
        .into_iter()
        .map(|(practitioner_id, appointments)| PractitionerSchedule {
            practitioner_id,
            practitioner_name: format!("Practitioner {}", &practitioner_id.to_string()[..8]),
            appointments,
        })
        .collect();

    practitioners.sort_by(|a, b| a.practitioner_name.cmp(&b.practitioner_name));

    Ok(CalendarDayView {
        date,
        practitioners,
    })
}

async fn fetch_consultations(
    api_client: std::sync::Arc<crate::api::ApiClient>,
    patient_id: uuid::Uuid,
    limit: u32,
) -> Result<Vec<Consultation>, ApiTaskError> {
    let mut page = 1;
    let mut collected = Vec::new();

    loop {
        let response = api_client
            .get_consultations(patient_id, page, limit)
            .await
            .map_err(|e| {
                ApiTaskError::from_client_error(e, "Failed to fetch consultations")
            })?;
        let page_count = response.data.len();

        for item in response.data {
            collected.push(Consultation {
                id: item.id,
                patient_id: item.patient_id,
                practitioner_id: item.practitioner_id,
                appointment_id: item.appointment_id,
                consultation_date: item.consultation_date,
                reason: item.reason,
                clinical_notes: item.clinical_notes,
                is_signed: item.is_signed,
                signed_at: None,
                signed_by: None,
                created_at: item.consultation_date,
                updated_at: item.consultation_date,
                version: 1,
                created_by: item.practitioner_id,
                updated_by: None,
            });
        }

        if collected.len() as u64 >= response.total || page_count == 0 {
            break;
        }

        page += 1;
    }

    Ok(collected)
}

fn parse_gender(value: &str) -> Gender {
    match value.trim().to_lowercase().as_str() {
        "male" => Gender::Male,
        "female" => Gender::Female,
        "other" => Gender::Other,
        _ => Gender::PreferNotToSay,
    }
}

fn parse_appointment_status(value: &str) -> AppointmentStatus {
    match value.trim().to_lowercase().as_str() {
        "scheduled" => AppointmentStatus::Scheduled,
        "confirmed" => AppointmentStatus::Confirmed,
        "arrived" => AppointmentStatus::Arrived,
        "in_progress" | "inprogress" => AppointmentStatus::InProgress,
        "completed" => AppointmentStatus::Completed,
        "no_show" | "noshow" => AppointmentStatus::NoShow,
        "cancelled" => AppointmentStatus::Cancelled,
        "rescheduled" => AppointmentStatus::Rescheduled,
        _ => AppointmentStatus::Scheduled,
    }
}

fn parse_appointment_type(value: &str) -> AppointmentType {
    match value.trim().to_lowercase().as_str() {
        "brief" => AppointmentType::Brief,
        "standard" => AppointmentType::Standard,
        "long" => AppointmentType::Long,
        "new_patient" | "newpatient" => AppointmentType::NewPatient,
        "health_assessment" | "healthassessment" => AppointmentType::HealthAssessment,
        "chronic_disease_review" | "chronicdiseasereview" => AppointmentType::ChronicDiseaseReview,
        "mental_health_plan" | "mentalhealthplan" => AppointmentType::MentalHealthPlan,
        "immunisation" => AppointmentType::Immunisation,
        "procedure" => AppointmentType::Procedure,
        "telephone" => AppointmentType::Telephone,
        "telehealth" => AppointmentType::Telehealth,
        "home_visit" | "homevisit" => AppointmentType::HomeVisit,
        "emergency" => AppointmentType::Emergency,
        _ => AppointmentType::Standard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::{Query, State},
        http::StatusCode,
        routing::{get, post},
        Json, Router,
    };
    use chrono::Utc;
    use opengp_domain::domain::api::{
        ApiErrorResponse, AppointmentResponse, AuthenticatedUserResponse, ConsultationResponse,
        LoginRequest, LoginResponse, PaginatedResponse, PatientResponse,
    };
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::{sleep, Duration};

    #[derive(Clone, Default)]
    struct TestState {
        patient_calls: Arc<Mutex<u32>>,
        appointment_calls: Arc<Mutex<u32>>,
        consultation_calls: Arc<Mutex<u32>>,
    }

    #[tokio::test]
    async fn app_patient_refresh_uses_api_client() {
        let state = TestState::default();
        let app_router = Router::new()
            .route("/api/v1/patients", get(patients_handler))
            .with_state(state.clone());

        let (base_url, _server) = spawn_server(app_router).await;
        let api_client = Arc::new(crate::api::ApiClient::new(base_url));
        api_client
            .set_session_token(Some("test-token".to_string()))
            .await;

        let mut app = App::new(
            Some(api_client),
            None,
            None,
            None,
            opengp_config::CalendarConfig::default(),
        );
        app.request_refresh_patients();

        for _ in 0..20 {
            app.poll_api_tasks().await;
            if !app.patient_list_patients().is_empty() {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }

        assert_eq!(app.patient_list_patients().len(), 1);
        assert_eq!(*state.patient_calls.lock().await, 1);
    }

    #[tokio::test]
    async fn app_appointment_and_consultation_refresh_use_api_client() {
        let state = TestState::default();
        let app_router = Router::new()
            .route("/api/v1/appointments", get(appointments_handler))
            .route("/api/v1/consultations", get(consultations_handler))
            .with_state(state.clone());

        let (base_url, _server) = spawn_server(app_router).await;
        let api_client = Arc::new(crate::api::ApiClient::new(base_url));
        api_client
            .set_session_token(Some("test-token".to_string()))
            .await;

        let mut app = App::new(
            Some(api_client),
            None,
            None,
            None,
            opengp_config::CalendarConfig::default(),
        );

        let patient_id = uuid::Uuid::new_v4();
        app.request_refresh_appointments(Utc::now().date_naive());
        app.request_refresh_consultations(patient_id);

        for _ in 0..20 {
            app.poll_api_tasks().await;
            if app.appointment_state.schedule_data.is_some()
                && !app.clinical_state.consultations.is_empty()
            {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }

        assert!(app.appointment_state.schedule_data.is_some());
        assert_eq!(app.clinical_state.consultations.len(), 1);
        assert_eq!(*state.appointment_calls.lock().await, 1);
        assert_eq!(*state.consultation_calls.lock().await, 1);
    }

    #[tokio::test]
    async fn app_login_success_sets_authentication_and_token() {
        let app_router = Router::new()
            .route("/api/v1/auth/login", post(login_success_handler))
            .route("/api/v1/patients", get(patients_handler))
            .with_state(TestState::default());

        let (base_url, _server) = spawn_server(app_router).await;
        let api_client = Arc::new(crate::api::ApiClient::new(base_url));

        let mut app = App::new(
            Some(api_client.clone()),
            None,
            None,
            None,
            opengp_config::CalendarConfig::default(),
        );
        app.set_authenticated(false);

        enter_login_credentials(&mut app, "dr_smith", "correct-password");

        for _ in 0..30 {
            app.poll_api_tasks().await;
            if app.is_authenticated() {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }

        assert!(app.is_authenticated());
        assert_eq!(
            api_client.current_session_token().await.as_deref(),
            Some("session-token")
        );
    }

    #[tokio::test]
    async fn app_login_failure_keeps_user_unauthenticated() {
        let app_router = Router::new().route("/api/v1/auth/login", post(login_failure_handler));

        let (base_url, _server) = spawn_server(app_router).await;
        let api_client = Arc::new(crate::api::ApiClient::new(base_url));

        let mut app = App::new(
            Some(api_client.clone()),
            None,
            None,
            None,
            opengp_config::CalendarConfig::default(),
        );
        app.set_authenticated(false);

        enter_login_credentials(&mut app, "dr_smith", "wrong-password");

        for _ in 0..20 {
            app.poll_api_tasks().await;
            sleep(Duration::from_millis(10)).await;
        }

        assert!(!app.is_authenticated());
        assert!(api_client.current_session_token().await.is_none());
    }

    #[tokio::test]
    async fn app_handles_unauthorized_api_response_by_logging_out() {
        let app_router = Router::new().route("/api/v1/patients", get(patients_unauthorized_handler));

        let (base_url, _server) = spawn_server(app_router).await;
        let api_client = Arc::new(crate::api::ApiClient::new(base_url));
        api_client
            .set_session_token(Some("test-token".to_string()))
            .await;

        let mut app = App::new(
            Some(api_client.clone()),
            None,
            None,
            None,
            opengp_config::CalendarConfig::default(),
        );
        app.set_authenticated(true);
        app.request_refresh_patients();

        for _ in 0..20 {
            app.poll_api_tasks().await;
            if !app.is_authenticated() {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }

        assert!(!app.is_authenticated());
        assert!(api_client.current_session_token().await.is_none());
    }

    #[test]
    fn pending_form_data_is_preserved_while_logged_out_and_available_after_relogin() {
        let mut app = App::new(None, None, None, None, opengp_config::CalendarConfig::default());

        let patient_id = uuid::Uuid::new_v4();
        let practitioner_id = uuid::Uuid::new_v4();

        app.pending_patient_data = Some(PendingPatientData::New(
            opengp_domain::domain::patient::NewPatientData {
                ihi: None,
                medicare_number: None,
                medicare_irn: None,
                medicare_expiry: None,
                title: None,
                first_name: "Jane".to_string(),
                middle_name: None,
                last_name: "Citizen".to_string(),
                preferred_name: None,
                date_of_birth: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).expect("valid date"),
                gender: Gender::Female,
                address: opengp_domain::domain::patient::Address::default(),
                phone_home: None,
                phone_mobile: Some("0400000000".to_string()),
                email: None,
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: None,
                interpreter_required: None,
                aboriginal_torres_strait_islander: None,
            },
        ));

        app.pending_appointment_save = Some(opengp_domain::domain::appointment::NewAppointmentData {
            patient_id,
            practitioner_id,
            start_time: Utc::now(),
            duration: chrono::Duration::minutes(15),
            appointment_type: AppointmentType::Standard,
            reason: Some("Follow-up".to_string()),
            is_urgent: false,
        });

        app.pending_clinical_save_data = Some(PendingClinicalSaveData::Consultation {
            patient_id,
            practitioner_id,
            appointment_id: None,
            reason: Some("Review".to_string()),
            clinical_notes: Some("Doing well".to_string()),
        });

        app.set_authenticated(false);

        assert!(app.take_pending_patient_data().is_none());
        assert!(app.take_pending_appointment_save().is_none());
        assert!(app.take_pending_clinical_save_data().is_none());

        assert!(app.pending_patient_data.is_some());
        assert!(app.pending_appointment_save.is_some());
        assert!(app.pending_clinical_save_data.is_some());

        app.set_authenticated(true);

        assert!(app.take_pending_patient_data().is_some());
        assert!(app.take_pending_appointment_save().is_some());
        assert!(app.take_pending_clinical_save_data().is_some());
    }

    #[tokio::test]
    async fn app_shows_server_unavailable_error_and_retries_patient_refresh() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let api_client = Arc::new(crate::api::ApiClient::new("http://127.0.0.1:9"));
        api_client
            .set_session_token(Some("test-token".to_string()))
            .await;

        let mut app = App::new(
            Some(api_client),
            None,
            None,
            None,
            opengp_config::CalendarConfig::default(),
        );

        app.request_refresh_patients();

        for _ in 0..30 {
            app.poll_api_tasks().await;
            if app.server_unavailable_error_message().is_some() {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }

        let error = app.server_unavailable_error_message().unwrap_or_default();
        assert!(error.contains("Cannot connect to server"));

        let action = app.handle_key_event(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
        assert_eq!(action, crate::ui::keybinds::Action::Refresh);
        assert!(app.server_unavailable_error_message().is_none());

        app.poll_api_tasks().await;
        assert!(app.patient_list_fetch_task.is_some());
    }

    async fn patients_handler(
        State(state): State<TestState>,
    ) -> Json<PaginatedResponse<PatientResponse>> {
        *state.patient_calls.lock().await += 1;
        Json(PaginatedResponse {
            data: vec![PatientResponse {
                id: uuid::Uuid::new_v4(),
                first_name: "Jane".to_string(),
                last_name: "Citizen".to_string(),
                date_of_birth: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).expect("valid date"),
                gender: "female".to_string(),
                phone_mobile: Some("0400000000".to_string()),
                email: None,
                is_active: true,
                version: 1,
            }],
            total: 1,
            page: 1,
            limit: 100,
        })
    }

    async fn appointments_handler(
        State(state): State<TestState>,
        Query(_): Query<HashMap<String, String>>,
    ) -> Json<PaginatedResponse<AppointmentResponse>> {
        *state.appointment_calls.lock().await += 1;
        let patient_id = uuid::Uuid::new_v4();
        let practitioner_id = uuid::Uuid::new_v4();
        let start_time = Utc::now();
        Json(PaginatedResponse {
            data: vec![AppointmentResponse {
                id: uuid::Uuid::new_v4(),
                patient_id,
                practitioner_id,
                start_time,
                end_time: start_time + chrono::Duration::minutes(15),
                status: "scheduled".to_string(),
                appointment_type: "standard".to_string(),
                is_urgent: false,
                reason: Some("Review".to_string()),
                version: 1,
            }],
            total: 1,
            page: 1,
            limit: 100,
        })
    }

    async fn consultations_handler(
        State(state): State<TestState>,
        Query(query): Query<HashMap<String, String>>,
    ) -> Json<PaginatedResponse<ConsultationResponse>> {
        *state.consultation_calls.lock().await += 1;
        let patient_id = query
            .get("patient_id")
            .and_then(|value| uuid::Uuid::parse_str(value).ok())
            .unwrap_or_else(uuid::Uuid::new_v4);
        Json(PaginatedResponse {
            data: vec![ConsultationResponse {
                id: uuid::Uuid::new_v4(),
                patient_id,
                practitioner_id: uuid::Uuid::new_v4(),
                appointment_id: None,
                consultation_date: Utc::now(),
                reason: Some("Follow-up".to_string()),
                clinical_notes: Some("Doing well".to_string()),
                is_signed: false,
                version: 1,
            }],
            total: 1,
            page: 1,
            limit: 100,
        })
    }

    async fn login_success_handler(Json(_): Json<LoginRequest>) -> Json<LoginResponse> {
        Json(LoginResponse {
            access_token: "session-token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in_seconds: 3600,
            user: AuthenticatedUserResponse {
                id: uuid::Uuid::new_v4(),
                username: "dr_smith".to_string(),
                role: "doctor".to_string(),
                display_name: "Dr Smith".to_string(),
            },
        })
    }

    async fn login_failure_handler(
        Json(_): Json<LoginRequest>,
    ) -> (axum::http::StatusCode, Json<ApiErrorResponse>) {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            Json(ApiErrorResponse {
                status: 401,
                message: "Invalid credentials".to_string(),
                code: "invalid_credentials".to_string(),
            }),
        )
    }

    async fn patients_unauthorized_handler() -> (StatusCode, Json<ApiErrorResponse>) {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiErrorResponse {
                status: 401,
                message: "Session expired".to_string(),
                code: "unauthorized".to_string(),
            }),
        )
    }

    fn enter_login_credentials(app: &mut App, username: &str, password: &str) {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        for ch in username.chars() {
            let _ = app.handle_key_event(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
        }
        let _ = app.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        for ch in password.chars() {
            let _ = app.handle_key_event(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
        }
        let _ = app.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        let _ = app.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    }

    async fn spawn_server(app: Router) -> (String, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let addr = listener
            .local_addr()
            .expect("listener should have local address");
        let server_task = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("test server should run")
        });

        (
            format!("http://{}", socket_addr_to_host_port(addr)),
            server_task,
        )
    }

    fn socket_addr_to_host_port(addr: SocketAddr) -> String {
        format!("{}:{}", addr.ip(), addr.port())
    }
}
