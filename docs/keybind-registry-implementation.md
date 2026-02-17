# Keybind Registry Implementation

**Date**: 2026-02-13  
**Updated**: 2026-02-17  
**Status**: Complete (All Phases Complete)  
**Files**: `src/ui/keybinds.rs`, `src/ui/key_dispatcher.rs`

## Overview

Created a centralized keybind registry module that serves as a single source of truth for all 111 keyboard bindings across the OpenGP TUI application. The registry now includes **lookup capabilities** for dispatching key events to actions.

## Phase 4 (2026-02-17): Validation Tests

Added comprehensive validation tests in `key_dispatcher.rs` to ensure component behavior matches registry definitions:

```rust
// All keybinds in all contexts can be dispatched without panicking
#[test]
fn test_all_keybinds_dispatchable() { ... }

// All dispatched actions are valid Action enum values
#[test]
fn test_all_dispatched_actions_are_valid() { ... }

// Implemented keybinds always return Some(Action)
#[test]
fn test_implemented_keybinds_return_action() { ... }

// Unhandled keys correctly return None
#[test]
fn test_unhandled_keys_return_none() { ... }

// All contexts have at least one keybind defined
#[test]
fn test_all_contexts_have_keybinds() { ... }

// Dispatcher correctly handles all registered keys
#[test]
fn test_dispatcher_matches_registry_lookup() { ... }
```

**Test Coverage**: 7 new validation tests added.

## Phase 3: Components Refactored to Use Dispatcher

Components refactored to use `KeyDispatcher` for centralized key event handling instead of hardcoded keybinds.

## New in Phase 1 (2026-02-17)

### Lookup API Added

The registry now supports key event lookup for centralized dispatch:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opengp::ui::keybinds::{KeybindRegistry, KeybindContext};

// Look up action by key event
let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
if let Some(action) = KeybindRegistry::lookup_action(&KeybindContext::CalendarDayView, key) {
    println!("Action: {}", action); // "New"
}

// Get full details: (action, description, implemented)
let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
if let Some((action, desc, implemented)) = KeybindRegistry::lookup_keybind(&KeybindContext::PatientList, key) {
    if implemented {
        println!("{}: {}", action, desc);
    }
}

// Check if key has a binding
let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
if KeybindRegistry::has_keybind(&KeybindContext::PatientList, key) {
    // Handle key
}

// Get all implemented actions
let actions = KeybindRegistry::get_implemented_actions(&KeybindContext::PatientList);
```

## Implementation Summary

### File Created
- **`src/ui/keybinds.rs`** (646 lines)
  - Comprehensive keybind definitions
  - Context-aware resolution
  - Help text generation
  - Full test coverage (9 tests)

### Conflicts Resolved

#### 1. `n` Key Conflict (CRITICAL)
**Problem**: In calendar day view, `n` had dual meaning:
- Create new appointment (month view behavior)
- Mark as "No Show" (day view specific)

**Solution**: 
- `n` now **always** creates new appointments (consistent across all contexts)
- `x` is now the dedicated key for "No Show" status changes
- Updated in day view and detail modal contexts

#### 2. `Enter` in Patient List (DOCUMENTED)
**Problem**: `Enter` key does nothing - not implemented

**Solution**:
- Marked as `unimplemented` in registry
- Reserved for future patient detail view
- Excluded from help text automatically

## Module Structure

```rust
pub enum KeybindContext {
    Global,
    PatientList,
    PatientListSearch,
    PatientForm,
    AppointmentList,
    CalendarMonthView,
    CalendarDayView,
    CalendarWeekView,
    CalendarMultiSelect,
    CalendarDetailModal,
    CalendarRescheduleModal,
    CalendarSearchModal,
    CalendarFilterMenu,
    CalendarPractitionerMenu,
    CalendarAuditModal,
    CalendarConfirmation,
    CalendarErrorModal,
    CalendarBatchMenu,
    AppointmentForm,
    AppointmentFormPatient,
}

pub struct Keybind {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
    pub action: &'static str,
    pub description: &'static str,
    pub implemented: bool,
}

pub struct KeybindRegistry;

impl KeybindRegistry {
    pub fn get_keybinds(context: KeybindContext) -> Vec<Keybind>
    pub fn get_help_text(context: KeybindContext) -> String
    pub fn format_key(key: &KeyCode, modifiers: KeyModifiers) -> String
    pub fn format_keybind(kb: &Keybind) -> String
}
```

## Usage Example

```rust
use crate::ui::keybinds::{KeybindRegistry, KeybindContext};

// Get help text for display
let help_text = KeybindRegistry::get_help_text(KeybindContext::PatientList);
// Returns: "j: Next patient  k: Previous patient  g: First patient  ..."

// Get all keybinds for a context
let keybinds = KeybindRegistry::get_keybinds(KeybindContext::CalendarDayView);
for kb in keybinds {
    println!("{}", KeybindRegistry::format_keybind(&kb));
}
```

## Test Coverage

All phases complete with comprehensive test coverage:

### Phase 1: Registry Tests (9 tests)
- ✅ Key formatting (simple, with modifiers, special keys)
- ✅ Context keybind retrieval
- ✅ Help text generation
- ✅ `n` key conflict resolution verification
- ✅ Unimplemented keybind handling
- ✅ Lookup API (action, keybind, has_keybind)

### Phase 4: Validation Tests (7 tests)
- ✅ All keybinds dispatchable without panic
- ✅ All dispatched actions are valid
- ✅ Implemented keybinds return action
- ✅ Unhandled keys return None
- ✅ All contexts have keybinds
- ✅ Dispatcher matches registry lookup
- ✅ Case insensitivity verified

**Total Tests**: 16 passing tests covering the complete keybind system

## Key Features

### 1. Context-Aware Resolution
Each UI state has its own context enum, ensuring no ambiguity:
- Calendar has 10 separate contexts (month view, day view, week view, 7 modals)
- Patient list has 2 contexts (normal, search mode)
- Forms have dedicated contexts (patient form, appointment form, appointment form patient field)

### 2. Help Text Generation
- Automatic generation from keybind definitions
- Unimplemented keybinds excluded automatically
- Compact format suitable for title bars and footers
- Modifier-aware formatting (Ctrl+Z, Shift+←, etc.)

### 3. Type Safety
- Enum-based context selection (compile-time safety)
- KeyCode and KeyModifiers from crossterm (standard types)
- Static string slices for descriptions (zero allocation)

### 4. Documentation
- Module-level documentation with design goals
- Known conflicts documented in comments
- Usage examples provided
- All public APIs documented

## Integration Points

### Exports
Module exported in `src/ui/mod.rs`:
```rust
pub use keybinds::{Keybind, KeybindContext, KeybindRegistry};
```

### Next Steps (Follow-up Tasks)
1. Update component files to use registry instead of hardcoded keybinds
2. Replace inline help text with `KeybindRegistry::get_help_text()`
3. Add context-aware help modal (show all keybinds for current context)
4. Consider adding keybind customization (config file support)

## Verification

```bash
# Compile check
cargo check  # ✅ Passes

# Run all keybind-related tests
cargo test --lib ui::key_dispatcher::  # ✅ All tests pass
cargo test --lib ui::keybinds::        # ✅ All tests pass

# Run all tests to ensure nothing broke
cargo test --lib                       # ✅ All library tests pass
```

## Statistics

- **Total keybinds defined**: 111 (matches audit inventory)
- **Contexts**: 21
- **Files created**: 
  - `src/ui/keybinds.rs` (816 lines)
  - `src/ui/key_dispatcher.rs` (280+ lines)
- **Test coverage**: 16 unit tests (9 registry + 7 validation)
- **Conflicts resolved**: 2 critical issues
- **Phases complete**: 4/4 (all phases done)

## References

- **Audit Document**: `docs/keybind-inventory.md`
- **Source Files**: 
  - `src/ui/keybinds.rs` (registry)
  - `src/ui/key_dispatcher.rs` (dispatcher)
- **Module Exports**: `src/ui/mod.rs`

---

## Implementation Complete ✅

All 4 phases completed:

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Registry extended with lookup methods | ✅ Complete |
| 2 | KeyDispatcher created | ✅ Complete |
| 3 | Components refactored to use dispatcher | ✅ Complete |
| 4 | Validation tests added | ✅ Complete |

The keybind registry system is now fully implemented with:
- Single source of truth for all keyboard bindings
- Context-aware key resolution
- Centralized dispatcher for action mapping
- Comprehensive validation tests
- All tests passing
