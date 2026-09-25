# Catalog operations

Renewal, publication and withdrawal share one durable transaction boundary.
Scheduled automation, public probes and an explicit backup reconciliation command
are the remaining plan 04 work. No production release or deployment has been
performed by this implementation.

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
43. New requests require the current sequence; retry the **same** operation with
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
cannot create a new release below the recorded generation. The dedicated
reconciliation/restore operation is still being implemented.

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
