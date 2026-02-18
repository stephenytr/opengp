# OpenGP TUI Implementation Plan

## Overview

This plan outlines the implementation of a production-ready Terminal User Interface (TUI) for OpenGP using:
- **ratatui 0.29** - Core UI framework
- **ratatui-interact 0.4** - Mouse support, focus management, and interactive widgets
- **crossterm** - Event handling

## Architecture Summary

```
src/ui/
├── main.rs              # Entry point, terminal setup, main loop
├── app.rs               # App state, routing, global keybinds
├── theme.rs             # Global theme configuration
├── config.rs            # UI configuration
├── events.rs            # Event handling setup
│
├── components/          # UI Components
│   ├── mod.rs
│   ├── tabs.rs         # Tab navigation (Patient/Appointments/Clinical/Billing)
│   ├── status_bar.rs   # Bottom status bar
│   └── help.rs         # F1 help overlay
│
├── patient/             # Patient Tab
│   ├── mod.rs
│   ├── list.rs         # Patient list with fuzzy search
│   ├── form.rs         # New patient form
│   └── state.rs        # Patient component state
│
├── appointment/         # Appointments Tab
│   ├── mod.rs
│   ├── calendar.rs     # 30-day date picker calendar
│   ├── schedule.rs     # Practitioner columns with time rows
│   ├── block.rs        # Appointment block rendering
│   └── state.rs        # Appointment component state
│
├── clinical/            # Clinical Tab (stub)
│   └── mod.rs
│
├── billing/            # Billing Tab (stub)
│   └── mod.rs
│
├── widgets/            # Reusable widgets
│   ├── mod.rs
│   ├── form_field.rs  # Reusable form field with validation
│   ├── search_input.rs # Fuzzy search input
│   └── loading.rs     # Async loading indicator
│
└── services/           # UI services (bridge to domain)
    ├── mod.rs
    ├── patient_service.rs
    ├── appointment_service.rs
    └── practitioner_service.rs
```

## Key Design Decisions

### 1. Technology Stack
- **ratatui-interact 0.4**: Provides mouse support, focus management, and ready-to-use interactive widgets
- **ratatui 0.29**: Core rendering framework
- **crossterm**: Event handling

### 2. Mouse Support
- Enabled via `crossterm::event::EnableMouseCapture`
- ratatui-interact provides click handling for all interactive widgets
- Custom calendar/schedule components need manual mouse hit-testing

### 3. Keybind Strategy
- **Context-specific**: Tab to navigate between focusable elements, Enter to activate
- **Global shortcuts**: F1=Help, Ctrl+N=New, Ctrl+F=Search, Ctrl+Q=Quit
- **Escape**: Always goes back/cancels (as specified)
- **Arrow keys**: Navigate lists, calendars, schedules
- **ratatui-interact**: Handles Tab/Shift+Tab for focus management

### 4. Data Strategy
- **Async loading**: All database operations via Tokio tasks
- **Always refetch**: On tab switch, on form load
- **Pagination**: Calculated based on terminal height
- **Loading states**: Show spinner while loading

### 5. Theme System
- Global configurable theme with color presets
- Support for light/dark variants
- Status-based coloring (appointment status colors)

### 6. Form Validation
- Inline errors displayed below fields
- Form cannot be submitted until valid

---

## Implementation Details

### 1. Theme Configuration (`theme.rs`)

```rust
// Global theme with configurable colors
pub struct Theme {
    pub colors: ColorPalette,
    pub fonts: FontConfig,
    pub spacing: SpacingConfig,
}

pub struct ColorPalette {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub foreground: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub // Appointment status colors
    appointment_scheduled: Color,
    appointment_confirmed: Color,
    appointment_arrived: Color,
    appointment_in_progress: Color,
    appointment_completed: Color,
    appointment_cancelled: Color,
}

// Predefined themes
impl Theme {
    pub fn dark() -> Self { ... }
    pub fn light() -> Self { ... }
}
```

### 2. Centralized Keybinds (`app.rs`)

```rust
// Keybind definitions - single source of truth
pub struct Keybind {
    pub key: KeyEvent,
    pub action: Action,
    pub context: KeyContext,  // Global or specific component
    pub description: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyContext {
    Global,
    PatientList,
    PatientForm,
    Calendar,
    Schedule,
}

pub enum Action {
    Quit,
    OpenHelp,
    NewPatient,
    Search,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    Enter,
    Escape,
    TabNext,
    TabPrev,
    // ... more actions
}

// Keybind registry
pub struct KeybindRegistry {
    binds: Vec<Keybind>,
}

impl KeybindRegistry {
    pub fn global(&self, key: KeyEvent) -> Option<Action>;
    pub fn for_context(&self, key: KeyEvent, context: KeyContext) -> Option<Action>;
}
```

### 3. Tab Navigation (`components/tabs.rs`)

```rust
pub struct TabBar {
    tabs: Vec<Tab>,
    selected: usize,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Tab {
    Patient,
    Appointment,
    Clinical,
    Billing,
}

impl TabBar {
    pub fn new() -> Self;
    pub fn next(&mut self);
    pub fn prev(&mut self);
    pub fn select(&mut self, tab: Tab);
}
```

### 4. Patient List (`patient/list.rs`)

```rust
// Uses ratatui-interact Input for search
// Uses ratatui Table with ListState for patient list
// Columns: Name, DOB, Medicare #, Phone, Last Visit

pub struct PatientList {
    patients: Vec<Patient>,
    filtered: Vec<Patient>,
    search_query: String,
    list_state: ListState,
    search_input: InputState,  // From ratatui-interact
    loading: bool,
}

impl PatientList {
    pub fn new() -> Self;
    pub async fn load_patients(&mut self, repo: &Arc<dyn PatientRepository>);
    pub fn filter(&mut self);
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Msg>;
    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<Msg>;
}
```

### 5. Patient Form (`patient/form.rs`)

```rust
// Single comprehensive form using ratatui-interact components
// Fields validated inline

pub struct PatientForm {
    // Form fields with validation states
    first_name: InputState,
    last_name: InputState,
    date_of_birth: InputState,
    gender: SelectState<Gender>,
    medicare_number: InputState,
    // ... all patient fields
    
    // Validation errors (inline)
    errors: HashMap<Field, String>,
    
    // Form state
    mode: FormMode,  // Create or Edit
    saving: bool,
}

impl PatientForm {
    pub fn new() -> Self;
    pub fn validate(&self) -> bool;
    pub fn errors_for(&self, field: Field) -> Option<&str>;
}
```

### 6. Calendar Component (`appointment/calendar.rs`)

```rust
// 30-day date picker style calendar
// Custom widget (ratatui-interact doesn't have calendar)

pub struct Calendar {
    current_month: NaiveDate,
    selected_date: Option<NaiveDate>,
    visible_dates: Vec<CalendarDay>,
}

pub struct CalendarDay {
    date: NaiveDate,
    is_current_month: bool,
    is_today: bool,
    is_selected: bool,
    has_appointments: bool,
}

impl Calendar {
    pub fn new() -> Self;
    pub fn next_month(&mut self);
    pub fn prev_month(&mut self);
    pub fn select_date(&mut self, date: NaiveDate);
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Msg>;
    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<Msg>;
}
```

### 7. Schedule Component (`appointment/schedule.rs`)

```rust
// Practitioner columns with time rows
// Appointment blocks with height proportional to duration

pub struct Schedule {
    practitioners: Vec<Practitioner>,
    selected_date: NaiveDate,
    appointments: HashMap<Uuid, Vec<Appointment>>,  // By practitioner
    viewport_start_hour: u8,  // Typically 8am
    viewport_end_hour: u8,    // Typically 6pm
    selected_practitioner: Option<Uuid>,
    selected_appointment: Option<Uuid>,
}

pub struct TimeSlot {
    hour: u8,
    minute: u8,
}

impl Schedule {
    pub fn new() -> Self;
    pub async fn load_day(&mut self, date: NaiveDate);
    pub fn navigate_time(&mut self, direction: Direction);
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Msg>;
    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<Msg>;
}
```

### 8. Async Service Layer (`services/`)

```rust
// Bridge between UI and domain layer
// Handles async database operations

pub struct PatientUiService {
    repository: Arc<dyn PatientRepository>,
}

impl PatientUiService {
    pub async fn list_patients(&self, page: usize, page_size: usize) 
        -> Result<Vec<Patient>>;
    
    pub async fn search_patients(&self, query: &str) 
        -> Result<Vec<Patient>>;
    
    pub async fn create_patient(&self, data: NewPatientData) 
        -> Result<Patient>;
}

pub struct AppointmentUiService {
    repository: Arc<dyn AppointmentRepository>,
    practitioner_repository: Arc<dyn PractitionerRepository>,
}

impl AppointmentUiService {
    pub async fn get_schedule(&self, date: NaiveDate) 
        -> Result<ScheduleData>;
    
    pub async fn get_calendar_data(&self, month: NaiveDate) 
        -> Result<CalendarData>;
}
```

---

## Keybind Mapping

### Global Keybinds (Work Everywhere)

| Key | Action | Description |
|-----|--------|-------------|
| `F1` | OpenHelp | Show help overlay |
| `Ctrl+N` | New | Create new (patient, appointment, etc.) |
| `Ctrl+F` | Search | Focus search input |
| `Ctrl+Q` | Quit | Exit application |
| `Tab` | TabNext | Navigate to next focusable element |
| `Shift+Tab` | TabPrev | Navigate to previous focusable element |
| `Escape` | Escape | Go back / Cancel (always) |

### Patient List Context

| Key | Action | Description |
|-----|--------|-------------|
| `j` / `Down` | NavigateDown | Move selection down |
| `k` / `Up` | NavigateUp | Move selection up |
| `Enter` | OpenPatient | Open selected patient |
| `n` | NewPatient | Open new patient form |
| `/` | Search | Focus search input |
| `r` | Refresh | Reload patient list |

### Patient Form Context

| Key | Action | Description |
|-----|--------|-------------|
| `Tab` | NextField | Move to next field |
| `Shift+Tab` | PrevField | Move to previous field |
| `Enter` | Submit | Submit form |
| `Escape` | Cancel | Cancel and go back |
| `Ctrl+S` | Save | Save patient |

### Calendar Context

| Key | Action | Description |
|-----|--------|-------------|
| `h` / `Left` | PrevDay | Go to previous day |
| `l` / `Right` | NextDay | Go to next day |
| `j` / `Down` | NextWeek | Go to next week |
| `k` / `Up` | PrevWeek | Go to previous week |
| `Enter` | SelectDate | Select highlighted date |
| `t` | Today | Go to today |

### Schedule Context

| Key | Action | Description |
|-----|--------|-------------|
| `h` / `Left` | PrevPractitioner | Move to previous practitioner column |
| `l` / `Right` | NextPractitioner | Move to next practitioner column |
| `j` / `Down` | NextTimeSlot | Move to next time slot |
| `k` / `Up` | PrevTimeSlot | Move to previous time slot |
| `Enter` | SelectAppointment | Open selected appointment |
| `n` | NewAppointment | Create new appointment at time |

---

## Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                      Main Loop                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │ Poll Events │ -> │  Update     │ -> │   Render   │    │
│  └─────────────┘    └─────────────┘    └─────────────┘    │
│        │                  │                  │             │
│        v                  v                  v             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              App State (Single Source of Truth)     │   │
│  │  - Current tab                                       │   │
│  │  - Tab-specific state (PatientList, Calendar, etc.) │   │
│  │  - Focus manager (ratatui-interact)                 │   │
│  │  - Theme                                            │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              v
┌─────────────────────────────────────────────────────────────┐
│                    Async Services                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Patient     │  │ Appointment  │  │ Practitioner │     │
│  │  Service     │  │  Service     │  │  Service     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│           │                 │                 │             │
│           v                 v                 v             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Domain Layer (Read/Write)              │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Startup Flow

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load config
    let config = Config::from_env()?;
    
    // 2. Initialize logging
    init_logging(&config.log_level);
    
    // 3. Create database pool
    let pool = create_pool(&config.database).await?;
    
    // 4. Run migrations on startup
    run_migrations(&pool).await?;
    
    // 5. Initialize repositories
    let patient_repo = SqlxPatientRepository::new(pool.clone());
    let appointment_repo = SqlxAppointmentRepository::new(pool.clone());
    let practitioner_repo = SqlxPractitionerRepository::new(pool.clone());
    
    // 6. Initialize UI services
    let patient_service = Arc::new(PatientUiService::new(patient_repo));
    let appointment_service = Arc::new(AppointmentUiService::new(
        appointment_repo, 
        practitioner_repo
    ));
    
    // 7. Create app state
    let mut app = App::new(patient_service, appointment_service);
    
    // 8. Enable mouse capture
    crossterm::execute!(stdout(), EnableMouseCapture)?;
    
    // 9. Initialize terminal
    let terminal = ratatui::init();
    
    // 10. Main loop
    app.run(terminal).await?;
    
    Ok(())
}
```

---

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Project structure setup
- [ ] Theme system implementation
- [ ] Centralized keybind system
- [ ] Tab navigation
- [ ] Status bar
- [ ] Basic event handling with mouse support

### Phase 2: Patient Tab (Week 1-2)
- [ ] Patient list with fuzzy search
- [ ] Patient form with validation
- [ ] Database integration
- [ ] Async loading states

### Phase 3: Appointments Tab (Week 2-3)
- [ ] 30-day calendar component
- [ ] Schedule view (practitioner columns)
- [ ] Appointment block rendering
- [ ] Mouse interaction for calendar/schedule

### Phase 4: Clinical & Billing Tabs (Week 3)
- [ ] Clinical tab stub (placeholder)
- [ ] Billing tab stub (placeholder)

### Phase 5: Polish (Week 4)
- [ ] Help overlay (F1)
- [ ] Error handling
- [ ] Edge cases
- [ ] Testing

---

## Open Questions for Later

1. **Clinical Tab**: What specific features should be implemented? (SOAP notes, medical history, allergies)
2. **Billing Tab**: What specific features should be implemented? (Invoicing, Medicare claims)
3. **Authentication**: Should there be a login screen?
4. **Window Management**: Support for resizing, minimizing?

---

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
ratatui = "0.29"
ratatui-interact = "0.4"
crossterm = "0.29"

# Already in project:
# tokio = { version = "1", features = ["full"] }
# async-trait = "0.1"
# etc.
```

---

## File Summary

| File | Purpose | Lines Est. |
|------|---------|------------|
| `src/ui/main.rs` | Entry point, terminal init, main loop | 80 |
| `src/ui/app.rs` | App state, keybind handling, routing | 200 |
| `src/ui/theme.rs` | Theme configuration | 100 |
| `src/ui/events.rs` | Event listener setup | 60 |
| `src/ui/components/tabs.rs` | Tab navigation | 80 |
| `src/ui/components/status_bar.rs` | Status bar | 50 |
| `src/ui/components/help.rs` | Help overlay | 60 |
| `src/ui/patient/list.rs` | Patient list | 200 |
| `src/ui/patient/form.rs` | Patient form | 300 |
| `src/ui/patient/state.rs` | Patient state | 80 |
| `src/ui/appointment/calendar.rs` | Calendar | 250 |
| `src/ui/appointment/schedule.rs` | Schedule view | 250 |
| `src/ui/appointment/block.rs` | Appointment blocks | 100 |
| `src/ui/appointment/state.rs` | Appointment state | 80 |
| `src/ui/widgets/form_field.rs` | Reusable form field | 80 |
| `src/ui/widgets/search_input.rs` | Search input | 60 |
| `src/ui/widgets/loading.rs` | Loading indicator | 40 |
| `src/ui/services/mod.rs` | Service exports | 30 |
| `src/ui/services/patient_service.rs` | Patient UI service | 100 |
| `src/ui/services/appointment_service.rs` | Appointment UI service | 100 |

**Total Estimated**: ~2,200 lines
