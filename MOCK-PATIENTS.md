# Patient Data Generation

## Overview

OpenGP includes a comprehensive patient generation system for creating realistic test data. Mock patients have been removed from production code and replaced with a flexible generation system.

**See [PATIENT-GENERATION.md](./PATIENT-GENERATION.md) for full documentation.**

## Next Steps

The following patient generation approaches are planned:

### 1. Database Seed Data
- Create SQL migration with initial patient data
- Use realistic Australian names and healthcare identifiers
- Include diverse demographics (age, gender, ethnicity)

### 2. Test Data Generator
- Programmatic patient generation for testing
- Configurable parameters (count, demographics)
- Valid Australian healthcare identifiers (IHI, Medicare)

### 3. CSV Import
- Support for bulk patient import
- Validation of imported data
- Error reporting for invalid entries

## Requirements for Patient Generation

When implementing patient generation logic, ensure:

### Data Quality
- Valid IHI format (16 digits)
- Valid Medicare numbers (10 digits with Luhn checksum)
- Medicare IRN (1-9)
- Australian phone formats (02/03/07/08 landlines, 04 mobiles)
- Realistic age distribution

### Demographics Coverage
- Children (under 18)
- Adults (18-65)
- Seniors (65+)
- Gender diversity
- Name diversity (Anglo-Saxon, Asian, European, etc.)

### Test Scenarios
Patient data should support testing:
- Search by first name, last name, preferred name
- Search by Medicare number
- Navigation and scrolling
- Pagination for large datasets
- Selection state management

## Previous Mock Data (Removed)

The previous implementation included 20 hardcoded mock patients in:
- `src/components/patient/list.rs::generate_mock_patients()` (removed)
- `src/components/appointment/form.rs::generate_mock_patients()` (removed)

These have been removed to make way for proper patient generation logic.
