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
    /// Appointment calendar
    Calendar,
    /// Appointment schedule
    Schedule,
    /// Clinical notes
    Clinical,
    /// Clinical form (allergy, medical history, vitals, family history)
    ClinicalForm,
    /// Billing screen
    Billing,
    /// Search/modal dialogs
    Search,
    /// Help overlay
    Help,
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

    // Tab actions
    /// Switch to Patient tab
    SwitchToPatient,
    /// Switch to Appointments tab
    SwitchToAppointments,
    /// Switch to Clinical tab
    SwitchToClinical,
    /// Switch to Billing tab
    SwitchToBilling,

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
    /// Select appointment
    SelectAppointment,
    /// Create new appointment
    NewAppointment,

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
            key: KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
            action: Action::New,
            context: KeyContext::Global,
            description: "Create new item",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
            action: Action::Search,
            context: KeyContext::Global,
            description: "Focus search input",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL),
            action: Action::Refresh,
            context: KeyContext::Global,
            description: "Refresh current view",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            action: Action::TabNext,
            context: KeyContext::Global,
            description: "Move to next focusable element",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT),
            action: Action::TabPrev,
            context: KeyContext::Global,
            description: "Move to previous focusable element",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            action: Action::Escape,
            context: KeyContext::Global,
            description: "Go back / Cancel",
        });

        // Tab switching
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE),
            action: Action::SwitchToPatient,
            context: KeyContext::Global,
            description: "Switch to Patient tab",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE),
            action: Action::SwitchToAppointments,
            context: KeyContext::Global,
            description: "Switch to Appointments tab",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::F(4), KeyModifiers::NONE),
            action: Action::SwitchToClinical,
            context: KeyContext::Global,
            description: "Switch to Clinical tab",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE),
            action: Action::SwitchToBilling,
            context: KeyContext::Global,
            description: "Switch to Billing tab",
        });

        // Navigation keybinds (work in most contexts)
        for context in &[
            KeyContext::PatientList,
            KeyContext::Calendar,
            KeyContext::Search,
        ] {
            self.register(Keybind {
                key: KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
                action: Action::NavigateDown,
                context: *context,
                description: "Move selection down",
            });
            self.register(Keybind {
                key: KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
                action: Action::NavigateUp,
                context: *context,
                description: "Move selection up",
            });
            self.register(Keybind {
                key: KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                action: Action::NavigateDown,
                context: *context,
                description: "Move selection down",
            });
            self.register(Keybind {
                key: KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
                action: Action::NavigateUp,
                context: *context,
                description: "Move selection up",
            });
        }

        // Schedule-specific: j/k for time slot navigation
        // This overrides the shared NavigateUp/Down mappings for Schedule context
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            action: Action::NextTimeSlot,
            context: KeyContext::Schedule,
            description: "Move to next time slot",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
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
            key: KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            action: Action::PrevTimeSlot,
            context: KeyContext::Schedule,
            description: "Move to previous time slot",
        });

        // Calendar-specific keybinds
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            action: Action::PrevDay,
            context: KeyContext::Calendar,
            description: "Go to previous day",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
            action: Action::NextDay,
            context: KeyContext::Calendar,
            description: "Go to next day",
        });
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
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
            action: Action::Today,
            context: KeyContext::Calendar,
            description: "Go to today",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            action: Action::PrevMonth,
            context: KeyContext::Calendar,
            description: "Go to previous month",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('M'), KeyModifiers::SHIFT),
            action: Action::NextMonth,
            context: KeyContext::Calendar,
            description: "Go to next month",
        });
        // Calendar: Enter selects focused date
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            action: Action::Enter,
            context: KeyContext::Calendar,
            description: "Select focused date",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            action: Action::NewAppointment,
            context: KeyContext::Calendar,
            description: "Create new appointment",
        });

        // Patient list keybinds
        // Note: '/' is handled directly in PatientList for search input
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
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            action: Action::Delete,
            context: KeyContext::PatientList,
            description: "Delete selected patient",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            action: Action::Enter,
            context: KeyContext::PatientList,
            description: "Open patient clinical record",
        });

        // Patient form keybinds
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            action: Action::Save,
            context: KeyContext::PatientForm,
            description: "Save patient",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            action: Action::Escape,
            context: KeyContext::PatientForm,
            description: "Cancel and go back",
        });

        // Schedule keybinds
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            action: Action::NewAppointment,
            context: KeyContext::Schedule,
            description: "Create new appointment",
        });
        // Schedule horizontal navigation (h/l for practitioner columns)
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            action: Action::PrevPractitioner,
            context: KeyContext::Schedule,
            description: "Go to previous practitioner",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
            action: Action::NextPractitioner,
            context: KeyContext::Schedule,
            description: "Go to next practitioner",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            action: Action::PrevPractitioner,
            context: KeyContext::Schedule,
            description: "Go to previous practitioner",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            action: Action::NextPractitioner,
            context: KeyContext::Schedule,
            description: "Go to next practitioner",
        });
        // Schedule: Enter selects appointment at current slot
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            action: Action::Enter,
            context: KeyContext::Schedule,
            description: "Select appointment at current time slot",
        });
        // Schedule: PageUp/PageDown for viewport scrolling
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE),
            action: Action::ScrollViewportUp,
            context: KeyContext::Schedule,
            description: "Scroll viewport to earlier hours",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
            action: Action::ScrollViewportDown,
            context: KeyContext::Schedule,
            description: "Scroll viewport to later hours",
        });
        // Schedule: Month navigation
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            action: Action::PrevMonth,
            context: KeyContext::Schedule,
            description: "Go to previous month",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('M'), KeyModifiers::SHIFT),
            action: Action::NextMonth,
            context: KeyContext::Schedule,
            description: "Go to next month",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
            action: Action::Today,
            context: KeyContext::Schedule,
            description: "Go to today",
        });

        // Clinical keybinds
        // Navigation
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            action: Action::NavigateDown,
            context: KeyContext::Clinical,
            description: "Move selection down",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
            action: Action::NavigateUp,
            context: KeyContext::Clinical,
            description: "Move selection up",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            action: Action::NavigateDown,
            context: KeyContext::Clinical,
            description: "Move selection down",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            action: Action::NavigateUp,
            context: KeyContext::Clinical,
            description: "Move selection up",
        });
        // Actions
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            action: Action::New,
            context: KeyContext::Clinical,
            description: "Create new clinical note",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            action: Action::Edit,
            context: KeyContext::Clinical,
            description: "Edit selected item",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            action: Action::TabNext,
            context: KeyContext::Clinical,
            description: "Cycle to next sub-view",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT),
            action: Action::TabPrev,
            context: KeyContext::Clinical,
            description: "Cycle to previous sub-view",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            action: Action::Enter,
            context: KeyContext::Clinical,
            description: "Open selected item",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
            action: Action::Save,
            context: KeyContext::Clinical,
            description: "Sign consultation",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            action: Action::Delete,
            context: KeyContext::Clinical,
            description: "Deactivate item",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            action: Action::Escape,
            context: KeyContext::Clinical,
            description: "Go back / Cancel",
        });
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
            action: Action::Search,
            context: KeyContext::Clinical,
            description: "Search patients",
        });

        // Clinical: Number keys 1-7 to jump to specific views
        // 1 = Patient Summary
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE),
            action: Action::SwitchToPatientSummary,
            context: KeyContext::Clinical,
            description: "Go to Patient Summary",
        });
        // 2 = Consultations
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE),
            action: Action::SwitchToConsultations,
            context: KeyContext::Clinical,
            description: "Go to Consultations",
        });
        // 3 = Allergies
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
            action: Action::SwitchToAllergies,
            context: KeyContext::Clinical,
            description: "Go to Allergies",
        });
        // 4 = Medical History
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('4'), KeyModifiers::NONE),
            action: Action::SwitchToMedicalHistory,
            context: KeyContext::Clinical,
            description: "Go to Medical History",
        });
        // 5 = Vital Signs
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE),
            action: Action::SwitchToVitalSigns,
            context: KeyContext::Clinical,
            description: "Go to Vital Signs",
        });
        // 6 = Social History
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('6'), KeyModifiers::NONE),
            action: Action::SwitchToSocialHistory,
            context: KeyContext::Clinical,
            description: "Go to Social History",
        });
        // 7 = Family History
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('7'), KeyModifiers::NONE),
            action: Action::SwitchToFamilyHistory,
            context: KeyContext::Clinical,
            description: "Go to Family History",
        });

        // Clinical: Quick actions
        // a = Add/View Allergies
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
            action: Action::ViewAllergies,
            context: KeyContext::Clinical,
            description: "View allergies",
        });
        // c = Add/View Conditions (Medical History)
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
            action: Action::ViewConditions,
            context: KeyContext::Clinical,
            description: "View conditions",
        });
        // v = View Vital Signs
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE),
            action: Action::ViewVitals,
            context: KeyContext::Clinical,
            description: "View vital signs",
        });
        // o = View Observations (recent consultations)
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE),
            action: Action::ViewObservations,
            context: KeyContext::Clinical,
            description: "View recent consultations",
        });
        // f = View Family History
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE),
            action: Action::ViewFamilyHistory,
            context: KeyContext::Clinical,
            description: "View family history",
        });
        // h = View Social History
        self.register(Keybind {
            key: KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            action: Action::ViewSocialHistory,
            context: KeyContext::Clinical,
            description: "View social history",
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
    fn test_keybind_lookup_context_specific() {
        let registry = KeybindRegistry::new();
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let result = registry.lookup(key, KeyContext::PatientList);
        assert!(result.is_some());
        assert_eq!(result.unwrap().action, Action::NavigateDown);
    }

    #[test]
    fn test_keybind_lookup_schedule_j() {
        let registry = KeybindRegistry::new();
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let result = registry.lookup(key, KeyContext::Schedule);
        assert!(
            result.is_some(),
            "j should be registered for Schedule context"
        );
        assert_eq!(result.unwrap().action, Action::NextTimeSlot);
    }

    #[test]
    fn test_keybind_lookup_schedule_k() {
        let registry = KeybindRegistry::new();
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let result = registry.lookup(key, KeyContext::Schedule);
        assert!(
            result.is_some(),
            "k should be registered for Schedule context"
        );
        assert_eq!(result.unwrap().action, Action::PrevTimeSlot);
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
}
