# Ratatui Popup Composition & Modal Patterns: Analysis & Recommendations

**Date:** 2026-04-14  
**Context:** Review of reschedule feature plan for OpenGP appointment modal  
**Scope:** Ratatui patterns, Rust lifetime parameters, TUI component design

---

## EXECUTIVE SUMMARY

### Your 5 Questions Answered

#### 1. Common Ratatui Patterns for Composing Popups?

**Answer:** Owned sub-widgets in parent struct (no lifetimes).

```rust
pub struct AppointmentForm {
    date_picker: DatePickerPopup,      // ← Owned, not borrowed
    time_picker: TimePickerPopup,      // ← Owned, not borrowed
}
```

**Why:** No borrow checker friction, visibility state is local, dispatch is straightforward.

**Evidence:** OpenGP's `AppointmentForm` (1700 lines), `AppointmentDetailModal` (875 lines), `tui-popup` crate, `tui-overlay` crate.

---

#### 2. Risks of Lifetime Parameters on Helper Structs?

**Answer:** `InlinePicker<'a>` will cause borrow checker errors. Don't use it.

**Critical issue:**
```rust
pub struct InlinePicker<'a> {
    date_picker: &'a mut DatePickerPopup,
    time_picker: &'a mut TimePickerPopup,
}

impl AppointmentForm {
    pub fn handle_key(&mut self, key: KeyEvent) {
        // ❌ BORROW CHECKER ERROR:
        let mut picker = InlinePicker {
            date_picker: &mut self.date_picker,  // Can't borrow sub-field
            time_picker: &mut self.time_picker,  // while parent is mutable
        };
    }
}
```

**Why it fails:** Rust won't allow mutable references to sub-fields while parent is also mutable. The lifetime `'a` doesn't solve this — it's a fundamental ownership issue.

---

#### 3. Established Patterns for "Optional Edit Mode"?

**Answer:** Use a mode enum (not a trait).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailViewMode {
    ReadOnly,
    Editing,
}

pub struct AppointmentDetailModal {
    appointment: CalendarAppointment,
    mode: DetailViewMode,
    edit_form: Option<AppointmentForm>,
}
```

**Advantages:** Clear state machine, compiler enforces correctness, easy to test each mode independently.

**Real-world example:** OpenGP's `AppointmentState` uses `AppointmentView` enum (Calendar vs. Schedule).

---

#### 4. Pitfalls When Adding Mutable State to Read-Only Modal?

**Top 4 pitfalls:**

1. **Forgetting to reset state on close** — `close()` should reset mode and clear edit_form
2. **Inconsistent data between view and edit form** — Update original when saving
3. **Validation errors not displayed** — Render shows validation errors
4. **Modal becomes too large** — Separate concerns (form handles its own popups)

---

#### 5. Risks of Over-Engineering a Trait for "Future Use"?

**Answer:** Don't create `EditableDetailModal` trait. Wait until you have 3+ use cases.

**Why it's a trap:**
- Premature abstraction (no second use case yet)
- Trait bloat (8 methods, each impl uses 3)
- Harder to test (need to mock trait)
- Harder to refactor (changing trait breaks all impls)
- Hides implementation details

**When traits ARE worth it:** 3+ implementations with identical behavior, small trait (5 methods), actual polymorphism usage.

---

## DETAILED ANALYSIS

### 1. Popup Composition Patterns

#### Pattern 1A: Owned Sub-Widgets (Recommended)

```rust
pub struct AppointmentForm {
    date_picker: DatePickerPopup,
    time_picker: TimePickerPopup,
    patient_picker: SearchableListState<PatientListItem>,
    practitioner_picker: SearchableListState<PractitionerViewItem>,
}

impl AppointmentForm {
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppointmentFormAction> {
        // 1. Check if date picker is visible
        if self.date_picker.is_visible() {
            if let Some(action) = self.date_picker.handle_key(key) {
                match action {
                    DatePickerAction::Selected(date) => {
                        self.set_value_by_id(FIELD_DATE, format_date(date));
                        return Some(AppointmentFormAction::ValueChanged);
                    }
                    DatePickerAction::Dismissed => {
                        return Some(AppointmentFormAction::FocusChanged);
                    }
                }
            }
            return Some(AppointmentFormAction::FocusChanged);
        }
        
        // 2. Handle form-level keys
        match key.code {
            KeyCode::Tab => { /* navigate */ }
            KeyCode::Enter => { /* submit */ }
            _ => None,
        }
    }

    pub fn render(self, area: Rect, buf: &mut Buffer) {
        // ... render form fields ...
        
        // Render popups LAST (on top)
        if self.date_picker.is_visible() {
            self.date_picker.render(area, buf);
        }
        if self.time_picker.is_visible() {
            self.time_picker.render(area, buf);
        }
    }
}
```

**Advantages:**
- ✅ No borrow checker friction
- ✅ Popups can be cloned (for state snapshots)
- ✅ Clear ownership semantics
- ✅ Easy to test

**Disadvantages:**
- ❌ Parent struct gets larger (but manageable)
- ❌ Must manually manage visibility state

---

#### Pattern 1B: Visibility Enum (Alternative)

```rust
pub enum ModalState {
    Closed,
    DatePickerOpen,
    TimePickerOpen,
    PatientPickerOpen,
}

pub struct AppointmentForm {
    modal_state: ModalState,
    date_picker: DatePickerPopup,
    time_picker: TimePickerPopup,
}
```

**When to use:** Only ONE popup can be open at a time.  
**When NOT to use:** Multiple independent popups (date + time simultaneously).

---

### 2. Lifetime Parameters: Critical Issues

#### Issue 1: Borrow Checker Friction

```rust
pub struct InlinePicker<'a> {
    date_picker: &'a mut DatePickerPopup,
    time_picker: &'a mut TimePickerPopup,
}

impl AppointmentForm {
    pub fn handle_key(&mut self, key: KeyEvent) {
        // ❌ ERROR: Can't borrow self as mutable while picker holds &mut
        let mut picker = InlinePicker {
            date_picker: &mut self.date_picker,
            time_picker: &mut self.time_picker,
        };
        picker.handle_key(key);
        
        // ❌ Can't access self.other_field here because picker still holds &mut
    }
}
```

**Why it fails:**
- `InlinePicker<'a>` holds mutable references to sub-fields
- Rust won't let you hold a mutable reference to a sub-field while also mutating the parent
- The lifetime `'a` doesn't help — the borrow checker still sees the conflict

---

#### Issue 2: Lifetime Elision Confusion

```rust
pub struct InlinePicker<'a> {
    date_picker: &'a mut DatePickerPopup,
    time_picker: &'a mut TimePickerPopup,
}

impl<'a> InlinePicker<'a> {
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<PickerAction> {
        // self is &mut InlinePicker<'a>
        // self.date_picker is &'a mut DatePickerPopup
        // But what's the lifetime of the returned action?
        // If it references self.date_picker, it's 'a
        // If it references self, it's the lifetime of &mut self
        // This gets confusing fast
    }
}
```

---

#### Issue 3: Cloning Becomes Impossible

OpenGP's forms implement `Clone`:

```rust
impl Clone for AppointmentForm {
    fn clone(&self) -> Self {
        Self {
            date_picker: self.date_picker.clone(),
            time_picker: self.time_picker.clone(),
            // ...
        }
    }
}
```

**With `InlinePicker<'a>`, you can't clone** because you can't clone borrowed references.

---

#### When Lifetimes DO Make Sense

```rust
pub struct WidgetRenderer<'a> {
    widget: &'a dyn Widget,
    theme: &'a Theme,
}

impl<'a> WidgetRenderer<'a> {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        self.widget.render(area, buf);
    }
}
```

**Why this works:**
- No mutation needed
- Renderer is temporary (created, used, dropped)
- No state management

---

### 3. Optional Edit Mode Patterns

#### Pattern 3A: Mode Enum (Recommended)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailViewMode {
    ReadOnly,
    Editing,
}

pub struct AppointmentDetailModal {
    appointment: CalendarAppointment,
    mode: DetailViewMode,
    edit_form: Option<AppointmentForm>,
}

impl AppointmentDetailModal {
    pub fn toggle_edit(&mut self) {
        match self.mode {
            DetailViewMode::ReadOnly => {
                self.mode = DetailViewMode::Editing;
                self.edit_form = Some(AppointmentForm::from_appointment(&self.appointment));
            }
            DetailViewMode::Editing => {
                self.mode = DetailViewMode::ReadOnly;
                self.edit_form = None;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        match self.mode {
            DetailViewMode::ReadOnly => self.handle_key_readonly(key),
            DetailViewMode::Editing => {
                if let Some(form) = &mut self.edit_form {
                    form.handle_key(key)
                } else {
                    None
                }
            }
        }
    }

    pub fn render(self, area: Rect, buf: &mut Buffer) {
        match self.mode {
            DetailViewMode::ReadOnly => self.render_readonly(area, buf),
            DetailViewMode::Editing => {
                if let Some(form) = self.edit_form {
                    form.render(area, buf);
                }
            }
        }
    }
}
```

**Advantages:**
- ✅ Clear state machine
- ✅ Compiler enforces mode-specific logic
- ✅ Easy to test each mode independently
- ✅ No lifetime parameters needed

---

#### Pattern 3B: Trait-Based Polymorphism (Not Recommended)

```rust
pub trait DetailView {
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn handle_key(&mut self, key: KeyEvent) -> Option<Action>;
}

pub struct ReadOnlyDetail { /* ... */ }
pub struct EditableDetail { /* ... */ }

impl DetailView for ReadOnlyDetail { /* ... */ }
impl DetailView for EditableDetail { /* ... */ }

pub struct AppointmentDetailModal {
    view: Box<dyn DetailView>,
}
```

**⚠️ Not recommended:**
- ❌ Runtime dispatch overhead
- ❌ Harder to test (need to mock trait)
- ❌ Loses type information
- ❌ Cloning becomes complex

---

### 4. Pitfalls When Adding Mutable State

#### Pitfall 1: Forgetting to Reset State on Close

```rust
// ❌ BAD: Edit form persists after closing
pub struct AppointmentDetailModal {
    appointment: CalendarAppointment,
    mode: DetailViewMode,
    edit_form: Option<AppointmentForm>,
}

impl AppointmentDetailModal {
    pub fn close(&mut self) {
        // Forgot to reset mode!
        // Next time modal opens, it's still in Editing mode
    }
}

// ✅ GOOD:
pub fn close(&mut self) {
    self.mode = DetailViewMode::ReadOnly;
    self.edit_form = None;
}
```

---

#### Pitfall 2: Inconsistent Data Between View and Edit Form

```rust
// ❌ BAD: User edits form, but original appointment is never updated
pub fn save_changes(&mut self) -> Result<(), Error> {
    if let Some(form) = &self.edit_form {
        // Form has new data, but self.appointment is unchanged
        // Next render shows stale data
    }
}

// ✅ GOOD:
pub fn save_changes(&mut self) -> Result<(), Error> {
    if let Some(form) = &self.edit_form {
        let updated = form.to_appointment()?;
        self.appointment = updated;
        self.mode = DetailViewMode::ReadOnly;
        self.edit_form = None;
    }
    Ok(())
}
```

---

#### Pitfall 3: Validation Errors Not Displayed

```rust
// ❌ BAD: Form validates but errors aren't shown
pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
    match self.mode {
        DetailViewMode::Editing => {
            if let Some(form) = &mut self.edit_form {
                if key.code == KeyCode::Enter {
                    if form.validate() {
                        return Some(Action::Save);
                    }
                    // Errors are in form.errors, but render() doesn't show them!
                }
            }
        }
        _ => {}
    }
    None
}

// ✅ GOOD: Render shows validation errors
pub fn render(self, area: Rect, buf: &mut Buffer) {
    match self.mode {
        DetailViewMode::Editing => {
            if let Some(form) = self.edit_form {
                form.render(area, buf);  // Form renders its own errors
            }
        }
        _ => {}
    }
}
```

---

#### Pitfall 4: Modal Becomes Too Large

```rust
// ❌ BAD: 500+ lines, mixing read-only and edit logic
pub struct AppointmentDetailModal {
    // Read-only fields
    appointment: CalendarAppointment,
    // Edit fields
    edit_form: Option<AppointmentForm>,
    // Popup fields
    date_picker: DatePickerPopup,
    time_picker: TimePickerPopup,
    // Status dropdown
    status_dropdown: DropdownWidget,
    // ... 20 more fields
}

// ✅ GOOD: Separate concerns
pub struct AppointmentDetailModal {
    appointment: CalendarAppointment,
    mode: DetailViewMode,
    edit_form: Option<AppointmentForm>,
}

// AppointmentForm handles its own popups and dropdowns
pub struct AppointmentForm {
    date_picker: DatePickerPopup,
    time_picker: TimePickerPopup,
    // ...
}
```

---

### 5. Over-Engineering Traits for "Future Use"

#### The Trap: EditableDetailModal Trait

```rust
// ❌ OVER-ENGINEERED: Trait for a single use case
pub trait EditableDetailModal {
    fn get_data(&self) -> DetailData;
    fn set_data(&mut self, data: DetailData);
    fn validate(&self) -> Result<(), ValidationError>;
    fn render_readonly(&self, area: Rect, buf: &mut Buffer);
    fn render_editable(&self, area: Rect, buf: &mut Buffer);
    fn handle_key_readonly(&mut self, key: KeyEvent) -> Option<Action>;
    fn handle_key_editable(&mut self, key: KeyEvent) -> Option<Action>;
    fn toggle_edit_mode(&mut self);
}

pub struct AppointmentDetailModal { /* ... */ }
impl EditableDetailModal for AppointmentDetailModal { /* ... */ }

pub struct PatientDetailModal { /* ... */ }
impl EditableDetailModal for PatientDetailModal { /* ... */ }
```

**Why this is a trap:**

1. **Premature abstraction** — You don't have a second use case yet
2. **Trait bloat** — The trait has 8 methods, but each impl only uses 3
3. **Harder to test** — You need to mock the trait
4. **Harder to refactor** — Changing the trait breaks all impls
5. **Hides implementation details** — You can't see what's actually happening

**Real-world consequence:**
- 6 months later, you add `ReferralDetailModal`
- It needs slightly different validation logic
- The trait doesn't support it
- You either:
  - Add more methods to the trait (bloat)
  - Create a new trait (duplication)
  - Abandon the trait and implement directly (wasted effort)

---

#### When Traits ARE Worth It

```rust
// ✅ GOOD: All popups have the same interface
pub trait Popup {
    fn is_visible(&self) -> bool;
    fn open(&mut self);
    fn close(&mut self);
    fn handle_key(&mut self, key: KeyEvent) -> Option<PopupAction>;
    fn render(&self, area: Rect, buf: &mut Buffer);
}

pub struct DatePickerPopup { /* ... */ }
pub struct TimePickerPopup { /* ... */ }
pub struct ConfirmDialog { /* ... */ }

impl Popup for DatePickerPopup { /* ... */ }
impl Popup for TimePickerPopup { /* ... */ }
impl Popup for ConfirmDialog { /* ... */ }

// Now you can store them in a Vec:
pub struct ModalStack {
    popups: Vec<Box<dyn Popup>>,
}
```

**Why this works:**
- All popups have identical behavior (open/close/render/handle_key)
- You have 3+ implementations
- The trait is small (5 methods)
- You actually use the polymorphism (Vec<Box<dyn Popup>>)

---

## RECOMMENDATIONS FOR RESCHEDULE FEATURE

### ✅ DO THIS:

1. **Keep popups owned by the form:**
   ```rust
   pub struct AppointmentForm {
       date_picker: DatePickerPopup,
       time_picker: TimePickerPopup,
   }
   ```

2. **Use a mode enum for edit state:**
   ```rust
   pub enum DetailModalMode {
       ReadOnly,
       Rescheduling,
   }
   ```

3. **No lifetime parameters:**
   - Owned data is simpler
   - No borrow checker friction
   - Easier to clone for snapshots

4. **Keep the modal simple:**
   - `AppointmentDetailModal` handles read-only display + mode toggle
   - `RescheduleForm` handles editing + popups
   - Clear separation of concerns

5. **Test each mode independently:**
   ```rust
   #[test]
   fn test_readonly_mode_navigation() { /* ... */ }
   
   #[test]
   fn test_reschedule_mode_validation() { /* ... */ }
   ```

### ❌ DON'T DO THIS:

1. **Don't use `InlinePicker<'a>`** — borrow checker will fight you
2. **Don't create `EditableDetailModal` trait** — wait until you have 3+ use cases
3. **Don't mix read-only and edit logic** — separate into different structs
4. **Don't forget to reset state** — `close()` should reset mode and clear edit_form
5. **Don't use trait objects** — concrete types are faster and clearer

---

## RATATUI ECOSYSTEM PATTERNS

| Crate | Pattern | Visibility | Key Handling |
|-------|---------|------------|--------------|
| **tui-popup** (v0.7.4) | Owned widget, no lifetimes | `visible: bool` flag | Returns `PopupAction` enum |
| **tui-overlay** (v0.1.2) | Composable overlay, no content ownership | Managed by parent | Parent handles keys, overlay positions |
| **OpenGP** | Owned sub-widgets in parent struct | Each popup tracks `is_visible` | Parent checks visibility, delegates |

**Consensus:** Owned sub-widgets with visibility flags is the most practical pattern.

---

## IMPLEMENTATION CHECKLIST

### Phase 1: Add Reschedule Form Widget
- [ ] Create `RescheduleForm` struct with date/time pickers
- [ ] Implement `Clone` for `RescheduleForm`
- [ ] Add validation logic (date must be in future, etc.)
- [ ] Add tests for form validation

### Phase 2: Update AppointmentDetailModal
- [ ] Add `DetailModalMode` enum
- [ ] Add `mode` field to modal
- [ ] Add `reschedule_form: Option<RescheduleForm>` field
- [ ] Implement `toggle_reschedule()` method
- [ ] Split `handle_key()` into `handle_key_readonly()` and `handle_key_rescheduling()`
- [ ] Split `render()` into `render_readonly()` and `render_rescheduling()`
- [ ] Update `AppointmentDetailModalAction` enum

### Phase 3: Update Modal Rendering
- [ ] Add "Reschedule" button to read-only view
- [ ] Render date/time picker fields in reschedule view
- [ ] Show validation errors in reschedule view
- [ ] Add help text for reschedule mode

### Phase 4: Integration
- [ ] Handle `ConfirmReschedule` action in app event handler
- [ ] Call `AppointmentService::reschedule_appointment()`
- [ ] Update appointment state on success
- [ ] Show error message on failure

### Phase 5: Testing
- [ ] Test read-only mode navigation (unchanged)
- [ ] Test toggle to reschedule mode
- [ ] Test date picker interaction
- [ ] Test time picker interaction
- [ ] Test validation errors
- [ ] Test confirm reschedule
- [ ] Test cancel reschedule

---

## KEY TAKEAWAYS

1. **Owned sub-widgets are simpler than borrowed references** — no borrow checker friction
2. **Mode enums are clearer than trait abstraction** — compiler enforces correctness
3. **Don't over-engineer for "future use"** — wait until you have 3+ use cases
4. **Follow OpenGP's existing patterns** — consistency matters
5. **Test each mode independently** — easier to debug and maintain

---

## REFERENCES

### OpenGP Codebase
- `crates/opengp-ui/src/ui/components/appointment/detail_modal.rs` (875 lines)
- `crates/opengp-ui/src/ui/components/appointment/form.rs` (1700 lines)
- `crates/opengp-ui/src/ui/widgets/date_picker_popup.rs` (543 lines)
- `crates/opengp-ui/src/ui/widgets/time_picker_popup.rs` (401 lines)

### Ratatui Ecosystem
- `tui-popup` (v0.7.4, 2026-04-04) — https://github.com/ratatui/tui-widgets
- `tui-overlay` (v0.1.2, 2026-04-07) — https://github.com/jharsono/tui-overlay
- Ratatui popup example — https://github.com/ratatui/ratatui/tree/main/examples/apps/popup

