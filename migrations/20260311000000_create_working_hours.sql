-- Working hours table for OpenGP
-- Stores practitioner working hours schedules for each day of the week

CREATE TABLE IF NOT EXISTS working_hours (
    id UUID PRIMARY KEY,
    practitioner_id UUID NOT NULL,

    -- Day of week (0 = Monday, 6 = Sunday)
    day_of_week INTEGER NOT NULL CHECK(day_of_week BETWEEN 0 AND 6),

    -- Start time stored as TEXT (HH:MM:SS format)
    start_time TEXT NOT NULL,

    -- End time stored as TEXT (HH:MM:SS format)
    end_time TEXT NOT NULL,

    -- Is this working hours entry active?
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Audit fields
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (practitioner_id) REFERENCES users(id),
    UNIQUE(practitioner_id, day_of_week)
);

-- Indexes for common queries
CREATE INDEX idx_working_hours_practitioner ON working_hours(practitioner_id);
CREATE INDEX idx_working_hours_day ON working_hours(practitioner_id, day_of_week);
CREATE INDEX idx_working_hours_active ON working_hours(is_active);
