-- Migration: Add clinical tables for allergies, medical history, vital signs, and family history
-- Issue #53: Clinical Tab Phase 2 - Repository Layer Implementation

-- Allergies table with encrypted sensitive fields
CREATE TABLE IF NOT EXISTS allergies (
    id BLOB PRIMARY KEY,
    patient_id BLOB NOT NULL,
    allergen TEXT NOT NULL,
    allergy_type TEXT NOT NULL CHECK(allergy_type IN ('Drug', 'Food', 'Environmental', 'Other')),
    severity TEXT NOT NULL CHECK(severity IN ('Mild', 'Moderate', 'Severe')),
    reaction BLOB,
    onset_date DATE,
    notes BLOB,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BLOB NOT NULL,
    updated_by BLOB,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (updated_by) REFERENCES users(id)
);

CREATE INDEX idx_allergies_patient ON allergies(patient_id);
CREATE INDEX idx_allergies_active ON allergies(patient_id, is_active) WHERE is_active = TRUE;

-- Medical history table with encrypted notes
CREATE TABLE IF NOT EXISTS medical_history (
    id BLOB PRIMARY KEY,
    patient_id BLOB NOT NULL,
    condition TEXT NOT NULL,
    diagnosis_date DATE,
    status TEXT NOT NULL CHECK(status IN ('Active', 'Resolved', 'Chronic', 'Recurring', 'InRemission')),
    severity TEXT CHECK(severity IN ('Mild', 'Moderate', 'Severe')),
    notes BLOB,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BLOB NOT NULL,
    updated_by BLOB,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (updated_by) REFERENCES users(id)
);

CREATE INDEX idx_medical_history_patient ON medical_history(patient_id);
CREATE INDEX idx_medical_history_active ON medical_history(patient_id, is_active) WHERE is_active = TRUE;

-- Vital signs table (numeric data, no encryption needed)
CREATE TABLE IF NOT EXISTS vital_signs (
    id BLOB PRIMARY KEY,
    patient_id BLOB NOT NULL,
    consultation_id BLOB,
    measured_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    systolic_bp INTEGER,
    diastolic_bp INTEGER,
    heart_rate INTEGER,
    respiratory_rate INTEGER,
    temperature REAL,
    oxygen_saturation INTEGER,
    height_cm INTEGER,
    weight_kg REAL,
    bmi REAL,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BLOB NOT NULL,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (consultation_id) REFERENCES consultations(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX idx_vital_signs_patient ON vital_signs(patient_id);
CREATE INDEX idx_vital_signs_measured_at ON vital_signs(measured_at);

-- Family history table with encrypted notes
CREATE TABLE IF NOT EXISTS family_history (
    id BLOB PRIMARY KEY,
    patient_id BLOB NOT NULL,
    relative_relationship TEXT NOT NULL,
    condition TEXT NOT NULL,
    age_at_diagnosis INTEGER,
    notes BLOB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BLOB NOT NULL,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX idx_family_history_patient ON family_history(patient_id);
