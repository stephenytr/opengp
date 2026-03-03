-- Migration: Add reason field to consultations table
-- Adds a nullable reason column to capture the presenting reason for a consultation

ALTER TABLE consultations ADD COLUMN reason TEXT;
