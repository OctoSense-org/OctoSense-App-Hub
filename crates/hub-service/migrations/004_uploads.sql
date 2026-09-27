CREATE TABLE uploads (
    id TEXT PRIMARY KEY,
    app_id TEXT NOT NULL REFERENCES app_claims(app_id),
    publisher_id TEXT NOT NULL REFERENCES publishers(id),
    initiated_by TEXT NOT NULL REFERENCES accounts(id),
    expected_digest TEXT NOT NULL,
    expected_bytes INTEGER NOT NULL CHECK (expected_bytes > 0),
    status TEXT NOT NULL CHECK (status IN ('pending', 'complete')),
    created_at INTEGER NOT NULL,
    completed_at INTEGER,
    UNIQUE (app_id, expected_digest)
) STRICT;

CREATE INDEX uploads_publisher ON uploads(publisher_id, created_at);
