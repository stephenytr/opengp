# Pagination Patterns Audit — opengp-ui

**Date:** 2026-04-21  
**Scope:** All pagination approaches in `crates/opengp-ui`  
**Status:** ✅ Confirmed — Three intentionally different patterns

---

## SUMMARY

OpenGP-UI uses **three distinct pagination patterns**, each optimized for its use case:

1. **PaginatedState** — Page-based offset pagination (patient/appointment/billing lists)
2. **PaginatedList<T>** — Selection-based list with wrapping navigation (billing item selection)
3. **ClinicalTableList<T>** — Scroll-based table with keyboard/mouse support (clinical records)

All three patterns coexist intentionally. No inconsistencies detected.

---

## PATTERN 1: PaginatedState (Page-Based Offset)

### Definition
**File:** `crates/opengp-ui/src/ui/components/shared/mod.rs:1-53`

```rust
pub struct PaginatedState {
    pub page: usize,           // Current page number (0-indexed)
    pub page_size: usize,      // Items per page (default: 20)
    pub loading: bool,         // Loading indicator
    pub error: Option<String>, // Error message
}
```

### Key Methods
- `new()` — Initialize with page=0, page_size=20
- `set_page_size(height)` — Dynamically adjust page_size based on terminal height
- `total_pages(total_items)` — Calculate total pages
- `page_offset()` → `page * page_size` — Offset for database query
- `next_page(total_items)` — Move to next page (bounded)
- `prev_page()` — Move to previous page (saturating)

### Use Cases
**Patient List** — `crates/opengp-ui/src/ui/components/patient/state.rs:27`
```rust
pub struct PatientState {
    pub pagination: PaginatedState,
    // ...
}
```

**Appointment List** — `crates/opengp-ui/src/ui/components/workspace/appointment_state.rs:9`
```rust
pub struct AppointmentState {
    pub pagination: PaginatedState,
    // ...
}
```

**Billing List** — `crates/opengp-ui/src/ui/components/billing/state.rs:21`
```rust
pub struct BillingState {
    pub pagination: PaginatedState,
    // ...
}
```

**Patient Billing** — `crates/opengp-ui/src/ui/components/billing/patient_billing_state.rs:15`
```rust
pub struct PatientBillingState {
    pub pagination: PaginatedState,
    // ...
}
```

**Appointments View** — `crates/opengp-ui/src/ui/components/workspace/appointments_view.rs:139,195`
```rust
use crate::ui::components::shared::PaginatedState;
let mut state = PaginatedState::new();
```

### Why This Pattern
- **Offset-based**: Suitable for large datasets (patients, appointments, billing records)
- **Dynamic sizing**: Page size adapts to terminal height
- **Stateless queries**: Backend can fetch `LIMIT page_size OFFSET page_offset()`
- **Simple navigation**: Next/prev with bounds checking

---

## PATTERN 2: PaginatedList<T> (Selection-Based with Wrapping)

### Definition
**File:** `crates/opengp-ui/src/ui/components/billing/paginated_list.rs:1-61`

```rust
pub struct PaginatedList<T: Clone> {
    pub items: Vec<T>,           // All items in memory
    pub selected_index: usize,   // Currently selected item index
    pub scroll_state: ListState, // Ratatui ListState for rendering
    pub hovered_index: Option<usize>, // Hover tracking
}
```

### Key Methods
- `new(items)` — Initialize with all items loaded
- `select_next_wrap()` — Move to next item, wrap to start at end
- `select_prev_wrap()` — Move to previous item, wrap to end at start
- `selected()` → `Option<&T>` — Get currently selected item

### Use Cases
**Payment List** — `crates/opengp-ui/src/ui/components/billing/payment_list.rs:13,23,28`
```rust
use super::paginated_list::PaginatedList;

pub struct PaymentListState {
    paginated_list: PaginatedList<Payment>,
}

let paginated_list = PaginatedList::new(payments.clone());
```

**Claim List** — `crates/opengp-ui/src/ui/components/billing/claim_list.rs:13,23,37`
```rust
use super::paginated_list::PaginatedList;

pub struct ClaimListState {
    paginated_list: PaginatedList<MedicareClaim>,
}

let paginated_list = PaginatedList::new(claims.clone());
```

### Why This Pattern
- **Selection-focused**: For choosing one item from a small set (payments, claims)
- **Wrapping navigation**: Circular navigation (next at end → start, prev at start → end)
- **In-memory**: All items loaded; no pagination needed
- **Ratatui integration**: Uses `ListState` for native list rendering

---

## PATTERN 3: ClinicalTableList<T> (Scroll-Based Table)

### Definition
**File:** `crates/opengp-ui/src/ui/widgets/clinical_table_list.rs:1-399`

```rust
pub struct ClinicalTableList<T> {
    pub items: Vec<T>,                    // All items in memory
    pub columns: Vec<ColumnDef<T>>,       // Column definitions
    pub selected_index: usize,            // Selected row index
    pub scroll_offset: usize,             // Scroll position (top visible row)
    pub theme: Theme,
    pub title: String,
    pub loading: bool,
    pub empty_message: String,
    pub hovered_index: Option<usize>,     // Hover tracking
    pub double_click_detector: DoubleClickDetector,
}
```

### Key Methods
- `new(items, columns, theme, title, sort_fn)` — Initialize with optional sorting
- `move_up()` / `move_down()` — Move selection up/down (no wrap)
- `move_first()` / `move_last()` — Jump to first/last row
- `adjust_scroll(visible_rows)` — Keep selected row in view
- `handle_key(key)` → `Option<ListAction<T>>` — Keyboard navigation (↑↓ Home End PgUp PgDn Enter n e d i)
- `handle_mouse(mouse, area)` → `Option<ListAction<T>>` — Mouse scroll, click, double-click, right-click

### ListAction Enum
```rust
pub enum ListAction<T> {
    Select(usize),           // Selection changed
    Open(T),                 // Enter or double-click
    New,                     // 'n' key
    Edit(T),                 // 'e' key
    Delete(T),               // 'd' key
    ToggleInactive,          // 'i' key
    ContextMenu { index, x, y }, // Right-click
}
```

### Use Cases
**Vitals List** — `crates/opengp-ui/src/ui/components/clinical/vitals_list.rs:2,113,114`
```rust
use crate::ui::widgets::{ClinicalTableList, ColumnDef, ListAction};

fn table(&self) -> ClinicalTableList<VitalSigns> {
    let mut table = ClinicalTableList::new(/* ... */);
}
```

**Medical History List** — `crates/opengp-ui/src/ui/components/clinical/medical_history_list.rs:2,53,58,59`
```rust
use crate::ui::widgets::{ClinicalTableList, ColumnDef, ListAction};

fn sync_nav(&mut self, list: &ClinicalTableList<MedicalHistory>) { /* ... */ }
fn table_list(&self) -> ClinicalTableList<MedicalHistory> {
    let mut list = ClinicalTableList::new(/* ... */);
}
```

**Consultation List** — `crates/opengp-ui/src/ui/components/clinical/consultation_list.rs:2,151,152`
```rust
use crate::ui::widgets::{ClinicalTableList, ColumnDef, ListAction};

fn table(&self) -> ClinicalTableList<Consultation> {
    let mut table = ClinicalTableList::new(/* ... */);
}
```

**Allergy List** — `crates/opengp-ui/src/ui/components/clinical/allergy_list.rs:2,97,98`
```rust
use crate::ui::widgets::{ClinicalTableList, ColumnDef, ListAction};

fn table(&self) -> ClinicalTableList<Allergy> {
    let mut table = ClinicalTableList::new(/* ... */);
}
```

**Family History List** — `crates/opengp-ui/src/ui/components/clinical/family_history_list.rs:3,78,79`
```rust
use crate::ui::widgets::{ClinicalTableList, ColumnDef, ListAction};

fn as_table(&self) -> ClinicalTableList<FamilyHistory> {
    let mut table = ClinicalTableList::new(/* ... */);
}
```

### Why This Pattern
- **Table-first**: Columnar display with configurable headers
- **Scroll-based**: Tracks `scroll_offset` to show visible rows
- **Rich interaction**: Keyboard (↑↓ Home End PgUp PgDn), mouse (scroll, click, double-click, right-click)
- **Clinical focus**: Designed for clinical records (vitals, allergies, history)
- **In-memory**: All items loaded; sorting applied at construction

---

## COMPARISON TABLE

| Aspect | PaginatedState | PaginatedList<T> | ClinicalTableList<T> |
|--------|---|---|---|
| **Navigation** | Page-based (next/prev page) | Item-based with wrapping | Row-based with scroll |
| **Data Size** | Large (paginated from DB) | Small (all in memory) | Medium (all in memory) |
| **Keyboard** | Custom (handled by component) | Ratatui ListState | Full (↑↓ Home End PgUp PgDn n e d i) |
| **Mouse** | None | None | Full (scroll, click, double-click, right-click) |
| **Selection** | Page + offset | Index with wrap | Index with scroll |
| **Rendering** | Custom (component-specific) | Ratatui List | Ratatui Table |
| **Use Case** | Patient/Appointment/Billing lists | Payment/Claim selection | Clinical records (vitals, allergies, history) |
| **Hover** | No | No | Yes |
| **Double-click** | No | No | Yes |
| **Context Menu** | No | No | Yes |

---

## CONSISTENCY ASSESSMENT

### ✅ No Inconsistencies Found

Each pattern is **intentionally different** and serves a distinct purpose:

1. **PaginatedState** — Designed for large, paginated datasets where offset-based queries are efficient
2. **PaginatedList<T>** — Designed for small, in-memory selections with circular navigation
3. **ClinicalTableList<T>** — Designed for clinical records with rich table rendering and interaction

### Design Rationale

- **Separation of concerns**: Each pattern encapsulates its own navigation logic
- **Appropriate complexity**: Simple patterns for simple use cases, rich patterns for complex ones
- **Ratatui integration**: Leverages native Ratatui widgets (`ListState`, `Table`) where appropriate
- **Keyboard/mouse support**: Varies by use case (clinical records need rich interaction; billing lists don't)

---

## RECOMMENDATIONS

### Current State
✅ **No changes needed.** The three patterns are well-designed and intentionally different.

### Future Considerations
1. **Documentation**: Add comments to each pattern explaining when to use it
2. **Naming clarity**: Consider renaming `PaginatedList<T>` to `SelectableList<T>` to avoid confusion with `PaginatedState`
3. **Trait abstraction**: If a fourth pattern emerges, consider extracting a common `ListNavigation` trait
4. **Testing**: All three patterns have comprehensive tests; maintain this standard

---

## EVIDENCE SUMMARY

### PaginatedState Usage (6 files)
- Definition: `crates/opengp-ui/src/ui/components/shared/mod.rs:1-53`
- Patient: `crates/opengp-ui/src/ui/components/patient/state.rs:27`
- Appointment: `crates/opengp-ui/src/ui/components/workspace/appointment_state.rs:9`
- Billing: `crates/opengp-ui/src/ui/components/billing/state.rs:21`
- Patient Billing: `crates/opengp-ui/src/ui/components/billing/patient_billing_state.rs:15`
- Appointments View: `crates/opengp-ui/src/ui/components/workspace/appointments_view.rs:139,195`

### PaginatedList<T> Usage (4 files)
- Definition: `crates/opengp-ui/src/ui/components/billing/paginated_list.rs:1-61`
- Export: `crates/opengp-ui/src/ui/components/billing/mod.rs:43`
- Payment List: `crates/opengp-ui/src/ui/components/billing/payment_list.rs:13,23,28`
- Claim List: `crates/opengp-ui/src/ui/components/billing/claim_list.rs:13,23,37`

### ClinicalTableList<T> Usage (7 files)
- Definition: `crates/opengp-ui/src/ui/widgets/clinical_table_list.rs:1-399`
- Export: `crates/opengp-ui/src/ui/widgets/mod.rs:31`
- Vitals List: `crates/opengp-ui/src/ui/components/clinical/vitals_list.rs:2,113,114`
- Medical History: `crates/opengp-ui/src/ui/components/clinical/medical_history_list.rs:2,53,58,59`
- Consultation List: `crates/opengp-ui/src/ui/components/clinical/consultation_list.rs:2,151,152`
- Allergy List: `crates/opengp-ui/src/ui/components/clinical/allergy_list.rs:2,97,98`
- Family History: `crates/opengp-ui/src/ui/components/clinical/family_history_list.rs:3,78,79`

---

## CONCLUSION

OpenGP-UI's pagination architecture is **well-designed and intentional**. The three patterns coexist without conflict, each optimized for its specific use case. No refactoring or consolidation is recommended.
