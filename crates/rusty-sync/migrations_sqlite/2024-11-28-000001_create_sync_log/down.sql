-- Drop the indexes
DROP INDEX IF EXISTS idx_sync_entity_id;
DROP INDEX IF EXISTS idx_sync_entity_version;

-- Drop the sync log table
DROP TABLE IF EXISTS _rhtmx_sync_log;
