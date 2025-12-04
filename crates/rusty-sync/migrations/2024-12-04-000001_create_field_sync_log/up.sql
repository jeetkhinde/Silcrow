-- Create the field sync log table for field-level change tracking
CREATE TABLE _rhtmx_field_sync_log (
    id BIGSERIAL PRIMARY KEY,
    entity VARCHAR NOT NULL,
    entity_id VARCHAR NOT NULL,
    field VARCHAR NOT NULL,
    value TEXT,
    action VARCHAR NOT NULL,
    version BIGINT NOT NULL,
    client_id VARCHAR,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create composite index for efficient field-level queries
CREATE INDEX idx_field_sync_entity_field
ON _rhtmx_field_sync_log(entity, entity_id, field, version);

-- Create index for version-based queries
CREATE INDEX idx_field_sync_version
ON _rhtmx_field_sync_log(entity, version);

-- Create index for latest field value queries
CREATE INDEX idx_field_sync_latest
ON _rhtmx_field_sync_log(entity, entity_id, field, id DESC);

-- Create a notification function for PostgreSQL LISTEN/NOTIFY (field-level)
CREATE OR REPLACE FUNCTION notify_field_sync_change()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        '_rhtmx_field_sync_' || NEW.entity,
        json_build_object(
            'id', NEW.id,
            'entity', NEW.entity,
            'entity_id', NEW.entity_id,
            'field', NEW.field,
            'action', NEW.action,
            'version', NEW.version,
            'client_id', NEW.client_id
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to notify on field inserts
CREATE TRIGGER field_sync_change_notify
AFTER INSERT ON _rhtmx_field_sync_log
FOR EACH ROW
EXECUTE FUNCTION notify_field_sync_change();
