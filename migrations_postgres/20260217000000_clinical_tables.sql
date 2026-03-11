CREATE TABLE IF NOT EXISTS allergies (
    id UUID PRIMARY KEY,
    patient_id UUID NOT NULL,
    allergen TEXT NOT NULL,
    allergy_type TEXT NOT NULL CHECK(allergy_type IN ('Drug', 'Food', 'Environmental', 'Other')),
    severity TEXT NOT NULL CHECK(severity IN ('Mild', 'Moderate', 'Severe')),
    reaction BYTEA,
    onset_date DATE,
    notes BYTEA,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NOT NULL,
    updated_by UUID,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (updated_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_allergies_patient ON allergies(patient_id);
CREATE INDEX IF NOT EXISTS idx_allergies_active ON allergies(patient_id, is_active) WHERE is_active = TRUE;

CREATE TABLE IF NOT EXISTS medical_history (
    id UUID PRIMARY KEY,
    patient_id UUID NOT NULL,
    condition TEXT NOT NULL,
    diagnosis_date DATE,
    status TEXT NOT NULL CHECK(status IN ('Active', 'Resolved', 'Chronic', 'Recurring', 'InRemission')),
    severity TEXT CHECK(severity IN ('Mild', 'Moderate', 'Severe')),
    notes BYTEA,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NOT NULL,
    updated_by UUID,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (updated_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_medical_history_patient ON medical_history(patient_id);
CREATE INDEX IF NOT EXISTS idx_medical_history_active ON medical_history(patient_id, is_active) WHERE is_active = TRUE;

CREATE TABLE IF NOT EXISTS vital_signs (
    id UUID PRIMARY KEY,
    patient_id UUID NOT NULL,
    consultation_id UUID,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    systolic_bp INTEGER,
    diastolic_bp INTEGER,
    heart_rate INTEGER,
    respiratory_rate INTEGER,
    temperature DOUBLE PRECISION,
    oxygen_saturation INTEGER,
    height_cm INTEGER,
    weight_kg DOUBLE PRECISION,
    bmi DOUBLE PRECISION,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NOT NULL,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (consultation_id) REFERENCES consultations(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_vital_signs_patient ON vital_signs(patient_id);
CREATE INDEX IF NOT EXISTS idx_vital_signs_measured_at ON vital_signs(measured_at);

CREATE TABLE IF NOT EXISTS family_history (
    id UUID PRIMARY KEY,
    patient_id UUID NOT NULL,
    relative_relationship TEXT NOT NULL,
    condition TEXT NOT NULL,
    age_at_diagnosis INTEGER,
    notes BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NOT NULL,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_family_history_patient ON family_history(patient_id);
