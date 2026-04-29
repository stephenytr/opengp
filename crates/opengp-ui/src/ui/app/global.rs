use std::sync::Arc;
use rat_salsa::{SalsaAppContext, SalsaContext};

use crate::ui::app::event::AppEvent;
use crate::ui::app::error::AppError;
use crate::ui::keybinds::KeybindRegistry;
use crate::ui::theme::Theme;
use crate::ui::services::{
    BillingUiService, ClinicalUiService, AppointmentUiService, PatientUiService,
};
use crate::api::ApiClient;

/// GlobalState holds long-lived dependencies and the rat-salsa execution context.
/// It does NOT contain mutable UI state — that belongs in AppState.
pub struct GlobalState {
    pub salsa_ctx: SalsaAppContext<AppEvent, AppError>,
    pub api_client: Option<Arc<ApiClient>>,
    pub billing_ui_service: Option<Arc<BillingUiService>>,
    pub clinical_ui_service: Option<Arc<ClinicalUiService>>,
    pub appointment_ui_service: Option<Arc<AppointmentUiService>>,
    pub patient_ui_service: Option<Arc<PatientUiService>>,
    pub practice_config: opengp_config::PracticeConfig,
    pub healthcare_config: opengp_config::healthcare::HealthcareConfig,
    pub patient_config: opengp_config::PatientConfig,
    pub allergy_config: opengp_config::AllergyConfig,
    pub clinical_config: opengp_config::ClinicalConfig,
    pub social_history_config: opengp_config::SocialHistoryConfig,
    pub theme: Theme,
    pub keybinds: &'static KeybindRegistry,
}

impl SalsaContext<AppEvent, AppError> for GlobalState {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, AppError>) {
        self.salsa_ctx = app_ctx;
    }

    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, AppError> {
        &self.salsa_ctx
    }
}
