# 04. Atomic catalog publication, renewal and signing operations Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Keep distribution fresh and recoverable without requiring an app release every two weeks.

**Architecture:** Retain a signed static data plane and add a single-writer release transaction plus scheduled renewal. Uploaded artifacts become immutable before a catalog pointer changes; an isolated signer consumes trusted release records, never developer scripts.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Implemented and verified locally (2026-09-25); deployment/alert routing remain unactivated. **Priority:** P0. **Phase:** A — Correctness. **Relative size:** M (complexity, not a delivery-date estimate).

**Prerequisites:** [01 — Publisher key continuity and signed release identity](2026-09-24-store-01-publisher-key-continuity.md)

**Review coverage:** R04, R15, R16 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Modify | [H/crates/app-hub/src/bin/hub.rs](../../crates/app-hub/src/bin/hub.rs) |
| Modify | [H/crates/app-hub/src/bin/hub-usage.txt](../../crates/app-hub/src/bin/hub-usage.txt) |
| Modify | [H/crates/app-hub/src/signing.rs](../../crates/app-hub/src/signing.rs) |
| Create | [H/crates/app-hub/src/release.rs](../../crates/app-hub/src/release.rs) |
| Modify | [H/crates/app-hub/src/lib.rs](../../crates/app-hub/src/lib.rs) |
| Create | [H/crates/app-hub/tests/release_transactions.rs](../../crates/app-hub/tests/release_transactions.rs) |
| Create | [H/.github/workflows/catalog-renewal.yml](../../.github/workflows/catalog-renewal.yml) |
| Create | [H/docs/operations/catalog.md](../../docs/operations/catalog.md) |
| Modify | [H/crates/app-hub/Cargo.toml](../../crates/app-hub/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
ReleaseTransaction {
    expected_previous_sequence,
    next_sequence,
    immutable_artifact_digests,
    approved_release_ids,
    catalog_digest,
    idempotency_key
}
// Publish artifacts -> verify reachability -> sign -> atomic catalog swap.
// Never decrease the sequence, including rollback or backup recovery.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Add renewal without changing releases

**Touch:** `H/crates/app-hub/src/release.rs`, `H/crates/app-hub/src/bin/hub.rs`, `H/crates/app-hub/tests/release_transactions.rs`.

1. **Write the regression/acceptance case** `renewal_advances_sequence_and_date`: Renew an empty and populated catalog; entries remain byte-equivalent in signed meaning while sequence/date advance. Future/bad dates and stale expected sequences fail.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add hub admin renew with an injected clock and compare-and-swap generation check. Make publisher commands separate from operator commands and update usage.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Make publication transactional

**Touch:** `H/crates/app-hub/src/release.rs`, `H/crates/app-hub/src/bin/hub.rs`.

1. **Write the regression/acceptance case** `crash_at_each_publish_boundary_keeps_valid_catalog`: Inject failures before artifact completion, signing, temporary catalog write and final pointer replacement. Consumers see the old complete generation or the new complete generation.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Store artifacts by digest with create-only writes; make catalog publication a serialized, durable transaction. Do not serve staging paths. Record retries by idempotency key.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Separate signing authority

**Touch:** `H/crates/app-hub/src/signing.rs`, `H/crates/app-hub/src/release.rs`.

1. **Write the regression/acceptance case** `unapproved_release_cannot_be_signed`: Attempt to sign unreviewed/mismatched digests and supply user-controlled reviewer commands. The signer must reject them without executing any app code.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Define a signer interface whose inputs are validated release transactions; production keys stay in a protected signing environment, with no PR checkout. Development uses temporary keys.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Automate renewal and alerts

**Touch:** `H/.github/workflows/catalog-renewal.yml`, `H/docs/operations/catalog.md`.

1. **Write the regression/acceptance case** `renewal_failure_alerts_before_expiry`: Advance a test clock across the 14-day window with no app publishes; successful renewal keeps installs allowed. Simulate missed runs and verify warning/escalation before expiry.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Schedule daily renewal with concurrency protection. Alert at 7 days without success and urgently at 12; verify public catalog signature/age/artifact probes after release. Keep provider credentials outside repository files.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Exercise restore and working-key rotation

**Touch:** `H/crates/app-hub/tests/release_transactions.rs`, `H/docs/operations/catalog.md`.

1. **Write the regression/acceptance case** `restore_never_replays_old_sequence`: Restore a backup older than the observed catalog and attempt to publish; require reconciliation above the durable sequence high-water mark. Rotate to a new anchor-certified working key.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Document backup retention, recovery ownership, healthy/failed signature handling and runbook commands. Planned compromise/anchor recovery is completed in Plan 19.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-hub --test release_transactions
cargo test --locked -p octosense-app-hub --test freshness
```

Expected after implementation: all listed suites pass with zero failures. The release/freshness suites passed as part of 117 Hub/policy tests on 2026-09-25. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [x] Renewal and warning/escalation behavior verified with test clocks; opt-in scheduled workflows provided. Production scheduler and delivered alerts require deployment validation.
- [x] Concurrent/retried publishes cannot lose entries or expose incomplete artifacts.
- [x] Recovery preserves monotonic trust history; signing jobs do not execute submission code.

## Rollout, migration and recovery

First run renewal against a local mirror/test anchor; validate a dry-run transaction before configuring the production signer. Deployment and credential/provider selection are explicit later execution decisions.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(hub): add atomic publication and catalog renewal`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.

## Implementation evidence

- Local renewal/publication slices: commits `11c0483` and `aa6b8d1`; recovery/monitoring checkpoint follows.
- 117 Hub/policy tests pass, including durable interruption/concurrency, recovery, signature/date/freshness, immutable artifact and key rotation cases. Native CLI publication plus real loopback HTTP healthy/damaged-pack probes pass.
- Workflow YAML, bash syntax and actual checksum refusal/success pass locally. Independent final code review found no remaining important issues. The hosted and self-hosted workflows were not executed or activated.
- `CatalogSigner` accepts validated candidates only; the included key-file adapter is an operator implementation. Authenticated service roles and a deployed remote signer are not claimed.
- See [catalog runbook](../operations/catalog.md) for configuration, retained-history recovery and bounded monitoring coverage.
