# OpenGP Keybind Architecture Redesign Plan

## Executive Summary

**Goal**: Create a unified, robust keybind system that works consistently across all screens without relying on tui-realm's focus-based event routing.

**Root Cause**: The current dual event handling system (tui-realm tick + direct crossterm poll) creates race conditions and "lost" key events when components don't handle navigation keys.

---

## Current Architecture Problems

### 1. Dual Event Handling
```
┌─────────────────────────────────────────────────────────────┐
│                      Main Event Loop                         │
├─────────────────────────────────────────────────────────────┤
│  1. self.inner.tick(PollStrategy::UpTo(1))                 │
│     → Routes to FOCUSED component only                      │
│     → PatientList gets Tab/Arrows but doesn't handle them   │
│                                                              │
│  2. poll(Duration::ZERO)                                    │
│     → Handles "global" keys (Tab, arrows, 1-4, q)          │
│     → Creates parallel handling path                          │
└─────────────────────────────────────────────────────────────┘
```

### 2. Focus-Based Routing Fails
- tui-realm only sends events to the focused component
- When PatientList has focus → Tab key goes to PatientList
- PatientList's `key_matches()` doesn't include Tab/Left/Right → key is IGNORED
- User presses Tab → nothing happens

### 3. Keybind Logic is Scattered
| Location | Purpose |
|----------|---------|
| `keybinds.rs` | Defines KeybindContext, Keybind structs, registry |
| `key_dispatcher.rs` | Maps key names to Action enum |
| `realm_patient_list.rs` | Custom `key_matches()` implementation |
| `realm_tabs.rs` | Tab-specific key handling |
| `app.rs` | Direct navigation key handling |

### 4. Inconsistent Key Matching
- Each component reimplements key matching differently
- `key_matches()` in patient list vs tabs vs forms
- No unified contract for "this component handles X keys"

---

## Proposed Solution: Centralized Event Dispatch

### Architecture Overview
```
┌─────────────────────────────────────────────────────────────┐
│                   Unified Event Loop                        │
├─────────────────────────────────────────────────────────────┤
│  1. Single poll(Duration::from_millis(10))                 │
│     → Non-blocking, catches all key events                   │
│                                                              │
│  2. App-level dispatcher uses KeybindRegistry               │
│     → Determines current context (PatientList, Tabs, etc)   │
│     → Looks up action for key                               │
│     → Routes to appropriate handler                          │
│                                                              │
│  3. Components only handle their SPECIFIC actions           │
│     → Not responsible for navigation                         │
│     → Pure UI: render + state management                    │
└─────────────────────────────────────────────────────────────┘
```

### Key Principles

1. **Single Event Source**: All key events handled at app level
2. **Centralized Dispatch**: KeybindRegistry determines what action to take
3. **Context-Aware**: Current screen + modal state determines context
4. **Components are Pure**: They receive actions, not raw keys

---

## Implementation Plan

### Phase 1: Refactor Event Loop (app.rs)

**Current** (problematic):
```rust
while !self.should_quit {
    tui.draw(|f| self.render(f))?;
    
    // Path 1: tui-realm tick
    let msgs = self.inner.tick(PollStrategy::UpTo(1));
    // ...
    
    // Path 2: Direct poll
    if poll(Duration::ZERO).unwrap_or(false) {
        // Handle navigation keys directly
    }
}
```

**New** (unified):
```rust
while !self.should_quit {
    tui.draw(|f| self.render(f))?;
    
    // Single event source
    if poll(Duration::from_millis(10)).unwrap_or(false) {
        if let Ok(Event::Key(key)) = crossterm::event::read() {
            self.dispatch_key_event(key);
        }
    }
}
```

### Phase 2: Create Unified Dispatcher

**New file: `src/ui/event_dispatcher.rs`**

```rust
pub struct EventDispatcher;

impl EventDispatcher {
    /// Determine current keybind context from app state
    pub fn get_context(app: &App) -> KeybindContext {
        // If modal is open → modal context
        // If patient form open → PatientForm
        // Otherwise → based on active_screen + focused component
    }
    
    /// Dispatch key event to appropriate handler
    pub fn dispatch(app: &mut App, key: KeyEvent) {
        let context = Self::get_context(app);
        
        if let Some(action) = KeyDispatcher::dispatch(&context, key) {
            Self::execute_action(app, action);
        } else {
            // Check global context as fallback
            if let Some(action) = KeyDispatcher::dispatch(&KeybindContext::Global, key) {
                Self::execute_action(app, action);
            }
        }
    }
    
    /// Execute action - modifies app state
    fn execute_action(app: &mut App, action: Action) {
        match action {
            Action::Quit => app.should_quit = true,
            Action::PatientCreate => app.show_patient_form = true,
            // ... all actions
        }
    }
}
```

### Phase 3: Simplify Components

**Before** (component handles raw keys):
```rust
impl Component<Msg, NoUserEvent> for RealmPatientList {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key) => self.handle_keyboard(key),
            // ...
        }
    }
    
    fn handle_keyboard(&mut self, key: KeyEvent) -> Option<Msg> {
        // Re-implements key matching logic
        for kb in &self.keybinds {
            if self.key_matches(kb.key, key.code) { ... }
        }
    }
}
```

**After** (component receives actions):
```rust
impl RealmPatientList {
    /// Handle navigation action from dispatcher
    pub fn handle_action(&mut self, action: &str) -> Option<Msg> {
        match action {
            "Next" => { self.move_selection(1); Some(Msg::Render) }
            "Previous" => { self.move_selection(-1); Some(Msg::Render) }
            "New" => Some(Msg::PatientCreate),
            "Edit" => self.selected_patient().map(|p| Msg::PatientEdit(p.id)),
            // ... no raw key handling
        }
    }
}
```

### Phase 4: Update Components to Use Action-Based Model

Update these components:
1. `RealmPatientList` - remove raw key handling, add `handle_action()`
2. `RealmTabs` - remove raw key handling, add `handle_action()`  
3. `RealmPatientForm` - simplify to action-based
4. Other components (appointment calendar, etc.)

---

## KeybindContext Mapping

```rust
fn get_keybind_context(app: &App) -> KeybindContext {
    // Priority: modals > forms > screen-specific
    
    if app.show_help_modal { return KeybindContext::Global; }
    if app.show_patient_form { return KeybindContext::PatientForm; }
    if app.show_appointment_form { return KeybindContext::AppointmentForm; }
    
    match app.active_screen {
        Screen::Patients => {
            if app.patient_list.is_search_mode() {
                KeybindContext::PatientListSearch
            } else {
                KeybindContext::PatientList
            }
        }
        Screen::Appointments => {
            // Calendar-specific contexts
            KeybindContext::CalendarDayView  // Default for now
        }
        Screen::Clinical => KeybindContext::Global,  // Not implemented
        Screen::Billing => KeybindContext::Global,
    }
}
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/ui/app.rs` | Replace event loop, use dispatcher |
| `src/ui/event_dispatcher.rs` | **NEW** - unified dispatch logic |
| `src/ui/key_dispatcher.rs` | Enhance with context detection |
| `src/ui/components/realm_patient_list.rs` | Remove raw key handling |
| `src/ui/components/realm_tabs.rs` | Remove raw key handling |
| `src/ui/components/realm_patient_form.rs` | Simplify key handling |

---

## Testing Strategy

1. **Unit Tests**: `KeyDispatcher` handles all keybinds in all contexts
2. **Integration Tests**: Event loop handles all key combinations
3. **Manual Testing**:
   - Tab/Arrows navigate tabs from ANY screen
   - n creates new patient from PatientList
   - e edits patient from PatientList
   - j/k navigate patient list
   - q quits from anywhere
   - 1-4 switch tabs directly

---

## Migration Path

1. **Step 1**: Add new `EventDispatcher` module
2. **Step 2**: Modify app.rs to use dispatcher (keep tui-realm for rendering)
3. **Step 3**: Update components to action-based model (one at a time)
4. **Step 4**: Remove dual event handling entirely
5. **Step 5**: Clean up unused code

---

## Benefits

1. **Predictable**: All keys handled by same logic
2. **Debuggable**: Single point of dispatch to trace key flows
3. **Maintainable**: Keybinds defined in ONE place
4. **Testable**: Can unit test dispatcher without UI
5. **Flexible**: Easy to add new screens/contexts

---

## Risk Mitigation

- **Keep tui-realm rendering**: Don't rewrite UI, just event handling
- **Incremental changes**: Update one component at a time
- **Feature flags**: Can toggle between old/new if needed
- **Extensive testing**: Manual + automated at each step
