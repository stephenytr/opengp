-- Migration: Add encrypted storage for sensitive patient data
-- Issue #3: Encrypt clinical notes, social history, and patient identifiers

-- Consultations table with encrypted SOAP notes
CREATE TABLE IF NOT EXISTS consultations (
    id UUID PRIMARY KEY,
    patient_id UUID NOT NULL,
    practitioner_id UUID NOT NULL,
    appointment_id UUID,
    consultation_date TIMESTAMP WITH TIME ZONE NOT NULL,
    
    -- Encrypted SOAP notes (UUID for encrypted bytes)
    soap_subjective UUID,
    soap_objective UUID,
    soap_assessment UUID,
    soap_plan UUID,
    
    is_signed BOOLEAN NOT NULL DEFAULT FALSE,
    signed_at TIMESTAMP WITH TIME ZONE,
    signed_by UUID,
    
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NOT NULL,
    updated_by UUID,
    
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
    
    -- Encrypted notes field
    notes UUID,
    
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_by UUID NOT NULL,
    
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (updated_by) REFERENCES users(id),
    
    -- Only one social history record per patient
    UNIQUE(patient_id)
);

CREATE INDEX idx_social_history_patient ON social_history(patient_id);
