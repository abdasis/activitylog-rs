CREATE TABLE IF NOT EXISTS activity_log (
    id VARCHAR(36) PRIMARY KEY NOT NULL,
    log_name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    subject_type VARCHAR(255),
    subject_id VARCHAR(36),
    causer_type VARCHAR(255),
    causer_id VARCHAR(36),
    properties TEXT NOT NULL,
    attribute_changes TEXT,
    event VARCHAR(64),
    batch_uuid VARCHAR(36),
    created_at VARCHAR(32) NOT NULL,
    updated_at VARCHAR(32)
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
