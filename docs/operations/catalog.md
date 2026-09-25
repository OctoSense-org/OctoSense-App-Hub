# Catalog operations

The renewal transaction is implemented. Migrating publication/withdrawal to the
same boundary, automated scheduling, artifact probes and explicit backup recovery
are the next plan 04 slices. Do not mix legacy catalog writers with a store whose
release history has been initialized; they do not yet participate in its lock.
No production renewal or deployment has been performed by this implementation.

## Renew an existing catalog

Keep the anchor offline. Supply its public key, an anchor-certified working key,
and a private durable state directory, separate from the public catalog backup.
Use an operator checkout/build; do not run this job from an app submission's code.

```sh
hub admin renew --catalog /srv/hub/catalog.json \
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
