ALTER TABLE validation_jobs ADD COLUMN lease_token_hash BLOB;
ALTER TABLE validation_jobs ADD COLUMN outcome_digest TEXT;

CREATE TABLE validation_evidence (
    submission_id TEXT PRIMARY KEY REFERENCES submissions(id),
    artifact_digest TEXT NOT NULL,
    result_digest TEXT NOT NULL,
    validator_id TEXT NOT NULL,
    policy_version TEXT NOT NULL,
    passed INTEGER NOT NULL CHECK (passed IN (0, 1)),
    report_json TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;
