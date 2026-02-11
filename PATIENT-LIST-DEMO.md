# Patient List Component - Demo Guide

## Overview

The Patient List component displays a table of patients with search and navigation capabilities.

## Features Implemented

✅ **Patient Table Display**
- Name (Last, First/Preferred)
- Date of Birth (DD/MM/YYYY format)
- Age (calculated)
- Medicare Number (with IRN)
- Phone Number (mobile or home)

✅ **Keyboard Navigation**
- `j` or `Down Arrow` - Move selection down
- `k` or `Up Arrow` - Move selection up
- `g` - Jump to first patient
- `G` (Shift+g) - Jump to last patient
- `Enter` - Select patient (placeholder)

✅ **Mock Data**
- 3 sample patients pre-loaded
- Realistic Australian healthcare data
- Medicare numbers, IHI, contact details

## How to Run

```bash
./run-dev.sh
```

Then press `1` to navigate to the Patients screen.

## Controls

| Key | Action |
|-----|--------|
| `j` or `↓` | Next patient |
| `k` or `↑` | Previous patient |
| `g` | First patient |
| `G` | Last patient |
| `Enter` | Select patient |
| `n` | New patient (not implemented) |
| `/` | Search (not implemented) |
| `1-4` | Switch screens |
| `q` | Quit |

## Mock Patients

1. **John David Smith**
   - DOB: 15/05/1980 (45 years old)
   - Medicare: 2123456781-1
   - Phone: 0412 345 678

2. **Sarah Johnson** (Sally)
   - DOB: 22/08/1992 (33 years old)
   - Medicare: 3234567892-1
   - Phone: 0423 456 789

3. **Michael James Chen**
   - DOB: 10/12/1975 (50 years old)
   - Medicare: 4345678903-2
   - Phone: 0434 567 890

## Architecture

```
PatientListComponent (UI)
    ↓
Patient (Domain Model)
    ↓
PatientRepository (Data Access)
    ↓
Database
```

The component follows the layered architecture:
- **UI Layer**: Ratatui table widget with stateful selection
- **Domain Layer**: Patient model with validation
- **Data Layer**: Repository pattern (mock data for now)

## What's Working

✅ Table rendering with proper column layout  
✅ Keyboard navigation (j/k/g/G)  
✅ Selection highlighting  
✅ Mock data loading  
✅ Age calculation  
✅ Medicare number formatting  

## What's Not Implemented

⏳ Search functionality  
⏳ Pagination for large datasets  
⏳ Patient details view (on Enter)  
⏳ Create new patient  
⏳ Edit patient  
⏳ Real database integration  
⏳ Loading states  
⏳ Error handling UI  

## Next Steps

To continue development:

1. **Add Search**: Implement `/` key to filter patients
2. **Patient Details**: Show detailed view on Enter
3. **Create Patient**: Form for new patient entry
4. **Database Integration**: Connect to SQLite via repository
5. **Pagination**: Handle 100+ patients efficiently

## Code Location

- Component: `src/components/patient/list.rs`
- Domain Model: `src/domain/patient/model.rs`
- Repository: `src/domain/patient/repository.rs`
- Wire-up: `src/app.rs` (init_components)
