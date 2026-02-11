# Mock Patient Data

## Overview

The patient list now includes 20 diverse mock patients for testing and demonstration.

## Patient Demographics

### Age Distribution
- Children (under 18): 2 patients
- Adults (18-65): 12 patients  
- Seniors (65+): 6 patients

### Gender Distribution
- Male: 11 patients
- Female: 9 patients

### Name Diversity
Mix of common Australian names including:
- Anglo-Saxon: Smith, Johnson, Brown, Williams, Taylor
- Asian: Chen, Nguyen, Lee
- European: Martinez, Garcia, O'Connor
- Various given names: John, Sarah, Michael, Emma, Liam, etc.

## Test Scenarios

### Search Testing

**By First Name:**
- `john` → Smith, John
- `sarah` → Johnson, Sarah
- `michael` → Chen, Michael

**By Last Name:**
- `smith` → Smith, John
- `chen` → Chen, Michael
- `nguyen` → Nguyen, Ava

**By Preferred Name:**
- `sally` → Johnson, Sarah (Sally)
- `jim` → Brown, James (Jim)
- `bill` → Taylor, William (Bill)
- `bella` → Wilson, Isabella (Bella)
- `ben` → Martin, Benjamin (Ben)
- `amy` → Thompson, Amelia (Amy)
- `charlie` → Harris, Charlotte (Charlie)
- `liv` → Martinez, Olivia (Liv)

**By Medicare Number:**
- `2123456781` → Smith, John
- `3234567892` → Johnson, Sarah
- `4345678903` → Chen, Michael

### Navigation Testing

**Scroll Testing:**
- Use `j` 20 times to scroll through all patients
- Use `k` to scroll back up
- Use `g` to jump to first (Smith)
- Use `G` to jump to last (Robinson)

**Pagination:**
- More than fits on one screen (tests scrolling)
- TableState handles selection across all rows

## Patient List

| # | Name | Preferred | Age | Gender | Medicare |
|---|------|-----------|-----|--------|----------|
| 1 | Smith, John | - | 45 | M | 2123456781-1 |
| 2 | Johnson, Sarah | Sally | 33 | F | 3234567892-1 |
| 3 | Chen, Michael | - | 50 | M | 4345678903-2 |
| 4 | Williams, Emma | - | 60 | F | 5456789014-1 |
| 5 | Brown, James | Jim | 67 | M | 6567890125-3 |
| 6 | Martinez, Olivia | Liv | 30 | F | 7678901236-1 |
| 7 | Taylor, William | Bill | 53 | M | 8789012347-2 |
| 8 | Anderson, Sophia | - | 37 | F | 9890123458-1 |
| 9 | O'Connor, Liam | - | 15 | M | 1901234569-4 |
| 10 | Nguyen, Ava | - | 27 | F | 2012345670-1 |
| 11 | Davis, Noah | - | 63 | M | 3123456781-2 |
| 12 | Wilson, Isabella | Bella | 55 | F | 4234567892-1 |
| 13 | Moore, Mason | - | 40 | M | 5345678903-3 |
| 14 | Lee, Mia | - | 20 | F | 6456789014-1 |
| 15 | White, Ethan | - | 34 | M | 7567890125-2 |
| 16 | Harris, Charlotte | Charlie | 70 | F | 8678901236-1 |
| 17 | Martin, Benjamin | Ben | 10 | M | 9789012347-4 |
| 18 | Thompson, Amelia | Amy | 32 | F | 1890123458-1 |
| 19 | Garcia, Lucas | - | 43 | M | 2901234569-2 |
| 20 | Robinson, Harper | - | 57 | F | 3012345670-1 |

## Data Characteristics

### Realistic Australian Data
- Valid IHI format (16 digits starting with 800360816669)
- Valid Medicare numbers (10 digits)
- Medicare IRN (1-9)
- Australian phone formats (02/03/07/08 landlines, 04 mobiles)
- Realistic age distribution

### Testing Coverage
- **Children**: Liam O'Connor (15), Benjamin Martin (10)
- **Young Adults**: Ava Nguyen (27), Mia Lee (20), Olivia Martinez (30)
- **Middle Age**: Wide variety
- **Seniors**: James Brown (67), Charlotte Harris (70), Noah Davis (63)

### Phone Number Patterns
- Landline area codes: 02 (NSW), 03 (VIC), 07 (QLD), 08 (SA/WA)
- Mobile: All start with 04
- Mix of patients with/without home phones
- Some patients mobile-only (younger demographic)

## Performance

With 20 patients, you can test:
- Scrolling performance
- Search responsiveness  
- Table rendering speed
- Selection state management

All operations should be instant at this scale.

## Future Expansion

To add more patients, simply extend the `generate_mock_patients()` function in:
`src/components/patient/list.rs`

For realistic large-scale testing, consider:
- Generating 100-1000 patients programmatically
- Loading from CSV file
- Connecting to real database with seed data
