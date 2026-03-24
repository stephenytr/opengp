-- Add source column to audit_logs table
-- Tracks whether data was accessed from cache or database
-- Defaults to 'database' for backward compatibility

ALTER TABLE audit_logs
ADD COLUMN source TEXT NOT NULL DEFAULT 'database';

-- Create index for source queries
CREATE INDEX idx_audit_source ON audit_logs(source);
