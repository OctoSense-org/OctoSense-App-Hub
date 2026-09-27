# Publisher control plane (development contract)

`hub-service` is a separate HTTP process backed by SQLite. Set
`OCTOSENSE_HUB_DB` to a writable database path; it binds to
`127.0.0.1:8790` unless `OCTOSENSE_HUB_BIND` is set. `/healthz` checks the
process and `/readyz` checks the migrated database. Schema migrations are
transactional and a newer database is refused rather than downgraded.
No operator, reviewer or developer account is seeded.

Set `OCTOSENSE_GITHUB_CLIENT_ID` to the client ID of a GitHub OAuth or GitHub
App with device flow enabled to accept developer logins. Without it the
service refuses login. The default bind address is local only; this is a
development service and has not been deployed as a public API.

`POST /v1/login/device` begins GitHub device authorization and returns a
device code, user code and verification URL. Clients should poll according to
GitHub's interval; the provider adapter enforces it and handles `slow_down`.
`POST /v1/login/token` accepts `{ "device_code": "..." }` and returns a
one-hour Bearer token after the provider verifies the person. GitHub's stable
numeric user ID is the account subject; login names are display information,
not ownership. `GET /v1/me`
reads that token; `POST /v1/logout` revokes it. The service stores only hashes
of device codes and tokens. A code can be consumed once; mismatched audience,
expired codes and expired/revoked tokens are refused. Requests have a 4 KiB
body limit. Tokens grant `publisher.read`, `publisher.write` and `apps.submit`;
they never grant operator access.

`POST /v1/publishers` accepts `{ "slug": "myteam", "display_name": "My Team" }`
with a Bearer token. Slugs are lowercase ASCII identifiers after normalization;
some first-party slugs are reserved. Publisher IDs are immutable and separate
from display names. `POST /v1/apps` accepts
`{ "publisher_id": "...", "app_id": "myteam.notes" }`. An app ID under that
publisher's slug becomes `active`; other IDs are recorded as
`review_required` and are **not** publication authority. Exact normalized app
IDs are unique and ownership is checked in a transaction. Repeating one's own
claim is idempotent; another publisher receives `409 already_claimed`.

Only an owner can enroll a release key. Post
`{ "public_key": "<32-byte Ed25519 public key, lowercase hex>" }` to
`/v1/publishers/{id}/keys/challenge`, then sign the returned UTF-8 `message`
and post `{ "challenge_id": "...", "signature": "<64-byte signature, hex>" }`
to `/v1/publishers/{id}/keys`. Challenges expire after five minutes and can be
used once. The v1 key ID equals the publisher ID; replacement requires a later
explicit rotation flow. Private keys never enter this service.

The default resource limits are 1,024 active device logins, 16 publishers per
account and 1,000 app claims per publisher. These can be changed through the
service `Limits` configuration. `429 limit_exceeded` signals a quota. HTTP
request-rate controls, owner inventory, live-provider conformance, operator
review of legacy IDs, artifact upload and submission are still needed before
public publishing.

GitHub's [device authorization protocol](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#device-flow)
and [authenticated user endpoint](https://docs.github.com/en/rest/users/users#get-the-authenticated-user)
define the external identity contract. Tests use a deterministic provider and
mock GitHub HTTP responses; no GitHub account or OAuth app is configured in CI.
