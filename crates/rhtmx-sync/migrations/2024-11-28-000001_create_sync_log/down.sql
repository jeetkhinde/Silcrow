-- Drop the trigger
DROP TRIGGER IF EXISTS sync_change_notify ON _rhtmx_sync_log;

-- Drop the notification function
DROP FUNCTION IF EXISTS notify_sync_change();

-- Drop the indexes
DROP INDEX IF EXISTS idx_sync_entity_id;
DROP INDEX IF EXISTS idx_sync_entity_version;

-- Drop the sync log table
DROP TABLE IF EXISTS _rhtmx_sync_log;
