-- Add version columns for optimistic locking
-- This migration adds the version field to patients and appointments tables

ALTER TABLE patients ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE appointments ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
