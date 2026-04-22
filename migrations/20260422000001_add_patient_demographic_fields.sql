-- Migration: Add patient demographic fields
-- Date: 2026-04-22
-- Description: Add occupation, employment_status, health_fund, and dva_card_type columns to patients table

ALTER TABLE patients
    ADD COLUMN IF NOT EXISTS occupation VARCHAR(255),
    ADD COLUMN IF NOT EXISTS employment_status VARCHAR(50),
    ADD COLUMN IF NOT EXISTS health_fund VARCHAR(255),
    ADD COLUMN IF NOT EXISTS dva_card_type VARCHAR(20);

-- Down migration
-- ALTER TABLE patients
--     DROP COLUMN IF EXISTS occupation,
--     DROP COLUMN IF EXISTS employment_status,
--     DROP COLUMN IF EXISTS health_fund,
--     DROP COLUMN IF EXISTS dva_card_type;
