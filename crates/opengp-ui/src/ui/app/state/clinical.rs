use crate::ui::app::App;

impl App {
    pub fn clinical_state_mut(&mut self) -> &mut crate::ui::components::clinical::ClinicalState {
        let needs_init = self
            .workspace_manager()
            .active()
            .map(|w| w.clinical.is_none())
            .unwrap_or(false);

        if needs_init {
            let theme = self.theme.clone();
            let healthcare_config = self.healthcare_config.clone();
            let allergy_config = self.allergy_config.clone();
            let clinical_config = self.clinical_config.clone();
            let social_history_config = self.social_history_config.clone();

            let workspace = self
                .workspace_manager_mut()
                .active_mut()
                .expect("No active workspace for clinical state access");
            workspace.clinical = Some(crate::ui::components::clinical::ClinicalState::new(
                theme,
                healthcare_config,
                allergy_config,
                clinical_config,
                social_history_config,
            ));
        }

        self.workspace_manager_mut()
            .active_mut()
            .expect("No active workspace for clinical state access")
            .clinical
            .as_mut()
            .expect("clinical state must be initialized")
    }
}
