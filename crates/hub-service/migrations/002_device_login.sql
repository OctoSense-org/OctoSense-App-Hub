CREATE TABLE device_logins (
    code_hash BLOB PRIMARY KEY,
    audience TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    consumed_at INTEGER
) STRICT;
