CREATE TABLE submissions (
    id TEXT PRIMARY KEY,
    publisher_id TEXT NOT NULL REFERENCES publishers(id),
    app_id TEXT NOT NULL REFERENCES app_claims(app_id),
    upload_id TEXT NOT NULL UNIQUE REFERENCES uploads(id),
    version TEXT NOT NULL,
    release_number INTEGER NOT NULL CHECK (release_number > 0),
    manifest_digest TEXT NOT NULL,
    bundle_digest TEXT NOT NULL,
    artifact_digest TEXT NOT NULL,
    source_repository TEXT,
    source_commit TEXT,
    idempotency_key TEXT NOT NULL,
    request_digest TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('submitted','validating','awaiting_review',
        'approved','publishing','published','rejected','failed','cancelled')),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    CHECK ((source_repository IS NULL) = (source_commit IS NULL)),
    UNIQUE (publisher_id, idempotency_key),
    UNIQUE (app_id, version),
    UNIQUE (app_id, release_number)
) STRICT;

CREATE TABLE validation_jobs (
    id INTEGER PRIMARY KEY,
    submission_id TEXT NOT NULL UNIQUE REFERENCES submissions(id),
    status TEXT NOT NULL CHECK (status IN ('queued','leased','complete','failed')),
    created_at INTEGER NOT NULL,
    lease_until INTEGER,
    attempts INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE TABLE submission_events (
    id INTEGER PRIMARY KEY,
    submission_id TEXT NOT NULL REFERENCES submissions(id),
    actor_account_id TEXT REFERENCES accounts(id),
    from_status TEXT,
    to_status TEXT NOT NULL,
    artifact_digest TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX submissions_publisher ON submissions(publisher_id, created_at);
CREATE INDEX validation_jobs_queue ON validation_jobs(status, created_at);
