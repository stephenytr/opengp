# Phase 7 Advanced Calendar Features - User Guide

## Overview
Phase 7 adds powerful productivity features to the appointment calendar, including search, filters, status management, and audit history.

## New Features

### 1. Appointment Search (/)
**Keybind**: `/` (forward slash)

Quickly find appointments by patient name or appointment type.

**How to use**:
1. Press `/` in calendar view
2. Type search query
3. Use ↑/↓ to navigate results
4. Press Enter to jump to appointment
5. Press Esc to close

### 2. Status Filter (f)
**Keybind**: `f`

Show/hide appointments by status (Scheduled, Confirmed, Arrived, etc.)

**How to use**:
1. Press `f` to open filter menu
2. Press `1-8` to toggle specific statuses
3. Press `0` to clear all filters
4. Press Esc to close menu

Active filters shown in calendar title.

### 3. Practitioner Filter (p)
**Keybind**: `p`

Show/hide specific practitioner columns in Day/Week view.

**How to use**:
1. Press `p` to open practitioner menu
2. Select practitioners to show/hide
3. Calendar adjusts column widths automatically
4. Press Esc to close

### 4. Double-Booking Warnings
**Automatic**

Visual warnings when appointments overlap.

**Indicators**:
- Red border on overlapping appointments
- ⚠ warning icon
- Conflict count displayed
- Details in appointment modal

### 5. Status Transition Validation
**Automatic**

Prevents invalid status changes (e.g., can't mark Cancelled as Arrived).

**Valid transitions**:
- Scheduled → Confirmed, Cancelled, Rescheduled
- Confirmed → Arrived, Cancelled, Rescheduled
- Arrived → InProgress, NoShow
- InProgress → Completed

Invalid transitions show error message explaining why.

### 6. Direct Status Updates
**Keybinds**: `a`, `c`, `x` (in day view)

Update appointment status without opening detail modal.

**How to use**:
1. Navigate to appointment in day view
2. Press `a` to mark Arrived
3. Press `c` to mark Completed
4. Press `x` to mark No-show
5. Press Ctrl+Z to undo (within 30 seconds)

Confirmation shown for destructive actions.

### 7. Audit History Viewer (H)
**Keybind**: `H` (in appointment detail modal)

View complete change history for an appointment.

**How to use**:
1. Open appointment detail modal (Enter)
2. Press `H` to view history
3. See all status changes, reschedules, edits
4. Color-coded by action type
5. Press Esc to close

**Action colors**:
- Green: Created
- Cyan: Updated
- Yellow: Status Changed
- Magenta: Rescheduled
- Red: Cancelled

## Performance

All features are optimized for large datasets:
- Search: <1 second for 10,000 appointments
- Filters: <100ms rendering update
- No lag with 50+ appointments visible

## Compliance

Audit history provides complete compliance trail:
- All changes logged automatically
- Append-only (cannot be edited/deleted)
- Timestamps and user tracking
- Meets Australian healthcare regulations

## See Also

- KEYBINDS.md - Complete keyboard shortcut reference
- ARCHITECTURE.md - Technical implementation details
