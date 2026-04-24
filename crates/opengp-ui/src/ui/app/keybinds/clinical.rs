use crate::ui::app::{App, PendingClinicalSaveData};
use crate::ui::components::clinical::{
    AllergyDetailModalAction, ClinicalFormView, ClinicalView,
    ConsultationFormAction, FamilyHistoryDetailModalAction, FamilyHistoryFormAction,
    MedicalHistoryDetailModalAction, MedicalHistoryFormAction,
    VitalsDetailModalAction,
};
use crate::ui::components::SubtabKind;
use crate::ui::keybinds::{Action, KeyContext};
use crate::ui::shared::FormAction;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::ui::widgets::FormNavigation;

impl App {
    pub(crate) fn handle_clinical_keys(&mut self, key: KeyEvent) -> Action {
        let Some(active_workspace) = self.workspace_manager().active() else {
            return Action::Unknown;
        };

        // Tab/Shift+Tab must pass through to the keybind system for clinical row cycling.
        // Do NOT trap these keys here - let them reach NextClinicalMenu/PrevClinicalMenu.
        if key.code == KeyCode::Tab || key.code == KeyCode::BackTab {
            return Action::Unknown;
        }

        if active_workspace.active_subtab != SubtabKind::Clinical {
            return Action::Unknown;
        }

        let patient_id = active_workspace.patient_id;
        let current_user_id = self.current_user_id;
        let theme = self.theme.clone();

        if self.clinical_state_mut().selected_patient_id != Some(patient_id) {
            self.clinical_state_mut().set_patient(patient_id);
        }

        if self.clinical_state_mut().has_open_detail_modal() {
            if key.code == KeyCode::Esc {
                self.clinical_state_mut().close_allergy_detail();
                self.clinical_state_mut().close_medical_history_detail();
                self.clinical_state_mut().close_vitals_detail();
                self.clinical_state_mut().close_family_history_detail();
                return Action::Enter;
            }

            let allergy_action = {
                let clinical_state = self.clinical_state_mut();
                clinical_state
                    .allergies
                    .allergy_detail_modal
                    .as_mut()
                    .and_then(|modal| modal.handle_key(key))
            };
            if let Some(action) = allergy_action {
                self.handle_allergy_modal_action(action);
                return Action::Enter;
            }

            let medical_history_action = {
                let clinical_state = self.clinical_state_mut();
                clinical_state
                    .medical_history
                    .medical_history_detail_modal
                    .as_mut()
                    .and_then(|modal| modal.handle_key(key))
            };
            if let Some(action) = medical_history_action {
                self.handle_medical_history_modal_action(action);
                return Action::Enter;
            }

            let vitals_action = {
                let clinical_state = self.clinical_state_mut();
                clinical_state
                    .vitals
                    .vitals_detail_modal
                    .as_mut()
                    .and_then(|modal| modal.handle_key(key))
            };
            if let Some(action) = vitals_action {
                self.handle_vitals_modal_action(action);
                return Action::Enter;
            }

            let family_history_action = {
                let clinical_state = self.clinical_state_mut();
                clinical_state
                    .family_history
                    .family_history_detail_modal
                    .as_mut()
                    .and_then(|modal| modal.handle_key(key))
            };
            if let Some(action) = family_history_action {
                self.handle_family_history_modal_action(action);
                return Action::Enter;
            }
        }

        if self.clinical_state_mut().is_form_open() {
            let form_view = self.clinical_state_mut().current_form_view().clone();
            match form_view {
                ClinicalFormView::AllergyForm => {
                    let form_action = {
                        let clinical_state = self.clinical_state_mut();
                        clinical_state
                            .allergies
                            .allergy_form
                            .as_mut()
                            .and_then(|form| form.handle_key(key))
                    };
                    if let Some(action) = form_action {
                        match action {
                            FormAction::FocusChanged | FormAction::ValueChanged => {}
                            FormAction::Submit => {
                                let payload = {
                                    let clinical_state = self.clinical_state_mut();
                                    clinical_state.allergies.allergy_form.as_mut().and_then(|form| {
                                        if form.validate() {
                                            Some(PendingClinicalSaveData::Allergy {
                                                patient_id,
                                                allergy: form.to_allergy(patient_id, current_user_id),
                                            })
                                        } else {
                                            None
                                        }
                                    })
                                };

                                if let Some(payload) = payload {
                                    self.pending_clinical_save_data = Some(payload);
                                    self.clinical_state_mut().close_form();
                                    self.status_bar.clear_error();
                                } else {
                                    self.status_bar
                                        .set_error(Some("Please fill in required fields".to_string()));
                                }
                            }
                            FormAction::Cancel => {
                                self.clinical_state_mut().close_form();
                                self.status_bar.clear_error();
                            }
                        }
                        return Action::Enter;
                    }
                }
                ClinicalFormView::MedicalHistoryForm => {
                    let form_action = {
                        let clinical_state = self.clinical_state_mut();
                        clinical_state
                            .medical_history
                            .medical_history_form
                            .as_mut()
                            .and_then(|form| form.handle_key(key))
                    };
                    if let Some(action) = form_action {
                        match action {
                            MedicalHistoryFormAction::FocusChanged
                            | MedicalHistoryFormAction::ValueChanged => {}
                            MedicalHistoryFormAction::Submit => {
                                let payload = {
                                    let clinical_state = self.clinical_state_mut();
                                    clinical_state
                                        .medical_history
                                        .medical_history_form
                                        .as_mut()
                                        .and_then(|form| {
                                            form.to_medical_history(patient_id, current_user_id)
                                                .map(|history| {
                                                    PendingClinicalSaveData::MedicalHistory {
                                                        patient_id,
                                                        history,
                                                    }
                                                })
                                        })
                                };

                                if let Some(payload) = payload {
                                    self.pending_clinical_save_data = Some(payload);
                                    self.clinical_state_mut().close_form();
                                    self.status_bar.clear_error();
                                } else {
                                    self.status_bar
                                        .set_error(Some("Please fill in required fields".to_string()));
                                }
                            }
                            MedicalHistoryFormAction::Cancel => {
                                self.clinical_state_mut().close_form();
                                self.status_bar.clear_error();
                            }
                        }
                        return Action::Enter;
                    }
                }
                ClinicalFormView::VitalSignsForm => {
                    let form_action = {
                        let clinical_state = self.clinical_state_mut();
                        clinical_state
                            .vitals
                            .vitals_form
                            .as_mut()
                            .and_then(|form| form.handle_key(key))
                    };
                    if let Some(action) = form_action {
                        match action {
                            FormAction::FocusChanged | FormAction::ValueChanged => {}
                            FormAction::Submit => {
                                let payload = {
                                    let clinical_state = self.clinical_state_mut();
                                    clinical_state.vitals.vitals_form.as_mut().and_then(|form| {
                                        if form.validate() {
                                            Some(PendingClinicalSaveData::VitalSigns {
                                                patient_id,
                                                vitals: form.to_vital_signs(patient_id, current_user_id),
                                            })
                                        } else {
                                            None
                                        }
                                    })
                                };

                                if let Some(payload) = payload {
                                    self.pending_clinical_save_data = Some(payload);
                                    self.clinical_state_mut().close_form();
                                    self.status_bar.clear_error();
                                } else {
                                    self.status_bar
                                        .set_error(Some("Please fill in required fields".to_string()));
                                }
                            }
                            FormAction::Cancel => {
                                self.clinical_state_mut().close_form();
                                self.status_bar.clear_error();
                            }
                        }
                        return Action::Enter;
                    }
                }
                ClinicalFormView::FamilyHistoryForm => {
                    let form_action = {
                        let clinical_state = self.clinical_state_mut();
                        clinical_state
                            .family_history
                            .family_history_form
                            .as_mut()
                            .and_then(|form| form.handle_key(key))
                    };
                    if let Some(action) = form_action {
                        match action {
                            FamilyHistoryFormAction::FocusChanged
                            | FamilyHistoryFormAction::ValueChanged => {}
                            FamilyHistoryFormAction::Submit => {
                                let payload = {
                                    let clinical_state = self.clinical_state_mut();
                                    clinical_state
                                        .family_history
                                        .family_history_form
                                        .as_mut()
                                        .map(|form| PendingClinicalSaveData::FamilyHistory {
                                            patient_id,
                                            entry: form.to_family_history(patient_id, current_user_id),
                                        })
                                };

                                if let Some(payload) = payload {
                                    self.pending_clinical_save_data = Some(payload);
                                    self.clinical_state_mut().close_form();
                                    self.status_bar.clear_error();
                                } else {
                                    self.status_bar
                                        .set_error(Some("Please fill in required fields".to_string()));
                                }
                            }
                            FamilyHistoryFormAction::Cancel => {
                                self.clinical_state_mut().close_form();
                                self.status_bar.clear_error();
                            }
                        }
                        return Action::Enter;
                    }
                }
                ClinicalFormView::ConsultationForm => {
                    let form_action = {
                        let clinical_state = self.clinical_state_mut();
                        clinical_state
                            .consultations
                            .consultation_form
                            .as_mut()
                            .and_then(|form| form.handle_key(key))
                    };

                    if let Some(action) = form_action {
                        match action {
                            ConsultationFormAction::FocusChanged
                            | ConsultationFormAction::ValueChanged => {}
                            ConsultationFormAction::Submit => {
                                let payload = {
                                    let clinical_state = self.clinical_state_mut();
                                    clinical_state
                                        .consultations
                                        .consultation_form
                                        .as_mut()
                                        .and_then(|form| {
                                            if form.validate() {
                                                let consultation = form.to_consultation(
                                                    patient_id,
                                                    uuid::Uuid::nil(),
                                                    current_user_id,
                                                );
                                                Some(PendingClinicalSaveData::Consultation {
                                                    patient_id,
                                                    practitioner_id: consultation.practitioner_id,
                                                    appointment_id: consultation.appointment_id,
                                                    reason: consultation.reason,
                                                    clinical_notes: consultation.clinical_notes,
                                                })
                                            } else {
                                                None
                                            }
                                        })
                                };

                                if let Some(payload) = payload {
                                    self.pending_clinical_save_data = Some(payload);
                                    self.clinical_state_mut().close_consultation_form();
                                    self.status_bar.clear_error();
                                } else {
                                    self.status_bar
                                        .set_error(Some("Please fill in required fields".to_string()));
                                }
                            }
                            ConsultationFormAction::Cancel => {
                                self.clinical_state_mut().close_consultation_form();
                                self.status_bar.clear_error();
                            }
                        }
                        return Action::Enter;
                    }
                }
                ClinicalFormView::SocialHistoryForm => {
                    if key.code == KeyCode::Esc {
                        self.clinical_state_mut().close_social_history_form();
                        return Action::Enter;
                    }
                }
                ClinicalFormView::None => {}
            }
        }

        let registry = crate::ui::keybinds::KeybindRegistry::global();
        if let Some(keybind) = registry.lookup(key, KeyContext::ClinicalSubView) {
            match keybind.action {
                Action::CycleClinicalView => {
                    if let Some(workspace) = self.workspace_manager.active_mut() {
                        workspace.active_clinical_menu = workspace.active_clinical_menu.next();
                    }
                    self.sync_clinical_view_to_menu();
                    return Action::Enter;
                }
                Action::CycleClinicalViewReverse => {
                    if let Some(workspace) = self.workspace_manager.active_mut() {
                        workspace.active_clinical_menu = workspace.active_clinical_menu.prev();
                    }
                    self.sync_clinical_view_to_menu();
                    return Action::Enter;
                }
                Action::ToggleConsultationTimer => {
                    let timer_payload = {
                        let clinical_state = self.clinical_state_mut();
                        let selected = clinical_state.consultations.consultation_list.selected().cloned();
                        selected.map(|c| {
                            if c.consultation_started_at.is_some() && c.consultation_ended_at.is_none() {
                                PendingClinicalSaveData::TimerStop { consultation_id: c.id }
                            } else {
                                PendingClinicalSaveData::TimerStart { consultation_id: c.id }
                            }
                        })
                    };
                    if let Some(payload) = timer_payload {
                        self.pending_clinical_save_data = Some(payload);
                        return Action::Enter;
                    }
                }
                _ => {}
            }
        }

        let view = self.clinical_state_mut().view.clone();
        match view {
            ClinicalView::PatientSummary => {}
            ClinicalView::Consultations => {
                let list_action = self
                    .clinical_state_mut()
                    .consultations
                    .consultation_list
                    .handle_key(key);
                if let Some(action) = list_action {
                    match action {
                        crate::ui::components::clinical::ConsultationListAction::Open(_) => {}
                        crate::ui::components::clinical::ConsultationListAction::New => {
                            self.clinical_state_mut().open_consultation_form();
                        }
                        crate::ui::components::clinical::ConsultationListAction::Select(_)
                        | crate::ui::components::clinical::ConsultationListAction::NextPage
                        | crate::ui::components::clinical::ConsultationListAction::PrevPage
                        | crate::ui::components::clinical::ConsultationListAction::ContextMenu { .. } => {}
                    }
                    return Action::Enter;
                }
            }
            ClinicalView::Allergies => {
                let list_action = self
                    .clinical_state_mut()
                    .allergies
                    .allergy_list
                    .handle_key(key);
                if let Some(action) = list_action {
                    match action {
                        crate::ui::components::clinical::AllergyListAction::Open(allergy) => {
                            self.clinical_state_mut().open_allergy_detail(allergy, &theme);
                        }
                        crate::ui::components::clinical::AllergyListAction::New => {
                            self.clinical_state_mut().open_allergy_form();
                        }
                        crate::ui::components::clinical::AllergyListAction::Select(_)
                        | crate::ui::components::clinical::AllergyListAction::ToggleInactive
                        | crate::ui::components::clinical::AllergyListAction::Delete(_)
                        | crate::ui::components::clinical::AllergyListAction::ContextMenu { .. } => {}
                    }
                    return Action::Enter;
                }
            }
            ClinicalView::MedicalHistory => {
                let list_action = self
                    .clinical_state_mut()
                    .medical_history
                    .medical_history_list
                    .handle_key(key);
                if let Some(action) = list_action {
                    match action {
                        crate::ui::widgets::ListAction::Open(condition) => {
                            self.clinical_state_mut()
                                .open_medical_history_detail(condition, &theme);
                        }
                        crate::ui::widgets::ListAction::New => {
                            self.clinical_state_mut().open_medical_history_form();
                        }
                        crate::ui::widgets::ListAction::Select(_)
                        | crate::ui::widgets::ListAction::Edit(_)
                        | crate::ui::widgets::ListAction::Delete(_)
                        | crate::ui::widgets::ListAction::ToggleInactive
                        | crate::ui::widgets::ListAction::ContextMenu { .. } => {}
                    }
                    return Action::Enter;
                }
            }
            ClinicalView::VitalSigns => {
                let list_action = self.clinical_state_mut().vitals.vitals_list.handle_key(key);
                if let Some(action) = list_action {
                    match action {
                        crate::ui::components::clinical::VitalSignsListAction::Open(vitals) => {
                            self.clinical_state_mut().open_vitals_detail(vitals, &theme);
                        }
                        crate::ui::components::clinical::VitalSignsListAction::New => {
                            self.clinical_state_mut().open_vitals_form();
                        }
                        crate::ui::components::clinical::VitalSignsListAction::Select(_)
                        | crate::ui::components::clinical::VitalSignsListAction::NextPage
                        | crate::ui::components::clinical::VitalSignsListAction::PrevPage
                        | crate::ui::components::clinical::VitalSignsListAction::ContextMenu { .. } => {}
                    }
                    return Action::Enter;
                }
            }
            ClinicalView::SocialHistory => {
                if key.code == KeyCode::Char('e') || key.code == KeyCode::Char('n') {
                    self.clinical_state_mut().open_social_history_form();
                    return Action::Enter;
                }
            }
            ClinicalView::FamilyHistory => {
                let list_action = self
                    .clinical_state_mut()
                    .family_history
                    .family_history_list
                    .handle_key(key);
                if let Some(action) = list_action {
                    match action {
                        crate::ui::components::clinical::FamilyHistoryListAction::Open(entry) => {
                            self.clinical_state_mut()
                                .open_family_history_detail(entry, &theme);
                        }
                        crate::ui::components::clinical::FamilyHistoryListAction::New => {
                            self.clinical_state_mut().open_family_history_form();
                        }
                        crate::ui::components::clinical::FamilyHistoryListAction::Select(_)
                        | crate::ui::components::clinical::FamilyHistoryListAction::Delete(_)
                        | crate::ui::components::clinical::FamilyHistoryListAction::ContextMenu { .. } => {}
                    }
                    return Action::Enter;
                }
            }
        }

        Action::Unknown
    }

    fn handle_allergy_modal_action(
        &mut self,
        action: AllergyDetailModalAction,
    ) {
        match action {
            AllergyDetailModalAction::Close => {
                self.clinical_state_mut().close_allergy_detail();
            }
            AllergyDetailModalAction::Edit => {
                self.clinical_state_mut().close_allergy_detail();
                self.clinical_state_mut().open_allergy_form();
            }
        }
    }

    fn handle_medical_history_modal_action(
        &mut self,
        action: MedicalHistoryDetailModalAction,
    ) {
        match action {
            MedicalHistoryDetailModalAction::Close => {
                self.clinical_state_mut().close_medical_history_detail();
            }
            MedicalHistoryDetailModalAction::Edit => {
                self.clinical_state_mut().close_medical_history_detail();
                self.clinical_state_mut().open_medical_history_form();
            }
        }
    }

    fn handle_vitals_modal_action(
        &mut self,
        action: VitalsDetailModalAction,
    ) {
        match action {
            VitalsDetailModalAction::Close => {
                self.clinical_state_mut().close_vitals_detail();
            }
            VitalsDetailModalAction::Edit => {
                self.clinical_state_mut().close_vitals_detail();
                self.clinical_state_mut().open_vitals_form();
            }
        }
    }

    fn handle_family_history_modal_action(
        &mut self,
        action: FamilyHistoryDetailModalAction,
    ) {
        match action {
            FamilyHistoryDetailModalAction::Close => {
                self.clinical_state_mut().close_family_history_detail();
            }
            FamilyHistoryDetailModalAction::Edit => {
                self.clinical_state_mut().close_family_history_detail();
                self.clinical_state_mut().open_family_history_form();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::clinical::ClinicalFormView;
    use crate::ui::theme::Theme;
    use crate::ui::view_models::PatientListItem;
    use chrono::NaiveDate;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use opengp_config::CalendarConfig;
    use opengp_domain::domain::patient::Gender;

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

    fn make_patient() -> PatientListItem {
        PatientListItem {
            id: uuid::Uuid::new_v4(),
            full_name: "Clinical Test".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            gender: Gender::Male,
            medicare_number: None,
            medicare_irn: None,
            ihi: None,
            phone_mobile: None,
        }
    }

    #[test]
    fn clinical_new_shortcut_opens_consultation_form_in_workspace() {
        let mut app = make_app();
        let patient = make_patient();
        let _ = app.workspace_manager.open_patient(patient);
        app.clinical_state_mut().show_consultations();

        let action = app.handle_clinical_keys(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));

        assert_eq!(action, Action::Enter);
        assert!(matches!(app.clinical_state_mut().form_view, ClinicalFormView::ConsultationForm));
        assert!(app.clinical_state_mut().consultations.consultation_form.is_some());
    }
}
