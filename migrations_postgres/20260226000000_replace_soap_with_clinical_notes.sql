ALTER TABLE consultations ADD COLUMN IF NOT EXISTS clinical_notes BYTEA;
ALTER TABLE consultations DROP COLUMN IF EXISTS soap_subjective;
ALTER TABLE consultations DROP COLUMN IF EXISTS soap_objective;
ALTER TABLE consultations DROP COLUMN IF EXISTS soap_assessment;
ALTER TABLE consultations DROP COLUMN IF EXISTS soap_plan;
