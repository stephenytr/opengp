-- Add medicare_search_hash column and index for efficient lookups
-- This column stores HMAC-SHA256 hash of Medicare number for indexed searches
-- without requiring decryption of the encrypted medicare_number field

ALTER TABLE patients ADD COLUMN medicare_search_hash VARCHAR(64);

-- Partial index on active patients with non-null hash
-- This is efficient because:
-- 1. Only indexes active patients (WHERE is_active = TRUE)
-- 2. Only indexes rows with a hash value (WHERE medicare_search_hash IS NOT NULL)
-- 3. Avoids indexing NULL values which are common during backfill
CREATE INDEX idx_patients_medicare_hash ON patients(medicare_search_hash) 
WHERE is_active = TRUE AND medicare_search_hash IS NOT NULL;

-- Document the column purpose
COMMENT ON COLUMN patients.medicare_search_hash IS 'HMAC-SHA256 hash of Medicare number for indexed lookups without decryption';
