ALTER TABLE sessions ADD COLUMN IF NOT EXISTS token TEXT;

UPDATE sessions
SET token = md5(random()::text || clock_timestamp()::text) || md5(random()::text || clock_timestamp()::text)
WHERE token IS NULL OR token = '';

CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
