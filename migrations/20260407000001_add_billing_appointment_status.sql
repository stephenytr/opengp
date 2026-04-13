-- Migration: Add 'Billing' status to appointments table

-- Drop the existing status CHECK constraint and add an updated one
ALTER TABLE appointments DROP CONSTRAINT IF EXISTS appointments_status_check;

ALTER TABLE appointments ADD CONSTRAINT appointments_status_check
    CHECK (status IN (
        'Scheduled', 'Confirmed', 'Arrived', 'InProgress',
        'Completed', 'NoShow', 'Cancelled', 'Rescheduled', 'Billing'
    ));
