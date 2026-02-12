-- Seed mock users for testing appointments
-- These match the hardcoded practitioners in AppointmentFormComponent

-- Insert mock users (practitioners)
INSERT OR IGNORE INTO users (id, username, password_hash, role, is_active, created_at, updated_at)
VALUES 
    -- Dr Sarah Johnson
    (
        x'a1b2c3d4e5f64789a1b2c3d4e5f64789',
        's.johnson',
        '$2b$12$dummy.hash.for.testing.purposes.only.no.real.auth',
        'Doctor',
        TRUE,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    -- Dr Michael Chen
    (
        x'b2c3d4e5f6a789a1b2c3d4e5f6a789a1',
        'm.chen',
        '$2b$12$dummy.hash.for.testing.purposes.only.no.real.auth',
        'Doctor',
        TRUE,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    -- Dr Emily Williams
    (
        x'c3d4e5f6a789a1b2c3d4e5f6a789a1b2',
        'e.williams',
        '$2b$12$dummy.hash.for.testing.purposes.only.no.real.auth',
        'Doctor',
        TRUE,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    );
