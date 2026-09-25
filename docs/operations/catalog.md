# Catalog operations

Renewal, publication and withdrawal share one durable transaction boundary.
Daily renewal, independent monitoring and backup reconciliation are implemented.
The workflows are opt-in and have not been activated; no production release or
deployment has been performed by this implementation.

Use a private state directory **outside the public catalog directory**, on the
same filesystem. Serve only the public directory. The CLI refuses state beneath
the catalog directory, but it cannot inspect your web server's broader mounts.
New state directories are created with Unix mode 0700. Back up release state
independently from the public artifact/catalog backup.

## Review and publish a release

These are privileged operator commands, not a publisher submission API. Reviewer
identity must come from your authenticated operator process; do not copy a
publisher's claimed reviewer fields into this command. Publisher accounts and
service-enforced review roles are separate work in plans 11–13.

Run review separately (`hub scan` may call your trusted reviewer). Record the
review decision, then publish its exact app bytes:

```sh
hub admin publish /private/reviewed-bundle \
  --catalog /srv/hub/public/catalog.json \
  --state-dir /var/lib/octosense-hub/release-state \
  --expected-sequence 42 --idempotency-key release-example-2 \
  --anchor "$HUB_ANCHOR_PUBLIC" --key "$HUB_WORKING_KEY_FILE" \
  --anchor-cert "$HUB_WORKING_KEY_CERTIFICATE" \
  --publisher example --publisher-key "example=$PUBLISHER_PUBLIC_KEY" \
  --reviewed-by operator-id --review-id decision-123 \
  --validator /opt/octosense/bin/app-validator
```

Use expected sequence 0 for the first catalog. The trusted anchor is mandatory
from the first release. `hub publish` remains an operator alias. The old
`--reviewed` flag and inline `--reviewer` command are refused on publication;
review cannot bypass required native checks. The worker runs before the local
signing key is loaded. Production signer implementations consume only the
validated transaction through `CatalogSigner`; use a protected operator
installation and never run the signing job from a submission/PR checkout.

Publication rechecks publisher continuity and duplicate versions under the lock,
then stages the complete approved artifact, pack and evidence privately. The
artifact path is addressed by a hash of the approved bytes/metadata. Existing
objects are verified and synchronized on retry, never overwritten. An artifact
failure before signing does not reserve the catalog sequence. Orphan complete
artifacts from an interrupted pre-signing attempt may be retained for later
collection; they are not catalog releases.

After staging, the transaction reserves its candidate, signs it, persists its
receipt/head and swaps the catalog. A concurrent operation with a stale expected
sequence must re-read the catalog and retry. The same successful request does
not duplicate a release. An out-of-process signer adapter can implement the
interface without executing any Card or reviewer command; the included key-file
adapter is the local operator implementation.

## Withdraw an exact version

```sh
hub admin withdraw example --version 2.0.0 --reason "This version has been retired" \
  --catalog /srv/hub/public/catalog.json \
  --state-dir /var/lib/octosense-hub/release-state \
  --expected-sequence 43 --idempotency-key retire-example-2 \
  --anchor "$HUB_ANCHOR_PUBLIC" --key "$HUB_WORKING_KEY_FILE" \
  --anchor-cert "$HUB_WORKING_KEY_CERTIFICATE"
```

Withdrawal retains the publisher binding, immutable artifacts and exact version
history. `hub remove` is disabled: deleting released identity/history and freeing
its version for reuse is not a supported release operation.

## Renew an existing catalog

Keep the anchor offline. Supply its public key, an anchor-certified working key,
and a private durable state directory, separate from the public catalog backup.
Use an operator checkout/build; do not run this job from an app submission's code.

```sh
hub admin renew --catalog /srv/hub/public/catalog.json \
  --state-dir /var/lib/octosense-hub/release-state \
  --expected-sequence 42 --idempotency-key renewal-2026-09-25 \
  --anchor "$HUB_ANCHOR_PUBLIC" --key "$HUB_WORKING_KEY_FILE" \
  --anchor-cert "$HUB_WORKING_KEY_CERTIFICATE"
```

The command uses the operator's current UTC date, verifies the old catalog and
working-key certificate, preserves all release entries, then publishes sequence
43. The scheduled job uses `--expected-sequence auto` to select the current
sequence under the same lock; a repeated daily request retains its original
generation. Manual new requests require the current sequence; retry the **same** operation with
its original expected sequence and idempotency key. Reusing a key for a different
request is refused. A new certified working key can perform the next renewal.

All writers must share the catalog's sibling lock and the same state directory.
The transaction reserves a generation before signing, stores a create-only retry
receipt, synchronizes the durable head, and atomically replaces the catalog.
Readers see a complete old or new catalog. A prepared generation blocks other
operations until its original request finishes. A corrected clock cannot install
future-dated prepared output; retry after the trusted clock is valid.

State contains `pending.json` while an operation is reserved, `head.json` as the
durable generation high-water mark, and `requests/` with retry receipts. Do not
hand-edit or delete these files to bypass a refusal. Keep this directory outside
public hosting and back it up independently. Restoring an older public catalog
cannot create a new release below the recorded generation. Use the reconciliation command below after an older public backup is restored.

Catalog transactions use compact bounded JSON: at most 64 MiB per state record,
with 4 KiB reserved for signature and transaction metadata when preparing a
catalog. Missing/corrupt signatures, impossible dates, sequence overflow and
conflicting durable history fail before catalog replacement. The local runner
currently requires Unix and a filesystem supporting locks, hard links, atomic
renames and fsync. Object-store deployment needs its own conditional-write adapter.

## Verification recorded for renewal

CLI tests cover empty/populated renewal, unchanged release entries, malformed or
future dates, signature failures, idempotency conflicts, restored sequence refusal
and certified working-key rotation. Transaction tests inject interruptions before
intent/signing/receipt/head/catalog writes and after catalog replacement, then
retry. Concurrent requests cannot reuse a generation. Additional regressions
cover a competing request after interruption, clock correction, signer mutation
and oversized serialized records. These are local tests with temporary keys.

## Recover a public backup

The release operator owns recovery. Retain public artifacts, packs and validation
sidecars, plus an independent backup of the private head, pending intent and all
request receipts. Back up the working-key certificate and custody configuration;
keep the offline anchor separate. Select retention according to your recovery
objectives; do not garbage-collect objects referenced by retained generations.

Stop ordinary publication while restoring. Restore immutable artifacts and packs
first. Inspect the retained signed `head.json` to obtain its sequence; do not
copy a number from the older public backup. Then run:

```sh
hub admin recover --catalog /srv/hub/public/catalog.json \
  --state-dir /var/lib/octosense-hub/release-state \
  --expected-sequence 45 --idempotency-key restore-incident-123 \
  --anchor "$HUB_ANCHOR_PUBLIC" --key "$HUB_WORKING_KEY_FILE" \
  --anchor-cert "$HUB_WORKING_KEY_CERTIFICATE"
```

Recovery verifies and synchronizes every retained bundle/pack, restores the exact
signed durable head (including withdrawals), then signs a newer generation.
It never constructs history from the older backup. An interrupted recovery can
retry its original request. A different pending operation must finish using its
original inputs first; preserve reviewed bytes, review record and validator build
for publication retries. Do not delete the pending intent to unblock the queue.

A missing public pointer is recoverable. A corrupt or wrongly signed public
pointer is refused: stop serving it, preserve it as incident evidence outside the
public root, and recover the missing pointer from authenticated durable state.
A newer public generation than the retained head, conflicting signatures, missing
state, or corrupt state requires recovering authoritative history first. This
command cannot infer a lost high-water mark. Compromise/anchor recovery is plan 19.
After recovery, run the public probe with `--expected-catalog` before reopening
publication. A new anchor-certified working key can perform renewal/recovery;
never replace the trusted anchor merely to bypass a signature failure.

## Freshness and public probes

```sh
hub admin status --catalog /srv/hub/public/catalog.json --anchor "$HUB_ANCHOR_PUBLIC"
hub admin probe --base https://apps.example.org \
  --anchor "$HUB_ANCHOR_PUBLIC" \
  --expected-catalog /srv/hub/public/catalog.json --max-artifacts 10
```

Both commands emit JSON and return nonzero when attention is required. At seven
days the catalog is `warning`, at twelve `critical`, and after fourteen `expired`.
The probe authenticates the public catalog, checks its age and sequence, downloads
up to ten newest offered packs, and verifies complete manifests and payloads.
Its `sampled` field explicitly reports when it did not check every offered pack.
The public origin must use HTTPS; `--allow-local-http` permits only explicit
localhost/127.0.0.1 fixture servers.

An independent monitor can use `--minimum-sequence N` without access to operator
files. Keep that floor updated from trusted releases. The default zero checks
signatures/freshness but cannot detect replay of a still-fresh older generation.
A probe failure may also mean a stale CDN cache; inspect the JSON report before
changing any release state. The operator's post-renewal probe compares with the
local signed catalog to detect that lag.

## Configure renewal and independent monitoring

The two workflows are disabled unless their repository variables are explicitly
`true`. They run only from the default branch. Provision and review the following
before enabling them; deployment and alert routing remain operator setup work.

| Workflow | Required configuration |
| --- | --- |
| `catalog-renewal.yml` | Protected `catalog-signing` environment; dedicated Linux self-hosted runner labeled `octosense-catalog-operator`; preinstalled reviewed `/opt/octosense/bin/hub`; durable public/private filesystem; protected working-key file. |
| Renewal variables | `CATALOG_HUB_SHA256`, `CATALOG_FILE`, `CATALOG_STATE_DIR`, `CATALOG_PUBLIC_BASE`, `CATALOG_ANCHOR_PUBLIC`, `CATALOG_WORKING_KEY_FILE`, `CATALOG_WORKING_KEY_CERTIFICATE`; enable with `CATALOG_RENEWAL_ENABLED=true`. |
| `catalog-monitor.yml` | Independent GitHub-hosted Linux job; reviewed Linux Hub binary available at an HTTPS URL, pinned by SHA-256; no signing credentials or private filesystem. |
| Monitor variables | `CATALOG_MONITOR_BINARY_URL`, `CATALOG_MONITOR_BINARY_SHA256`, `CATALOG_PUBLIC_BASE`, `CATALOG_ANCHOR_PUBLIC`, `CATALOG_MONITOR_MINIMUM_SEQUENCE`; enable with `CATALOG_MONITOR_ENABLED=true`. |

Do not put key contents in repository variables. Restrict the signing runner to
trusted operator workflows; never schedule submission/PR builds on it. Neither
workflow checks out repository code. A binary checksum failure prevents execution,
including the renewal job's always-run probe. The included signer is the local
key-file adapter; a remote custody adapter is not deployed by this change.

Daily renewal runs at 06:17 UTC and probes the public origin afterward. The
independent monitor runs every six hours at minute 37, even if the signing runner
is unavailable. Configure Actions failure notifications/on-call routing and test
a warning with a test catalog before activation; JSON output and error annotations
alone are not evidence that a human received an alert. Both jobs still depend on
GitHub scheduling. An external read-only cron running the same probe is needed
for monitoring during a GitHub outage. Scheduled workflows can be delayed; see
[GitHub schedule behavior](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#schedule).
Filesystem locking and durable requests remain necessary even with workflow
[concurrency protection](https://docs.github.com/en/actions/concepts/workflows-and-actions/concurrency).

## Local validation scope

Tests cover interrupted publication/renewal/recovery, concurrent writers,
idempotency conflicts, private state, withdrawal preservation, restored artifact
synchronization, replay, certified key rotation, 7/12/14-day boundaries, bad
signatures and damaged packs. A temporary signed native Card release was probed
through a real loopback HTTP server, including a damaged-pack refusal. Workflow
YAML and shell blocks were checked locally, including checksum refusal. These
checks do not constitute a deployed scheduler, delivered alert, remote signer,
Linux runtime validator or device qualification.
