-- Add revoked_at column to license_keys table
-- This allows us to track when licenses were revoked without deleting them

USE receipt_extractor;

-- Add the revoked_at column
ALTER TABLE license_keys 
ADD COLUMN revoked_at TIMESTAMP NULL DEFAULT NULL AFTER assigned_at;

-- Add index for faster queries on revoked licenses
ALTER TABLE license_keys 
ADD INDEX idx_revoked (revoked_at);

-- Update the statistics view to exclude revoked licenses
DROP VIEW IF EXISTS license_statistics;
CREATE VIEW license_statistics AS
SELECT 
    tier,
    COUNT(*) as total_keys,
    SUM(CASE WHEN is_used = TRUE AND revoked_at IS NULL THEN 1 ELSE 0 END) as used_keys,
    SUM(CASE WHEN is_used = FALSE AND revoked_at IS NULL THEN 1 ELSE 0 END) as available_keys,
    SUM(CASE WHEN revoked_at IS NOT NULL THEN 1 ELSE 0 END) as revoked_keys
FROM license_keys
GROUP BY tier;

