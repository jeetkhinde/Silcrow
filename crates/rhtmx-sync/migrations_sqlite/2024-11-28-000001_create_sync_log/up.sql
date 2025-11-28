-- Create the sync log table for tracking changes (SQLite version)
CREATE TABLE _rhtmx_sync_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    action TEXT NOT NULL,
    data TEXT,
    version INTEGER NOT NULL,
    client_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create index for efficient querying by entity and version
CREATE INDEX idx_sync_entity_version ON _rhtmx_sync_log(entity, version);

-- Create index for querying by entity_id
CREATE INDEX idx_sync_entity_id ON _rhtmx_sync_log(entity, entity_id);
