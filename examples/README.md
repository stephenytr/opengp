# Patient Generation Examples

This directory contains examples for generating realistic patient data in OpenGP.

## Quick Start

### Generate and Display Patients

Generate 20 patients and display their information:

```bash
cargo run --example generate_patients
```

### Seed Database

Populate your database with 50 generated patients:

```bash
cargo run --example seed_database
```

**Note**: Requires `DATABASE_URL` to be set in your environment or `.env` file.

## What Gets Generated

Each patient includes:

- **Personal Details**: Name (first, middle, last), preferred name, title, gender, date of birth
- **Healthcare IDs**: Medicare number with IRN and expiry, IHI number  
- **Contact Info**: Mobile phone, home phone, email address
- **Address**: Full Australian address with street, suburb, state, postcode
- **Demographics**: Age ranges from newborns to seniors (100+)

## Example Output

```
Generated 20 patients:

  1. Smith, John                   Age:  45 (M) Medicare: 2123456781-1   Phone: 0412 345 678
  2. Johnson, Sarah                Age:  33 (F) Medicare: 3234567892-1   Phone: 0423 456 789
  3. Chen, Michael                 Age:  50 (M) Medicare: 4345678903-2   Phone: 0434 567 890
  ...

Statistics:
  Gender: 10 Male, 10 Female
  Age Distribution: 2 Children, 12 Adults, 6 Seniors
  With Medicare: 19 (95.0%)
  With IHI: 18 (90.0%)
```

## Configuration

See [PATIENT-GENERATION.md](../PATIENT-GENERATION.md) for detailed configuration options and programmatic usage.

## Troubleshooting

**Error: "Failed to connect to database"**

Solution: Set your `DATABASE_URL`:
```bash
export DATABASE_URL="postgres://postgres:postgres@localhost/opengp"
```

**Error: "UNIQUE constraint failed"**

Solution: Medicare numbers are randomly generated and may occasionally collide. Clear your database or handle duplicates in your application logic.

## More Information

- [Full Documentation](../PATIENT-GENERATION.md)
- [Architecture Guide](../ARCHITECTURE.md)
- [Requirements](../REQUIREMENTS.md)
