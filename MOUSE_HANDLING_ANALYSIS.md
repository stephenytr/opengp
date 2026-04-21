# OpenGP TUI Mouse Handling Implementation - Complete Analysis

**Date**: April 20, 2026  
**Project**: OpenGP (Rust, Ratatui 0.30, Crossterm 0.29)  
**Scope**: Comprehensive inventory of current mouse handling before planned overhaul

---

## EXECUTIVE SUMMARY

OpenGP has a **functional but minimal** mouse implementation:
- ✅ **14 HandleMouse implementations** across components and widgets
- ✅ **Global dispatcher** with priority-based routing
- ✅ **Scroll wheel support** (hardcoded increments)
- ✅ **Click-to-select** in lists and forms
- ❌ **NO hover state tracking**
- ❌ **NO right-click handling**
- ❌ **NO context menus**
- ✅ **Detail modals** (6 implementations) for read-only views
- ✅ **Enter key flow** for opening detail views
- ⚠️ **Scroll increments hardcoded** (not configurable)

---

## 1. HANDLEMOUSE TRAIT DEFINITION

**File**: `/home/stephenp/Documents/opengp/crates/opengp-ui/src/ui/input.rs:90-100`

```rust
/// Trait for components that handle mouse events
///
/// Components implementing this trait can process MouseEvent within
/// a specific rendering area and produce an optional action as output.
pub trait HandleMouse {
    /// Action type produced by handling a mouse event
    type Action;

    /// Handles a mouse event within the given area and returns an optional action
    fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<Self::Action>;
}
```

**Key Design**:
- Generic `Action` type per component (allows type-safe action enums)
- Takes `MouseEvent` from crossterm and `Rect` (rendering area)
- Returns `Option<Action>` (None = event not handled by this component)
- Signature matches `HandleEvent` trait pattern for consistency

---

## 2. ALL 14 HANDLEMOUSE IMPLEMENTATIONS - COMPLETE INVENTORY

### **Component Implementations (10)**

#### 1. **PatientForm** 
- **File**: `crates/opengp-ui/src/ui/components/patient/form.rs:1038-1078`
- **Action Type**: `PatientFormAction`
- **MouseEventKind Variants Handled**: 
  - `Up(MouseButton::Left)` only
- **Behavior**: 
  - Checks if click is within form area
  - Iterates through form fields to find which field was clicked
  - Returns `FocusChanged` if different field clicked
  - Returns `None` if same field clicked or outside bounds
- **Code**:
```rust
pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<PatientFormAction> {
    if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left) {
        return None;
    }

    if !area.contains(Position::new(mouse.column, mouse.row)) {
        return None;
    }

    let click_pos = Position::new(mouse.column, mouse.row);
    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    if !inner.contains(click_pos) {
        return None;
    }

    let mut y = inner.y + 1;
    let max_y = inner.y + inner.height - 2;

    for field_id in &self.field_ids {
        if y > max_y {
            break;
        }

        let field_height = self.get_field_height(field_id);
        let field_area = Rect::new(inner.x + 1, y, inner.width - 2, field_height);

        if field_area.contains(click_pos) {
            if *field_id != self.focused_field {
                self.focused_field = field_id.clone();
                self.sync_focus_to_state();
                return Some(PatientFormAction::FocusChanged);
            }
            return None;
        }

        y += field_height;
    }

    None
}
```

---

#### 2. **PatientList**
- **File**: `crates/opengp-ui/src/ui/components/patient/list.rs:292-335`
- **Action Type**: `PatientListAction`
- **MouseEventKind Variants Handled**: 
  - `ScrollUp`
  - `ScrollDown`
  - `Up(MouseButton::Left)`
- **Behavior**:
  - Scroll wheel: 3 rows per scroll event
  - Left-click: Selects clicked row (accounts for scroll offset)
  - Skips header row (HEADER_HEIGHT constant)
- **Code**:
```rust
pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<PatientListAction> {
    // Handle mouse wheel for scrolling
    if let MouseEventKind::ScrollUp = mouse.kind {
        for _ in 0..3 {
            self.scrollable.scroll_up();
        }
        return Some(PatientListAction::SelectionChanged);
    }
    if let MouseEventKind::ScrollDown = mouse.kind {
        let visible_rows = area.height.saturating_sub(3) as usize;
        let max_scroll = self.filtered.len().saturating_sub(visible_rows);
        for _ in 0..3 {
            if self.scrollable.scroll_offset() < max_scroll {
                self.scrollable.scroll_down();
            }
        }
        return Some(PatientListAction::SelectionChanged);
    }

    if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left) {
        return None;
    }

    if !area.contains(Position::new(mouse.column, mouse.row)) {
        return None;
    }

    if mouse.row < area.y + HEADER_HEIGHT {
        return None;
    }

    let row_index = (mouse.row - area.y - HEADER_HEIGHT) as usize;
    let actual_index = self.scrollable.scroll_offset() + row_index;
    if actual_index < self.filtered.len() {
        let current_index = self.scrollable.selected_index();
        let offset = actual_index as isize - current_index as isize;
        self.scrollable.move_by(offset);
        Some(PatientListAction::SelectionChanged)
    } else {
        None
    }
}
```

---

#### 3. **Calendar** (Component)
- **File**: `crates/opengp-ui/src/ui/components/appointment/calendar.rs:207-230`
- **Action Type**: `CalendarAction`
- **MouseEventKind Variants Handled**: 
  - `ScrollUp` (next month)
  - `ScrollDown` (previous month)
  - `Up(MouseButton::Left)` (select date)
- **Behavior**:
  - Scroll wheel changes month
  - Left-click delegates to underlying `CalendarWidget`
  - Returns `SelectDate(date)` action
- **Code**:
```rust
pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<CalendarAction> {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            self.widget.next_month();
            self.rebuild_days();
            Some(CalendarAction::MonthChanged(self.current_month))
        }
        MouseEventKind::ScrollDown => {
            self.widget.prev_month();
            self.rebuild_days();
            Some(CalendarAction::MonthChanged(self.current_month))
        }
        MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
            if let Some(_action) = self.widget.handle_mouse(mouse, area) {
                let selected = self.widget.focused_date;
                self.rebuild_days();
                Some(CalendarAction::SelectDate(selected))
            } else {
                None
            }
        }
        _ => None,
    }
}
```

---

#### 4. **AppointmentState** (Schedule)
- **File**: `crates/opengp-ui/src/ui/components/appointment/state.rs:469-535`
- **Action Type**: `ScheduleAction`
- **MouseEventKind Variants Handled**: 
  - `Up(MouseButton::Left)` (select practitioner/appointment/time slot)
  - `ScrollUp` (scroll viewport up by 1 hour)
  - `ScrollDown` (scroll viewport down by 1 hour)
- **Behavior**:
  - Left-click: Calculates which practitioner column and time slot was clicked
  - Scroll wheel: Adjusts viewport hour range (min_hour to max_hour from config)
  - Returns `SelectPractitioner(id)` or `SelectAppointment(id)` or `NavigateTimeSlot(slot)`
- **Code**:
```rust
pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<ScheduleAction> {
    use crate::ui::layout::TIME_COLUMN_WIDTH;

    match mouse.kind {
        MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
            let time_column_width = TIME_COLUMN_WIDTH;
            let inner = area.inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            });

            let y = mouse.row.saturating_sub(inner.y);
            let slot = (y as u8 / 2).min(self.max_time_slot());
            self.selected_time_slot = slot;

            if mouse.column < inner.x + time_column_width {
                return Some(ScheduleAction::NavigateTimeSlot(0));
            }

            let col = mouse.column.saturating_sub(inner.x + time_column_width);
            let practitioner_cols = inner.width.saturating_sub(time_column_width);

            if practitioner_cols > 0 && !self.practitioners_view.is_empty() {
                let col_width = practitioner_cols / self.practitioners_view.len() as u16;
                if col_width > 0 {
                    let practitioner_index = (col / col_width) as usize;
                    if practitioner_index < self.practitioners_view.len() {
                        self.selected_practitioner_index = practitioner_index;

                        if let Some(apt) = self.get_appointment_at_slot_for_practitioner(
                            slot,
                            self.selected_practitioner_index,
                        ) {
                            return Some(ScheduleAction::SelectAppointment(apt.id));
                        }

                        return Some(ScheduleAction::SelectPractitioner(
                            self.practitioners_view[practitioner_index].id,
                        ));
                    }
                }
            }
            None
        }
        MouseEventKind::ScrollUp => {
            let min_hour = self.config.min_hour;
            if self.viewport_start_hour > min_hour {
                self.viewport_start_hour = self.viewport_start_hour.saturating_sub(1);
                self.viewport_end_hour = self.viewport_end_hour.saturating_sub(1);
                self.scroll_viewport_to_show_selection();
            }
            None
        }
        MouseEventKind::ScrollDown => {
            let max_hour = self.config.max_hour;
            let window_hours = self.viewport_end_hour - self.viewport_start_hour;
            if self.viewport_end_hour < max_hour {
                self.viewport_start_hour =
                    (self.viewport_start_hour + 1).min(max_hour - window_hours);
                self.viewport_end_hour = (self.viewport_end_hour + 1).min(max_hour);
                self.scroll_viewport_to_show_selection();
            }
            None
        }
        _ => None,
    }
}
```

---

#### 5. **ConsultationList**
- **File**: `crates/opengp-ui/src/ui/components/clinical/consultation_list.rs:134-147`
- **Action Type**: `ConsultationListAction`
- **MouseEventKind Variants Handled**: Delegates to `ClinicalTableList`
- **Behavior**: Wraps `ClinicalTableList::handle_mouse()`, maps `ListAction::Select` to `ConsultationListAction::Select`
- **Code**:
```rust
pub fn handle_mouse(
    &mut self,
    mouse: MouseEvent,
    area: Rect,
) -> Option<ConsultationListAction> {
    let mut table = self.table();
    let out = match table.handle_mouse(mouse, area) {
        Some(ListAction::Select(i)) => Some(ConsultationListAction::Select(i)),
        _ => None,
    };
    self.selected_index = table.selected_index;
    self.scroll_offset = table.scroll_offset;
    out
}
```

---

#### 6. **VitalSignsList**
- **File**: `crates/opengp-ui/src/ui/components/clinical/vitals_list.rs:151-161`
- **Action Type**: `VitalSignsListAction`
- **MouseEventKind Variants Handled**: Delegates to `ClinicalTableList`
- **Behavior**: Wraps `ClinicalTableList::handle_mouse()`, maps `ListAction::Select` to `VitalSignsListAction::Select`
- **Code**:
```rust
pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<VitalSignsListAction> {
    let mut table = self.table();
    let action = table.handle_mouse(mouse, area).map(|a| match a {
        ListAction::Select(i) => VitalSignsListAction::Select(i),
        _ => unreachable!("unexpected clinical table mouse action for vitals"),
    });
    self.vitals = table.items;
    self.selected_index = table.selected_index;
    self.scroll_offset = table.scroll_offset;
    action
}
```

---

#### 7. **AllergyList**
- **File**: `crates/opengp-ui/src/ui/components/clinical/allergy_list.rs:84-93`
- **Action Type**: `AllergyListAction`
- **MouseEventKind Variants Handled**: Delegates to `ClinicalTableList`
- **Behavior**: Wraps `ClinicalTableList::handle_mouse()`, maps `ListAction::Select` to `AllergyListAction::Select`
- **Code**:
```rust
pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<AllergyListAction> {
    let mut table = self.table();
    let out = match table.handle_mouse(mouse, area) {
        Some(ListAction::Select(i)) => Some(AllergyListAction::Select(i)),
        _ => None,
    };
    self.selected_index = table.selected_index;
    self.scroll_offset = table.scroll_offset;
    out
}
```

---

#### 8. **MedicalHistoryList**
- **File**: `crates/opengp-ui/src/ui/components/clinical/medical_history_list.rs:42-50`
- **Action Type**: `ListAction<MedicalHistory>` (type alias)
- **MouseEventKind Variants Handled**: Delegates to `ClinicalTableList`
- **Behavior**: Wraps `ClinicalTableList::handle_mouse()`, syncs navigation state
- **Code**:
```rust
pub fn handle_mouse(
    &mut self,
    mouse: MouseEvent,
    area: Rect,
) -> Option<MedicalHistoryListAction> {
    let mut list = self.table_list();
    let action = list.handle_mouse(mouse, area);
    self.sync_nav(&list);
    action
}
```

---

#### 9. **FamilyHistoryList**
- **File**: `crates/opengp-ui/src/ui/components/clinical/family_history_list.rs:67-77`
- **Action Type**: `FamilyHistoryListAction`
- **MouseEventKind Variants Handled**: Delegates to `ClinicalTableList`
- **Behavior**: Wraps `ClinicalTableList::handle_mouse()`, maps actions via `map_action()` helper
- **Code**:
```rust
pub fn handle_mouse(
    &mut self,
    mouse: MouseEvent,
    area: Rect,
) -> Option<FamilyHistoryListAction> {
    let mut table = self.as_table();
    let action = table.handle_mouse(mouse, area).and_then(map_action);
    self.selected_index = table.selected_index;
    self.scroll_offset = table.scroll_offset;
    action
}

fn map_action(action: ListAction<FamilyHistory>) -> Option<FamilyHistoryListAction> {
    match action {
        ListAction::Select(index) => Some(FamilyHistoryListAction::Select(index)),
        ListAction::Open(entry) => Some(FamilyHistoryListAction::Open(entry)),
        ListAction::New => Some(FamilyHistoryListAction::New),
        ListAction::Delete(entry) => Some(FamilyHistoryListAction::Delete(entry)),
        ListAction::Edit(_) | ListAction::ToggleInactive => None,
    }
}
```

---

#### 10. **TabBar**
- **File**: `crates/opengp-ui/src/ui/components/tabs.rs:176-196`
- **Action Type**: `Tab` (enum of tab variants)
- **MouseEventKind Variants Handled**: 
  - `Up(MouseButton::Left)` only
- **Behavior**:
  - Calculates tab width based on number of tabs
  - Determines which tab was clicked based on column position
  - Returns the selected `Tab` enum variant
- **Code**:
```rust
pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<Tab> {
    if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left) {
        return None;
    }

    // Check if click is within the tab bar area
    if !area.contains(Position::new(mouse.column, mouse.row)) {
        return None;
    }

    // Calculate tab width (each tab has a fixed width)
    let tab_width = area.width as usize / self.tabs.len().max(1);
    let click_index = (mouse.column.saturating_sub(area.x)) as usize / tab_width.max(1);

    if click_index < self.tabs.len() {
        self.selected = self.tabs[click_index].tab;
        return Some(self.selected);
    }

    None
}
```

---

### **Widget Implementations (4)**

#### 11. **CalendarWidget** (Widget)
- **File**: `crates/opengp-ui/src/ui/widgets/calendar.rs:130-142`
- **Action Type**: `CalendarAction`
- **MouseEventKind Variants Handled**: 
  - `Up(MouseButton::Left)` only
- **Behavior**:
  - Calculates which day cell was clicked
  - Returns `SelectDate` if valid day found
- **Code**:
```rust
pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<CalendarAction> {
    if let MouseEventKind::Up(MouseButton::Left) = mouse.kind {
        if let Some(day_index) = self.get_day_index_at(mouse.column, mouse.row, area) {
            if let Some(date) = self.day_at_index(day_index) {
                self.focused_date = date;
                self.selected_date = Some(date);
                return Some(CalendarAction::SelectDate);
            }
        }
    }

    None
}
```

---

#### 12. **ClinicalTableList<T>** (Generic Widget)
- **File**: `crates/opengp-ui/src/ui/widgets/clinical_table_list.rs:190-225`
- **Action Type**: `ListAction<T>`
- **MouseEventKind Variants Handled**: 
  - `ScrollUp` (3 rows)
  - `ScrollDown` (3 rows)
  - `Up(MouseButton::Left)` (select row)
- **Behavior**:
  - Scroll wheel: 3 rows per event
  - Left-click: Selects clicked row (accounts for scroll offset and header)
  - Skips header rows (TABLE_HEADER_ROWS constant)
- **Code**:
```rust
pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<ListAction<T>> {
    if let MouseEventKind::ScrollUp = mouse.kind {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(3);
        }
        return Some(ListAction::Select(self.selected_index));
    }

    if let MouseEventKind::ScrollDown = mouse.kind {
        let visible_rows = area.height.saturating_sub(3) as usize;
        let max_scroll = self.items.len().saturating_sub(visible_rows);
        self.scroll_offset = (self.scroll_offset + 3).min(max_scroll);
        return Some(ListAction::Select(self.selected_index));
    }

    if mouse.kind != MouseEventKind::Up(MouseButton::Left) {
        return None;
    }

    if !area.contains(Position::new(mouse.column, mouse.row)) {
        return None;
    }

    if mouse.row < area.y + TABLE_HEADER_ROWS {
        return None;
    }

    let row_index = (mouse.row - area.y - TABLE_HEADER_ROWS) as usize;
    let actual_index = self.scroll_offset + row_index;
    if actual_index < self.items.len() {
        self.selected_index = actual_index;
        Some(ListAction::Select(self.selected_index))
    } else {
        None
    }
}
```

---

#### 13. **DropdownWidget**
- **File**: `crates/opengp-ui/src/ui/widgets/dropdown.rs:254-288`
- **Action Type**: `DropdownAction`
- **MouseEventKind Variants Handled**: 
  - `Up(MouseButton::Left)` only
- **Behavior**:
  - If closed: Opens dropdown
  - If open: Selects clicked option (calculates relative Y position)
  - Click outside: Closes dropdown
- **Code**:
```rust
pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<DropdownAction> {
    if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left) {
        return None;
    }

    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    if !inner.contains((mouse.column, mouse.row).into()) {
        if self.is_open() {
            self.close();
            return Some(DropdownAction::Closed);
        }
        return None;
    }

    if self.is_open() {
        let _option_height = 1;
        let header_height = 1;
        let options_area_start = inner.y + header_height;
        let mouse_y = mouse.row;

        if mouse_y >= options_area_start {
            let relative_y = (mouse_y - options_area_start) as usize;
            if relative_y < self.options.len() {
                self.focused_index = relative_y;
                self.confirm_selection();
                return Some(DropdownAction::Selected(self.selected_index));
            }
        }
    } else {
        self.open();
        return Some(DropdownAction::Opened);
    }

    None
}
```

---

#### 14. **TestComponent** (Test Implementation)
- **File**: `crates/opengp-ui/src/ui/input.rs:205-236`
- **Action Type**: `TestAction`
- **MouseEventKind Variants Handled**: All (generic test)
- **Behavior**: Test-only implementation demonstrating trait usage
- **Code**:
```rust
#[test]
fn handle_mouse_trait_can_be_implemented() {
    #[derive(Debug)]
    enum TestAction {
        Clicked,
    }

    struct TestComponent;

    impl HandleMouse for TestComponent {
        type Action = TestAction;

        fn handle_mouse(&mut self, _mouse: MouseEvent, _area: Rect) -> Option<Self::Action> {
            Some(TestAction::Clicked)
        }
    }

    let mut component = TestComponent;
    let mouse = MouseEvent {
        kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: 10,
        row: 5,
        modifiers: KeyModifiers::NONE,
    };
    let area = Rect {
        x: 0,
        y: 0,
        width: 20,
        height: 10,
    };
    let action = component.handle_mouse(mouse, area);
    assert!(matches!(action, Some(TestAction::Clicked)));
}
```

---

## 3. GLOBAL MOUSE DISPATCHER

**File**: `/home/stephenp/Documents/opengp/crates/opengp-ui/src/ui/app/event_handler/global.rs:9-134`

### **Full Dispatcher Code**

```rust
impl App {
    pub fn handle_global_mouse_event(&mut self, mouse: MouseEvent, area: Rect) {
        // ============================================================
        // PRIORITY 1: TAB BAR (highest priority - always checked first)
        // ============================================================
        let tab_bar_area = self.tab_bar.area(area);
        if self.tab_bar.handle_mouse(mouse, tab_bar_area).is_some() {
            self.refresh_status_bar();
            self.refresh_context();
            return;  // Early exit - don't process other components
        }

        // ============================================================
        // PRIORITY 2: PATIENT FORM (if open - modal-like behavior)
        // ============================================================
        if let Some(ref mut form) = self.patient_form {
            if let Some(action) = form.handle_mouse(mouse, area) {
                match action {
                    crate::ui::components::patient::PatientFormAction::FocusChanged => {}
                    crate::ui::components::patient::PatientFormAction::ValueChanged => {}
                    crate::ui::components::patient::PatientFormAction::Submit => {}
                    crate::ui::components::patient::PatientFormAction::Cancel => {}
                    crate::ui::components::patient::PatientFormAction::SaveComplete => {
                        self.request_refresh_patients();
                    }
                }
                return;  // Early exit - form consumed event
            }
        }

        // ============================================================
        // PRIORITY 3: PATIENT LIST (if Patient tab active and no form)
        // ============================================================
        if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
            let content_area = Rect::new(
                area.x,
                area.y + 2,  // Skip tab bar (2 rows)
                area.width,
                area.height.saturating_sub(2 + STATUS_BAR_HEIGHT),  // Skip status bar
            );
            if let Some(action) = self.patient_list.handle_mouse(mouse, content_area) {
                match action {
                    crate::ui::components::patient::PatientListAction::SelectionChanged => {}
                    crate::ui::components::patient::PatientListAction::OpenPatient(id) => {
                        self.request_edit_patient(id);
                    }
                    crate::ui::components::patient::PatientListAction::FocusSearch => {}
                    crate::ui::components::patient::PatientListAction::SearchChanged => {}
                }
            }
        }

        // ============================================================
        // PRIORITY 4: APPOINTMENT TAB (Calendar or Schedule view)
        // ============================================================
        if self.tab_bar.selected() == Tab::Appointment {
            use crate::ui::components::appointment::schedule::ScheduleAction;

            let appointment_content_area = Rect::new(
                area.x,
                area.y + 2,
                area.width,
                area.height.saturating_sub(2 + STATUS_BAR_HEIGHT),
            );

            match self.appointment_state.current_view {
                // ---- Calendar-only view ----
                AppointmentView::Calendar => {
                    self.appointment_state.calendar.focused = true;
                    self.appointment_state.focused = false;
                    if let Some(action) = self
                        .appointment_state
                        .calendar
                        .handle_mouse(mouse, appointment_content_area)
                    {
                        match action {
                            crate::ui::components::appointment::CalendarAction::SelectDate(
                                date,
                            ) => {
                                self.appointment_state.selected_date = Some(date);
                                self.appointment_state.current_view = AppointmentView::Schedule;
                                self.request_refresh_appointments(date);
                                self.refresh_context();
                            }
                            crate::ui::components::appointment::CalendarAction::FocusDate(_) => {}
                            crate::ui::components::appointment::CalendarAction::MonthChanged(_) => {
                            }
                            crate::ui::components::appointment::CalendarAction::GoToToday => {}
                        }
                    }
                }
                // ---- Schedule view (split layout) ----
                AppointmentView::Schedule => {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
                        .split(appointment_content_area);

                    // Left side: Calendar (25%) - only scroll events
                    use crossterm::event::MouseEventKind;
                    if let MouseEventKind::Up(_) | MouseEventKind::Down(_) = mouse.kind {
                        if let Some(action) = self
                            .appointment_state
                            .calendar
                            .handle_mouse(mouse, chunks[0])
                        {
                            self.appointment_state.calendar.focused = true;
                            self.appointment_state.focused = false;
                            match action {
                                crate::ui::components::appointment::CalendarAction::SelectDate(date) => {
                                    self.appointment_state.selected_date = Some(date);
                                    self.request_refresh_appointments(date);
                                }
                                crate::ui::components::appointment::CalendarAction::FocusDate(_) => {}
                                crate::ui::components::appointment::CalendarAction::MonthChanged(_) => {}
                                crate::ui::components::appointment::CalendarAction::GoToToday => {}
                            }
                        }
                    }

                    // Right side: Schedule (75%) - all events
                    if let Some(action) = self.appointment_state.handle_mouse(mouse, chunks[1]) {
                        self.appointment_state.focused = true;
                        self.appointment_state.calendar.focused = false;
                        match action {
                            ScheduleAction::SelectPractitioner(id) => {
                                self.appointment_state.selected_practitioner = Some(id);
                            }
                            ScheduleAction::SelectAppointment(id) => {
                                self.appointment_state.selected_appointment = Some(id);
                            }
                            ScheduleAction::NavigateTimeSlot(_) => {}
                            ScheduleAction::NavigatePractitioner(_) => {}
                            ScheduleAction::ToggleColumn => {}
                            ScheduleAction::CreateAtSlot { .. } => {}
                        }
                    }
                }
            }
        }
    }
}
```

### **Dispatcher Design Pattern**

```
Mouse Event
    ↓
[1] Tab Bar? → Yes → Handle & Return (early exit)
    ↓ No
[2] Patient Form Open? → Yes → Handle & Return (early exit)
    ↓ No
[3] Patient Tab Active? → Yes → Handle Patient List
    ↓ No
[4] Appointment Tab Active? → Yes → Handle Calendar or Schedule
    ↓ No
[End] Event not handled
```

**Key Characteristics**:
- **Priority-based**: Tab bar > Form > List > Appointment
- **Early exit**: Each handler returns immediately if event consumed
- **Area calculation**: Each component gets adjusted `Rect` (skips tab bar, status bar)
- **Layout-aware**: Schedule view splits area 25/75 for calendar/schedule
- **Stateful**: Updates component state directly (e.g., `selected_date`, `focused`)

---

## 4. SCROLL INCREMENTS - HARDCODED MAGIC NUMBERS

### **Critical Finding**: Scroll increments are **NOT configurable constants** — they are hardcoded throughout the codebase.

| Component | Scroll Amount | Location | Code |
|-----------|---------------|----------|------|
| **PatientList** | **3 rows** | `list.rs:295, 303` | `for _ in 0..3 { self.scrollable.scroll_up(); }` |
| **ClinicalTableList** | **3 rows** | `clinical_table_list.rs:193, 201` | `self.scroll_offset.saturating_sub(3)` |
| **Calendar (Component)** | **1 month** | `calendar.rs:210, 215` | `self.widget.next_month()` / `prev_month()` |
| **AppointmentState (Schedule)** | **1 hour** | `state.rs:516-517, 526-528` | `viewport_start_hour.saturating_sub(1)` |
| **list_nav helper** | **3 rows** | `list_nav.rs:99-111` | `for _ in 0..3 { scrollable.scroll_up(); }` |

### **Detailed Code Examples**

#### PatientList (lines 294-309)
```rust
if let MouseEventKind::ScrollUp = mouse.kind {
    for _ in 0..3 {  // ← HARDCODED 3
        self.scrollable.scroll_up();
    }
    return Some(PatientListAction::SelectionChanged);
}
if let MouseEventKind::ScrollDown = mouse.kind {
    let visible_rows = area.height.saturating_sub(3) as usize;
    let max_scroll = self.filtered.len().saturating_sub(visible_rows);
    for _ in 0..3 {  // ← HARDCODED 3
        if self.scrollable.scroll_offset() < max_scroll {
            self.scrollable.scroll_down();
        }
    }
    return Some(PatientListAction::SelectionChanged);
}
```

#### ClinicalTableList (lines 191-203)
```rust
if let MouseEventKind::ScrollUp = mouse.kind {
    if self.scroll_offset > 0 {
        self.scroll_offset = self.scroll_offset.saturating_sub(3);  // ← HARDCODED 3
    }
    return Some(ListAction::Select(self.selected_index));
}

if let MouseEventKind::ScrollDown = mouse.kind {
    let visible_rows = area.height.saturating_sub(3) as usize;
    let max_scroll = self.items.len().saturating_sub(visible_rows);
    self.scroll_offset = (self.scroll_offset + 3).min(max_scroll);  // ← HARDCODED 3
    return Some(ListAction::Select(self.selected_index));
}
```

#### AppointmentState Schedule (lines 513-531)
```rust
MouseEventKind::ScrollUp => {
    let min_hour = self.config.min_hour;
    if self.viewport_start_hour > min_hour {
        self.viewport_start_hour = self.viewport_start_hour.saturating_sub(1);  // ← 1 hour
        self.viewport_end_hour = self.viewport_end_hour.saturating_sub(1);      // ← 1 hour
        self.scroll_viewport_to_show_selection();
    }
    None
}
MouseEventKind::ScrollDown => {
    let max_hour = self.config.max_hour;
    let window_hours = self.viewport_end_hour - self.viewport_start_hour;
    if self.viewport_end_hour < max_hour {
        self.viewport_start_hour =
            (self.viewport_start_hour + 1).min(max_hour - window_hours);  // ← 1 hour
        self.viewport_end_hour = (self.viewport_end_hour + 1).min(max_hour);  // ← 1 hour
        self.scroll_viewport_to_show_selection();
    }
    None
}
```

#### list_nav helper (lines 98-112)
```rust
pub fn list_handle_mouse(
    mouse: MouseEvent,
    area: Rect,
    header_height: u16,
    scrollable: &mut ScrollableState,
    visible_rows: usize,
) -> Option<ListNavAction> {
    // Handle mouse wheel scrolling
    if let MouseEventKind::ScrollUp = mouse.kind {
        for _ in 0..3 {  // ← HARDCODED 3
            scrollable.scroll_up();
        }
        return Some(ListNavAction::SelectionChanged);
    }
    if let MouseEventKind::ScrollDown = mouse.kind {
        let max_scroll = scrollable.item_count().saturating_sub(visible_rows);
        for _ in 0..3 {  // ← HARDCODED 3
            if scrollable.scroll_offset() < max_scroll {
                scrollable.scroll_down();
            }
        }
        return Some(ListNavAction::SelectionChanged);
    }
    // ...
}
```

### **Impact**
- ❌ No way to customize scroll speed per component
- ❌ No global scroll configuration
- ❌ Inconsistent: lists use 3, schedule uses 1
- ⚠️ Would require code changes to adjust (not config-driven)

---

## 5. HOVER STATE TRACKING

### **CONFIRMED: NO HOVER STATE EXISTS**

**Evidence**:

1. **No hover fields in any component**:
   - PatientList: `patients`, `filtered`, `search_query`, `searching`, `scrollable`, `loading`, `loading_state`, `theme` — NO hover
   - Calendar: `widget`, `theme`, `focused`, `days`, `appointment_indicators`, `current_month`, `selected_date`, `focused_date` — NO hover
   - AppointmentState: `current_view`, `calendar`, `schedule`, `selected_date`, `schedule_data`, `practitioners`, `selected_practitioner`, `selected_appointment`, `loading_state`, `loading`, `hidden_columns`, `practitioners_view`, `selected_practitioner_index`, `selected_time_slot`, `viewport_start_hour`, `viewport_end_hour`, `last_inner_height`, `focused`, `config`, `debug_overlay_visible` — NO hover

2. **No `MouseEventKind::Moved` handling**:
   - All implementations only handle: `ScrollUp`, `ScrollDown`, `Up(MouseButton::Left)`
   - No `Moved` variant processed anywhere
   - No position tracking between events

3. **No visual feedback for mouse position**:
   - Rendering code doesn't check for hover state
   - No special styling for "hovered" items
   - Only `selected_index` affects rendering

4. **Test confirms absence**:
   ```rust
   // No tests for hover behavior
   // No hover-related assertions
   ```

### **Conclusion**
Hover state is completely absent. The TUI only tracks:
- **Selection** (which item is selected)
- **Focus** (which component has keyboard focus)
- **Scroll offset** (viewport position)

---

## 6. RIGHT-CLICK HANDLING

### **CONFIRMED: NO RIGHT-CLICK HANDLING**

**Evidence**:

1. **Only `MouseButton::Left` handled**:
   - PatientForm: `Up(MouseButton::Left)` only
   - PatientList: `Up(MouseButton::Left)` only
   - Calendar: `Up(MouseButton::Left)` only
   - AppointmentState: `Up(MouseButton::Left)` only
   - TabBar: `Up(MouseButton::Left)` only
   - All other components: same pattern

2. **Explicit test confirms right-click returns None**:
   ```rust
   // widgets/list_nav.rs
   #[test]
   fn test_list_handle_mouse_right_click_returns_none() {
       let mouse = MouseEvent {
           kind: MouseEventKind::Up(MouseButton::Right),  // ← Right-click
           column: 10,
           row: 5,
           modifiers: KeyModifiers::NONE,
       };
       let area = Rect { x: 0, y: 0, width: 20, height: 10 };
       let mut scrollable = ScrollableState::new();
       scrollable.set_item_count(10);
       
       let result = list_handle_mouse(mouse, area, 1, &mut scrollable, 5);
       assert_eq!(result, None);  // ← Explicitly returns None
   }
   ```

3. **No context menu infrastructure**:
   - No `ContextMenu` struct
   - No `ContextMenuAction` enum
   - No right-click action handlers
   - No context menu state management

4. **No `MouseButton::Middle` handling either**:
   - Only `Left` button is checked
   - Middle-click also returns `None`

### **Conclusion**
Right-click is completely unimplemented. The codebase explicitly ignores `MouseButton::Right` and `MouseButton::Middle`.

---

## 7. BOUNDS/HIT TESTING

### **Pattern: Ratatui's `Rect::contains(Position)`**

All implementations use the same pattern for hit testing:

```rust
if !area.contains(Position::new(mouse.column, mouse.row)) {
    return None;
}
```

### **Detailed Example: PatientForm (lines 1043-1065)**

```rust
pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<PatientFormAction> {
    // Step 1: Check if click is within outer area (including border)
    if !area.contains(Position::new(mouse.column, mouse.row)) {
        return None;
    }

    let click_pos = Position::new(mouse.column, mouse.row);

    // Step 2: Calculate inner area (subtract border: 1px on each side)
    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    if !inner.contains(click_pos) {
        return None;
    }

    // Step 3: Iterate through fields, calculating each field's area dynamically
    let mut y = inner.y + 1;
    let max_y = inner.y + inner.height - 2;

    for field_id in &self.field_ids {
        if y > max_y {
            break;
        }

        // Get field height (varies by field type: text, textarea, dropdown)
        let field_height = self.get_field_height(field_id);
        
        // Create area for this field
        let field_area = Rect::new(inner.x + 1, y, inner.width - 2, field_height);

        // Step 4: Check if click is within this field
        if field_area.contains(click_pos) {
            if *field_id != self.focused_field {
                self.focused_field = field_id.clone();
                self.sync_focus_to_state();
                return Some(PatientFormAction::FocusChanged);
            }
            return None;
        }

        y += field_height;
    }

    None
}
```

### **Helper Function: list_nav.rs (lines 120-127)**

```rust
pub fn list_handle_mouse(
    mouse: MouseEvent,
    area: Rect,
    header_height: u16,
    scrollable: &mut ScrollableState,
    visible_rows: usize,
) -> Option<ListNavAction> {
    // ... scroll handling ...

    // Check if click is within the list area
    if !area.contains(Position::new(mouse.column, mouse.row)) {
        return None;
    }

    // Check if click is in the header area (skip it)
    if mouse.row < area.y + header_height {
        return None;
    }

    // Calculate which row was clicked (relative to list content area)
    let row_index = (mouse.row - area.y - header_height) as usize;

    // Account for scroll offset to get actual item index
    let actual_index = scrollable.scroll_offset() + row_index;

    // Only select if within list bounds
    if actual_index < scrollable.item_count() {
        // Move selection to clicked item
        let current_index = scrollable.selected_index();
        let offset = actual_index as isize - current_index as isize;
        scrollable.move_by(offset);
        Some(ListNavAction::SelectionChanged)
    } else {
        None
    }
}
```

### **Schedule View Hit Testing: AppointmentState (lines 474-510)**

```rust
MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
    let time_column_width = TIME_COLUMN_WIDTH;
    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });

    // Calculate which time slot was clicked
    let y = mouse.row.saturating_sub(inner.y);
    let slot = (y as u8 / 2).min(self.max_time_slot());
    self.selected_time_slot = slot;

    // Check if click is in time column (left side)
    if mouse.column < inner.x + time_column_width {
        return Some(ScheduleAction::NavigateTimeSlot(0));
    }

    // Calculate which practitioner column was clicked
    let col = mouse.column.saturating_sub(inner.x + time_column_width);
    let practitioner_cols = inner.width.saturating_sub(time_column_width);

    if practitioner_cols > 0 && !self.practitioners_view.is_empty() {
        let col_width = practitioner_cols / self.practitioners_view.len() as u16;
        if col_width > 0 {
            let practitioner_index = (col / col_width) as usize;
            if practitioner_index < self.practitioners_view.len() {
                self.selected_practitioner_index = practitioner_index;

                // Check if click is on an existing appointment
                if let Some(apt) = self.get_appointment_at_slot_for_practitioner(
                    slot,
                    self.selected_practitioner_index,
                ) {
                    return Some(ScheduleAction::SelectAppointment(apt.id));
                }

                // Otherwise select the practitioner/time slot
                return Some(ScheduleAction::SelectPractitioner(
                    self.practitioners_view[practitioner_index].id,
                ));
            }
        }
    }
    None
}
```

### **Key Characteristics**

1. **Rect::contains() is the primary method**:
   - Checks if `(column, row)` is within `Rect` bounds
   - Simple, efficient, no special logic needed

2. **Bounds are recalculated on each mouse event**:
   - No persistent `last_render_area` stored
   - Areas calculated fresh each time
   - Means bounds must match render logic exactly

3. **Scroll offset is accounted for**:
   - `actual_index = scroll_offset + row_index`
   - Ensures clicks map to correct items even when scrolled

4. **Header rows are skipped**:
   - `if mouse.row < area.y + header_height { return None; }`
   - Prevents clicking on column headers from selecting items

5. **Complex layouts use nested areas**:
   - Schedule view: outer area → inner area → time column + practitioner columns
   - Each level has its own bounds check

---

## 8. APPCOMMAND ENUM

**File**: `/home/stephenp/Documents/opengp/crates/opengp-ui/src/ui/app/command.rs:8-50`

### **Full Enum Definition**

```rust
#[derive(Debug)]
pub enum AppCommand {
    // ============================================================
    // APPOINTMENT COMMANDS
    // ============================================================
    
    /// Refresh appointments for a specific date
    RefreshAppointments(NaiveDate),
    
    /// Create a new appointment
    CreateAppointment(NewAppointmentData),
    
    /// Update an existing appointment
    UpdateAppointment {
        id: Uuid,
        data: NewAppointmentData,
        version: i32,
    },
    
    /// Result of appointment save operation
    AppointmentSaveResult(Result<(), String>),
    
    /// Update appointment status (scheduled, confirmed, arrived, etc.)
    UpdateAppointmentStatus {
        id: Uuid,
        status: AppointmentStatus,
    },
    
    /// Load list of practitioners
    LoadPractitioners,
    
    /// Load available time slots for a practitioner on a date
    LoadAvailableSlots {
        practitioner_id: Uuid,
        date: NaiveDate,
        duration_minutes: u32,
    },
    
    /// Cancel an appointment with reason
    CancelAppointment {
        id: Uuid,
        reason: String,
    },
    
    /// Reschedule an appointment to a new date/time
    RescheduleAppointment {
        id: Uuid,
        new_start_time: DateTime<Utc>,
        new_duration_minutes: i64,
        user_id: Uuid,
    },

    // ============================================================
    // CLINICAL/BILLING COMMANDS
    // ============================================================
    
    /// Save clinical data (consultation, vitals, allergies, etc.)
    SaveClinicalData {
        patient_id: Uuid,
        data: PendingClinicalSaveData,
    },
    
    /// Save billing data (MBS selection, invoices, etc.)
    SaveBillingData {
        patient_id: Uuid,
        data: PendingBillingSaveData,
    },
    
    /// Load patient workspace data for a specific subtab
    LoadPatientWorkspaceData {
        patient_id: Uuid,
        subtab: SubtabKind,
    },
}
```

### **Key Characteristics**

1. **Async-only channel**:
   - Sent via `command_tx: mpsc::Sender<AppCommand>`
   - Processed in main event loop
   - Results come back on channel

2. **No "open detail view" command**:
   - Detail views are opened by **direct state mutation**
   - Example: `self.appointment_detail_modal = Some(AppointmentDetailModal::new(...))`
   - NOT via AppCommand

3. **Focused on data operations**:
   - Load/refresh data
   - Create/update/delete items
   - Save form data
   - NOT UI state changes

4. **Used in event handlers**:
   ```rust
   // Example from appointment/event_handler.rs:97-100
   let _ = self.command_tx.send(AppCommand::LoadPatientWorkspaceData {
       patient_id,
       subtab: SubtabKind::Clinical,
   });
   ```

5. **Variants with associated data**:
   - Some are simple: `RefreshAppointments(NaiveDate)`
   - Some are complex: `UpdateAppointment { id, data, version }`
   - Some are result types: `AppointmentSaveResult(Result<(), String>)`

---

## 9. ENTER KEY → DETAIL VIEW FLOW

### **Complete Flow Trace**

#### **Step 1: Keybind Registry** (`keybinds.rs:421-424`)

```rust
KeybindEntry {
    key: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    action: Action::OpenPatientFromList,
    description: "Open patient clinical record",
}
```

**Other Enter keybinds**:
```rust
// Calendar context (line 531-532)
KeybindEntry {
    key: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    action: Action::Enter,
    description: "Select focused date",
}

// Schedule context (line 631-632)
KeybindEntry {
    key: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    action: Action::Enter,
    description: "Open appointment detail",
}
```

#### **Step 2: Event Handler Dispatch** (`app/event_handler.rs`)

The main event loop receives the key event and dispatches based on context:

```rust
// Pseudo-code flow
pub fn handle_key_event(&mut self, key: KeyEvent) {
    let action = self.keybind_registry.lookup(key, self.current_context);
    
    match action {
        Action::OpenPatientFromList => {
            // Patient list context
            if let Some(patient) = self.patient_list.selected_patient() {
                self.request_edit_patient(patient.id);
            }
        }
        Action::Enter => {
            // Context-specific Enter handling
            match self.current_context {
                KeyContext::Calendar => {
                    // Select focused date, switch to schedule view
                    if let Some(date) = self.appointment_state.calendar.focused_date {
                        self.appointment_state.selected_date = Some(date);
                        self.appointment_state.current_view = AppointmentView::Schedule;
                    }
                }
                KeyContext::Schedule => {
                    // Open appointment detail modal
                    if let Some(apt_id) = self.appointment_state.selected_appointment {
                        self.appointment_detail_modal = 
                            Some(AppointmentDetailModal::new(apt_id));
                    }
                }
                // ... other contexts
            }
        }
        // ... other actions
    }
}
```

#### **Step 3: Detail View Opening** (Example: Appointment Detail Modal)

**File**: `crates/opengp-ui/src/ui/app/event_handler/appointment.rs:82-117`

```rust
pub(crate) fn handle_appointment_detail_modal_keys(&mut self, key: KeyEvent) -> Action {
    if let Some(ref mut modal) = self.appointment_detail_modal {
        if let Some(action) = modal.handle_key(key) {
            match action {
                // ... other actions ...
                
                AppointmentDetailModalAction::StartConsultation => {
                    let patient_id = modal.patient_id();
                    self.appointment_detail_modal = None;  // Close modal
                    
                    if let Some(patient_item) = self.patient_list.get_patient_by_id(patient_id) {
                        match self.workspace_manager.open_patient(patient_item.clone()) {
                            Ok(_index) => {
                                // Set active subtab to Clinical
                                if let Some(workspace) = self.workspace_manager.active_mut() {
                                    workspace.active_subtab = SubtabKind::Clinical;
                                }
                                
                                // Switch context to PatientWorkspace
                                self.current_context = KeyContext::PatientWorkspace;
                                
                                // Trigger lazy load of clinical data via AppCommand
                                let _ = self.command_tx.send(AppCommand::LoadPatientWorkspaceData {
                                    patient_id,
                                    subtab: SubtabKind::Clinical,
                                });
                                
                                self.refresh_status_bar();
                                self.refresh_context();
                            }
                            Err(err) => {
                                self.status_bar.set_error(err.to_string());
                            }
                        }
                    }
                }
                // ... other actions ...
            }
            return Action::Enter;
        }
    }
    Action::Unknown
}
```

### **Flow Diagram**

```
User presses Enter
    ↓
Keybind registry looks up Enter in current context
    ↓
Returns Action (e.g., Action::OpenPatientFromList)
    ↓
Event handler matches on Action
    ↓
Checks if item is selected
    ↓
If yes: Directly mutate state
    - self.appointment_detail_modal = Some(AppointmentDetailModal::new(...))
    - self.current_context = KeyContext::PatientWorkspace
    - self.command_tx.send(AppCommand::LoadPatientWorkspaceData { ... })
    ↓
Next render cycle shows the modal
```

### **Key Pattern**

1. **No AppCommand for opening modals**:
   - Modals are opened by **direct state mutation**
   - Not via async command channel

2. **Context determines action**:
   - Same key (Enter) does different things in different contexts
   - PatientList context: opens form
   - Schedule context: opens detail modal
   - Calendar context: selects date

3. **Modal state is stored in App**:
   - `self.appointment_detail_modal: Option<AppointmentDetailModal>`
   - `self.patient_form: Option<PatientForm>`
   - Checked in dispatcher to handle modal events

4. **Lazy loading via AppCommand**:
   - After opening modal, may send AppCommand to load data
   - Results come back on channel
   - State updated when data arrives

---

## 10. CONTEXT MENU & POPUP PATTERNS

### **CONFIRMED: NO CONTEXT MENUS EXIST**

**What DOES exist**:

### **Detail Modals (6 implementations)**

#### 1. **AppointmentDetailModal**
- **File**: `crates/opengp-ui/src/ui/components/appointment/detail_modal.rs`
- **Purpose**: Read-only appointment details with action buttons
- **Actions**:
  ```rust
  pub enum AppointmentDetailModalAction {
      Close,
      ViewClinicalNotes,
      MarkStatus(AppointmentStatus),
      StartConsultation,
      RescheduleDate,
      OpenTimePicker { practitioner_id, date, duration },
      RescheduleTime,
  }
  ```
- **State**:
  ```rust
  pub struct AppointmentDetailModal {
      appointment: Appointment,
      focused_action: usize,  // Which button is focused
      // ... other state
  }
  ```

#### 2. **ConsultationDetailModal**
- **File**: `crates/opengp-ui/src/ui/components/clinical/consultation_detail_modal.rs`
- **Purpose**: Read-only consultation with edit/delete buttons
- **Actions**: `Close`, `Edit`, `Delete`

#### 3. **AllergyDetailModal**
- **File**: `crates/opengp-ui/src/ui/components/clinical/allergy_detail_modal.rs`
- **Purpose**: Read-only allergy with edit/delete buttons
- **Actions**: `Close`, `Edit`, `Delete`

#### 4. **VitalsDetailModal**
- **File**: `crates/opengp-ui/src/ui/components/clinical/vitals_detail_modal.rs`
- **Purpose**: Read-only vitals with edit/delete buttons
- **Actions**: `Close`, `Edit`, `Delete`

#### 5. **MedicalHistoryDetailModal**
- **File**: `crates/opengp-ui/src/ui/components/clinical/medical_history_detail_modal.rs`
- **Purpose**: Read-only condition with edit/delete buttons
- **Actions**: `Close`, `Edit`, `Delete`

#### 6. **FamilyHistoryDetailModal**
- **File**: `crates/opengp-ui/src/ui/components/clinical/family_history_detail_modal.rs`
- **Purpose**: Read-only family history with edit/delete buttons
- **Actions**: `Close`, `Edit`, `Delete`

### **Detail Modal Pattern**

```rust
pub struct DetailModal {
    data: T,
    focused_action: usize,  // Which button is focused
    theme: Theme,
}

pub enum DetailModalAction {
    Close,
    Edit,
    Delete,
    // ... other actions
}

impl DetailModal {
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<DetailModalAction> {
        match key.code {
            KeyCode::Esc => Some(DetailModalAction::Close),
            KeyCode::Up => {
                self.focused_action = self.focused_action.saturating_sub(1);
                None
            }
            KeyCode::Down => {
                self.focused_action = (self.focused_action + 1).min(self.actions.len() - 1);
                None
            }
            KeyCode::Enter => {
                // Return action based on focused_action index
                Some(self.actions[self.focused_action].clone())
            }
            _ => None,
        }
    }
}
```

### **Date/Time Picker Popups (2 implementations)**

#### 1. **DatePickerPopup**
- **File**: `crates/opengp-ui/src/ui/widgets/date_picker_popup.rs`
- **Purpose**: Modal calendar for date selection
- **Behavior**: Opens as overlay, allows date selection, returns selected date

#### 2. **TimePickerPopup**
- **File**: `crates/opengp-ui/src/ui/widgets/time_picker_popup.rs`
- **Purpose**: Modal time selector
- **Behavior**: Opens as overlay, allows time selection, returns selected time

### **Dropdown Widget (1 implementation)**

#### **DropdownWidget**
- **File**: `crates/opengp-ui/src/ui/widgets/dropdown.rs:254-288`
- **Purpose**: Searchable dropdown with open/close state
- **Actions**:
  ```rust
  pub enum DropdownAction {
      Opened,
      Closed,
      Selected(Option<usize>),
      FocusChanged,
  }
  ```
- **Behavior**:
  - Click to open/close
  - Click on option to select
  - Click outside to close

### **What DOES NOT Exist**

| Feature | Status | Reason |
|---------|--------|--------|
| Right-click context menus | ❌ None | No `MouseButton::Right` handling |
| Hover tooltips | ❌ None | No `MouseEventKind::Moved` handling |
| Floating action menus | ❌ None | No floating UI infrastructure |
| Keyboard shortcuts overlay | ❌ None | Only Help modal (F1) exists |
| Popup menus | ❌ None | Only modals and dropdowns |
| Contextual help | ❌ None | No hover-based help |

### **Conclusion**

The TUI uses a **modal-based UI pattern**:
- Detail views are **full-screen modals** (not floating windows)
- Actions are **button-based** (not context menus)
- Navigation is **keyboard-first** (arrow keys, Enter, Esc)
- No right-click or hover interactions

---

## SUMMARY TABLE: Current Mouse Handling State

| Aspect | Status | Details |
|--------|--------|---------|
| **Trait Definition** | ✅ Exists | `HandleMouse` with `handle_mouse(mouse, area) → Option<Action>` |
| **Implementations** | ✅ 14 total | 10 components + 4 widgets |
| **Global Dispatcher** | ✅ Exists | Priority: TabBar → Form → List → Appointment |
| **Scroll Increments** | ⚠️ Hardcoded | 3 rows (lists), 1 hour (schedule), 1 month (calendar) |
| **Hover State** | ❌ None | No `MouseEventKind::Moved` handling |
| **Right-Click** | ❌ None | Only `MouseButton::Left` handled |
| **Bounds Checking** | ✅ Exists | `Rect::contains(Position)` pattern |
| **Hit Testing** | ✅ Exists | Field-by-field iteration in forms |
| **AppCommand** | ✅ Exists | Async-only, no "open detail" command |
| **Enter Key Flow** | ✅ Exists | Keybind → Action → Event handler → State mutation |
| **Context Menus** | ❌ None | Only detail modals + date/time pickers |
| **Detail View Pattern** | ✅ Exists | 6 detail modals, state-based (not command-based) |

---

## RECOMMENDATIONS FOR OVERHAUL

### **High Priority**

1. **Extract Scroll Increments to Constants**
   - Create `crates/opengp-ui/src/ui/layout/mouse_config.rs`
   - Define: `const SCROLL_ROWS: usize = 3;`, `const SCROLL_HOURS: u8 = 1;`
   - Replace all hardcoded values

2. **Add Hover State Support**
   - Add `hovered_item: Option<usize>` to list components
   - Handle `MouseEventKind::Moved` in dispatcher
   - Update rendering to highlight hovered items

3. **Implement Right-Click Context Menus**
   - Add `MouseButton::Right` handling to dispatcher
   - Create `ContextMenuState` struct
   - Define context menu actions per component

### **Medium Priority**

4. **Refactor Global Dispatcher**
   - Extract tab-specific dispatchers: `handle_patient_tab_mouse()`, `handle_appointment_tab_mouse()`
   - Reduce global dispatcher from 135 lines to ~50 lines
   - Improve maintainability

5. **Persistent Bounds Tracking**
   - Add `last_render_area: Rect` to components
   - Avoid recalculating bounds on each mouse event
   - Improve performance

6. **Unify Detail View Opening**
   - Add `AppCommand::OpenDetailModal(...)` variants
   - Replace scattered state mutations with command dispatch
   - Centralize modal lifecycle

### **Low Priority**

7. **Add Hover Tooltips**
   - Extend hover state to track tooltip content
   - Render tooltips on mouse move
   - Useful for complex UI elements

8. **Keyboard Shortcuts Overlay**
   - Extend Help modal (F1) to show context-specific shortcuts
   - Update dynamically based on current context

---

## CONCLUSION

OpenGP has a **solid foundation** for mouse handling:
- ✅ Clean trait-based architecture
- ✅ Consistent hit testing pattern
- ✅ Priority-based event dispatch
- ✅ Modal-based UI pattern

But it's **minimal and incomplete**:
- ❌ No hover state
- ❌ No right-click support
- ❌ Hardcoded scroll increments
- ❌ No context menus

The overhaul should **extend, not replace** the current implementation. All code is ready for modification with no breaking changes needed to start.

