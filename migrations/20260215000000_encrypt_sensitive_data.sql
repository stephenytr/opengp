-- Migration: Add encrypted storage for sensitive patient data
-- Issue #3: Encrypt clinical notes, social history, and patient identifiers

-- Consultations table with encrypted SOAP notes
CREATE TABLE IF NOT EXISTS consultations (
    id BLOB PRIMARY KEY,
    patient_id BLOB NOT NULL,
    practitioner_id BLOB NOT NULL,
    appointment_id BLOB,
    consultation_date TIMESTAMP WITH TIME ZONE NOT NULL,
    
    -- Encrypted SOAP notes (BLOB for encrypted bytes)
    soap_subjective BLOB,
    soap_objective BLOB,
    soap_assessment BLOB,
    soap_plan BLOB,
    
    is_signed BOOLEAN NOT NULL DEFAULT FALSE,
    signed_at TIMESTAMP WITH TIME ZONE,
    signed_by BLOB,
    
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BLOB NOT NULL,
    updated_by BLOB,
    
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (practitioner_id) REFERENCES users(id),
    FOREIGN KEY (appointment_id) REFERENCES appointments(id),
    FOREIGN KEY (signed_by) REFERENCES users(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (updated_by) REFERENCES users(id)
);

CREATE INDEX idx_consultations_patient ON consultations(patient_id);
CREATE INDEX idx_consultations_date ON consultations(consultation_date);
CREATE INDEX idx_consultations_practitioner ON consultations(practitioner_id);

-- Social history table with encrypted notes
CREATE TABLE IF NOT EXISTS social_history (
    id BLOB PRIMARY KEY,
    patient_id BLOB NOT NULL,
    
    smoking_status TEXT,
    cigarettes_per_day INTEGER,
    smoking_quit_date DATE,
    
    alcohol_status TEXT,
    standard_drinks_per_week INTEGER,
    
    exercise_frequency TEXT,
    occupation TEXT,
    living_situation TEXT,
    support_network TEXT,
    
    -- Encrypted notes field
    notes BLOB,
    
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_by BLOB NOT NULL,
    
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (updated_by) REFERENCES users(id),
    
    -- Only one social history record per patient
    UNIQUE(patient_id)
);

CREATE INDEX idx_social_history_patient ON social_history(patient_id);
