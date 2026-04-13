-- Add missing patient clinical and administrative fields
ALTER TABLE patients ADD COLUMN IF NOT EXISTS concession_type TEXT;
ALTER TABLE patients ADD COLUMN IF NOT EXISTS concession_number TEXT;
ALTER TABLE patients ADD COLUMN IF NOT EXISTS preferred_language TEXT NOT NULL DEFAULT 'English';
ALTER TABLE patients ADD COLUMN IF NOT EXISTS interpreter_required BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE patients ADD COLUMN IF NOT EXISTS atsi_status TEXT;
