CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    provider_subject TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    disabled_at INTEGER,
    UNIQUE (provider, provider_subject)
) STRICT;

CREATE TABLE publishers (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    owner_account_id TEXT NOT NULL REFERENCES accounts(id),
    created_at INTEGER NOT NULL,
    disabled_at INTEGER
) STRICT;

CREATE TABLE publisher_memberships (
    publisher_id TEXT NOT NULL REFERENCES publishers(id),
    account_id TEXT NOT NULL REFERENCES accounts(id),
    role TEXT NOT NULL CHECK (role IN ('owner', 'submitter')),
    created_at INTEGER NOT NULL,
    PRIMARY KEY (publisher_id, account_id)
) STRICT;

CREATE TABLE signing_keys (
    id TEXT PRIMARY KEY,
    publisher_id TEXT NOT NULL REFERENCES publishers(id),
    public_key BLOB NOT NULL UNIQUE,
    enrolled_by TEXT NOT NULL REFERENCES accounts(id),
    enrolled_at INTEGER NOT NULL,
    revoked_at INTEGER
) STRICT;

CREATE TABLE key_challenges (
    id TEXT PRIMARY KEY,
    publisher_id TEXT NOT NULL REFERENCES publishers(id),
    account_id TEXT NOT NULL REFERENCES accounts(id),
    public_key BLOB NOT NULL,
    nonce BLOB NOT NULL,
    expires_at INTEGER NOT NULL,
    consumed_at INTEGER
) STRICT;

CREATE TABLE app_claims (
    app_id TEXT PRIMARY KEY,
    publisher_id TEXT NOT NULL REFERENCES publishers(id),
    claimed_by TEXT NOT NULL REFERENCES accounts(id),
    claimed_at INTEGER NOT NULL,
    imported_from_catalog INTEGER NOT NULL DEFAULT 0 CHECK (imported_from_catalog IN (0, 1))
) STRICT;

CREATE TABLE api_sessions (
    token_hash BLOB PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    scopes TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    revoked_at INTEGER
) STRICT;

CREATE TABLE audit_events (
    id INTEGER PRIMARY KEY,
    actor_account_id TEXT REFERENCES accounts(id),
    publisher_id TEXT REFERENCES publishers(id),
    event TEXT NOT NULL,
    subject TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX app_claims_publisher ON app_claims(publisher_id);
CREATE INDEX signing_keys_publisher ON signing_keys(publisher_id);
CREATE INDEX audit_events_publisher ON audit_events(publisher_id, id);
