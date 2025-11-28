-- Create the sync log table for tracking changes
CREATE TABLE _rhtmx_sync_log (
    id BIGSERIAL PRIMARY KEY,
    entity VARCHAR NOT NULL,
    entity_id VARCHAR NOT NULL,
    action VARCHAR NOT NULL,
    data TEXT,
    version BIGINT NOT NULL,
    client_id VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create index for efficient querying by entity and version
CREATE INDEX idx_sync_entity_version ON _rhtmx_sync_log(entity, version);

-- Create index for querying by entity_id
CREATE INDEX idx_sync_entity_id ON _rhtmx_sync_log(entity, entity_id);

-- Create a notification function for PostgreSQL LISTEN/NOTIFY
CREATE OR REPLACE FUNCTION notify_sync_change()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        '_rhtmx_sync_' || NEW.entity,
        json_build_object(
            'id', NEW.id,
            'entity', NEW.entity,
            'entity_id', NEW.entity_id,
            'action', NEW.action,
            'version', NEW.version,
            'client_id', NEW.client_id
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to notify on inserts
CREATE TRIGGER sync_change_notify
AFTER INSERT ON _rhtmx_sync_log
FOR EACH ROW
EXECUTE FUNCTION notify_sync_change();
