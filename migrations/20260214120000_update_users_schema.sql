-- Update users table schema to add missing User model fields
-- This migration adds columns for user profile information, authentication tracking, and permissions

-- Add first_name column (required)
ALTER TABLE users ADD COLUMN IF NOT EXISTS first_name TEXT NOT NULL DEFAULT '';
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_name TEXT NOT NULL DEFAULT '';
ALTER TABLE users ADD COLUMN IF NOT EXISTS email TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_locked BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login TIMESTAMP WITH TIME ZONE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_changed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE users ADD COLUMN IF NOT EXISTS additional_permissions TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash_new TEXT;
UPDATE users SET password_hash_new = password_hash;

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Create index on is_locked for finding locked accounts
CREATE INDEX IF NOT EXISTS idx_users_locked ON users(is_locked) WHERE is_locked = TRUE;

-- Create index on last_login for activity tracking
CREATE INDEX IF NOT EXISTS idx_users_last_login ON users(last_login);
