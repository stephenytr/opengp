# Ratatui 0.30 Quick Reference for OpenGP Redesign

## 1. 4-Row Layout (Copy-Paste Ready)

```rust
use ratatui::layout::{Constraint, Layout, Rect};

let [top_nav, sub_nav, content, status] = Layout::vertical([
    Constraint::Length(1),    // Fixed 1 row
    Constraint::Length(1),    // Fixed 1 row
    Constraint::Fill(1),      // Flexible (takes remaining)
    Constraint::Length(1),    // Fixed 1 row
]).areas(frame.area());
```

**Key Rule**: `Length` > `Fill` in priority. Order doesn't matter.

---

## 2. Mouse Event Dispatch (Copy-Paste Ready)

```rust
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use ratatui::layout::Position;

fn handle_mouse(&mut self, event: MouseEvent) {
    let pos = Position::new(event.column, event.row);
    
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if self.top_nav_area.contains(pos) {
                self.handle_top_nav_click();
            } else if self.sub_nav_area.contains(pos) {
                self.handle_sub_nav_click();
            } else if self.content_area.contains(pos) {
                self.handle_content_click();
            }
        }
        MouseEventKind::ScrollUp => {
            if self.content_area.contains(pos) {
                self.scroll_up();
            }
        }
        _ => {}
    }
}
```

**Enable mouse**: `execute!(stdout(), EnableMouseCapture)?;`

---

## 3. F-Key Tab Switching (F4-F9 for 6 Tabs)

```rust
use crossterm::event::{KeyCode, KeyEvent};

fn handle_key(&mut self, key: KeyEvent) {
    match key.code {
        KeyCode::F(4) => self.switch_to_tab(0),  // F4
        KeyCode::F(5) => self.switch_to_tab(1),  // F5
        KeyCode::F(6) => self.switch_to_tab(2),  // F6
        KeyCode::F(7) => self.switch_to_tab(3),  // F7
        KeyCode::F(8) => self.switch_to_tab(4),  // F8
        KeyCode::F(9) => self.switch_to_tab(5),  // F9
        _ => {}
    }
}
```

**Why F4-F9?** Avoids F1-F3 (system), F10-F12 (terminal).

---

## 4. Tab/Shift+Tab Cycling (Sub-Nav)

```rust
#[derive(Debug, Clone, Copy)]
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
            Self::History => Self::Vitals,  // Wrap
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

// In event handler:
match key.code {
    KeyCode::Tab => self.sub_nav_focus = self.sub_nav_focus.next(),
    KeyCode::BackTab => self.sub_nav_focus = self.sub_nav_focus.prev(),
    _ => {}
}
```

---

## 5. Dynamic Patient Tabs (Vec-Based)

```rust
struct PatientTab {
    id: u32,
    name: String,
}

struct AppState {
    patient_tabs: Vec<PatientTab>,
    active_tab_index: usize,
}

impl AppState {
    fn open_tab(&mut self, id: u32, name: String) {
        if !self.patient_tabs.iter().any(|t| t.id == id) {
            self.patient_tabs.push(PatientTab { id, name });
            self.active_tab_index = self.patient_tabs.len() - 1;
        }
    }
    
    fn close_tab(&mut self, index: usize) {
        self.patient_tabs.remove(index);
        if self.active_tab_index >= self.patient_tabs.len() && !self.patient_tabs.is_empty() {
            self.active_tab_index = self.patient_tabs.len() - 1;
        }
    }
    
    fn switch_to_tab(&mut self, index: usize) {
        if index < self.patient_tabs.len() {
            self.active_tab_index = index;
        }
    }
}
```

**Max 6 tabs**: Check `self.patient_tabs.len() >= 6` before opening.

---

## Constraint Priority (Highest to Lowest)

1. `Min(n)` - Minimum size
2. `Max(n)` - Maximum size
3. `Length(n)` - Fixed size ← **Use for headers/footers**
4. `Percentage(n)` - % of total
5. `Ratio(n, d)` - Fraction
6. `Fill(n)` - Proportional fill ← **Use for content**

---

## MouseEvent Fields

```rust
pub struct MouseEvent {
    pub column: u16,              // X (0-based from left)
    pub row: u16,                 // Y (0-based from top)
    pub kind: MouseEventKind,     // Down, Up, Drag, Moved, Scroll*
    pub modifiers: KeyModifiers,  // Ctrl, Alt, Shift
}
```

---

## Common Gotchas

| Gotcha | Fix |
|--------|-----|
| Mouse clicks not detected | Call `execute!(stdout(), EnableMouseCapture)?` |
| F-keys not working | Use F4-F9 (avoid F1-F3, F10-F12) |
| Layout shrinks unexpectedly | Use `Fill(1)` for flexible areas, not `Min(0)` |
| Tab cycling wraps wrong way | Implement `next()` and `prev()` explicitly |
| Mouse position off by 1 | Remember: 0-based indexing (column 0 = leftmost) |

---

## Files to Reference

- **Layout**: https://docs.rs/ratatui/latest/ratatui/layout/index.html
- **Mouse Example**: https://github.com/ratatui/ratatui/blob/main/examples/apps/mouse-drawing/src/main.rs
- **Form Example**: https://github.com/ratatui/ratatui/blob/main/examples/apps/input-form/src/main.rs
- **Tabs Example**: https://github.com/ratatui/ratatui/blob/main/examples/apps/demo2/src/tabs.rs

