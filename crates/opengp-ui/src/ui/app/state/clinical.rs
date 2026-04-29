use crate::ui::app::App;

impl App {
    pub fn take_pending_clinical_save_data(
        &mut self,
    ) -> Option<crate::ui::app::PendingClinicalSaveData> {
        if !self.authenticated {
            return None;
        }
        self.pending_clinical_save_data.take()
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::app::PendingClinicalSaveData;
    use crate::ui::theme::Theme;
    use opengp_config::CalendarConfig;

    fn make_app() -> App {
        App::new(
            None,
            CalendarConfig::default(),
            Theme::dark(),
            opengp_config::healthcare::HealthcareConfig::default(),
            opengp_config::PatientConfig::default(),
            opengp_config::AllergyConfig::default(),
            opengp_config::ClinicalConfig::default(),
            opengp_config::SocialHistoryConfig::default(),
            None,
            None,
            opengp_config::PracticeConfig::default(),
            8,
        )
    }

    #[test]
    fn take_pending_clinical_save_data_returns_and_clears_pending_value() {
        let mut app = make_app();
        let consultation_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        app.pending_clinical_save_data = Some(PendingClinicalSaveData::SignConsultation {
            consultation_id,
            user_id,
        });

        let first = app.take_pending_clinical_save_data();
        let second = app.take_pending_clinical_save_data();

        assert!(matches!(
            first,
            Some(PendingClinicalSaveData::SignConsultation {
                consultation_id: id,
                user_id: uid,
            }) if id == consultation_id && uid == user_id
        ));
        assert!(second.is_none());
    }
}
