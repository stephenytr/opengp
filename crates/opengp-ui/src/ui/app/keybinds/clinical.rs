use crate::ui::app::{App, AppointmentStatusTransition, PendingClinicalSaveData};
use crate::ui::components::tabs::Tab;
use crate::ui::keybinds::Action;
use crate::ui::widgets::FormNavigation;
use crossterm::event::KeyEvent;
use opengp_domain::domain::appointment::AppointmentStatus;

impl App {
    pub(crate) fn handle_clinical_keys(&mut self, key: KeyEvent) -> Action {
        use crate::ui::components::clinical::{
            AllergyFormAction, ClinicalFormView, ClinicalView, ConsultationFormAction,
            FamilyHistoryFormAction, MedicalHistoryFormAction, VitalSignsFormAction,
        };
        use crate::ui::keybinds::{Action as KeyAction, KeyContext, KeybindRegistry};
        use crate::ui::widgets::SearchableListState;
        use crossterm::event::KeyCode;

        if self.clinical_state.is_form_open() {
            match self.clinical_state.form_view.clone() {
                ClinicalFormView::AllergyForm => {
                    if let Some(ref mut form) = self.clinical_state.allergy_form {
                        if let Some(action) = form.handle_key(key) {
                            match action {
                                AllergyFormAction::FocusChanged
                                | AllergyFormAction::ValueChanged => {}
                                AllergyFormAction::Submit => {
                                    let form_valid = form.validate();
                                    if form_valid {
                                        if let Some(patient_id) =
                                            self.clinical_state.selected_patient_id
                                        {
                                            let system_user_id = self.current_user_id;
                                            let allergy =
                                                form.to_allergy(patient_id, system_user_id);
                                            self.pending_clinical_save_data =
                                                Some(PendingClinicalSaveData::Allergy {
                                                    patient_id,
                                                    allergy,
                                                });
                                            self.clinical_state.close_form();
                                            self.current_context = KeyContext::Clinical;
                                            self.status_bar.clear_error();
                                        } else {
                                            self.status_bar
                                                .set_error("Please fill in required fields");
                                        }
                                    } else {
                                        self.status_bar.set_error("Please fill in required fields");
                                    }
                                }
                                AllergyFormAction::Cancel => {
                                    self.clinical_state.close_form();
                                    self.current_context = KeyContext::Clinical;
                                    self.status_bar.clear_error();
                                }
                            }
                            return Action::Enter;
                        }
                    }
                }
                ClinicalFormView::MedicalHistoryForm => {
                    if let Some(ref mut form) = self.clinical_state.medical_history_form {
                        if let Some(action) = form.handle_key(key) {
                            match action {
                                MedicalHistoryFormAction::FocusChanged
                                | MedicalHistoryFormAction::ValueChanged => {}
                                MedicalHistoryFormAction::Submit => {
                                    if let Some(patient_id) =
                                        self.clinical_state.selected_patient_id
                                    {
                                        let system_user_id = self.current_user_id;
                                        if let Some(history) =
                                            form.to_medical_history(patient_id, system_user_id)
                                        {
                                            self.pending_clinical_save_data =
                                                Some(PendingClinicalSaveData::MedicalHistory {
                                                    patient_id,
                                                    history,
                                                });
                                            self.clinical_state.close_form();
                                            self.current_context = KeyContext::Clinical;
                                            self.status_bar.clear_error();
                                        } else {
                                            self.status_bar
                                                .set_error("Please fill in required fields");
                                        }
                                    } else {
                                        self.status_bar.set_error("Please fill in required fields");
                                    }
                                }
                                MedicalHistoryFormAction::Cancel => {
                                    self.clinical_state.close_form();
                                    self.current_context = KeyContext::Clinical;
                                    self.status_bar.clear_error();
                                }
                            }
                            return Action::Enter;
                        }
                    }
                }
                ClinicalFormView::VitalSignsForm => {
                    if let Some(ref mut form) = self.clinical_state.vitals_form {
                        if let Some(action) = form.handle_key(key) {
                            match action {
                                VitalSignsFormAction::FocusChanged
                                | VitalSignsFormAction::ValueChanged => {}
                                VitalSignsFormAction::Submit => {
                                    let form_valid = form.validate();
                                    if form_valid {
                                        if let Some(patient_id) =
                                            self.clinical_state.selected_patient_id
                                        {
                                            let system_user_id = self.current_user_id;
                                            let vitals =
                                                form.to_vital_signs(patient_id, system_user_id);
                                            self.pending_clinical_save_data =
                                                Some(PendingClinicalSaveData::VitalSigns {
                                                    patient_id,
                                                    vitals,
                                                });
                                            self.clinical_state.close_form();
                                            self.current_context = KeyContext::Clinical;
                                            self.status_bar.clear_error();
                                        } else {
                                            self.status_bar
                                                .set_error("Please fill in required fields");
                                        }
                                    } else {
                                        self.status_bar.set_error("Please fill in required fields");
                                    }
                                }
                                VitalSignsFormAction::Cancel => {
                                    self.clinical_state.close_form();
                                    self.current_context = KeyContext::Clinical;
                                    self.status_bar.clear_error();
                                }
                            }
                            return Action::Enter;
                        }
                    }
                }
                ClinicalFormView::FamilyHistoryForm => {
                    if let Some(ref mut form) = self.clinical_state.family_history_form {
                        if let Some(action) = form.handle_key(key) {
                            match action {
                                FamilyHistoryFormAction::FocusChanged
                                | FamilyHistoryFormAction::ValueChanged => {}
                                FamilyHistoryFormAction::Submit => {
                                    let form_valid = form.validate();
                                    if form_valid {
                                        if let Some(patient_id) =
                                            self.clinical_state.selected_patient_id
                                        {
                                            let system_user_id = self.current_user_id;
                                            let entry =
                                                form.to_family_history(patient_id, system_user_id);
                                            self.pending_clinical_save_data =
                                                Some(PendingClinicalSaveData::FamilyHistory {
                                                    patient_id,
                                                    entry,
                                                });
                                            self.clinical_state.close_form();
                                            self.current_context = KeyContext::Clinical;
                                            self.status_bar.clear_error();
                                        } else {
                                            self.status_bar
                                                .set_error("Please fill in required fields");
                                        }
                                    } else {
                                        self.status_bar.set_error("Please fill in required fields");
                                    }
                                }
                                FamilyHistoryFormAction::Cancel => {
                                    self.clinical_state.close_form();
                                    self.current_context = KeyContext::Clinical;
                                    self.status_bar.clear_error();
                                }
                            }
                            return Action::Enter;
                        }
                    }
                }
                ClinicalFormView::ConsultationForm => {
                    if let Some(ref mut form) = self.clinical_state.consultation_form {
                        if let Some(action) = form.handle_key(key) {
                            match action {
                                ConsultationFormAction::FocusChanged
                                | ConsultationFormAction::ValueChanged => {}
                                ConsultationFormAction::Submit => {
                                    let form_valid = form.validate();
                                    if form_valid {
                                        if let Some(patient_id) =
                                            self.clinical_state.selected_patient_id
                                        {
                                            let system_user_id = self.current_user_id;
                                            let practitioner_id = uuid::Uuid::nil();
                                            let consultation = form.to_consultation(
                                                patient_id,
                                                practitioner_id,
                                                system_user_id,
                                            );
                                            self.pending_clinical_save_data =
                                                Some(PendingClinicalSaveData::Consultation {
                                                    patient_id,
                                                    practitioner_id,
                                                    appointment_id: consultation.appointment_id,
                                                    reason: consultation.reason,
                                                    clinical_notes: consultation.clinical_notes,
                                                });
                                            self.clinical_state.close_consultation_form();
                                            self.current_context = KeyContext::Clinical;
                                            self.status_bar.clear_error();
                                        } else {
                                            self.status_bar.set_error("No patient selected");
                                        }
                                    } else {
                                        self.status_bar.set_error("Please fill in required fields");
                                    }
                                }
                                ConsultationFormAction::Cancel => {
                                    self.clinical_state.close_consultation_form();
                                    self.current_context = KeyContext::Clinical;
                                    self.status_bar.clear_error();
                                }
                            }
                            return Action::Enter;
                        }
                    }
                }
                ClinicalFormView::SocialHistoryForm => {
                    use crate::ui::components::clinical::SocialHistoryAction;
                    if let Some(ref mut form) = self.clinical_state.social_history_form {
                        if let Some(action) = form.handle_key(key) {
                            match action {
                                SocialHistoryAction::Edit => {
                                    form.start_editing();
                                }
                                SocialHistoryAction::Save => {
                                    if let Some(patient_id) =
                                        self.clinical_state.selected_patient_id
                                    {
                                        let system_user_id = self.current_user_id;
                                        let social_history_data =
                                            form.to_social_history(patient_id, system_user_id);
                                        self.pending_clinical_save_data =
                                            Some(PendingClinicalSaveData::SocialHistory {
                                                patient_id,
                                                history:
                                                    opengp_domain::domain::clinical::SocialHistory {
                                                        id: uuid::Uuid::new_v4(),
                                                        patient_id,
                                                        smoking_status: social_history_data
                                                            .smoking_status,
                                                        cigarettes_per_day: social_history_data
                                                            .cigarettes_per_day,
                                                        smoking_quit_date: social_history_data
                                                            .smoking_quit_date,
                                                        alcohol_status: social_history_data
                                                            .alcohol_status,
                                                        standard_drinks_per_week:
                                                            social_history_data
                                                                .standard_drinks_per_week,
                                                        exercise_frequency: social_history_data
                                                            .exercise_frequency,
                                                        occupation: social_history_data.occupation,
                                                        living_situation: social_history_data
                                                            .living_situation,
                                                        support_network: social_history_data
                                                            .support_network,
                                                        notes: social_history_data.notes,
                                                        updated_by: system_user_id,
                                                        updated_at:
                                                            chrono::DateTime::<chrono::Utc>::from(
                                                                std::time::SystemTime::now(),
                                                            ),
                                                    },
                                            });
                                        self.clinical_state.close_social_history_form();
                                        self.current_context = KeyContext::Clinical;
                                        self.status_bar.clear_error();
                                    }
                                }
                                SocialHistoryAction::Cancel => {
                                    self.clinical_state.close_social_history_form();
                                    self.current_context = KeyContext::Clinical;
                                    self.status_bar.clear_error();
                                }
                                SocialHistoryAction::FieldChanged
                                | SocialHistoryAction::FocusChanged => {}
                            }
                            return Action::Enter;
                        }
                    }
                }
                ClinicalFormView::None => {}
            }
            return Action::Unknown;
        }

        if let Some(ref mut search) = self.clinical_state.patient_search {
            if search.is_open() {
                use crate::ui::widgets::{SearchableList, SearchableListAction};
                let mut picker = SearchableList::new(search, &self.theme, "Select Patient", true);
                let action = picker.handle_key(key);
                match action {
                    SearchableListAction::Selected(id, _name) => {
                        self.clinical_state.set_patient(id);
                        self.clinical_state.patient_search = None;
                        return Action::Enter;
                    }
                    SearchableListAction::Cancelled => {
                        self.clinical_state.patient_search = None;
                        return Action::Enter;
                    }
                    SearchableListAction::None => {
                        return Action::Enter;
                    }
                }
            }
        }

        let registry = KeybindRegistry::global();
        if let Some(keybind) = registry.lookup(key, KeyContext::Clinical) {
            match keybind.action {
                KeyAction::SwitchToPatientSummary => {
                    self.clinical_state.show_patient_summary();
                    return Action::Enter;
                }
                KeyAction::SwitchToConsultations => {
                    self.clinical_state.show_consultations();
                    return Action::Enter;
                }
                KeyAction::SwitchToAllergies => {
                    self.clinical_state.show_allergies();
                    return Action::Enter;
                }
                KeyAction::SwitchToMedicalHistory => {
                    self.clinical_state.show_medical_history();
                    return Action::Enter;
                }
                KeyAction::SwitchToVitalSigns => {
                    self.clinical_state.show_vital_signs();
                    return Action::Enter;
                }
                KeyAction::SwitchToSocialHistory => {
                    self.clinical_state.show_social_history();
                    return Action::Enter;
                }
                KeyAction::SwitchToFamilyHistory => {
                    self.clinical_state.show_family_history();
                    return Action::Enter;
                }
                KeyAction::ViewAllergies => {
                    self.clinical_state.show_allergies();
                    return Action::Enter;
                }
                KeyAction::ViewConditions => {
                    self.clinical_state.show_medical_history();
                    return Action::Enter;
                }
                KeyAction::ViewVitals => {
                    self.clinical_state.show_vital_signs();
                    return Action::Enter;
                }
                KeyAction::ViewObservations => {
                    self.clinical_state.show_consultations();
                    return Action::Enter;
                }
                KeyAction::ViewFamilyHistory => {
                    self.clinical_state.show_family_history();
                    return Action::Enter;
                }
                KeyAction::ViewSocialHistory => {
                    self.clinical_state.show_social_history();
                    return Action::Enter;
                }
                KeyAction::Search => {
                    if self.clinical_state.patient_search.is_none()
                        && !self.clinical_state.is_form_open()
                    {
                        let patients: Vec<_> = self
                            .patient_list
                            .patients()
                            .iter()
                            .map(|p| crate::ui::view_models::PatientListItem {
                                id: p.id,
                                full_name: p.full_name.clone(),
                                date_of_birth: p.date_of_birth,
                                gender: p.gender,
                                medicare_number: p.medicare_number.clone(),
                                medicare_irn: p.medicare_irn,
                                ihi: p.ihi.clone(),
                                phone_mobile: p.phone_mobile.clone(),
                            })
                            .collect();
                        let mut search = SearchableListState::new(patients);
                        search.open();
                        self.clinical_state.patient_search = Some(search);
                    }
                    return Action::Enter;
                }
                KeyAction::FinishAppointment => {
                    if let Some(appointment_id) = self.clinical_state.active_appointment_id {
                        self.pending_appointment_status_transition = Some((
                            appointment_id,
                            AppointmentStatusTransition::SetStatus(AppointmentStatus::Completed),
                        ));
                        self.clinical_state.clear_active_appointment();
                        self.tab_bar.select(Tab::Appointment);
                        self.request_refresh_appointments(chrono::Utc::now().date_naive());
                        self.refresh_status_bar();
                        self.refresh_context();
                        return Action::Enter;
                    }
                }
                KeyAction::ToggleTimer => {
                    let consultation_id = if self.clinical_state.view == ClinicalView::Consultations
                    {
                        self.clinical_state.consultation_list.selected_id()
                    } else if self.clinical_state.view == ClinicalView::ConsultationSummary {
                        self.clinical_state
                            .active_appointment_id
                            .and_then(|appointment_id| {
                                self.clinical_state
                                    .consultations
                                    .iter()
                                    .find(|c| c.appointment_id == Some(appointment_id))
                                    .map(|c| c.id)
                            })
                    } else {
                        None
                    };

                    if let Some(consultation_id) = consultation_id {
                        if let Some(consultation) = self
                            .clinical_state
                            .consultations
                            .iter_mut()
                            .find(|c| c.id == consultation_id)
                        {
                            let is_running = consultation.consultation_started_at.is_some()
                                && consultation.consultation_ended_at.is_none();

                            if is_running {
                                consultation.consultation_ended_at = Some(chrono::Utc::now());
                                self.pending_clinical_save_data =
                                    Some(PendingClinicalSaveData::TimerStop { consultation_id });
                            } else {
                                consultation.consultation_started_at = Some(chrono::Utc::now());
                                consultation.consultation_ended_at = None;
                                self.pending_clinical_save_data =
                                    Some(PendingClinicalSaveData::TimerStart { consultation_id });
                            }
                        }
                        return Action::Enter;
                    }
                }
                _ => {}
            }
        }

        if key.code == KeyCode::Right {
            self.clinical_state.cycle_view();
            return Action::Enter;
        }
        if key.code == KeyCode::Left {
            self.clinical_state.cycle_view_reverse();
            return Action::Enter;
        }

        match self.clinical_state.view {
            ClinicalView::PatientSummary => {}
            ClinicalView::ConsultationSummary => {}
            ClinicalView::Consultations => {
                if let Some(action) = self.clinical_state.consultation_list.handle_key(key) {
                    match action {
                        crate::ui::components::clinical::ConsultationListAction::Select(_)
                        | crate::ui::components::clinical::ConsultationListAction::Open(_)
                        | crate::ui::components::clinical::ConsultationListAction::New
                        | crate::ui::components::clinical::ConsultationListAction::NextPage
                        | crate::ui::components::clinical::ConsultationListAction::PrevPage => {
                            return Action::Enter;
                        }
                    }
                }
            }
            ClinicalView::Allergies => {
                if let Some(action) = self.clinical_state.allergy_list.handle_key(key) {
                    match action {
                        crate::ui::components::clinical::AllergyListAction::New => {
                            self.clinical_state.open_allergy_form();
                            self.current_context = KeyContext::ClinicalForm;
                            return Action::Enter;
                        }
                        crate::ui::components::clinical::AllergyListAction::Select(_)
                        | crate::ui::components::clinical::AllergyListAction::Open(_)
                        | crate::ui::components::clinical::AllergyListAction::ToggleInactive
                        | crate::ui::components::clinical::AllergyListAction::Delete(_) => {
                            return Action::Enter;
                        }
                    }
                }
            }
            ClinicalView::MedicalHistory => {
                if let Some(action) = self.clinical_state.medical_history_list.handle_key(key) {
                    match action {
                        crate::ui::components::clinical::MedicalHistoryListAction::New => {
                            self.clinical_state.open_medical_history_form();
                            self.current_context = KeyContext::ClinicalForm;
                            return Action::Enter;
                        }
                        crate::ui::components::clinical::MedicalHistoryListAction::Select(_)
                        | crate::ui::components::clinical::MedicalHistoryListAction::Open(_)
                        | crate::ui::components::clinical::MedicalHistoryListAction::Edit(_)
                        | crate::ui::components::clinical::MedicalHistoryListAction::Delete(_) => {
                            return Action::Enter;
                        }
                        crate::ui::components::clinical::MedicalHistoryListAction::ToggleInactive => {
                            return Action::Enter;
                        }
                    }
                }
            }
            ClinicalView::VitalSigns => {
                if let Some(action) = self.clinical_state.vitals_list.handle_key(key) {
                    match action {
                        crate::ui::components::clinical::VitalSignsListAction::New => {
                            self.clinical_state.open_vitals_form();
                            self.current_context = KeyContext::ClinicalForm;
                            return Action::Enter;
                        }
                        crate::ui::components::clinical::VitalSignsListAction::Select(_)
                        | crate::ui::components::clinical::VitalSignsListAction::Open(_)
                        | crate::ui::components::clinical::VitalSignsListAction::NextPage
                        | crate::ui::components::clinical::VitalSignsListAction::PrevPage => {
                            return Action::Enter;
                        }
                    }
                }
            }
            ClinicalView::SocialHistory => {
                if let KeyCode::Char(c) = key.code {
                    if c == 'e' || c == 'n' {
                        self.clinical_state.open_social_history_form();
                        self.current_context = KeyContext::ClinicalForm;
                        return Action::Enter;
                    }
                }
            }
            ClinicalView::FamilyHistory => {
                if let Some(action) = self.clinical_state.family_history_list.handle_key(key) {
                    match action {
                        crate::ui::components::clinical::FamilyHistoryListAction::New => {
                            self.clinical_state.open_family_history_form();
                            self.current_context = KeyContext::ClinicalForm;
                            return Action::Enter;
                        }
                        crate::ui::components::clinical::FamilyHistoryListAction::Select(_)
                        | crate::ui::components::clinical::FamilyHistoryListAction::Open(_)
                        | crate::ui::components::clinical::FamilyHistoryListAction::Delete(_) => {
                            return Action::Enter;
                        }
                    }
                }
            }
        }

        Action::Unknown
    }
}
