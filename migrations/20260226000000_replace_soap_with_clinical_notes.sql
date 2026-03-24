-- Migration: Replace SOAP notes with clinical_notes field
-- Replaces 4 separate SOAP note columns with a single clinical_notes column
-- Data is discarded (fresh start as per user request)

-- Add clinical_notes column (UUID for encrypted data)
ALTER TABLE consultations ADD COLUMN IF NOT EXISTS clinical_notes UUID;

-- Drop old SOAP note columns
ALTER TABLE consultations DROP COLUMN IF EXISTS soap_subjective;
ALTER TABLE consultations DROP COLUMN IF EXISTS soap_objective;
ALTER TABLE consultations DROP COLUMN IF EXISTS soap_assessment;
ALTER TABLE consultations DROP COLUMN IF EXISTS soap_plan;