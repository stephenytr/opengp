//! Clinical Component Module
//!
//! Clinical records UI components for patient consultations, allergies,
//! medical history, vital signs, social history, and family history.

pub mod allergy_form;
pub mod allergy_list;
pub mod consultation_detail;
pub mod consultation_list;
pub mod family_history_form;
pub mod family_history_list;
pub mod medical_history_form;
pub mod medical_history_list;
pub mod patient_summary;
pub mod social_history;
pub mod state;
pub mod vitals_form;
pub mod vitals_list;

pub use allergy_form::{AllergyForm, AllergyFormAction};
pub use allergy_list::{AllergyList, AllergyListAction};
pub use consultation_detail::{ConsultationDetail, ConsultationDetailAction};
pub use consultation_list::{ConsultationList, ConsultationListAction};
pub use family_history_form::{FamilyHistoryForm, FamilyHistoryFormAction};
pub use family_history_list::{FamilyHistoryList, FamilyHistoryListAction};
pub use medical_history_form::{MedicalHistoryForm, MedicalHistoryFormAction};
pub use medical_history_list::{MedicalHistoryList, MedicalHistoryListAction};
pub use patient_summary::PatientSummaryComponent;
pub use social_history::{SocialHistoryAction, SocialHistoryComponent};
pub use state::{ClinicalFormView, ClinicalState, ClinicalView};
pub use vitals_form::{VitalSignsForm, VitalSignsFormAction};
pub use vitals_list::{VitalSignsList, VitalSignsListAction};
