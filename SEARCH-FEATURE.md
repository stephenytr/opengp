# Search Functionality - Implementation Guide

## Overview

The Patient List now includes real-time search/filter functionality with live input.

## Features Implemented

✅ **Live Search**
- Press `/` to enter search mode
- Type to filter patients in real-time
- Search across name, preferred name, and Medicare number
- Case-insensitive matching

✅ **Search UI**
- Yellow search bar when in input mode (with cursor █)
- Green filter bar when filter is applied
- Results counter in table title
- Clear visual feedback

✅ **Search Controls**
- `/` - Enter search mode
- Type characters - Add to search query
- Backspace - Remove last character
- Enter or Esc - Exit search mode (keep filter)
- Esc (when not in search mode) - Clear filter

## How to Use

1. Run the application: `./run-dev.sh`
2. Press `1` to go to Patients screen
3. Press `/` to start searching
4. Type patient name or Medicare number
5. Press Enter to exit search mode (filter remains)
6. Press Esc to clear the filter

## Search Examples

**Search by first name:**
```
Press: /
Type: john
Result: Shows "Smith, John"
```

**Search by last name:**
```
Press: /
Type: johnson
Result: Shows "Johnson, Sally"
```

**Search by Medicare number:**
```
Press: /
Type: 2123456781
Result: Shows patient with matching Medicare number
```

**Search by preferred name:**
```
Press: /
Type: sally
Result: Shows "Johnson, Sally"
```

## UI States

### 1. Normal Mode
```
┌─ Patient List (j/k:navigate g/G:first/last /:search Esc:clear) ─┐
│ Name              │ DOB        │ Age │ Medicare      │ Phone     │
│>> Smith, John     │ 15/05/1980 │ 45  │ 2123456781-1  │ 0412 ... │
└───────────────────────────────────────────────────────────────────┘
```

### 2. Search Mode (Active Input)
```
┌─ Search ──────────────────────────────────────────────────────────┐
│ Search: john█                                                      │
└───────────────────────────────────────────────────────────────────┘
┌─ Patient List - 1 results (Esc:clear) ────────────────────────────┐
│ Name              │ DOB        │ Age │ Medicare      │ Phone     │
│>> Smith, John     │ 15/05/1980 │ 45  │ 2123456781-1  │ 0412 ... │
└───────────────────────────────────────────────────────────────────┘
```

### 3. Filter Applied (Not in Search Mode)
```
┌─ Search ──────────────────────────────────────────────────────────┐
│ Filter: john (/ to edit, Esc to clear)                           │
└───────────────────────────────────────────────────────────────────┘
┌─ Patient List - 1 results (Esc:clear) ────────────────────────────┐
│ Name              │ DOB        │ Age │ Medicare      │ Phone     │
│>> Smith, John     │ 15/05/1980 │ 45  │ 2123456781-1  │ 0412 ... │
└───────────────────────────────────────────────────────────────────┘
```

## Implementation Details

### Architecture

```rust
pub struct PatientListComponent {
    all_patients: Vec<Patient>,        // Complete dataset
    filtered_patients: Vec<Patient>,   // Filtered results
    search_query: String,               // Current search text
    search_mode: bool,                  // Input mode active?
}
```

### Search Algorithm

1. User types character → add to query
2. Query converted to lowercase
3. Filter patients where:
   - Full name contains query, OR
   - Preferred name contains query, OR
   - Medicare number contains query
4. Update filtered_patients list
5. Reset selection to first result
6. Re-render

### Performance

- **Current**: Linear search O(n) - fine for <10,000 patients
- **Future**: For larger datasets, consider:
  - Debouncing (wait 100ms after typing stops)
  - Full-text search indexes
  - Pagination with server-side filtering

## Keyboard Shortcuts Summary

| Key | Action |
|-----|--------|
| `/` | Enter search mode |
| `a-z, 0-9` | Add character to search |
| `Backspace` | Remove last character |
| `Enter` | Exit search mode (keep filter) |
| `Esc` (in search) | Exit search mode (keep filter) |
| `Esc` (not in search) | Clear filter |
| `j/k` | Navigate filtered results |

## Code Location

- Component: `src/components/patient/list.rs`
- Search methods:
  - `apply_search_filter()` - Core filtering logic
  - `enter_search_mode()` - Activate input mode
  - `exit_search_mode()` - Deactivate input mode
  - `handle_search_input()` - Process keystrokes
  - `render_search_bar_static()` - Render search UI

## Future Enhancements

⏳ Advanced search features:
- Multiple criteria (name AND age range)
- Date range filtering (DOB)
- Search by phone number
- Regular expression support
- Search history (up/down arrows)
- Fuzzy matching (typo tolerance)
