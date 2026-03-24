ALTER TABLE sessions ADD COLUMN IF NOT EXISTS token TEXT;

UPDATE sessions
SET token = lower(hex(randomblob(32)))
WHERE token IS NULL OR token = '';

CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
