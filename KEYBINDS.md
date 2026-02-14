# Keybinds Reference Guide

This document provides a comprehensive reference for all keyboard shortcuts and navigation patterns in OpenGP. The keybind system follows a vim-inspired philosophy with consistent mnemonics across components.

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Quick Reference](#quick-reference)
3. [Global Keybinds](#global-keybinds)
4. [Navigation Patterns](#navigation-patterns)
5. [Component-Specific Keybinds](#component-specific-keybinds)
6. [Modal Keybinds](#modal-keybinds)
7. [Form Navigation](#form-navigation)
8. [Conflict Resolution](#conflict-resolution)
9. [Tips for New Users](#tips-for-new-users)

---

## Design Philosophy

OpenGP's keybind system is built on these principles:

- **Vim-inspired**: `hjkl` for directional navigation, `g`/`G` for jumping to first/last items
- **Mnemonic keys**: Keys relate to their actions (`n`=New, `a`=Arrived, `c`=Completed, `r`=Reschedule)
- **Context awareness**: Keys have different meanings in different modal states
- **Consistency**: Same actions use same keys across components where possible
- **No conflicts**: Within the same context, each key has a unique meaning

---

## Quick Reference

| Key | Global | Lists | Calendar | Forms | Modals |
|-----|--------|-------|----------|-------|--------|
| `q` / `Ctrl+C` | Quit | - | - | Cancel | Close |
| `1-4` | Navigate screens | - | - | - | - |
| `j` / `↓` | - | Next item | Next day/slot | Next field | Navigate |
| `k` / `↑` | - | Previous item | Previous day/slot | Previous field | Navigate |
| `h` | - | - | Previous month | - | - |
| `l` | - | - | Next month | - | - |
| `g` | - | First item | - | - | - |
| `G` | - | Last item | - | - | - |
| `n` | - | New item | New appointment | - | - |
| `t` | - | - | Today | - | - |
| `/` | - | Search | Search | - | Search |
| `Esc` | - | Clear search | Return/Cancel | Cancel | Close |
| `Enter` | - | Select | Switch view/Select | Submit/Confirm | Select/Confirm |
| `Tab` | - | - | Toggle view | Next field | - |
| `a` | - | - | - | - | Mark Arrived |
| `i` | - | - | - | - | Mark In Progress |
| `c` | - | - | - | - | Mark Completed |
| `x` | - | - | - | - | Mark No-show |
| `r` | - | - | - | - | Reschedule |
| `v` | - | - | Toggle view | - | - |
| `f` | - | - | Status filter | - | - |
| `p` | - | - | Practitioner filter | - | - |
| `Ctrl+S` | - | - | - | Submit | - |
| `F10` | - | - | - | Submit | - |
| `Shift+Tab` | - | - | - | Previous field | - |
| `+` | - | - | - | - | Increase duration |
| `-` | - | - | - | - | Decrease duration |
| `0-8` | - | - | - | - | Filter options |
| `Ctrl+Z` | - | - | Undo status change | - | - |
| `Shift+←/→` | - | - | Previous/Next week | - | - |

---

## Global Keybinds

These keybinds work everywhere in the application:

| Key | Action |
|-----|--------|
| **`q`** | Quit application |
| **`Ctrl+C`** | Quit application |
| **`1`** | Navigate to Patients screen |
| **`2`** | Navigate to Appointments screen |
| **`3`** | Navigate to Clinical screen |
| **`4`** | Navigate to Billing screen |

---

## Navigation Patterns

### Vim-Inspired Navigation

OpenGP adopts vim-like navigation for consistency and efficiency:

| Keys | Action | Mnemonic |
|------|--------|----------|
| **`j`** / **`↓`** | Move down/next item | "j" looks like down arrow |
| **`k`** / **`↑`** | Move up/previous item | "k" looks like up arrow |
| **`h`** | Move left/previous | "h" for left |
| **`l`** | Move right/next | "l" for right |
| **`g`** | Jump to first item | "g" for "go to top" |
| **`G`** | Jump to last item | "G" for "go to bottom" |

### List Navigation

Standard navigation patterns apply to all list components:

| Keys | Action |
|------|--------|
| **`j`** / **`Down`** | Move to next item |
| **`k`** / **`Up`** | Move to previous item |
| **`g`** | Jump to first item |
| **`G`** | Jump to last item |

---

## Component-Specific Keybinds

### Patient List Component

| Key | Action |
|-----|--------|
| **`j`** / **`Down`** | Move to next patient |
| **`k`** / **`Up`** | Move to previous patient |
| **`g`** | Jump to first patient |
| **`G`** | Jump to last patient |
| **`n`** | Create new patient |
| **`/`** | Enter search mode |
| **`Esc`** | Clear search (when in search mode) |
| **`Enter`** | Select patient (reserved for future use) |

### Appointment List Component

| Key | Action |
|-----|--------|
| **`j`** / **`Down`** | Move to next appointment |
| **`k`** / **`Up`** | Move to previous appointment |
| **`g`** | Jump to first appointment |
| **`G`** | Jump to last appointment |
| **`n`** | Create new appointment |

### Appointment Calendar - Month View

| Key | Action |
|-----|--------|
| **`↑`** / **`↓`** | Navigate to previous/next day |
| **`h`** | Navigate to previous month |
| **`l`** | Navigate to next month |
| **`t`** | Jump to today's date |
| **`n`** | Create new appointment |
| **`Enter`** | Switch to day view |
| **`Tab`** | Switch to day view |

### Appointment Calendar - Day View

| Key | Action |
|-----|--------|
| **`j`** / **`k`** / **`↑`** / **`↓`** | Navigate between time slots |
| **`v`** | Toggle between day/week view |
| **`n`** | Create new appointment |
| **`Tab`** / **`Esc`** | Return to month view |
| **`Enter`** | Open selected appointment details |
| **`/`** | Open search modal |
| **`f`** | Open status filter menu |
| **`p`** | Open practitioner filter menu |
| **`Ctrl+Z`** | Undo last status change |
| **`Shift+←`** | Previous week (week view only) |
| **`Shift+→`** | Next week (week view only) |

---

## Modal Keybinds

### Appointment Detail Modal

| Key | Action | Case Sensitivity |
|-----|--------|------------------|
| **`a`** / **`A`** | Mark as Arrived | Case-insensitive |
| **`i`** / **`I`** | Mark as In Progress | Case-insensitive |
| **`c`** / **`C`** | Mark as Completed | Case-insensitive |
| **`x`** / **`X`** | Mark as No-show | Case-insensitive |
| **`r`** / **`R`** | Reschedule appointment | Case-insensitive |
| **`Esc`** | Close modal | - |

### Appointment Reschedule Modal

| Key | Action |
|-----|--------|
| **`↑`** | Move appointment time earlier (15-minute increments) |
| **`↓`** | Move appointment time later (15-minute increments) |
| **`+`** | Increase appointment duration (15-minute increments) |
| **`-`** | Decrease appointment duration (15-minute increments) |
| **`Enter`** | Confirm reschedule |
| **`Esc`** | Cancel reschedule |

### Appointment Search Modal

| Key | Action |
|-----|--------|
| **Character keys** | Type search query |
| **`Backspace`** | Delete last character |
| **`↑`** / **`↓`** | Navigate search results |
| **`Enter`** | Jump to selected appointment |
| **`Esc`** | Close search modal |

### Filter Menus (Status/Practitioner)

| Key | Action |
|-----|--------|
| **`0`** | Clear all filters |
| **`1-8`** | Toggle specific filter (number corresponds to option) |
| **`Esc`** | Close filter menu |

---

## Form Navigation

All form components follow consistent navigation patterns:

### Patient Form Component

| Key | Action |
|-----|--------|
| **`Esc`** | Cancel form and discard changes |
| **`Enter`** | Submit form |
| **`F10`** | Submit form (alternative) |
| **`Tab`** | Move to next field |
| **`Shift+Tab`** | Move to previous field |
| **`↑`** / **`↓`** | Navigate between fields |
| **Character keys** | Input text in active field |
| **`Backspace`** | Delete character in text field |

### Appointment Form Component

| Key | Action |
|-----|--------|
| **`Esc`** | Cancel form and discard changes |
| **`Ctrl+S`** | Submit form |
| **`Tab`** | Move to next field |
| **`Enter`** | Confirm selection (context-dependent) |
| **`↑`** / **`↓`** | Navigate dropdown options |
| **Character keys** | Input text in active field |
| **`Backspace`** | Delete character in text field |

---

## Conflict Resolution

### Key Conflict Resolutions

#### 1. 'n' Key Conflict
- **Issue**: Originally used for both "New appointment" AND "Mark no-show"
- **Resolution**: Changed no-show to **`x`** (more mnemonic for "cancel/reject")
- **Rationale**: `x` is commonly used for "delete/cancel" operations and avoids conflict with the universal "New" action

#### 2. 'g'/'G' Standardization
- **Issue**: Appointment list lacked jump-to-first/last functionality
- **Resolution**: Added **`g`**/**`G`** to appointment list to match patient list behavior
- **Rationale**: Consistent navigation across all list views

#### 3. Tab Dual-Purpose
- **Issue**: Tab used for both form navigation and view switching
- **Resolution**: Kept dual usage due to different contexts
- **Rationale**: Forms and calendar views have fundamentally different interaction patterns, making context-dependent Tab usage acceptable

### Case Sensitivity Policy

Most action keys are case-insensitive to improve usability:
- **`a`**/**`A`**: Mark as Arrived
- **`i`**/**`I`**: Mark as In Progress
- **`c`**/**`C`**: Mark as Completed  
- **`x`**/**`X`**: Mark as No-show
- **`r`**/**`R`**: Reschedule

This reduces cognitive load and prevents errors from accidental Caps Lock.

---

## Tips for New Users

### Getting Started

1. **Learn the basics first**: Master `j`/`k` navigation and `q` to quit
2. **Use mnemonics**: Remember that keys relate to their actions
3. **Practice context switching**: Different screens have different keybinds
4. **Start with global keys**: `1-4` for navigation between main screens

### Efficiency Tips

1. **Use `g`/`G` for quick jumps**: Instantly go to first/last items
2. **Learn search**: `/` to search in lists, `Esc` to clear
3. **Master modals**: `Esc` almost always closes/cancels
4. **Use undo**: `Ctrl+Z` to undo appointment status changes

### Common Workflows

#### Quick Patient Lookup
1. Press `1` to go to Patients screen
2. Press `/` to enter search mode
3. Type patient name
4. Press `Enter` to select
5. Press `Esc` to clear search when done

#### Appointment Management
1. Press `2` to go to Appointments screen
2. Use `h`/`l` to navigate months
3. Press `Enter` to switch to day view
4. Use `j`/`k` to find the appointment
5. Press `a`/`c`/`x` to update status

#### Creating Records
1. Navigate to appropriate screen (`1-4`)
2. Press `n` to create new record
3. Fill form using `Tab` to navigate fields
4. Press `Enter` or `Ctrl+S` to submit
5. Press `Esc` to cancel if needed

#### Appointment Status Workflow
1. Press `2` to go to Appointments screen
2. Select an appointment in day view
3. Press `a` to mark as Arrived (when patient arrives)
4. Press `i` to mark as In Progress (when consultation starts)
5. Press `c` to mark as Completed (when finished)
6. Or press `x` if patient doesn't show

### Troubleshooting

- **Stuck in a mode?** Press `Esc` to exit most modes/modals
- **Want to quit?** Press `q` or `Ctrl+C` from anywhere
- **Lost your place?** Press `g` to go to first item, `G` for last
- **Accidental change?** Use `Ctrl+Z` to undo appointment status changes

---

*This document covers all keybinds as of OpenGP v1.0. For the most current information, check the application source code or run `opengp --help-keybinds` if available.*