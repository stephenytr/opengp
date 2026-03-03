-- Update users table schema to add missing User model fields
-- This migration adds columns for user profile information, authentication tracking, and permissions

-- Add first_name column (required)
ALTER TABLE users ADD COLUMN first_name TEXT NOT NULL DEFAULT '';

-- Add last_name column (required)
ALTER TABLE users ADD COLUMN last_name TEXT NOT NULL DEFAULT '';

-- Add email column (optional)
ALTER TABLE users ADD COLUMN email TEXT;

-- Add is_locked column (tracks account lockout status)
ALTER TABLE users ADD COLUMN is_locked BOOLEAN NOT NULL DEFAULT FALSE;

-- Add failed_login_attempts column (tracks failed login attempts for lockout logic)
ALTER TABLE users ADD COLUMN failed_login_attempts INTEGER NOT NULL DEFAULT 0;

-- Add last_login column (tracks when user last logged in)
ALTER TABLE users ADD COLUMN last_login TIMESTAMP WITH TIME ZONE;

-- Add password_changed_at column (tracks when password was last changed)
ALTER TABLE users ADD COLUMN password_changed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- Add additional_permissions column (stores JSON array of additional permissions beyond role)
ALTER TABLE users ADD COLUMN additional_permissions TEXT;

-- Modify password_hash to allow NULL (for future password implementation flexibility)
-- SQLite doesn't support direct column modification, so we use a workaround:
-- Create a new column, copy data, drop old column, rename new column
ALTER TABLE users ADD COLUMN password_hash_new TEXT;
UPDATE users SET password_hash_new = password_hash;
-- Note: SQLite doesn't support DROP COLUMN in older versions, but modern SQLite (3.35.0+) does
-- For compatibility, we'll keep both columns and use password_hash_new going forward
-- The old password_hash column will be deprecated

-- Create index on email for faster lookups
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Create index on is_locked for finding locked accounts
CREATE INDEX IF NOT EXISTS idx_users_locked ON users(is_locked) WHERE is_locked = TRUE;

-- Create index on last_login for activity tracking
CREATE INDEX IF NOT EXISTS idx_users_last_login ON users(last_login);
