-- Migration: Create dva_claims table
-- Date: 2026-04-22
-- Description: Store DVA (Department of Veterans' Affairs) claim records

CREATE TABLE IF NOT EXISTS dva_claims (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES patients(id),
    practitioner_id UUID NOT NULL,
    consultation_id UUID REFERENCES consultations(id),
    dva_file_number VARCHAR(50) NOT NULL,
    card_type VARCHAR(20) NOT NULL,
    service_date DATE NOT NULL,
    items JSONB NOT NULL,
    total_claimed DECIMAL(10, 2) NOT NULL,
    status VARCHAR(20) NOT NULL,
    submitted_at TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_dva_claims_patient_id ON dva_claims(patient_id);
CREATE INDEX IF NOT EXISTS idx_dva_claims_practitioner_id ON dva_claims(practitioner_id);
CREATE INDEX IF NOT EXISTS idx_dva_claims_service_date ON dva_claims(service_date);

-- Down migration
-- DROP TABLE IF EXISTS dva_claims;
