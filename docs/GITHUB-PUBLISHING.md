# Publish a catalog through GitHub

English | [简体中文](GITHUB-PUBLISHING.zh-CN.md)

An App Hub repository admin authorizes the exact catalog digest. GitHub
Actions signs those bytes with a short-lived identity certificate and records
the signature in a public transparency log. No admin stores a separate Hub
private key. This is GitHub-managed artifact signing, not an SSH signature or
the green Verified badge on a commit.

**Delivery status:** the workflow and native verifier are implemented here.
An official workflow run, production environment protection and an updated
OctoSense release must be verified before the new channel replaces the old
one. This document does not claim that migration has happened.

## Who authorizes publication

Only a current repository **admin** may dispatch or rerun
[`publish-catalog.yml`](../.github/workflows/publish-catalog.yml) on `main`.
The workflow checks the original actor's numeric identity and both actors'
current admin permissions. A writer or ordinary app publisher cannot sign a
catalog through this workflow.

Before enabling publication, configure the `app-hub-catalog` GitHub environment
to allow deployment only from `main`, require an App Hub admin as reviewer,
and disable protection bypass. Keep its reviewers aligned with current admin
membership. Protect workflow and publication-tool changes through repository
review rules. The native verifier requires the signed certificate's environment
claim; removing the environment from a modified workflow cannot bypass it.

Approval covers a complete commit, candidate name and prepared SHA-256, not
an app name or mutable branch. Admin review still includes the
[normal admission checks](SUBMITTING.md#8-what-reviewers-check).

## Prepare the reviewed candidate

Keep a candidate in a reviewable commit on `main`:

```text
catalog-candidates/<name>/catalog.json
catalog-candidates/<name>/artifacts/<app>-<version>.bundle/...
catalog-candidates/<name>/artifacts/<app>-<version>.bundle.pack.json
catalog-candidates/<name>/index/<app>-<version>.json
```

The candidate `catalog.json` is an unsigned payload: schema 1, no `key` or
`signature`, sequence exactly one above the authenticated base, and today's
UTC publication date. Retain all prior entries. Existing entries may only
change from offered to withdrawn with a reason. Never remove history,
replace existing bytes or re-offer a withdrawn version. For a freshness-only
update, no new artifact files are needed.

For new versions, include the exact reviewed bundle, matching pack and index.
`hub catalog-prepare` re-runs admission and publisher continuity checks and
compares every derived entry and packed file. The workflow extracts candidate
files as data; it never executes their scripts or build commands.

After [preparing the native workspace](FIRST-APP.md#1-prepare-the-tools-and-an-app-repository):

```sh
cargo build --locked --release -p octosense-app-hub
# Use catalog-v2.json as the base once it exists.
target/release/hub catalog-prepare --base catalog.json \
  --candidate catalog-candidates/<name>/catalog.json \
  --artifact-root catalog-candidates/<name> \
  --out /tmp/catalog-v2.payload.json
```

Record `payload_sha256` from the JSON receipt and the full 40-character
candidate commit. Review the resulting payload and all added files before
authorizing that digest. Re-prepare if the base changes or the UTC date rolls
over. The command itself is covered by native integration tests; production
dispatch remains unverified until recorded separately.

## Sign, verify and publish

Open **Actions → Publish catalog → Run workflow**, select `main`, then enter
the candidate name, full commit and exact `payload_sha256`. Start with
`dry_run: true`. Approve the protected environment using an admin account.

The first job checks the admin identity, builds trusted tooling, verifies the
candidate and binds the packet to the reviewed digest. The environment-gated
job signs `catalog-v2.payload.json` using GitHub OIDC and Sigstore, then runs
the native verifier over the returned proof. A dry run retains the verified
`catalog-v2.json` envelope and receipt as workflow artifacts without changing
the public catalog.

After that proof passes consumer acceptance, run the same reviewed transaction
with `dry_run: false`. The workflow commits only the envelope and its newly
admitted artifacts/index files. Its non-force push must advance the exact main
commit it checked. Concurrent changes stop publication; they are never
silently rebased or retried. A new dispatch requires review against the new
base. Legacy `catalog.json` is untouched.

## What OctoSense verifies

The native verifier checks the exact payload SHA-256, certificate chain,
certificate transparency proof, signed transparency-log entry and inclusion
proof, GitHub issuer, App Hub repository and owner numeric IDs, workflow path,
`main` ref, manual dispatch, GitHub-hosted runner and protected environment.
The signed statement must describe the exact catalog subject and approved
workflow. Network mirrors cannot substitute another repository or workflow.

For pre-release acceptance, set `OCTOSENSE_HUB_CATALOG=github-v2` before
starting the host. The default remains legacy until production proof and
consumer acceptance pass. Once a library caches v2, removing the environment
variable retains v2; explicitly requesting `legacy` refuses the downgrade.

The v2 client is selected by host configuration before fetching. It
refuses legacy or malformed documents instead of falling back. Cache the whole
envelope, including its proof, atomically; preserve the largest accepted
sequence across restarts. Different contents at the same sequence are also
refused. The existing 14-day freshness limit still gates new installations.

## Migration and app publishers

Older releases understand the anchor-signed `catalog.json` only. They need a
host update to consume `catalog-v2.json`; replacing the old file in place would
break them. Maintain separate channel caches during migration.

This change replaces the **Hub's catalog signing key**. It does not replace
existing app publisher signatures or transfer ownership of published apps.
Their [continuity rules](PUBLISHING.md#signing) still apply. A GitHub-based app
publisher identity flow is separate work; do not drop a recorded publisher
key merely because an admin can authorize a catalog.

## Validation

Run `python3 -m unittest discover -s tools -p test_catalog_publish.py` for
admin authorization, input/path refusal, exact-packet binding and real Git
concurrent-update rejection. The PR workflow runs these without signing or
write permissions. Native proof, tamper, admission, rollback and cache tests
live in `crates/app-hub`. Keep production proof and device acceptance claims
separate from those automated checks.
