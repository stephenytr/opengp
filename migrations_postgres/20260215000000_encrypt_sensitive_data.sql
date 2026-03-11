CREATE TABLE IF NOT EXISTS consultations (
    id UUID PRIMARY KEY,
    patient_id UUID NOT NULL,
    practitioner_id UUID NOT NULL,
    appointment_id UUID,
    consultation_date TIMESTAMPTZ NOT NULL,
    soap_subjective BYTEA,
    soap_objective BYTEA,
    soap_assessment BYTEA,
    soap_plan BYTEA,
    is_signed BOOLEAN NOT NULL DEFAULT FALSE,
    signed_at TIMESTAMPTZ,
    signed_by UUID,
    version INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NOT NULL,
    updated_by UUID,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (practitioner_id) REFERENCES users(id),
    FOREIGN KEY (appointment_id) REFERENCES appointments(id),
    FOREIGN KEY (signed_by) REFERENCES users(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (updated_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_consultations_patient ON consultations(patient_id);
CREATE INDEX IF NOT EXISTS idx_consultations_date ON consultations(consultation_date);
CREATE INDEX IF NOT EXISTS idx_consultations_practitioner ON consultations(practitioner_id);

CREATE TABLE IF NOT EXISTS social_history (
    id UUID PRIMARY KEY,
    patient_id UUID NOT NULL,
    smoking_status TEXT,
    cigarettes_per_day INTEGER,
    smoking_quit_date DATE,
    alcohol_status TEXT,
    standard_drinks_per_week INTEGER,
    exercise_frequency TEXT,
    occupation TEXT,
    living_situation TEXT,
    support_network TEXT,
    notes BYTEA,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_by UUID NOT NULL,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (updated_by) REFERENCES users(id),
    UNIQUE(patient_id)
);

CREATE INDEX IF NOT EXISTS idx_social_history_patient ON social_history(patient_id);
