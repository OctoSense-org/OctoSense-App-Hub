# Private artifact upload (development contract)

The current service stages App Hub pack JSON and creates a durable validation
queue record. It does not yet run the validator executable or publish an app.
A developer must first own an
`active` app claim and authenticate with an `apps.submit` token.

1. `POST /v1/apps/{app_id}/uploads` with JSON
   `{ "expected_digest": "<BLAKE3 of exact pack bytes>", "expected_bytes": 12345 }`.
   The response contains an upload ID and `pending` status. Repeating the same
   app ID and digest returns the same upload; a different declared size
   conflicts.
2. `PUT /v1/uploads/{upload_id}/content` with the raw pack JSON bytes and a
   Bearer token. The service streams the body into private quarantine, refuses
   an incomplete or oversized body, verifies the exact digest, and checks the
   pack schema, path safety, file count and unpacked-size limits. It then
   exposes a read-only blob under its digest and marks the upload `complete`.
3. `GET /v1/uploads/{upload_id}` returns the owner-visible upload status. An
   unsuccessful transfer remains `pending` and can be retried. No public blob
   URL is returned.

`POST /v1/apps/{app_id}/submissions` accepts JSON
`{ "upload_id": "...", "source_repository": "https://...", "source_commit": "..." }`
and requires an `Idempotency-Key` header. The source fields may both be
omitted. The service rechecks the exact stored pack, its bundle digest, v2
manifest, app ID and signature against the enrolled publisher key. It
transactionally reserves the manifest's version and release number and creates
one queued validation job and audit event. Repeating the same key and request
returns the same `submitted` record; changed input or a competing request for
the version returns `409 submission_conflict`. An API caller cannot set an
approval or publication status.

The raw pack ceiling is 12 MiB; the contained bundle uses the Hub's 8 MiB
unpacked limit. Blob storage lives next to the SQLite database in a private
`*.blobs` directory. The service never fetches a publisher URL or runs code
during upload. Worker execution, publisher status polling, cancellation and
review are subsequent steps.

The internal job state has a five-minute lease, a random per-attempt token
stored only as a hash, restart-safe reclaim, and a three-attempt failure
ceiling. A stale worker cannot write evidence. A matching retry after evidence
is committed is idempotent. Passing evidence moves a submission only to
`awaiting_review`; it is never approval. The service currently has no worker
credential exchange or trusted validator runner, so this internal method must
not be exposed as a publisher API.
