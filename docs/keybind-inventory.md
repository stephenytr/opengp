# OpenGP Keybind Inventory

**Generated**: 2026-02-13  
**Purpose**: Complete audit of all keyboard bindings across the OpenGP TUI application

---

## Table of Contents

1. [Global Keybinds](#global-keybinds)
2. [Patient List Screen](#patient-list-screen)
3. [Patient Form](#patient-form)
4. [Appointment List Screen](#appointment-list-screen)
5. [Appointment Calendar](#appointment-calendar)
6. [Appointment Form](#appointment-form)
7. [Identified Conflicts](#identified-conflicts)
8. [Identified Inconsistencies](#identified-inconsistencies)
9. [Missing Help Text](#missing-help-text)

---

## Global Keybinds

**Context**: Work everywhere unless a modal/form is open  
**File**: `src/app.rs` (lines 280-296)

| Key | Action | Works When | Notes |
|-----|--------|------------|-------|
| `Ctrl+C` | Quit application | Always | Hard quit |
| `q` | Quit application | Always | |
| `1` | Navigate to Patients screen | Not in form mode | |
| `2` | Navigate to Appointments screen | Not in form mode | |
| `3` | Navigate to Clinical screen | Not in form mode | Not implemented |
| `4` | Navigate to Billing screen | Not in form mode | Not implemented |

**Help Text**: Shown in placeholder screens, NOT shown in active screens

---

## Patient List Screen

**Context**: Patient list view (browsing patients)  
**File**: `src/components/patient/list.rs` (lines 202-246)

### Normal Mode (not searching)

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `j` or `↓` | Select next patient | Yes (in title) |
| `k` or `↑` | Select previous patient | Yes (in title) |
| `g` | Jump to first patient | Yes (in title) |
| `G` | Jump to last patient | Yes (in title) |
| `Enter` | View patient details | No - Action not implemented |
| `n` | Create new patient | Yes (in title) |
| `/` | Enter search mode | Yes (in title) |
| `Esc` | Clear search filter | Yes (in title) |

**Title Help Text**: `" Patients (j/k/↑↓: Nav, g/G: First/Last, n: New, /: Search, Esc: Clear) "`

### Search Mode

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| Any character | Add to search query | No |
| `Backspace` | Remove last character | No |
| `Enter` | Exit search mode | No |
| `Esc` | Exit search mode | No |
| `↑` | Select previous result | No |
| `↓` | Select next result | No |

**Search Help**: Shows inline text `"Search: {query}█"` but no key hints

---

## Patient Form

**Context**: Creating/editing patient  
**File**: `src/components/patient/form.rs` (lines 340-383)

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `Esc` | Cancel form | Yes (footer) |
| `Enter` | Submit form | Yes (footer) |
| `F10` | Submit form | Yes (footer) |
| `Tab` | Next field | Yes (footer) |
| `Shift+Tab` | Previous field | Yes (footer) |
| `↑` | Previous field / Cycle gender | Yes (footer) |
| `↓` | Next field / Cycle gender | Yes (footer) |
| Any character | Input text | N/A |
| `Backspace` | Delete character | N/A |

**Footer Help**: `"Tab/Shift+Tab: Next/Prev, ↑↓: Fields, Enter/F10: Submit, Esc: Cancel"`

**Notes**:
- `↑/↓` behavior changes based on current field (navigation vs gender selection)
- Field-specific validation for date/numbers

---

## Appointment List Screen

**Context**: Simple appointment list (not calendar)  
**File**: `src/components/appointment/list.rs` (lines 104-124)

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `j` or `↓` | Select next appointment | Yes (in title) |
| `k` or `↑` | Select previous appointment | Yes (in title) |
| `g` | Jump to first appointment | No |
| `G` | Jump to last appointment | No |
| `n` | Create new appointment | Yes (in title) |

**Title Help**: `" Appointments (j/k/↑↓: Nav, g/G: First/Last, n: New) "`

---

## Appointment Calendar

**File**: `src/components/appointment/calendar.rs` (lines 2550-2800)

### Base View (Month + Day/Week)

**No Modal Open**

| Key | Modifiers | Action | Context | Help Displayed? |
|-----|-----------|--------|---------|-----------------|
| `Ctrl+Z` | - | Undo last status change | Any | Yes (day view) |
| `/` | - | Open search modal | Any | Yes (day view) |
| `f` | - | Open filter menu | Any | Yes (day view) |
| `p` | - | Open practitioner filter | Any | Yes (day view) |
| `m` | - | Toggle multi-select mode | Any | Yes (day view) |

### Month View Focus

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `↑` | Previous week | Yes (title) |
| `↓` | Next week | Yes (title) |
| `←` | Previous day | No |
| `→` | Next day | No |
| `h` | Previous month | Yes (title) |
| `l` | Next month | Yes (title) |
| `t` | Jump to today | Yes (title) |
| `Enter` | Switch to day view | Yes (title) |
| `Tab` | Switch to day view | Yes (title) |
| `n` | Create new appointment | Yes (title) |

**Title Help**: `" {Month Year} (↑↓: Day, h/l: Month, t: Today, n: New, Enter/Tab: Day View) "`

### Day View Focus (Day Mode)

| Key | Modifiers | Action | Help Displayed? |
|-----|-----------|--------|-----------------|
| `k` or `↑` | - | Previous time slot | Yes (title) |
| `j` or `↓` | - | Next time slot | Yes (title) |
| `Tab` | - | Switch to month view | Yes (title) |
| `Esc` | - | Switch to month view | Yes (title) |
| `Enter` | - | Open appointment details | No |
| `a` | - | Mark as Arrived | No |
| `c` | - | Mark as Completed | No |
| `n` | - | Mark as No Show | No |
| `v` | - | Toggle Day/Week view | Yes (title - week) |
| `←` | Shift | Previous week (week view) | Yes (title - week) |
| `→` | Shift | Next week (week view) | Yes (title - week) |
| `Space` | - | Toggle appointment selection (multi-select) | Yes (multi-select title) |
| `a` | Ctrl | Select all appointments | Yes (multi-select title) |
| `b` | - | Open batch operations menu | Yes (multi-select title) |

**Day View Title (Normal)**: `" (j/k/↑↓: Nav, v: Week, n: New, Enter: Details, /: Search, f: Filter, p: Practitioner, m: Multi-Select, Ctrl+Z: Undo, Tab/Esc: Month) "`

**Day View Title (Multi-Select)**: `" Multi-Select: {count} selected (Space: Toggle, Ctrl+A: All, Esc: Exit, m: Toggle Mode) "`

**Week View Title**: `" (j/k/↑↓: Nav, v: Day, Shift+←→: Week, n: New, Tab/Esc: Month) "`

### Appointment Detail Modal

**File**: `src/components/appointment/calendar.rs` (lines 1053-1090)

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `Esc` | Close modal | Yes (footer) |
| `A` or `a` | Mark as Arrived | Yes (footer) |
| `C` or `c` | Mark as Completed | Yes (footer) |
| `X` or `x` | Mark as No Show | Yes (footer) |
| `H` or `h` | View audit history | Yes (footer) |
| `R` or `r` | Reschedule appointment | Yes (footer) |

**Footer Help**: Shows actions `"A: Arrived  C: Completed  X: No Show"` and `"R: Reschedule  H: History  Esc: Close"`

### Reschedule Modal

**File**: `src/components/appointment/calendar.rs` (lines 1146-1185)

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `Esc` | Cancel reschedule | Yes (footer) |
| `↑` | Move time earlier (15 min) | Yes (footer) |
| `↓` | Move time later (15 min) | Yes (footer) |
| `+` | Increase duration (15 min) | Yes (footer) |
| `-` | Decrease duration (15 min) | Yes (footer) |
| `Enter` | Save reschedule | Yes (footer) |

**Footer Help**: `"↑↓: Time  +/-: Duration  Enter: Save  Esc: Cancel"`

### Search Modal

**File**: `src/components/appointment/calendar.rs` (lines 1188-1234)

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `Esc` | Close search | Yes (footer) |
| `↑` | Previous result | Yes (footer) |
| `↓` | Next result | Yes (footer) |
| `Enter` | Navigate to selected | Yes (footer) |
| Any character | Add to query | No |
| `Backspace` | Remove character | No |

**Footer Help**: `"↑↓: Navigate  Enter: Select  Esc: Close"`

### Filter Menu (Status)

**File**: `src/components/appointment/calendar.rs` (lines 1262-1305)

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `Esc` | Close menu | Yes (footer) |
| `0` | Clear all filters | Yes (footer) |
| `1` | Toggle Scheduled | No |
| `2` | Toggle Confirmed | No |
| `3` | Toggle Arrived | No |
| `4` | Toggle InProgress | No |
| `5` | Toggle Completed | No |
| `6` | Toggle NoShow | No |
| `7` | Toggle Cancelled | No |
| `8` | Toggle Rescheduled | No |

**Footer Help**: `"0: Clear all  Esc: Close"`

**Note**: Shows checkboxes with `[1]`, `[2]`, etc. but not explicitly listed in help

### Practitioner Filter Menu

**File**: `src/components/appointment/calendar.rs` (lines 1308-1327)

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `Esc` | Close menu | Yes (footer) |
| `0` | Clear all filters | Yes (footer) |
| `1-9` | Toggle practitioner N | No |

**Footer Help**: `"0: Clear all  Esc: Close"`

### Audit History Modal

**File**: `src/components/appointment/calendar.rs` (lines 1093-1117)

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `Esc` | Close modal | Yes (footer) |
| `↑` | Previous entry | Yes (footer) |
| `↓` | Next entry | Yes (footer) |

**Footer Help**: `"↑/↓: Navigate  Esc: Close"`

### Confirmation Dialog

**File**: `src/components/appointment/calendar.rs` (lines 2168-2188)

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `Y` or `y` | Confirm action | Yes (footer) |
| `N` or `n` | Cancel action | Yes (footer) |
| `Esc` | Cancel action | Yes (footer) |

**Footer Help**: `"Y: Confirm  N: Cancel  Esc: Cancel"`

### Error Modal

**File**: `src/components/appointment/calendar.rs`

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `Esc` | Close error | Yes (footer) |
| `Enter` | Close error | Yes (footer) |

**Footer Help**: `"Press Esc or Enter to close"`

### Batch Operations Menu

**File**: `src/components/appointment/calendar.rs` (lines 1120-1143)

| Key | Action | Help Displayed? |
|-----|--------|-----------------|
| `Esc` | Close menu | Yes (footer) |
| `1` | Mark all as Arrived | Yes (list) |
| `2` | Mark all as Completed | Yes (list) |
| `3` | Cancel all (not implemented) | Yes (list, grayed) |

**Footer Help**: `"Select an option or press Esc to cancel"`

---

## Appointment Form

**Context**: Creating/editing appointments  
**File**: `src/components/appointment/form.rs` (lines 920-966)

| Key | Modifiers | Action | Field | Help Displayed? |
|-----|-----------|--------|-------|-----------------|
| `Esc` | - | Cancel form (or close dropdown) | Any | Yes (footer) |
| `Ctrl+S` | - | Submit form | Any | Yes (footer) |
| `Tab` | - | Next field | Any | Yes (footer) |
| `Tab` | Shift | Previous field | Any | Yes (footer) |
| `Enter` | - | Select dropdown item | Patient/Practitioner/Type | Yes (footer) |
| `Space` | - | Toggle dropdown | Practitioner/Type | Yes (footer) |
| `↑` | - | Navigate dropdown/results | Patient/Practitioner/Type | Yes (footer) |
| `↓` | - | Navigate dropdown/results | Patient/Practitioner/Type | Yes (footer) |
| `Ctrl+U` | - | Clear patient search | Patient | No |
| Any character | - | Input text | Text fields | N/A |
| `Backspace` | - | Delete character | Text fields | N/A |

**Patient Field Help** (when focused): `"[↑↓] Navigate  [Enter] Select  [Esc] Clear  [Ctrl+S] Submit"`

**Other Fields Help**: `"[Tab] Next  [Enter] Select  [↑↓] Navigate  [Ctrl+S] Submit  [Esc] Cancel"`

**Notes**:
- Patient field uses fuzzy search with sublime_fuzzy
- Date format: `YYYY-MM-DD`
- Time format: `HH:MM`
- Validation happens on submit

---

## Identified Conflicts

### 1. **`n` Key in Appointment Calendar Day View**

**Conflict**: Two different meanings in the same context

- **Normal mode**: Create new appointment
- **Day view focus**: Mark selected appointment as "No Show" 

**Location**: `src/components/appointment/calendar.rs`
- Line 2740: `KeyCode::Char('n') => Action::AppointmentCreate`
- Line 2780-2782: `KeyCode::Char('n') if !key.modifiers.contains(KeyModifiers::CONTROL) => self.initiate_status_change(AppointmentStatus::NoShow)`

**Impact**: When in day view with an appointment selected, pressing `n` marks as no-show instead of creating new. This is intentional but confusing.

**Recommendation**: Use different key for "No Show" (e.g., `x` or `X`)

### 2. **`a` Key Context Ambiguity**

**Conflict**: Different meaning with modifier

- **No modifier**: Mark as Arrived (single appointment)
- **Ctrl+A**: Select all appointments (multi-select mode)

**Location**: `src/components/appointment/calendar.rs`
- Line 2778: `KeyCode::Char('a') => self.initiate_status_change(AppointmentStatus::Arrived)`
- Line 2669: `KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL)`

**Impact**: Minimal - modifier makes it clear, but could be confusing

**Status**: Acceptable (standard Ctrl+A pattern)

### 3. **`Enter` in Patient List Does Nothing**

**Issue**: `Enter` on a patient does nothing (action not implemented)

**Location**: `src/components/patient/list.rs` lines 224-230

**Impact**: User expectation is that Enter opens patient details

**Recommendation**: Implement patient detail view or remove the keybind

---

## Identified Inconsistencies

### 1. **Navigation Keys Inconsistent**

**Issue**: Some screens support `j/k`, some don't

| Screen | `j/k` | Arrow keys | Both |
|--------|-------|------------|------|
| Patient List | ✅ | ✅ | ✅ |
| Appointment List | ✅ | ✅ | ✅ |
| Appointment Calendar (Day) | ✅ | ✅ | ✅ |
| Patient Form | ❌ | ✅ | ❌ |
| Appointment Form | ❌ | ✅ | ❌ |

**Recommendation**: Forms can stay arrows-only (standard form behavior), but all list views should support both

### 2. **Cancel/Quit Keys Inconsistent**

**Issue**: Different keys for similar "exit" actions

| Context | Key | Action |
|---------|-----|--------|
| Global | `q` | Quit app |
| Patient List search | `Esc` | Clear/exit search |
| Patient Form | `Esc` | Cancel form |
| Appointment Calendar modals | `Esc` | Close modal |
| Error modal | `Esc` OR `Enter` | Close |
| Confirmation dialog | `Esc` OR `n` | Cancel |

**Status**: Mostly consistent (Esc = cancel/close), but error modal also accepts Enter

**Recommendation**: Standardize on Esc for all modal closes

### 3. **Create Action Key Inconsistent**

**Issue**: All screens use `n` for "New", but it conflicts with "No Show" in calendar

| Screen | Key | Action |
|--------|-----|--------|
| Patient List | `n` | New patient |
| Appointment List | `n` | New appointment |
| Appointment Calendar (Month) | `n` | New appointment |
| Appointment Calendar (Day) | `n` | No Show OR New (context dependent) |

**Recommendation**: Use different key for "No Show" (e.g., `x`)

### 4. **Help Text Display Inconsistent**

**Issue**: Some screens show help in title bar, some in footer, some nowhere

| Screen | Help Location |
|--------|---------------|
| Patient List | Title bar |
| Patient Form | Footer |
| Appointment List | Title bar |
| Appointment Calendar | Title bar (very long) |
| Appointment Form | Footer |
| Patient List Search Mode | None |

**Recommendation**: Standardize on footer for all help text, reserve title for context info

### 5. **Status Change Keys Not Standard**

**Issue**: Single-letter keys for critical actions (no confirmation for some)

| Key | Action | Requires Confirmation? |
|-----|--------|------------------------|
| `a` | Mark Arrived | No |
| `c` | Mark Completed | No |
| `n` | Mark No Show | Yes |

**Recommendation**: Either require confirmation for all status changes, or use Shift+letter for destructive actions

---

## Missing Help Text

### 1. **Patient List Search Mode**

**Location**: `src/components/patient/list.rs`

**Issue**: When in search mode, no help text shown for:
- `↑/↓` to navigate results
- `Enter` to select
- `Esc` to exit search

**Current**: Only shows `"Search: {query}█"`

**Recommendation**: Show help text: `"↑↓: Navigate  Enter: Select  Esc: Exit  Backspace: Delete"`

### 2. **Appointment Calendar Month View**

**Location**: `src/components/appointment/calendar.rs`

**Issue**: Title shows some keys but omits:
- `←/→` for day navigation
- `n` for new appointment (shown but not obvious)

**Current Help**: `" {Month Year} (↑↓: Day, h/l: Month, t: Today, n: New, Enter/Tab: Day View) "`

**Recommendation**: Add `←→: Day` or abbreviate existing help

### 3. **Filter Menus**

**Location**: Calendar status and practitioner filters

**Issue**: Shows numbered options with checkboxes but doesn't explicitly say "Press 1-8 to toggle"

**Current**: Shows `[1] ☑ Scheduled` but footer only says `"0: Clear all  Esc: Close"`

**Recommendation**: Add to footer: `"1-8: Toggle  0: Clear all  Esc: Close"`

### 4. **Multi-Select Mode Entry**

**Location**: Appointment calendar

**Issue**: No help shown when first entering multi-select mode - user must already know `Space`, `Ctrl+A`, `b`

**Recommendation**: Show brief tutorial on first entry, or add persistent hint

### 5. **Form Field-Specific Keys**

**Location**: Appointment form patient field

**Issue**: `Ctrl+U` to clear patient search is not documented anywhere

**Recommendation**: Add to help text when patient field is focused

---

## Summary Statistics

### Total Keybinds by Screen

| Screen | Base Keys | Modal Keys | Total |
|--------|-----------|------------|-------|
| Global | 6 | 0 | 6 |
| Patient List | 8 | 6 (search) | 14 |
| Patient Form | 8 | 0 | 8 |
| Appointment List | 5 | 0 | 5 |
| Appointment Calendar (Base) | 17 | 0 | 17 |
| Calendar - Detail Modal | 6 | 0 | 6 |
| Calendar - Reschedule Modal | 6 | 0 | 6 |
| Calendar - Search Modal | 5 | 0 | 5 |
| Calendar - Filter Menu | 10 | 0 | 10 |
| Calendar - Practitioner Menu | 11 | 0 | 11 |
| Calendar - Audit Modal | 3 | 0 | 3 |
| Calendar - Confirmation | 3 | 0 | 3 |
| Calendar - Error Modal | 2 | 0 | 2 |
| Calendar - Batch Menu | 4 | 0 | 4 |
| Appointment Form | 11 | 0 | 11 |
| **TOTAL** | **105** | **6** | **111** |

### Conflict Count

- **Critical**: 1 (`n` key in calendar day view)
- **Minor**: 1 (Enter does nothing in patient list)
- **Acceptable**: 1 (`a` with Ctrl modifier)

### Inconsistency Count

- Navigation keys: 1
- Cancel/quit keys: 1 (minor)
- Create action: 1 (critical, same as conflict)
- Help text display: 1
- Status change UX: 1

### Missing Help Count

- **5 contexts** missing or incomplete help text

---

## Recommendations Priority

### High Priority

1. **Fix `n` key conflict in calendar** - Use `x` for "No Show"
2. **Implement or remove patient list Enter action**
3. **Add help text to search modes**
4. **Standardize status change confirmation**

### Medium Priority

5. **Add help text to filter menus** (show number keys)
6. **Shorten calendar day view help** (too long, wraps on small terminals)
7. **Document Ctrl+U in appointment form**

### Low Priority

8. **Standardize help text location** (footer vs title)
9. **Add multi-select mode tutorial**
10. **Make all list views support j/k consistently**

---

## Notes

- All keybinds extracted from source code as of 2026-02-13
- Clinical and Billing screens not yet implemented
- Some help text shown in title bars is very long and may wrap on smaller terminals
- Multi-select mode is a recent addition and may need UX refinement
- Undo functionality (Ctrl+Z) has 30-second timeout
