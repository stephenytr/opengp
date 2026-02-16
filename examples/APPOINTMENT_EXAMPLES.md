# Appointment Generator Example

Generate realistic appointment schedules using your existing patients and practitioners.

## Prerequisites

You need:
1. A running OpenGP database with patients and practitioners
2. `DATABASE_URL` environment variable set (or `.env` file)

### Quick Setup

```bash
# If you don't have patients yet:
export DATABASE_URL="sqlite://opengp.db"
cargo run --example seed_database  # Creates patients

# The database should already have practitioners from migrations
```

## Usage

```bash
cargo run --example generate_appointments_with_db
```

**What it does:**
- Connects to your database
- Fetches all active patients and practitioners
- Generates a 60% filled schedule for the next 7 days
- Shows statistics and sample appointments
- Prompts to save appointments to database

**Output:**
```
Found 1012 active patients
Found 5 active practitioners

==========================================
Configuration:
==========================================
  Fill rate: 60%
  Date range: 2026-02-15 to 2026-02-22 (7 days)
  Business hours: 9:00 - 17:00
  Slot duration: 15 minutes
  Using 1012 patients from database
  Using 5 practitioners from database

Generated 96 appointments

Schedule Statistics:
  Total slots: 160
  Filled: 96 (60.0%)
  Available: 64

  By Status:
    Scheduled: 61
    Confirmed: 35

Sample appointments (first 10):
  Date/Time            | Patient              | Type            | Status
  2026-02-16 09:15     | Tavares Kerluke      | Telephone       | Scheduled
                       | Practitioner: Dr. Johnson

Save appointments to database? (y/n): y
  Saved 96 appointments successfully!
```

## Configuration Options

### Fill Rate

Change `fill_rate` in the example (0.0 to 1.0):
- `0.20` = 20% filled (mostly empty schedule)
- `0.60` = 60% filled (moderately busy)
- `1.00` = 100% filled (fully booked)

### Date Range

Modify `start_date` and `end_date`:
```rust
start_date: Some(Utc::now()),                                    // Today
end_date: Some(Utc::now() + Duration::days(14)),                // 2 weeks
```

### Business Hours

Change the operating hours:
```rust
business_hours_start: 8,    // 8am
business_hours_end: 18,     // 6pm
```

### Slot Duration

Change appointment length:
```rust
slot_duration_minutes: 30,  // 30-minute slots
```

### Weekends

Include or exclude weekends:
```rust
exclude_weekends: true,   // Skip Saturday/Sunday
exclude_weekends: false,  // Include weekends
```

### Lunch Hour

Skip lunch hour (12pm-1pm):
```rust
exclude_lunch_hour: true,
```

## Customizing

Edit `examples/generate_appointments_with_db.rs` to change the configuration:

```rust
let fixture_config = AppointmentGeneratorConfig {
    fill_rate: 0.75,                    // 75% filled
    start_date: Some(Utc::now()),
    end_date: Some(Utc::now() + Duration::days(30)),  // 30 days
    patient_ids: Some(patient_ids),
    practitioner_ids: Some(practitioner_ids),
    slot_duration_minutes: 30,          // 30-minute appointments
    business_hours_start: 8,            // 8am start
    business_hours_end: 17,             // 5pm end
    exclude_weekends: true,
    exclude_lunch_hour: true,           // Skip lunch
    ..Default::default()
};
```

## Programmatic Usage

Use in your own code:

```rust
use opengp::infrastructure::fixtures::{
    AppointmentGenerator, AppointmentGeneratorConfig, GenerationStats
};
use chrono::{Duration, Utc};
use uuid::Uuid;

// Fetch your existing patients and practitioners from database
let patient_ids: Vec<Uuid> = vec![/* from database */];
let practitioner_ids: Vec<Uuid> = vec![/* from database */];

let config = AppointmentGeneratorConfig {
    fill_rate: 0.60,
    start_date: Some(Utc::now()),
    end_date: Some(Utc::now() + Duration::days(7)),
    patient_ids: Some(patient_ids),
    practitioner_ids: Some(practitioner_ids),
    ..Default::default()
};

let mut generator = AppointmentGenerator::new(config);
let (appointments, stats) = generator.generate_schedule();

println!("Generated {} appointments ({}% filled)", 
    stats.filled_slots, 
    stats.actual_fill_rate * 100.0
);

// Save to database
for appointment in appointments {
    appointment_repo.create(appointment).await?;
}
```

## Troubleshooting

**Error: "No patients found in database!"**

Run the patient seeder first:
```bash
cargo run --example seed_database
```

**Error: "Failed to connect to database"**

Set your DATABASE_URL:
```bash
export DATABASE_URL="sqlite://opengp.db"
# or for PostgreSQL:
export DATABASE_URL="postgres://user:pass@localhost/opengp"
```

**Error: "UNIQUE constraint failed"**

Some appointments may overlap. The generator tries to avoid this, but if you're saving to an existing schedule with appointments, you may get conflicts. Clear existing appointments first or adjust the date range.

## More Information

- [Full Documentation](../APPOINTMENT-GENERATION.md)
- [Architecture Guide](../ARCHITECTURE.md)
- [Database Schema](../migrations/)
