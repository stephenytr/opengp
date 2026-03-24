-- Initial schema for OpenGP database
-- This creates the base tables needed for patient management

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('Admin', 'Doctor', 'Nurse', 'Receptionist', 'Billing')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active = TRUE;

CREATE TABLE IF NOT EXISTS patients (
    id UUID PRIMARY KEY,
    
    ihi BYTEA,
    medicare_number BYTEA,
    medicare_irn INTEGER CHECK(medicare_irn BETWEEN 1 AND 9),
    medicare_expiry DATE,
    
    title TEXT,
    first_name TEXT NOT NULL,
    middle_name TEXT,
    last_name TEXT NOT NULL,
    preferred_name TEXT,
    date_of_birth DATE NOT NULL,
    gender TEXT NOT NULL CHECK(gender IN ('Male', 'Female', 'Other', 'PreferNotToSay')),
    
    address_line1 TEXT,
    address_line2 TEXT,
    suburb TEXT,
    state TEXT,
    postcode TEXT,
    country TEXT DEFAULT 'Australia',
    phone_home TEXT,
    phone_mobile TEXT,
    phone_work TEXT,
    email TEXT,
    
    emergency_contact_name TEXT,
    emergency_contact_phone TEXT,
    emergency_contact_relationship TEXT,
    
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_deceased BOOLEAN NOT NULL DEFAULT FALSE,
    
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID,
    updated_by UUID,
    
    UNIQUE(medicare_number, medicare_irn),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (updated_by) REFERENCES users(id)
);

CREATE INDEX idx_patients_name ON patients(last_name, first_name);
CREATE INDEX idx_patients_dob ON patients(date_of_birth);
CREATE INDEX idx_patients_medicare ON patients(medicare_number);
CREATE INDEX idx_patients_active ON patients(is_active) WHERE is_active = TRUE;

CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    action TEXT NOT NULL,
    entity_type TEXT,
    entity_id UUID,
    metadata TEXT,
    ip_address TEXT,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_audit_log_user ON audit_log(user_id);
CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);

CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
