# Ratatui 0.29-0.30 TUI Redesign Research Report

## Executive Summary

This report covers five critical areas for your 4-row layout redesign with F-key tab switching, dynamic patient tabs, and mouse event dispatch. All patterns are based on concrete code from ratatui 0.30.0 and production applications.

---

## 1. RATATUI LAYOUT CONSTRAINTS: Best Practices for 4-Row Layout

### Constraint Priority Hierarchy (Highest to Lowest)
```
1. Constraint::Min(n)        - Minimum size (highest priority)
2. Constraint::Max(n)        - Maximum size
3. Constraint::Length(n)     - Fixed size (exact cells)
4. Constraint::Percentage(n) - Proportional (% of total)
5. Constraint::Ratio(n, d)   - Fractional (n/d of total)
6. Constraint::Fill(n)       - Proportional fill (lowest priority)
```

### Recommended 4-Row Layout Pattern

**For your layout: top nav (fixed) → clinical sub-nav (fixed) → content (flexible) → status bar (fixed)**

```rust
use ratatui::layout::{Constraint, Layout, Rect};

fn create_4row_layout(area: Rect) -> [Rect; 4] {
    let layout = Layout::vertical([
        Constraint::Length(1),    // Top nav row (fixed 1 cell)
        Constraint::Length(1),    // Clinical sub-nav row (fixed 1 cell)
        Constraint::Fill(1),      // Content area (flexible, takes remaining)
        Constraint::Length(1),    // Status bar (fixed 1 cell)
    ]);
    
    let [top_nav, sub_nav, content, status] = area.areas(layout);
    [top_nav, sub_nav, content, status]
}
```

### Key Gotchas: Length vs Min/Fill

| Scenario | Use | Why |
|----------|-----|-----|
| Fixed-height header | `Constraint::Length(3)` | Exact size, highest priority after Min/Max |
| Flexible content area | `Constraint::Fill(1)` | Takes all remaining space after fixed constraints |
| Minimum space guarantee | `Constraint::Min(10)` | Ensures at least 10 cells, can grow larger |
| Avoid shrinking below N | `Constraint::Min(n)` + `Constraint::Fill(1)` | Prevents content collapse |

**Critical**: `Length` is higher priority than `Fill`. If you have:
```rust
[Constraint::Length(10), Constraint::Fill(1)]  // ✓ Correct
[Constraint::Fill(1), Constraint::Length(10)]  // ✓ Also correct (order doesn't matter for priority)
```

### Nested Layout Pattern (for sub-navigation row)

If your clinical sub-nav row has multiple tabs/buttons:

```rust
// Main 4-row split
let [top_nav, sub_nav, content, status] = Layout::vertical([
    Constraint::Length(1),
    Constraint::Length(1),
    Constraint::Fill(1),
    Constraint::Length(1),
]).areas(area);

// Split sub_nav horizontally for multiple nav items
let nav_items = Layout::horizontal([
    Constraint::Length(15),  // "Vitals" button
    Constraint::Length(15),  // "Allergies" button
    Constraint::Length(15),  // "History" button
    Constraint::Fill(1),     // Spacer
]).areas(sub_nav);
```

**Source**: [ratatui layout module](https://docs.rs/ratatui/latest/ratatui/layout/index.html)

---

## 2. RATATUI MOUSE EVENT HANDLING (0.30+)

### MouseEvent Structure

```rust
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};

// MouseEvent fields:
pub struct MouseEvent {
    pub column: u16,           // X coordinate (0-based from left)
    pub row: u16,              // Y coordinate (0-based from top)
    pub kind: MouseEventKind,  // Down, Up, Drag, Moved, Scroll*
    pub modifiers: KeyModifiers, // Ctrl, Alt, Shift
}

// MouseEventKind variants:
pub enum MouseEventKind {
    Down(MouseButton),      // Button pressed
    Up(MouseButton),        // Button released
    Drag(MouseButton),      // Button held + moved
    Moved,                  // Mouse moved (no button)
    ScrollUp,               // Wheel up
    ScrollDown,             // Wheel down
    ScrollLeft,             // Wheel left (rare)
    ScrollRight,            // Wheel right (rare)
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
}
```

### Pattern 1: Hit-Testing Against Rect Areas

**From ratatui mouse-drawing example** ([source](https://github.com/ratatui/ratatui/blob/main/examples/apps/mouse-drawing/src/main.rs)):

```rust
use ratatui::layout::{Position, Rect};
use crossterm::event::{MouseEvent, MouseEventKind};

fn handle_mouse_event(&mut self, event: MouseEvent, area: Rect) {
    let position = Position::new(event.column, event.row);
    
    // Check if click is within the area
    if area.contains(position) {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Handle left click
                self.on_left_click(position);
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                // Handle drag
                self.on_drag(position);
            }
            MouseEventKind::Up(MouseButton::Left) => {
                // Handle release
                self.on_release(position);
            }
            _ => {}
        }
    }
}
```

### Pattern 2: Dispatch to Multiple Rows (Your Use Case)

For your 4-row layout with different handlers per row:

```rust
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use ratatui::layout::Rect;

struct AppState {
    top_nav_area: Rect,
    sub_nav_area: Rect,
    content_area: Rect,
    status_area: Rect,
}

impl AppState {
    fn handle_mouse(&mut self, event: MouseEvent) {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Dispatch based on which row was clicked
                if self.top_nav_area.contains(Position::new(event.column, event.row)) {
                    self.handle_top_nav_click(event.column, event.row);
                } else if self.sub_nav_area.contains(Position::new(event.column, event.row)) {
                    self.handle_sub_nav_click(event.column, event.row);
                } else if self.content_area.contains(Position::new(event.column, event.row)) {
                    self.handle_content_click(event.column, event.row);
                } else if self.status_area.contains(Position::new(event.column, event.row)) {
                    self.handle_status_click(event.column, event.row);
                }
            }
            MouseEventKind::ScrollUp => {
                // Scroll in content area
                if self.content_area.contains(Position::new(event.column, event.row)) {
                    self.scroll_content_up();
                }
            }
            _ => {}
        }
    }
}
```

### Pattern 3: Click Region Registry (Advanced)

For complex layouts with many clickable regions, use a registry pattern:

```rust
use std::collections::HashMap;
use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ClickRegion {
    TopNavHome,
    TopNavSettings,
    SubNavVitals,
    SubNavAllergies,
    SubNavHistory,
    PatientTab(usize),  // For dynamic tabs
    ContentArea,
}

struct ClickRegistry {
    regions: HashMap<ClickRegion, Rect>,
}

impl ClickRegistry {
    fn new() -> Self {
        Self {
            regions: HashMap::new(),
        }
    }
    
    fn register(&mut self, region: ClickRegion, area: Rect) {
        self.regions.insert(region, area);
    }
    
    fn hit_test(&self, column: u16, row: u16) -> Option<ClickRegion> {
        for (&region, &area) in &self.regions {
            if area.contains(Position::new(column, row)) {
                return Some(region);
            }
        }
        None
    }
}

// In render loop:
fn render(&mut self, frame: &mut Frame) {
    let [top_nav, sub_nav, content, status] = self.layout_areas(frame.area());
    
    // Register clickable regions
    self.click_registry.clear();
    self.click_registry.register(ClickRegion::TopNavHome, top_nav_home_area);
    self.click_registry.register(ClickRegion::SubNavVitals, sub_nav_vitals_area);
    // ... register all regions
    
    // Render widgets
    frame.render_widget(top_nav_widget, top_nav);
    // ...
}

// In event handler:
fn handle_mouse(&mut self, event: MouseEvent) {
    if let MouseEventKind::Down(MouseButton::Left) = event.kind {
        if let Some(region) = self.click_registry.hit_test(event.column, event.row) {
            match region {
                ClickRegion::TopNavHome => self.navigate_home(),
                ClickRegion::SubNavVitals => self.show_vitals(),
                ClickRegion::PatientTab(idx) => self.switch_to_patient_tab(idx),
                _ => {}
            }
        }
    }
}
```

### Enable Mouse Capture

```rust
use crossterm::execute;
use crossterm::event::{EnableMouseCapture, DisableMouseCapture};
use std::io::stdout;

fn main() -> Result<()> {
    // Enable mouse at startup
    execute!(stdout(), EnableMouseCapture)?;
    
    // Your app loop
    
    // Disable mouse on exit
    execute!(stdout(), DisableMouseCapture)?;
    Ok(())
}
```

**Source**: [ratatui mouse-drawing example](https://github.com/ratatui/ratatui/blob/main/examples/apps/mouse-drawing/src/main.rs)

---

## 3. F-KEY HANDLING IN CROSSTERM/RATATUI

### F-Key Representation

In crossterm 0.29+, F-keys are represented as:

```rust
use crossterm::event::{KeyCode, KeyEvent};

// F1-F12 are available as:
KeyCode::F(1)   // F1
KeyCode::F(2)   // F2
KeyCode::F(3)   // F3
// ... up to F(24) for F24

// Some systems also support:
KeyCode::F1, KeyCode::F2, ... KeyCode::F12  // Direct variants (older API)
```

### Pattern: F-Key Tab Switching (F2-F9 for 6 Tabs)

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatientTab {
    Tab1,  // F4
    Tab2,  // F5
    Tab3,  // F6
    Tab4,  // F7
    Tab5,  // F8
    Tab6,  // F9
}

struct AppState {
    active_patient_tabs: Vec<PatientTab>,  // Currently open tabs
    current_tab_index: usize,               // Which tab is active
}

impl AppState {
    fn handle_key(&mut self, key: KeyEvent) {
        // F-key handling for tab switching
        match key.code {
            KeyCode::F(4) => self.switch_to_tab(0),  // F4 = Tab 1
            KeyCode::F(5) => self.switch_to_tab(1),  // F5 = Tab 2
            KeyCode::F(6) => self.switch_to_tab(2),  // F6 = Tab 3
            KeyCode::F(7) => self.switch_to_tab(3),  // F7 = Tab 4
            KeyCode::F(8) => self.switch_to_tab(4),  // F8 = Tab 5
            KeyCode::F(9) => self.switch_to_tab(5),  // F9 = Tab 6
            
            // Alternative: use Shift+F for closing tabs
            KeyCode::F(4) if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.close_tab(0);
            }
            
            _ => {}
        }
    }
    
    fn switch_to_tab(&mut self, index: usize) {
        if index < self.active_patient_tabs.len() {
            self.current_tab_index = index;
        }
    }
}
```

### Known F-Key Issues & Workarounds

| Issue | Symptom | Workaround |
|-------|---------|-----------|
| F1-F4 captured by terminal | F1-F4 don't reach app | Use F5-F12 instead, or remap in terminal |
| F10 opens menu (Windows) | F10 doesn't reach app | Use F9 or F11 instead |
| F11 fullscreen (browsers) | F11 doesn't reach app | Use F9 or F12 instead |
| Shift+F-key not detected | Shift+F5 seen as F5 | Some terminals don't support; test first |

**Recommendation**: Use F4-F9 for your 6 tabs (avoids F1-F3 system keys, F10-F12 terminal keys).

**Source**: [yazi keymap](https://github.com/sxyazi/yazi/blob/main/yazi-config/src/keymap/key.rs) - production TUI using F-keys

---

## 4. TAB/SHIFT+TAB CYCLING PATTERNS

### Pattern 1: Simple Enum-Based Cycling (Recommended)

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubNavFocus {
    Vitals,
    Allergies,
    History,
    MedicalHistory,
}

impl SubNavFocus {
    fn next(self) -> Self {
        match self {
            Self::Vitals => Self::Allergies,
            Self::Allergies => Self::History,
            Self::History => Self::MedicalHistory,
            Self::MedicalHistory => Self::Vitals,  // Wrap around
        }
    }
    
    fn prev(self) -> Self {
        match self {
            Self::Vitals => Self::MedicalHistory,
            Self::Allergies => Self::Vitals,
            Self::History => Self::Allergies,
            Self::MedicalHistory => Self::History,
        }
    }
}

struct SubNavState {
    focus: SubNavFocus,
}

impl SubNavState {
    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.focus = self.focus.next();
            }
            KeyCode::BackTab => {
                self.focus = self.focus.prev();
            }
            _ => {}
        }
    }
}
```

### Pattern 2: Index-Based Cycling (For Dynamic Tabs)

```rust
struct SubNavState {
    items: Vec<String>,  // Dynamic list of nav items
    focused_index: usize,
}

impl SubNavState {
    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.focused_index = (self.focused_index + 1) % self.items.len();
            }
            KeyCode::BackTab => {
                if self.focused_index == 0 {
                    self.focused_index = self.items.len() - 1;
                } else {
                    self.focused_index -= 1;
                }
            }
            _ => {}
        }
    }
}
```

### Pattern 3: Ratatui Input Form Example

**From ratatui input-form example** ([source](https://github.com/ratatui/ratatui/blob/main/examples/apps/input-form/src/main.rs)):

```rust
#[derive(Default, PartialEq, Eq)]
enum Focus {
    #[default]
    FirstName,
    LastName,
    Age,
}

impl Focus {
    const fn next(&self) -> Self {
        match self {
            Self::FirstName => Self::LastName,
            Self::LastName => Self::Age,
            Self::Age => Self::FirstName,  // Wrap
        }
    }
}

struct InputForm {
    focus: Focus,
    // ... other fields
}

impl InputForm {
    fn on_key_press(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Tab => self.focus = self.focus.next(),
            _ => match self.focus {
                Focus::FirstName => self.first_name.on_key_press(event),
                Focus::LastName => self.last_name.on_key_press(event),
                Focus::Age => self.age.on_key_press(event),
            },
        }
    }
}
```

### Detecting Tab vs BackTab

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn is_tab(key: &KeyEvent) -> bool {
    key.code == KeyCode::Tab && !key.modifiers.contains(KeyModifiers::SHIFT)
}

fn is_backtab(key: &KeyEvent) -> bool {
    key.code == KeyCode::BackTab || 
    (key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT))
}

// Usage:
if is_tab(&key) {
    self.focus = self.focus.next();
} else if is_backtab(&key) {
    self.focus = self.focus.prev();
}
```

**Source**: [ratatui input-form example](https://github.com/ratatui/ratatui/blob/main/examples/apps/input-form/src/main.rs)

---

## 5. DYNAMIC TAB BARS (Runtime Open/Close)

### Pattern 1: Vec-Based Dynamic Tabs

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PatientId(u32);

struct PatientTab {
    patient_id: PatientId,
    patient_name: String,
    is_active: bool,
}

struct AppState {
    patient_tabs: Vec<PatientTab>,
    active_tab_index: usize,
}

impl AppState {
    fn open_patient_tab(&mut self, patient_id: PatientId, name: String) {
        // Check if already open
        if !self.patient_tabs.iter().any(|t| t.patient_id == patient_id) {
            self.patient_tabs.push(PatientTab {
                patient_id,
                patient_name: name,
                is_active: true,
            });
            self.active_tab_index = self.patient_tabs.len() - 1;
        }
    }
    
    fn close_patient_tab(&mut self, index: usize) {
        if index < self.patient_tabs.len() {
            self.patient_tabs.remove(index);
            // Adjust active index
            if self.active_tab_index >= self.patient_tabs.len() && !self.patient_tabs.is_empty() {
                self.active_tab_index = self.patient_tabs.len() - 1;
            }
        }
    }
    
    fn switch_to_tab(&mut self, index: usize) {
        if index < self.patient_tabs.len() {
            self.active_tab_index = index;
        }
    }
}
```

### Pattern 2: Render Dynamic Tab Bar

```rust
use ratatui::widgets::{Tabs, Block, Borders};
use ratatui::style::{Style, Color};
use ratatui::text::Line;

fn render_patient_tabs(&self, frame: &mut Frame, area: Rect) {
    // Build tab titles from open tabs
    let tab_titles: Vec<Line> = self.patient_tabs
        .iter()
        .map(|tab| {
            let title = format!(" {} ", tab.patient_name);
            if tab.patient_id == self.patient_tabs[self.active_tab_index].patient_id {
                Line::from(title).style(Style::default().fg(Color::Yellow).bold())
            } else {
                Line::from(title).style(Style::default().fg(Color::Gray))
            }
        })
        .collect();
    
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(self.active_tab_index)
        .style(Style::default().fg(Color::White));
    
    frame.render_widget(tabs, area);
}
```

### Pattern 3: F-Key Dispatch to Dynamic Tabs

```rust
impl AppState {
    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            // F4-F9 map to tab indices 0-5
            KeyCode::F(4) => self.switch_to_tab(0),
            KeyCode::F(5) => self.switch_to_tab(1),
            KeyCode::F(6) => self.switch_to_tab(2),
            KeyCode::F(7) => self.switch_to_tab(3),
            KeyCode::F(8) => self.switch_to_tab(4),
            KeyCode::F(9) => self.switch_to_tab(5),
            
            // Shift+F to close tab
            KeyCode::F(4) if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.close_patient_tab(0);
            }
            
            // Ctrl+T to open new tab (example)
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_patient_tab(PatientId(999), "New Patient".to_string());
            }
            
            _ => {}
        }
    }
}
```

### Pattern 4: Tab Limit (Max 6 Tabs)

```rust
const MAX_PATIENT_TABS: usize = 6;

impl AppState {
    fn open_patient_tab(&mut self, patient_id: PatientId, name: String) {
        if self.patient_tabs.len() >= MAX_PATIENT_TABS {
            // Show error or close oldest tab
            self.patient_tabs.remove(0);
        }
        
        if !self.patient_tabs.iter().any(|t| t.patient_id == patient_id) {
            self.patient_tabs.push(PatientTab {
                patient_id,
                patient_name: name,
                is_active: true,
            });
            self.active_tab_index = self.patient_tabs.len() - 1;
        }
    }
}
```

**Source**: [ratatui demo2 tabs example](https://github.com/ratatui/ratatui/blob/main/examples/apps/demo2/src/tabs.rs)

---

## COMPLETE EXAMPLE: 4-Row Layout with F-Key Tabs and Mouse Dispatch

```rust
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubNavFocus {
    Vitals,
    Allergies,
    History,
}

impl SubNavFocus {
    fn next(self) -> Self {
        match self {
            Self::Vitals => Self::Allergies,
            Self::Allergies => Self::History,
            Self::History => Self::Vitals,
        }
    }
    
    fn prev(self) -> Self {
        match self {
            Self::Vitals => Self::History,
            Self::Allergies => Self::Vitals,
            Self::History => Self::Allergies,
        }
    }
}

struct PatientTab {
    id: u32,
    name: String,
}

struct AppState {
    should_exit: bool,
    patient_tabs: Vec<PatientTab>,
    active_tab_index: usize,
    sub_nav_focus: SubNavFocus,
    
    // Cached layout areas for mouse dispatch
    top_nav_area: Rect,
    sub_nav_area: Rect,
    content_area: Rect,
    status_area: Rect,
}

impl AppState {
    fn new() -> Self {
        Self {
            should_exit: false,
            patient_tabs: vec![
                PatientTab { id: 1, name: "John Doe".to_string() },
            ],
            active_tab_index: 0,
            sub_nav_focus: SubNavFocus::Vitals,
            top_nav_area: Rect::default(),
            sub_nav_area: Rect::default(),
            content_area: Rect::default(),
            status_area: Rect::default(),
        }
    }
    
    fn handle_event(&mut self) -> std::io::Result<()> {
        match event::read()? {
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            _ => {}
        }
        Ok(())
    }
    
    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            
            // F-key tab switching
            KeyCode::F(4) => self.switch_to_tab(0),
            KeyCode::F(5) => self.switch_to_tab(1),
            KeyCode::F(6) => self.switch_to_tab(2),
            KeyCode::F(7) => self.switch_to_tab(3),
            KeyCode::F(8) => self.switch_to_tab(4),
            KeyCode::F(9) => self.switch_to_tab(5),
            
            // Sub-nav cycling
            KeyCode::Tab => self.sub_nav_focus = self.sub_nav_focus.next(),
            KeyCode::BackTab => self.sub_nav_focus = self.sub_nav_focus.prev(),
            
            _ => {}
        }
    }
    
    fn handle_mouse(&mut self, event: MouseEvent) {
        let pos = Position::new(event.column, event.row);
        
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.top_nav_area.contains(pos) {
                    // Handle top nav click
                } else if self.sub_nav_area.contains(pos) {
                    // Handle sub-nav click
                    self.handle_sub_nav_click(event.column);
                } else if self.content_area.contains(pos) {
                    // Handle content click
                } else if self.status_area.contains(pos) {
                    // Handle status bar click
                }
            }
            MouseEventKind::ScrollUp => {
                if self.content_area.contains(pos) {
                    // Scroll content up
                }
            }
            _ => {}
        }
    }
    
    fn handle_sub_nav_click(&mut self, column: u16) {
        // Determine which sub-nav item was clicked based on column
        let vitals_start = self.sub_nav_area.x;
        let vitals_end = vitals_start + 10;
        let allergies_start = vitals_end;
        let allergies_end = allergies_start + 10;
        
        if column >= vitals_start && column < vitals_end {
            self.sub_nav_focus = SubNavFocus::Vitals;
        } else if column >= allergies_start && column < allergies_end {
            self.sub_nav_focus = SubNavFocus::Allergies;
        } else {
            self.sub_nav_focus = SubNavFocus::History;
        }
    }
    
    fn switch_to_tab(&mut self, index: usize) {
        if index < self.patient_tabs.len() {
            self.active_tab_index = index;
        }
    }
    
    fn render(&mut self, frame: &mut Frame) {
        let [top_nav, sub_nav, content, status] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]).areas(frame.area());
        
        // Cache areas for mouse dispatch
        self.top_nav_area = top_nav;
        self.sub_nav_area = sub_nav;
        self.content_area = content;
        self.status_area = status;
        
        // Render each row
        frame.render_widget(
            Paragraph::new("Top Nav").block(Block::default().borders(Borders::BOTTOM)),
            top_nav
        );
        
        frame.render_widget(
            Paragraph::new(format!("Sub-Nav: {:?}", self.sub_nav_focus))
                .block(Block::default().borders(Borders::BOTTOM)),
            sub_nav
        );
        
        frame.render_widget(
            Paragraph::new(format!("Patient: {}", self.patient_tabs[self.active_tab_index].name))
                .block(Block::default().borders(Borders::ALL)),
            content
        );
        
        frame.render_widget(
            Paragraph::new("Status Bar"),
            status
        );
    }
}

fn main() -> std::io::Result<()> {
    let mut app = AppState::new();
    let mut terminal = DefaultTerminal::new()?;
    
    while !app.should_exit {
        terminal.draw(|frame| app.render(frame))?;
        app.handle_event()?;
    }
    
    Ok(())
}
```

---

## SUMMARY TABLE

| Requirement | Pattern | Key Code |
|-------------|---------|----------|
| 4-row layout | `Layout::vertical([Length(1), Length(1), Fill(1), Length(1)])` | Constraint priority |
| Mouse dispatch | `Rect::contains(Position)` + match on area | Hit-testing |
| F-key tabs | `KeyCode::F(4..=9)` | F4-F9 for 6 tabs |
| Tab cycling | Enum with `next()`/`prev()` | Wrap-around logic |
| Dynamic tabs | `Vec<PatientTab>` + index | Open/close at runtime |

---

## References

- [Ratatui Layout Module](https://docs.rs/ratatui/latest/ratatui/layout/index.html)
- [Ratatui Mouse Drawing Example](https://github.com/ratatui/ratatui/blob/main/examples/apps/mouse-drawing/src/main.rs)
- [Ratatui Input Form Example](https://github.com/ratatui/ratatui/blob/main/examples/apps/input-form/src/main.rs)
- [Ratatui Demo2 Tabs](https://github.com/ratatui/ratatui/blob/main/examples/apps/demo2/src/tabs.rs)
- [Crossterm Event Documentation](https://docs.rs/crossterm/latest/crossterm/event/)
- [Yazi TUI F-Key Handling](https://github.com/sxyazi/yazi/blob/main/yazi-config/src/keymap/key.rs)

