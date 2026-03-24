-- Migration: Add performance indexes for clinical tables and patient search
-- Composite indexes for common query patterns and trigram search for patient names

-- Enable pg_trgm extension for trigram-based full-text search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Vital signs: composite index for patient + measured_at ordering
-- Supports queries filtering by patient and ordering by measurement time
CREATE INDEX IF NOT EXISTS idx_vital_signs_patient_measured_at 
ON vital_signs(patient_id, measured_at DESC);

-- Family history: composite index for patient + creation time ordering
-- Supports queries filtering by patient and ordering by creation date
CREATE INDEX IF NOT EXISTS idx_family_history_patient_created 
ON family_history(patient_id, created_at DESC);

-- Users: upgrade to composite index on role + active status
-- Drop the old single-column index first
DROP INDEX IF EXISTS idx_users_active;

-- Create new composite index for role-based queries with active filter
CREATE INDEX IF NOT EXISTS idx_users_role_active 
ON users(role, is_active) WHERE is_active = TRUE;

-- Appointments: composite index for practitioner scheduling queries
-- Supports time-range queries excluding cancelled/no-show appointments
CREATE INDEX IF NOT EXISTS idx_appointments_practitioner_time 
ON appointments(practitioner_id, start_time, end_time) 
WHERE status NOT IN ('Cancelled', 'NoShow');

-- Patients: trigram index for fuzzy name search
-- Supports ILIKE and similarity searches on concatenated first + last name
CREATE INDEX IF NOT EXISTS idx_patients_name_trgm 
ON patients USING gin ((first_name || ' ' || last_name) gin_trgm_ops) 
WHERE is_active = TRUE;
