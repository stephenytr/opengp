-- Add version columns for optimistic locking
-- This migration adds the version field to patients, appointments, and consultations tables

ALTER TABLE patients ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE appointments ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE consultations ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
