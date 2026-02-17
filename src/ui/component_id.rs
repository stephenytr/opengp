//! Component identification system for tui-realm.
//!
//! This module defines the `Id` enum that uniquely identifies each component
//! in the application. Components are mounted to the tui-realm Application
//! using these IDs, allowing for focused rendering and event routing.
//!
//! # Usage
//!
//! ```rust
//! use tuirealm::Application;
//! use crate::ui::component_id::Id;
//!
//! let mut app: Application<Id, Msg, NoUserEvent> = Application::init(...);
//!
//! // Mount components with IDs
//! app.mount(Id::PatientList, Box::new(patient_list), vec![]).unwrap();
//! app.mount(Id::AppointmentCalendar, Box::new(calendar), vec![]).unwrap();
//!
//! // Focus specific component
//! app.active(&Id::PatientList).unwrap();
//! ```

/// Unique identifier for each mounted component.
///
/// Each variant represents a distinct UI component that can be mounted
/// in the tui-realm Application. The enum implements necessary traits
/// for use as a generic parameter in Application<Id, Msg, UserEvent>.
///
/// # Naming Convention
/// - `CamelCase` for all variants
/// - Group related components by prefix (e.g., `Patient*`, `Appointment*`)
/// - Use descriptive names that reflect the component's purpose
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Id {
    // ========================================================================
    // Page-level components (main screens)
    // ========================================================================
    /// Patient list view
    PatientList,

    /// Patient create/edit form
    PatientForm,

    /// Appointment calendar view
    AppointmentCalendar,

    /// Appointment create/edit form
    AppointmentForm,

    /// Clinical records view
    Clinical,

    /// Billing view (placeholder)
    Billing,

    // ========================================================================
    // Shared/Modal components
    // ========================================================================
    /// Help modal showing keyboard shortcuts
    HelpModal,

    /// Search input component (used across views)
    SearchInput,

    /// Confirmation dialog component
    ConfirmDialog,

    /// Error display component
    ErrorDisplay,

    /// Detail view modal
    DetailModal,

    /// Reschedule appointment modal
    RescheduleModal,

    /// Filter selection modal
    FilterModal,

    /// Practitioner selection modal
    PractitionerSelectModal,

    /// Audit history modal
    AuditModal,

    /// Batch operations modal
    BatchModal,

    // ========================================================================
    // Navigation components
    // ========================================================================
    /// Main navigation/tabs component
    Navigation,

    // ========================================================================
    // Clinical sub-components
    // ========================================================================
    /// Patient selector in clinical view
    ClinicalPatientSelector,

    /// Consultation list in clinical view
    ClinicalConsultationList,

    /// Consultation editor in clinical view
    ClinicalConsultationEditor,

    /// Allergy list in clinical view
    ClinicalAllergyList,

    /// Allergy form in clinical view
    ClinicalAllergyForm,

    /// Medical history list
    ClinicalMedicalHistoryList,

    /// Medical history form
    ClinicalMedicalHistoryForm,

    /// Family history list
    ClinicalFamilyHistoryList,

    /// Family history form
    ClinicalFamilyHistoryForm,

    /// Social history editor
    ClinicalSocialHistoryEditor,

    /// Vital signs form
    ClinicalVitalSignsForm,

    /// Patient overview in clinical view
    ClinicalPatientOverview,
}

impl Id {
    /// Get a string representation of the ID for debugging/logging
    pub fn as_str(&self) -> &'static str {
        match self {
            // Pages
            Id::PatientList => "PatientList",
            Id::PatientForm => "PatientForm",
            Id::AppointmentCalendar => "AppointmentCalendar",
            Id::AppointmentForm => "AppointmentForm",
            Id::Clinical => "Clinical",
            Id::Billing => "Billing",

            // Shared
            Id::HelpModal => "HelpModal",
            Id::SearchInput => "SearchInput",
            Id::ConfirmDialog => "ConfirmDialog",
            Id::ErrorDisplay => "ErrorDisplay",
            Id::DetailModal => "DetailModal",
            Id::RescheduleModal => "RescheduleModal",
            Id::FilterModal => "FilterModal",
            Id::PractitionerSelectModal => "PractitionerSelectModal",
            Id::AuditModal => "AuditModal",
            Id::BatchModal => "BatchModal",

            // Navigation
            Id::Navigation => "Navigation",

            // Clinical sub-components
            Id::ClinicalPatientSelector => "ClinicalPatientSelector",
            Id::ClinicalConsultationList => "ClinicalConsultationList",
            Id::ClinicalConsultationEditor => "ClinicalConsultationEditor",
            Id::ClinicalAllergyList => "ClinicalAllergyList",
            Id::ClinicalAllergyForm => "ClinicalAllergyForm",
            Id::ClinicalMedicalHistoryList => "ClinicalMedicalHistoryList",
            Id::ClinicalMedicalHistoryForm => "ClinicalMedicalHistoryForm",
            Id::ClinicalFamilyHistoryList => "ClinicalFamilyHistoryList",
            Id::ClinicalFamilyHistoryForm => "ClinicalFamilyHistoryForm",
            Id::ClinicalSocialHistoryEditor => "ClinicalSocialHistoryEditor",
            Id::ClinicalVitalSignsForm => "ClinicalVitalSignsForm",
            Id::ClinicalPatientOverview => "ClinicalPatientOverview",
        }
    }

    /// Check if this ID represents a modal component
    pub fn is_modal(&self) -> bool {
        matches!(
            self,
            Id::HelpModal
                | Id::ConfirmDialog
                | Id::ErrorDisplay
                | Id::DetailModal
                | Id::RescheduleModal
                | Id::FilterModal
                | Id::PractitionerSelectModal
                | Id::AuditModal
                | Id::BatchModal
        )
    }

    /// Check if this ID represents a page-level component
    pub fn is_page(&self) -> bool {
        matches!(
            self,
            Id::PatientList
                | Id::PatientForm
                | Id::AppointmentCalendar
                | Id::AppointmentForm
                | Id::Clinical
                | Id::Billing
        )
    }

    /// Get the default parent ID for a component
    ///
    /// This is used when mounting components that should be children
    /// of another component in the focus hierarchy.
    pub fn default_parent(&self) -> Option<Id> {
        match self {
            // Modals have no parent (they overlay)
            Id::HelpModal
            | Id::ConfirmDialog
            | Id::ErrorDisplay
            | Id::DetailModal
            | Id::RescheduleModal
            | Id::FilterModal
            | Id::PractitionerSelectModal
            | Id::AuditModal
            | Id::BatchModal => None,

            // Forms are children of their list views
            Id::PatientForm => Some(Id::PatientList),
            Id::AppointmentForm => Some(Id::AppointmentCalendar),

            // Clinical sub-components are children of Clinical
            Id::ClinicalPatientSelector
            | Id::ClinicalConsultationList
            | Id::ClinicalConsultationEditor
            | Id::ClinicalAllergyList
            | Id::ClinicalAllergyForm
            | Id::ClinicalMedicalHistoryList
            | Id::ClinicalMedicalHistoryForm
            | Id::ClinicalFamilyHistoryList
            | Id::ClinicalFamilyHistoryForm
            | Id::ClinicalSocialHistoryEditor
            | Id::ClinicalVitalSignsForm
            | Id::ClinicalPatientOverview => Some(Id::Clinical),

            // Search can be used by multiple pages
            Id::SearchInput => None,

            // Top-level pages and navigation have no parent
            Id::PatientList
            | Id::AppointmentCalendar
            | Id::Clinical
            | Id::Billing
            | Id::Navigation => None,
        }
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_display() {
        assert_eq!(Id::PatientList.as_str(), "PatientList");
        assert_eq!(Id::HelpModal.as_str(), "HelpModal");
    }

    #[test]
    fn test_is_modal() {
        assert!(Id::HelpModal.is_modal());
        assert!(Id::ConfirmDialog.is_modal());
        assert!(!Id::PatientList.is_modal());
        assert!(!Id::Clinical.is_modal());
    }

    #[test]
    fn test_is_page() {
        assert!(Id::PatientList.is_page());
        assert!(Id::Clinical.is_page());
        assert!(!Id::HelpModal.is_page());
        assert!(!Id::SearchInput.is_page());
    }

    #[test]
    fn test_default_parent() {
        assert_eq!(Id::PatientForm.default_parent(), Some(Id::PatientList));
        assert_eq!(
            Id::ClinicalConsultationList.default_parent(),
            Some(Id::Clinical)
        );
        assert!(Id::HelpModal.default_parent().is_none());
    }

    #[test]
    fn test_id_equality() {
        let id1 = Id::PatientList;
        let id2 = Id::PatientList;
        assert_eq!(id1, id2);

        let id1 = Id::HelpModal;
        let id2 = Id::HelpModal;
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Id::PatientList);
        set.insert(Id::Clinical);
        set.insert(Id::PatientList); // duplicate

        assert_eq!(set.len(), 2);
    }
}
