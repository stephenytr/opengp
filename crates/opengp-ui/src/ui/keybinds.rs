//! OpenGP Keybind System
//!
//! Centralized keybind management with context-specific bindings.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Key context for context-specific keybinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum KeyContext {
    /// Global keybinds that work everywhere
    #[default]
    Global,
    /// Patient list screen
    PatientList,
    /// Patient form screen
    PatientForm,
    /// Patient workspace (open clinical record with tabs)
    PatientWorkspace,
    /// Patient subtab navigation (within workspace)
    PatientSubtab,
    /// Appointment calendar
    Calendar,
    /// Appointment schedule
    Schedule,
    /// Search/modal dialogs
    Search,
    /// Help overlay
    Help,
    /// Billing subtab (within patient workspace)
    Billing,
    /// Billing form/detail overlays
    BillingForm,
    /// Clinical sub-view navigation (consultations, allergies, etc.)
    ClinicalSubView,
    /// Tab bar focus
    TabBar,
}

/// Actions that can be triggered by keybinds
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Action {
    // Global actions
    /// Quit the application
    Quit,
    /// Open help overlay
    OpenHelp,
    /// Create new item (patient, appointment, etc.)
    New,
    /// Focus search input
    Search,
    /// Refresh current view
    Refresh,
    /// Open settings
    Settings,

    // Navigation actions
    /// Navigate up
    NavigateUp,
    /// Navigate down
    NavigateDown,
    /// Navigate left
    NavigateLeft,
    /// Navigate right
    NavigateRight,
    /// Go to previous item
    PrevItem,
    /// Go to next item
    NextItem,
    /// Go to previous page
    PrevPage,
    /// Go to next page
    NextPage,
    /// Go to first item
    Home,
    /// Go to last item
    End,

    // Focus actions
    /// Move to next focusable element
    TabNext,
    /// Move to previous focusable element
    TabPrev,
    /// Focus next component
    FocusNext,
    /// Focus previous component
    FocusPrev,

    // Interaction actions
    /// Select/confirm current item
    Enter,
    /// Cancel/go back
    Escape,
    /// Delete current item
    Delete,
    /// Edit current item
    Edit,
    /// Toggle selection
    Toggle,
    /// Submit form
    Submit,
    /// Clear input
    Clear,
    /// Start/stop consultation timer
    ToggleTimer,
    /// Finish appointment and return to appointments tab
    FinishAppointment,

    // Tab actions
    /// Switch to Schedule tab
    SwitchToSchedule,
    /// Switch to Patient Search tab
    SwitchToPatientSearch,
    /// Close current patient tab
    ClosePatientTab,
    /// Switch to next patient tab
    NextPatientTab,
    /// Switch to previous patient tab
    PrevPatientTab,
    /// Select specific patient tab by index
    SelectPatientTab(usize),

    // Clinical menu actions
    /// Navigate to next clinical menu item
    NextClinicalMenu,
    /// Navigate to previous clinical menu item
    PrevClinicalMenu,
    /// Open patient from list (clinical record)
    OpenPatientFromList,

    // Calendar actions
    /// Go to previous day
    PrevDay,
    /// Go to next day
    NextDay,
    /// Go to previous week
    PrevWeek,
    /// Go to next week
    NextWeek,
    /// Go to previous month
    PrevMonth,
    /// Go to next month
    NextMonth,
    /// Go to today
    Today,
    /// Select date
    SelectDate,

    // Appointment actions
    /// Move to previous practitioner column
    PrevPractitioner,
    /// Move to next practitioner column
    NextPractitioner,
    /// Move to previous time slot
    PrevTimeSlot,
    /// Move to next time slot
    NextTimeSlot,
    /// Toggle selected practitioner column visibility
    TogglePractitionerColumn,
    /// Select appointment
    SelectAppointment,
    /// Create new appointment
    NewAppointment,
    /// Mark selected appointment as No Show
    MarkNoShow,

    // Form actions
    /// Move to next field
    NextField,
    /// Move to previous field
    PrevField,
    /// Save form
    Save,
    /// Validate form
    Validate,

    // Clinical view actions (number keys 1-7)
    /// Switch to Patient Summary view
    SwitchToPatientSummary,
    /// Switch to Consultations view
    SwitchToConsultations,
    /// Switch to Allergies view
    SwitchToAllergies,
    /// Switch to Medical History view
    SwitchToMedicalHistory,
    /// Switch to Vital Signs view
    SwitchToVitalSigns,
    /// Switch to Social History view
    SwitchToSocialHistory,
    /// Switch to Family History view
    SwitchToFamilyHistory,

    // Clinical quick actions (letter keys)
    /// View allergies
    ViewAllergies,
    /// View conditions (medical history)
    ViewConditions,
    /// View vital signs
    ViewVitals,
    /// View observations (consultations)
    ViewObservations,
    /// View family history
    ViewFamilyHistory,
    /// View social history
    ViewSocialHistory,

    // Schedule viewport actions
    /// Scroll viewport up (earlier hours)
    ScrollViewportUp,
    /// Scroll viewport down (later hours)
    ScrollViewportDown,

    // Billing actions
    /// Cycle to next billing view (Invoice → Claim → Payment)
    NextBillingView,
    /// Cycle to previous billing view
    PrevBillingView,
    /// Create new invoice
    NewInvoice,
    /// Edit invoice
    EditInvoice,
    /// Process payment
    ProcessPayment,
    /// Void invoice
    VoidInvoice,
    /// Generate receipt
    GenerateReceipt,

    // Clinical sub-view actions
    /// Toggle consultation timer
    ToggleConsultationTimer,

    // Tab bar actions
    /// Go to first tab
    TabBarHome,
    /// Go to last tab
    TabBarEnd,
    /// Confirm tab selection
    TabBarConfirm,

    // Unknown action (fallback)
    #[default]
    Unknown,
}

/// A keybind definition
#[derive(Debug, Clone)]
pub struct Keybind {
    /// The key event that triggers this action
    pub key: KeyEvent,
    /// The action to perform
    pub action: Action,
    /// The context where this keybind is active
    pub context: KeyContext,
    /// Human-readable description of the action
    pub description: &'static str,
}

/// Keybind registry for looking up keybinds by key and context
#[derive(Debug, Default)]
pub struct KeybindRegistry {
    /// Global keybinds
    global: Vec<Keybind>,
    /// Context-specific keybinds
    context: HashMap<KeyContext, Vec<Keybind>>,
    /// Reverse lookup: context -> action -> keybind
    action_lookup: HashMap<(KeyContext, Action), Keybind>,
}

impl KeybindRegistry {
    /// Create a new keybind registry with default keybinds
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_defaults();
        registry
    }

    /// Register all default keybinds
    fn register_defaults(&mut self) {
        // Global keybinds
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
            action: Action::OpenHelp,
            context: KeyContext::Global,
            description: "Open help overlay",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
            action: Action::Quit,
            context: KeyContext::Global,
            description: "Quit application",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            action: Action::Quit,
            context: KeyContext::Global,
            description: "Quit application",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE),
            action: Action::SwitchToSchedule,
            context: KeyContext::Global,
            description: "Switch to Schedule view",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE),
            action: Action::SwitchToPatientSearch,
            context: KeyContext::Global,
            description: "Switch to Patient Search view",
        });

        // ── Schedule keybinds ─────────────────────────────────────────────────
        // Arrow keys — time slot and practitioner navigation
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            action: Action::PrevTimeSlot,
            context: KeyContext::Schedule,
            description: "Move to previous time slot",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            action: Action::NextTimeSlot,
            context: KeyContext::Schedule,
            description: "Move to next time slot",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            action: Action::PrevPractitioner,
            context: KeyContext::Schedule,
            description: "Move to previous practitioner column",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            action: Action::NextPractitioner,
            context: KeyContext::Schedule,
            description: "Move to next practitioner column",
        });
        // Vim-style navigation
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
            action: Action::PrevTimeSlot,
            context: KeyContext::Schedule,
            description: "Move to previous time slot",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            action: Action::NextTimeSlot,
            context: KeyContext::Schedule,
            description: "Move to next time slot",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            action: Action::PrevPractitioner,
            context: KeyContext::Schedule,
            description: "Move to previous practitioner column",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
            action: Action::NextPractitioner,
            context: KeyContext::Schedule,
            description: "Move to next practitioner column",
        });
        // Viewport scrolling
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE),
            action: Action::ScrollViewportUp,
            context: KeyContext::Schedule,
            description: "Scroll viewport up (earlier hours)",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
            action: Action::ScrollViewportDown,
            context: KeyContext::Schedule,
            description: "Scroll viewport down (later hours)",
        });
        // Interaction
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            action: Action::Enter,
            context: KeyContext::Schedule,
            description: "Select appointment or create new at slot",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            action: Action::NewAppointment,
            context: KeyContext::Schedule,
            description: "Create new appointment",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
            action: Action::TogglePractitionerColumn,
            context: KeyContext::Schedule,
            description: "Toggle selected practitioner column visibility",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            action: Action::Escape,
            context: KeyContext::Schedule,
            description: "Return to calendar view",
        });

        // ── Calendar keybinds ─────────────────────────────────────────────────
        // Day navigation
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            action: Action::PrevDay,
            context: KeyContext::Calendar,
            description: "Go to previous day",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            action: Action::NextDay,
            context: KeyContext::Calendar,
            description: "Go to next day",
        });
        // Week navigation
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            action: Action::PrevWeek,
            context: KeyContext::Calendar,
            description: "Go to previous week",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            action: Action::NextWeek,
            context: KeyContext::Calendar,
            description: "Go to next week",
        });
        // Month navigation
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE),
            action: Action::PrevMonth,
            context: KeyContext::Calendar,
            description: "Go to previous month",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
            action: Action::NextMonth,
            context: KeyContext::Calendar,
            description: "Go to next month",
        });
        // Select date / go to today
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            action: Action::Enter,
            context: KeyContext::Calendar,
            description: "Select focused date",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
            action: Action::Today,
            context: KeyContext::Calendar,
            description: "Go to today",
        });

        // ── Patient list keybinds ─────────────────────────────────────────────
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            action: Action::NavigateUp,
            context: KeyContext::PatientList,
            description: "Move selection up",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            action: Action::NavigateDown,
            context: KeyContext::PatientList,
            description: "Move selection down",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
            action: Action::NavigateUp,
            context: KeyContext::PatientList,
            description: "Move selection up",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            action: Action::NavigateDown,
            context: KeyContext::PatientList,
            description: "Move selection down",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Home, KeyModifiers::NONE),
            action: Action::Home,
            context: KeyContext::PatientList,
            description: "Go to first patient",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::End, KeyModifiers::NONE),
            action: Action::End,
            context: KeyContext::PatientList,
            description: "Go to last patient",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE),
            action: Action::PrevPage,
            context: KeyContext::PatientList,
            description: "Page up",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
            action: Action::NextPage,
            context: KeyContext::PatientList,
            description: "Page down",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            action: Action::Enter,
            context: KeyContext::PatientList,
            description: "Open selected patient",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
            action: Action::Search,
            context: KeyContext::PatientList,
            description: "Focus patient search",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            action: Action::New,
            context: KeyContext::PatientList,
            description: "Create new patient",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            action: Action::Edit,
            context: KeyContext::PatientList,
            description: "Edit selected patient",
        });
    }

    /// Register a keybind
    pub fn register(&mut self, keybind: Keybind) {
        let context = keybind.context;
        if context == KeyContext::Global {
            self.global.push(keybind.clone());
        } else {
            self.context
                .entry(context)
                .or_default()
                .push(keybind.clone());
        }
        self.action_lookup
            .insert((context, keybind.action.clone()), keybind);
    }

    /// Look up a keybind by key and context
    pub fn lookup(&self, key: KeyEvent, context: KeyContext) -> Option<&Keybind> {
        // First check context-specific keybinds
        if let Some(keybinds) = self.context.get(&context) {
            for kb in keybinds {
                if kb.key == key {
                    return Some(kb);
                }
            }
        }

        // Fall back to global keybinds
        self.global.iter().find(|kb| kb.key == key)
    }

    /// Get all keybinds for a context (including global)
    pub fn get_keybinds_for_context(&self, context: KeyContext) -> Vec<&Keybind> {
        let mut keybinds = Vec::new();

        // Add context-specific keybinds
        if let Some(context_kbs) = self.context.get(&context) {
            keybinds.extend(context_kbs);
        }

        // Add global keybinds
        keybinds.extend(&self.global);

        keybinds
    }

    /// Get keybind description for an action in a context
    pub fn get_description(&self, action: Action, context: KeyContext) -> Option<&'static str> {
        self.action_lookup
            .get(&(context, action.clone()))
            .or_else(|| self.action_lookup.get(&(KeyContext::Global, action)))
            .map(|kb| kb.description)
    }

    /// Get the global singleton keybind registry
    pub fn global() -> &'static KeybindRegistry {
        &DEFAULT_REGISTRY
    }
}

/// Global keybind registry singleton - lazily initialized
static DEFAULT_REGISTRY: LazyLock<KeybindRegistry> = LazyLock::new(KeybindRegistry::new);

/// Helper to create key events
#[macro_export]
macro_rules! key {
    (ctrl, $c:expr) => {
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char($c),
            crossterm::event::KeyModifiers::CONTROL,
        )
    };
    (alt, $c:expr) => {
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char($c),
            crossterm::event::KeyModifiers::ALT,
        )
    };
    (shift, $c:expr) => {
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char($c),
            crossterm::event::KeyModifiers::SHIFT,
        )
    };
    ($c:expr) => {
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char($c),
            crossterm::event::KeyModifiers::NONE,
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybind_lookup_global() {
        let registry = KeybindRegistry::new();
        let key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        let result = registry.lookup(key, KeyContext::Global);
        assert!(result.is_some());
        assert_eq!(result.unwrap().action, Action::OpenHelp);
    }



    #[test]
    fn test_keybind_lookup_fallback_to_global() {
        let registry = KeybindRegistry::new();
        let key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        // Should find global keybind even in different context
        let result = registry.lookup(key, KeyContext::PatientList);
        assert!(result.is_some());
    }

    #[test]
    fn test_get_keybinds_for_context() {
        let registry = KeybindRegistry::new();
        let keybinds = registry.get_keybinds_for_context(KeyContext::Global);
        assert!(!keybinds.is_empty());
    }

    // RED tests for new workspace navigation variants
    #[test]
    fn test_new_action_variants_exist() {
        // Verify new Action variants can be constructed
        let _ = Action::ClosePatientTab;
        let _ = Action::NextPatientTab;
        let _ = Action::PrevPatientTab;
        let _ = Action::SelectPatientTab(1);
        let _ = Action::NextClinicalMenu;
        let _ = Action::PrevClinicalMenu;
        let _ = Action::OpenPatientFromList;
    }

    #[test]
    fn test_new_key_context_variants_exist() {
        // Verify new KeyContext variants can be constructed
        let _ = KeyContext::PatientWorkspace;
        let _ = KeyContext::PatientSubtab;
    }



    #[test]
    fn test_billing_context_variants_exist() {
        let _ = KeyContext::Billing;
        let _ = KeyContext::BillingForm;
        let _ = Action::NextBillingView;
        let _ = Action::PrevBillingView;
        let _ = Action::NewInvoice;
        let _ = Action::EditInvoice;
        let _ = Action::ProcessPayment;
        let _ = Action::VoidInvoice;
        let _ = Action::GenerateReceipt;
    }

    #[test]
    fn test_clinical_subview_context_variants_exist() {
        let _ = KeyContext::ClinicalSubView;
        let _ = Action::ToggleConsultationTimer;
    }

    #[test]
    fn test_tab_bar_context_variants_exist() {
        let _ = KeyContext::TabBar;
        let _ = Action::TabBarHome;
        let _ = Action::TabBarEnd;
        let _ = Action::TabBarConfirm;
    }


}
