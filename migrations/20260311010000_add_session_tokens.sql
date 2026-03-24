ALTER TABLE sessions ADD COLUMN IF NOT EXISTS token TEXT;

UPDATE sessions
SET token = lower(md5(random()::text || now()::text))
WHERE token IS NULL OR token = '';

CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
