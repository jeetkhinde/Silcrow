-- Drop trigger first
DROP TRIGGER IF EXISTS field_sync_change_notify ON _rhtmx_field_sync_log;

-- Drop the notification function
DROP FUNCTION IF EXISTS notify_field_sync_change();

-- Drop indexes
DROP INDEX IF EXISTS idx_field_sync_latest;
DROP INDEX IF EXISTS idx_field_sync_version;
DROP INDEX IF EXISTS idx_field_sync_entity_field;

-- Drop the field sync log table
DROP TABLE IF EXISTS _rhtmx_field_sync_log;
