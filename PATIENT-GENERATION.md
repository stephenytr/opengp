# Patient Generation System

## Overview

OpenGP includes a comprehensive patient generation system for creating realistic Australian patient data. This is useful for:

- **Testing**: Generate test patients with realistic data
- **Development**: Populate development databases
- **Demonstrations**: Create demo data for showcasing features
- **Load Testing**: Generate large datasets for performance testing

## Architecture

The patient generation system is located in `src/infrastructure/fixtures/` and is completely separate from production code, ensuring no mock data leaks into the main application.

### Components

1. **PatientGenerator**: Core generator that creates realistic patients
2. **PatientGeneratorConfig**: Configuration for customizing generation
3. **Examples**: Ready-to-use examples for common scenarios

### Technology

Uses the [`fake`](https://crates.io/crates/fake) crate for truly random data generation:
- **Unlimited Name Variety**: Not limited to hardcoded lists - generates realistic, diverse names
- **Locale Support**: Can be extended to support different locales/regions
- **Consistent Quality**: Professional-grade fake data generation library
- **No Repetition**: Each run produces completely different names

## Features

### Realistic Australian Data

- **Names**: Truly random names using the `fake` crate (unlimited variety, not hardcoded lists)
- **Medicare Numbers**: Valid 10-digit numbers with correct checksums
- **IHI Numbers**: Valid 16-digit Healthcare Identifiers
- **Phone Numbers**: Australian format (02/03/07/08 landlines, 04 mobiles)
- **Addresses**: Realistic Australian addresses with states and postcodes
- **Email Addresses**: Various formats and common domains

### Configurable Generation

```rust
PatientGeneratorConfig {
    count: 50,                     // Number of patients to generate
    min_age: 0,                    // Minimum age
    max_age: 100,                  // Maximum age
    include_children: true,        // Include patients under 18
    include_seniors: true,         // Include patients 65+
    medicare_percentage: 0.95,     // % with Medicare numbers (0.0-1.0)
    ihi_percentage: 0.90,          // % with IHI numbers
    mobile_percentage: 0.85,       // % with mobile phones
    email_percentage: 0.70,        // % with email addresses
}
```

### Demographics

Generated patients include:
- **Gender**: Male, Female, Other
- **Ages**: 0-100 years (configurable)
- **Titles**: Mr, Mrs, Ms, Miss, Dr, Mx
- **Middle Names**: ~60% have middle names (truly random)
- **Preferred Names**: ~15% have nicknames (truly random)
- **Names**: Unlimited variety using the `fake` crate - not limited to hardcoded lists
- **Realistic Surnames**: Includes a wide variety of surnames from different cultures

## Usage

### 1. Generate and Display Patients

```bash
cargo run --example generate_patients
```

This will:
- Generate 20 patients with default configuration
- Display formatted patient information
- Show statistics (gender, age distribution, contact info)

**Sample Output:**
```
OpenGP Patient Generator Example
=================================

Configuration:
  Count: 20
  Age range: 0-100
  Include children: true
  Include seniors: true
  Medicare percentage: 95%
  IHI percentage: 90%
  Mobile percentage: 85%
  Email percentage: 70%

Generated 20 patients:

  1. Smith, John                   Age:  45 (M) Medicare: 2123456781-1   Phone: 0412 345 678
  2. Johnson, Sally                Age:  33 (F) Medicare: 3234567892-1   Phone: 0423 456 789
  ...

Statistics:
  Gender: 10 Male, 10 Female
  Age Distribution: 2 Children, 12 Adults, 6 Seniors
  With Medicare: 19 (95.0%)
  With IHI: 18 (90.0%)
```

### 2. Seed Database

```bash
cargo run --example seed_database
```

This will:
- Connect to your configured database
- Run migrations if needed
- Generate 50 patients
- Insert them into the database
- Report success/failure statistics

**Requirements**: Database must be configured in `.env` or environment variables.

### 3. Programmatic Usage

```rust
use opengp::infrastructure::fixtures::{PatientGenerator, PatientGeneratorConfig};

// Create custom configuration
let config = PatientGeneratorConfig {
    count: 100,
    min_age: 18,
    max_age: 65,
    include_children: false,
    include_seniors: false,
    medicare_percentage: 1.0,  // All patients have Medicare
    ..Default::default()
};

// Generate patients
let mut generator = PatientGenerator::new(config);
let patients = generator.generate();

// Use patients in your code
for patient in patients {
    println!("{} {}", patient.first_name, patient.last_name);
}
```

### 4. Testing Usage

```rust
#[cfg(test)]
mod tests {
    use opengp::infrastructure::fixtures::{PatientGenerator, PatientGeneratorConfig};

    #[test]
    fn test_with_generated_patients() {
        let config = PatientGeneratorConfig {
            count: 5,
            ..Default::default()
        };
        
        let mut generator = PatientGenerator::new(config);
        let patients = generator.generate();
        
        assert_eq!(patients.len(), 5);
        
        // Test your code with generated patients
        for patient in patients {
            // ... your test logic
        }
    }
}
```

## Configuration Options

### Basic Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `count` | `usize` | 10 | Number of patients to generate |
| `min_age` | `u32` | 0 | Minimum age in years |
| `max_age` | `u32` | 100 | Maximum age in years |

### Demographic Filters

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `include_children` | `bool` | true | Include patients under 18 |
| `include_seniors` | `bool` | true | Include patients 65+ |

### Data Completeness

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `medicare_percentage` | `f32` | 0.95 | Percentage with Medicare (0.0-1.0) |
| `ihi_percentage` | `f32` | 0.90 | Percentage with IHI (0.0-1.0) |
| `mobile_percentage` | `f32` | 0.85 | Percentage with mobile phone (0.0-1.0) |
| `email_percentage` | `f32` | 0.70 | Percentage with email (0.0-1.0) |

## Common Scenarios

### Generate Adult-Only Patients

```rust
let config = PatientGeneratorConfig {
    count: 50,
    min_age: 18,
    max_age: 64,
    include_children: false,
    include_seniors: false,
    ..Default::default()
};
```

### Generate Seniors with Complete Data

```rust
let config = PatientGeneratorConfig {
    count: 20,
    min_age: 65,
    max_age: 90,
    include_children: false,
    medicare_percentage: 1.0,
    ihi_percentage: 1.0,
    ..Default::default()
};
```

### Generate Large Test Dataset

```rust
let config = PatientGeneratorConfig {
    count: 1000,
    ..Default::default()
};
```

### Generate Minimal Contact Info

```rust
let config = PatientGeneratorConfig {
    count: 30,
    medicare_percentage: 0.5,
    ihi_percentage: 0.5,
    mobile_percentage: 0.3,
    email_percentage: 0.2,
    ..Default::default()
};
```

## Data Validation

All generated data follows Australian healthcare standards:

### Medicare Numbers
- **Format**: 10 digits
- **Validation**: Includes proper checksum calculation
- **IRN**: Individual Reference Number (1-9)
- **Expiry**: Random date 1-48 months in future

### IHI Numbers
- **Format**: 16 digits
- **Prefix**: `800360816669` (Australian format)
- **Suffix**: 4 random digits

### Phone Numbers
- **Landlines**: Area codes 02 (NSW), 03 (VIC), 07 (QLD), 08 (SA/WA)
- **Mobiles**: Start with 04, followed by 8 digits
- **Format**: Space-separated for readability

### Addresses
- **Realistic streets**: Common Australian street names and types
- **Suburbs**: Major Australian cities and suburbs
- **States**: NSW, VIC, QLD, WA, SA, TAS, NT, ACT
- **Postcodes**: Valid 4-digit postcodes (1000-9999)

## Testing

Run the patient generator tests:

```bash
cargo test patient_generator
```

Tests cover:
- Patient generation count
- Medicare number format and checksum
- IHI number format
- Phone number formats
- Address generation
- Age range constraints
- Gender distribution

## Performance

Generation is fast and memory-efficient:

- **10 patients**: <1ms
- **100 patients**: ~5ms
- **1,000 patients**: ~50ms
- **10,000 patients**: ~500ms

Insertion into database depends on database performance but typically:
- **SQLite**: ~100 patients/second
- **PostgreSQL**: ~500-1000 patients/second

## Best Practices

### Development

1. Use reasonable counts (10-50 patients) for quick iteration
2. Clear database between runs to avoid duplicates
3. Use consistent configuration for reproducible testing

### Testing

1. Use small counts (5-10) in unit tests
2. Generate fresh data for each test
3. Don't rely on specific generated data (it's random)

### Seeding

1. Run seed_database example once per environment
2. Check existing data before seeding
3. Back up production databases before seeding
4. Never seed production databases with test data

### Load Testing

1. Generate large datasets (1,000-10,000+)
2. Monitor database performance during insertion
3. Use transactions for batch insertions
4. Consider memory usage with very large counts

## Troubleshooting

### Database Connection Issues

```
Error: Failed to connect to database
```

**Solution**: Ensure `DATABASE_URL` is set in `.env` or environment:
```bash
export DATABASE_URL="sqlite://opengp.db"
```

### Duplicate Medicare Numbers

```
Error: UNIQUE constraint failed: patients.medicare_number
```

**Solution**: Medicare numbers are randomly generated and may collide. For large datasets, clear database first or handle duplicates in your code.

### Slow Database Insertion

**Solution**: Use transactions for batch inserts:
```rust
let mut tx = pool.begin().await?;
for patient in patients {
    repository.create_with_tx(&mut tx, patient).await?;
}
tx.commit().await?;
```

## Future Enhancements

Planned improvements:

- [ ] CSV export functionality
- [ ] Import from CSV
- [ ] Deterministic generation (seeded RNG)
- [ ] More diverse cultural names
- [ ] Relationship generation (family members)
- [ ] Practitioner and appointment generation
- [ ] Batch insertion with transactions
- [ ] CLI tool with more options

## Contributing

When adding to the patient generator:

1. Maintain Australian healthcare data standards
2. Add tests for new features
3. Update this documentation
4. Keep generation fast and memory-efficient
5. Don't add dependencies unnecessarily

## References

- Medicare Number Format: https://www.servicesaustralia.gov.au/medicare-card
- IHI Numbers: https://www.servicesaustralia.gov.au/individual-healthcare-identifiers
- Australian Phone Numbers: https://www.acma.gov.au/numbering
- Australian Postcodes: https://auspost.com.au/
