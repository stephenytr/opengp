CREATE TABLE IF NOT EXISTS appointments (
    id UUID PRIMARY KEY,
    patient_id UUID NOT NULL,
    practitioner_id UUID NOT NULL,

    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,

    appointment_type TEXT NOT NULL CHECK(appointment_type IN (
        'Standard', 'Long', 'Brief', 'NewPatient', 'HealthAssessment',
        'ChronicDiseaseReview', 'MentalHealthPlan', 'Immunisation',
        'Procedure', 'Telephone', 'Telehealth', 'HomeVisit', 'Emergency'
    )),

    status TEXT NOT NULL CHECK(status IN (
        'Scheduled', 'Confirmed', 'Arrived', 'InProgress',
        'Completed', 'NoShow', 'Cancelled', 'Rescheduled'
    )),

    reason TEXT,
    notes TEXT,
    is_urgent BOOLEAN NOT NULL DEFAULT FALSE,
    reminder_sent BOOLEAN NOT NULL DEFAULT FALSE,
    confirmed BOOLEAN NOT NULL DEFAULT FALSE,
    cancellation_reason TEXT,
    version INTEGER NOT NULL DEFAULT 0,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID,
    updated_by UUID,

    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (practitioner_id) REFERENCES users(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (updated_by) REFERENCES users(id),
    CONSTRAINT uq_appointments_practitioner_start UNIQUE (practitioner_id, start_time)
);

CREATE INDEX IF NOT EXISTS idx_appointments_patient ON appointments(patient_id);
CREATE INDEX IF NOT EXISTS idx_appointments_practitioner ON appointments(practitioner_id);
CREATE INDEX IF NOT EXISTS idx_appointments_start_time ON appointments(start_time);
CREATE INDEX IF NOT EXISTS idx_appointments_status ON appointments(status);
CREATE INDEX IF NOT EXISTS idx_appointments_patient_start ON appointments(patient_id, start_time);
