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

⏳ **Patient Data**
- Mock data has been removed
- Proper patient generation logic to be implemented
- Will use database or programmatic generation

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

## Patient Data

Mock patients have been removed from the codebase. The patient list now loads from the database via the PatientRepository.

To add test patients:
- Use the "New Patient" form (press `n`)
- Implement database seed data
- Use a test data generation script

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
