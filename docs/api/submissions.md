# Private artifact upload (development contract)

The current service stages App Hub pack JSON only. It does not create a
submission, start validation, or publish an app. A developer must first own an
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

The raw pack ceiling is 12 MiB; the contained bundle uses the Hub's 8 MiB
unpacked limit. Blob storage lives next to the SQLite database in a private
`*.blobs` directory. The service never fetches a publisher URL or runs code
during upload. A later submission endpoint will bind the completed blob to a
manifest, publisher signing key, idempotency key and validation job.
