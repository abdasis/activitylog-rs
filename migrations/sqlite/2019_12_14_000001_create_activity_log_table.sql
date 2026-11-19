CREATE TABLE IF NOT EXISTS activity_log (
    id TEXT PRIMARY KEY NOT NULL,
    log_name TEXT NOT NULL,
    description TEXT NOT NULL,
    subject_type TEXT,
    subject_id TEXT,
    causer_type TEXT,
    causer_id TEXT,
    properties TEXT NOT NULL,
    attribute_changes TEXT,
    event TEXT,
    batch_uuid TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT
);

CREATE INDEX IF NOT EXISTS activity_log_subject_index
    ON activity_log (subject_type, subject_id);

CREATE INDEX IF NOT EXISTS activity_log_causer_index
    ON activity_log (causer_type, causer_id);

CREATE INDEX IF NOT EXISTS activity_log_log_name_index
    ON activity_log (log_name);

CREATE INDEX IF NOT EXISTS activity_log_created_at_index
    ON activity_log (created_at);

CREATE INDEX IF NOT EXISTS activity_log_batch_uuid_index
    ON activity_log (batch_uuid);
