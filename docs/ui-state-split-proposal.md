## OpenGP UI App State Split Proposal

### Purpose

This document proposes a structured split of the `App` struct in `crates/opengp-ui/src/ui/app.rs` into focused state modules while keeping the external behavior of the UI stable.

The goal is to reduce the size and complexity of `App`, make tab specific behavior easier to evolve, and prepare the UI layer for future features without a risky rewrite.

### Current State Summary

`App` currently acts as a god object for the TUI.

- File size: roughly 2016 lines
- Pending fields: 8 distinct `pending_*` fields coordinating work between the UI and main loop
- `clone()` usage: 54 calls scattered across event handling, rendering, and view model code

Responsibilities include:

- Global concerns
  - Theme configuration (`Theme`)
  - Keybinding lookup (`KeybindRegistry` and `KeyContext`)
  - Tab bar selection (`TabBar`)
  - Status bar and help overlay
  - Terminal sizing and layout decisions
  - Application lifecycle flags (`should_quit`)

- Patient tab
  - `PatientState`
  - `PatientList`
  - `Option<PatientForm>`
  - `pending_patient_data: Option<PendingPatientData>`
  - `pending_edit_patient_id: Option<Uuid>`
  - Patient specific key handling and rendering

- Appointment tab
  - `AppointmentState`
  - `appointment_service: Option<AppointmentUiService>`
  - `patient_service: Option<PatientUiService>` (used from appointment context)
  - `pending_appointment_date: Option<NaiveDate>`
  - `pending_load_practitioners: bool`
  - `appointment_form: Option<AppointmentForm>`
  - `appointment_detail_modal: Option<AppointmentDetailModal>`
  - `pending_appointment_save: Option<NewAppointmentData>`
  - `pending_appointment_status_transition: Option<(Uuid, AppointmentStatusTransition)>`
  - Appointment specific key handling and rendering

- Clinical tab
  - `ClinicalState`
  - `clinical_service: Option<ClinicalUiService>`
  - `pending_clinical_patient_id: Option<Uuid>`
  - `pending_clinical_save_data: Option<PendingClinicalSaveData>`
  - Clinical specific key handling and rendering

All of this is wired directly inside `App::handle_key_event`, `App::draw`, and various helper methods. The result is a central type with:

- Many fields that are only relevant for a single tab
- Cross tab conditionals inside global key handling
- Multiple `pending_*` hand off points between UI and main loop
- Frequent cloning of view model data for rendering and event handling

### Proposed Split

Introduce three focused state structs owned by `App`.

- `PatientAppState`
- `AppointmentAppState`
- `ClinicalAppState`

`App` remains the root coordinator. It keeps global concerns (theme, keybinds, tab bar, status bar, help overlay, terminal size) and delegates tab specific work to these new modules.

#### PatientAppState

**Scope**

Encapsulates all patient tab behavior and state.

**Fields (initial mapping)**

- `patient_state: PatientState`
- `patient_list: PatientList`
- `patient_form: Option<PatientForm>`
- `pending_patient_data: Option<PendingPatientData>`
- `pending_edit_patient_id: Option<Uuid>`

**Key responsibilities**

- Loading patient list data
- Managing search, selection, and list navigation
- Opening and closing the patient form
- Tracking pending patient edits and exposing them to the main loop
- Rendering the patient tab content into a `Rect`
- Implementing `FormNavigation` integration for the patient form

**Example API surface**

These method signatures are illustrative. Exact naming can follow existing patterns.

- `fn load_patients(&mut self, patients: Vec<Patient>)`
- `fn request_edit_patient(&mut self, id: Uuid)`
- `fn take_pending_patient_data(&mut self) -> Option<PendingPatientData>`
- `fn take_pending_edit_patient_id(&mut self) -> Option<Uuid>`
- `fn handle_key(&mut self, key: KeyEvent) -> Action`
- `fn draw(&self, frame: &mut Frame, area: Rect)`

#### AppointmentAppState

**Scope**

Encapsulates schedule, calendar, and appointment form behavior.

**Fields (initial mapping)**

- `appointment_state: AppointmentState`
- `appointment_service: Option<AppointmentUiService>`
- `patient_service: Option<PatientUiService>`
- `pending_appointment_date: Option<NaiveDate>`
- `pending_load_practitioners: bool`
- `appointment_form: Option<AppointmentForm>`
- `appointment_detail_modal: Option<AppointmentDetailModal>`
- `pending_appointment_save: Option<NewAppointmentData>`
- `pending_appointment_status_transition: Option<(Uuid, AppointmentStatusTransition)>`

**Key responsibilities**

- Managing the current appointment view (calendar vs schedule)
- Handling appointment list navigation and selection
- Opening and closing the appointment creation form
- Opening and closing the appointment detail modal
- Tracking pending appointment changes for the main loop
- Coordinating practitioner and patient lookups for appointment forms
- Rendering the appointment tab content

**Example API surface**

- `fn appointment_state_mut(&mut self) -> &mut AppointmentState`
- `fn request_new_for_date(&mut self, date: NaiveDate)`
- `fn request_load_practitioners(&mut self)`
- `fn take_pending_appointment_date(&mut self) -> Option<NaiveDate>`
- `fn take_pending_load_practitioners(&mut self) -> bool`
- `fn set_patients(&mut self, patients: Vec<PatientListItem>)`
- `fn set_practitioners(&mut self, practitioners: Vec<PractitionerViewItem>)`
- `fn take_pending_save(&mut self) -> Option<NewAppointmentData>`
- `fn take_pending_status_transition(&mut self) -> Option<(Uuid, AppointmentStatusTransition)>`
- `fn handle_key(&mut self, key: KeyEvent) -> Action`
- `fn draw(&self, frame: &mut Frame, area: Rect)`

#### ClinicalAppState

**Scope**

Encapsulates clinical view behavior, including clinical forms and consultation context.

**Fields (initial mapping)**

- `clinical_state: ClinicalState`
- `clinical_service: Option<ClinicalUiService>`
- `pending_clinical_patient_id: Option<Uuid>`
- `pending_clinical_save_data: Option<PendingClinicalSaveData>`

**Key responsibilities**

- Managing which clinical form is open
- Handling clinical form navigation and validation
- Tracking which patient is active in the clinical view
- Capturing pending clinical records for the main loop
- Rendering the clinical tab content

**Example API surface**

- `fn clinical_state_mut(&mut self) -> &mut ClinicalState`
- `fn request_open_for_patient(&mut self, patient_id: Uuid)`
- `fn take_pending_patient_id(&mut self) -> Option<Uuid>`
- `fn take_pending_save(&mut self) -> Option<PendingClinicalSaveData>`
- `fn handle_key(&mut self, key: KeyEvent) -> Action`
- `fn draw(&self, frame: &mut Frame, area: Rect)`

### How App Changes

`App` keeps global concerns and becomes a coordinator around the tab specific modules.

**Fields**

Instead of holding all tab fields directly, `App` owns:

- `theme: Theme`
- `keybinds: &'static KeybindRegistry`
- `tab_bar: TabBar`
- `status_bar: StatusBar`
- `help_overlay: HelpOverlay`
- `current_context: KeyContext`
- `should_quit: bool`
- `title: String`
- `version: String`
- `terminal_size: Rect`
- `patient_app: PatientAppState`
- `appointment_app: AppointmentAppState`
- `clinical_app: ClinicalAppState`

The constructor `App::new` continues to take the same services and config as today but passes the relevant pieces into each sub state constructor.

**Key handling**

`App::handle_key_event` keeps the global routing logic:

- If help overlay visible, route keys to help overlay
- If global keybinds apply (tab switching, quitting), handle at the top
- Otherwise delegate to the active tab module based on `tab_bar.selected()`

Example shape of the delegation logic:

```rust
pub fn handle_key_event(&mut self, key: KeyEvent) -> Action {
    if self.help_overlay.is_visible() {
        return self.handle_help_overlay_keys(key);
    }

    if let Some(action) = self.handle_global_keybinds(key) {
        return action;
    }

    match self.tab_bar.selected() {
        Tab::Patient => self.patient_app.handle_key(key),
        Tab::Appointment => self.appointment_app.handle_key(key),
        Tab::Clinical => self.clinical_app.handle_key(key),
        Tab::Billing => Action::Unknown, // unchanged for now
    }
}
```

Existing public methods like `take_pending_patient_data`, `take_pending_appointment_save`, and `take_pending_clinical_save_data` remain on `App` but delegate to the new modules. This keeps the public surface area stable for the main loop and integration tests.

**Rendering**

`App::draw` continues to be responsible for the overall layout, but defers tab interior rendering to the modules.

Example shape:

```rust
fn draw(&mut self, frame: &mut Frame) {
    let areas = self.layout(frame.size());

    self.tab_bar.draw(frame, areas.tabs);
    self.status_bar.draw(frame, areas.status);

    match self.tab_bar.selected() {
        Tab::Patient => self.patient_app.draw(frame, areas.content),
        Tab::Appointment => self.appointment_app.draw(frame, areas.content),
        Tab::Clinical => self.clinical_app.draw(frame, areas.content),
        Tab::Billing => { /* existing behavior */ }
    }

    if self.help_overlay.is_visible() {
        self.help_overlay.draw(frame, areas.overlay);
    }
}
```

### Pros and Cons

#### Pros

1. **Smaller, focused types**
   - Each tab gets a dedicated state struct with a clear scope.
   - `App` becomes easier to understand and reason about.

2. **Better testability**
   - Tab specific behavior can be unit tested without bootstrapping the full application.
   - Pending hand off logic (for example `take_pending_*`) can be tested per module.

3. **Clearer ownership of pending state**
   - Each `pending_*` field lives inside the module where it originates.
   - The main loop interacts with a small set of `App` facade methods.

4. **Safer evolution of UI flows**
   - Changes to patient flows are less likely to impact appointment or clinical flows.
   - New forms and wizards can be added inside a module without growing `App` further.

5. **Opportunities to reduce `clone()` calls**
   - With better separation of concerns, it becomes easier to tighten lifetimes and reduce unnecessary cloning in each module.
   - View model creation and reuse can be localized to the tab that owns it.

6. **Aligns with Clean Architecture goals**
   - `App` resembles an orchestrator, and tab modules look more like presentation layer components.
   - The split mirrors domain boundaries (patient, appointment, clinical) which already exist in the project.

#### Cons

1. **More moving parts**
   - New types and files to navigate.
   - Developers need to learn where a particular piece of behavior lives.

2. **Indirection in the short term**
   - `App` will forward many methods to sub states during the migration.
   - Some call sites will see an extra level of function call until the API is simplified.

3. **Borrowing and lifetime complexity**
   - Splitting state can surface borrowing issues when multiple tab states need to be accessed at once.
   - This needs careful design of method signatures to avoid conflicting borrows.

4. **Initial migration cost**
   - Refactoring a 2000+ line file carries risk.
   - The migration needs to be incremental and test driven to avoid regressions.

### Non Breaking Migration Path

The migration should be incremental, with `App` as a stable facade so that callers and tests do not need to change all at once.

#### Phase 1: Introduce new structs (no behavior change)

1. Create three new modules in `crates/opengp-ui/src/ui/app/` (directory structure can match existing conventions):
   - `patient_app_state.rs`
   - `appointment_app_state.rs`
   - `clinical_app_state.rs`

2. Define empty skeleton structs with fields but minimal methods, for example constructors and basic accessors.
3. Update `App` to hold the new structs in addition to the existing fields, but keep current behavior unchanged.
4. Add unit tests for the new structs that simply assert construction and basic field wiring.

At the end of this phase, behavior is unchanged. The new structs exist but are not yet authoritative.

#### Phase 2: Move fields into modules

1. Move patient related fields from `App` into `PatientAppState`.
2. Move appointment related fields into `AppointmentAppState`.
3. Move clinical related fields into `ClinicalAppState`.
4. Update `App::new` to construct each module with the right values.
5. Add simple delegating getters on `App` so existing call sites continue to compile without changes.

Example: implement `App::appointment_state_mut` as a thin wrapper around `self.appointment_app.appointment_state_mut()`.

Run existing tests after each small move to keep the system green.

#### Phase 3: Move behavior into modules

1. Identify groups of methods that operate only on a single tab, for example:
   - Patient: `load_patients`, patient form open and submit helpers, patient list search handling
   - Appointment: appointment form open/close, save, cancel, status transitions
   - Clinical: clinical form routing, pending save extraction

2. For each group, follow this pattern:
   - Extract a method with the same behavior into the appropriate module.
   - Change the old `App` method to call into the module implementation.
   - Keep the old `App` method signature so external callers remain unchanged.

3. Gradually shrink `App::handle_key_event` by delegating to module level key handling methods.
4. Do the same for `App::draw`, delegating tab specific rendering.

After this phase, most of the tab logic lives inside the modules, but `App` still exposes the same API surface.

#### Phase 4: Simplify the App facade

Once all call sites have been updated to use more focused APIs (for example tests or the main loop may start calling `patient_app` methods directly), the `App` facade can be trimmed.

Possible cleanups:

- Remove obsolete forwarding methods.
- Collapse redundant helpers that simply pass through to a module.
- Revisit `clone()` sites inside each module to reduce unnecessary copying.

This phase is optional if consumers prefer to interact with `App` only.

#### Phase 5: Guardrails and documentation

1. Add module level docs summarizing the responsibilities of each `*AppState` type.
2. Add a short note in the main architecture docs linking to this design, so future contributors know where UI state lives.
3. Consider adding lightweight invariants as tests, for example:
   - Only one of `patient_form`, `appointment_form`, and clinical forms is open at a time.
   - `pending_*` fields are cleared when `take_*` methods are called.

### Impact on Callers and APIs

- Public methods on `App` that are used by the main loop and tests will be preserved during the migration.
- New helper methods on the modules will only be exposed as needed.
- No changes are required to domain or infrastructure layers.
- Keyboard shortcuts, keybinding configuration, and global layout entry points remain the same from the callers perspective.

### Summary

Splitting `App` into `PatientAppState`, `AppointmentAppState`, and `ClinicalAppState` brings the UI layer closer to the projects domain boundaries and reduces the risk of further growth in a single god object.

The proposed migration path is incremental and non breaking, with `App` acting as a stable facade while behavior moves into focused modules. This should reduce cognitive load, make tests more targeted, and create a safer place to evolve patient, appointment, and clinical workflows.
