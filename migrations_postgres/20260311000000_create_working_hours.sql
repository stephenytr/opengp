CREATE TABLE IF NOT EXISTS working_hours (
    id UUID PRIMARY KEY,
    practitioner_id UUID NOT NULL,
    day_of_week INTEGER NOT NULL CHECK(day_of_week BETWEEN 0 AND 6),
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (practitioner_id) REFERENCES users(id),
    UNIQUE(practitioner_id, day_of_week)
);

CREATE INDEX IF NOT EXISTS idx_working_hours_practitioner ON working_hours(practitioner_id);
CREATE INDEX IF NOT EXISTS idx_working_hours_day ON working_hours(practitioner_id, day_of_week);
CREATE INDEX IF NOT EXISTS idx_working_hours_active ON working_hours(is_active);
