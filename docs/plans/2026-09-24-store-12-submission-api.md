# 12. Immutable artifact upload and submission API Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Accept an outside developer's immutable signed release and provide reliable status without direct catalog access.

**Architecture:** Separate authenticated upload/submission from validation and signing. Use an idempotent state machine and durable job/outbox records; staging artifacts remain private until verified and approved.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P0. **Phase:** D — Self-service publishing. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [03 — Runnable bundle admission and actionable validation](2026-09-24-store-03-bundle-admission.md); [04 — Atomic catalog publication, renewal and signing operations](2026-09-24-store-04-catalog-release-operations.md); [11 — Publisher authentication, namespace ownership and registry](2026-09-24-store-11-publisher-accounts.md)

**Review coverage:** R04, R08, R16 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/hub-service/src/submissions.rs](../../crates/hub-service/src/submissions.rs) |
| Create | [H/crates/hub-service/src/artifacts.rs](../../crates/hub-service/src/artifacts.rs) |
| Create | [H/crates/hub-service/src/jobs.rs](../../crates/hub-service/src/jobs.rs) |
| Create | [H/crates/hub-service/migrations/002_submissions.sql](../../crates/hub-service/migrations/002_submissions.sql) |
| Create | [H/crates/hub-service/tests/submissions.rs](../../crates/hub-service/tests/submissions.rs) |
| Modify | [H/crates/hub-service/src/lib.rs](../../crates/hub-service/src/lib.rs) |
| Create | [H/docs/api/submissions.md](../../docs/api/submissions.md) |
| Modify | [H/crates/hub-service/Cargo.toml](../../crates/hub-service/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
POST /v1/apps/{app_id}/uploads
PUT  /v1/uploads/{upload_id}/content
POST /v1/apps/{app_id}/submissions   # Idempotency-Key required
GET  /v1/submissions/{submission_id}
# upload -> submitted -> validating -> awaiting_review -> approved
# -> publishing -> published; terminal rejected/failed/cancelled.
# Every transition records actor, digest, previous state and timestamp.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Stage bounded immutable uploads

**Touch:** `H/crates/hub-service/src/artifacts.rs`, `H/crates/hub-service/tests/submissions.rs`.

1. **Write the regression/acceptance case** `oversize_or_digest_mismatch_never_enters_queue`: Upload within limits, one byte over limit, a mismatched digest, path traversal pack and interrupted content. None of the invalid uploads is considered complete.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Stream into quarantined storage with total/file-count/unpacked-size limits; atomically finalize by digest. Begin with a filesystem BlobStore adapter; do not fetch arbitrary publisher URLs server-side.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Create idempotent submissions

**Touch:** `H/crates/hub-service/src/submissions.rs`, `H/crates/hub-service/migrations/002_submissions.sql`.

1. **Write the regression/acceptance case** `retry_creates_one_submission_and_release`: Repeat the same idempotency key/body, then reuse the key with changed bytes and concurrently submit the same app version. Return one result or a conflict, never duplicate releases.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Use transactional app ownership/version uniqueness checks and a request-body digest. Store source repository/commit/provenance plus the immutable artifact and publisher signature references.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Run durable validation jobs

**Touch:** `H/crates/hub-service/src/jobs.rs`, `H/crates/hub-service/src/submissions.rs`.

1. **Write the regression/acceptance case** `worker_restart_does_not_skip_validation`: Crash after claiming a job, after validation and before state persistence. Leases expire safely; replay does not auto-approve or create multiple release records.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add leased jobs and an outbox; workers run the shared validator with no production signing access. Store digest/runtime-bound evidence and structured diagnostics.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Expose status and cancellation

**Touch:** `H/crates/hub-service/src/submissions.rs`, `H/docs/api/submissions.md`.

1. **Write the regression/acceptance case** `publisher_can_read_only_own_submission`: Poll own/other publisher submissions, cancel queued work and race cancellation with publication. Show stable status/error/retry information without leaking artifacts.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Enforce ownership on all status/upload routes. Permit cancellation only before the publication commitment; return a published release reference once commit has occurred.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Connect publication through approved records

**Touch:** `H/crates/hub-service/src/jobs.rs`, `H/crates/hub-service/src/submissions.rs`.

1. **Write the regression/acceptance case** `unapproved_submission_cannot_publish`: Forge an approved field in an upload or skip review via direct API calls. Only a server-recorded approval for the exact evidence/digest can enqueue publication.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Connect Plan 04's signer transaction through an internal authenticated job interface; Plan 13 owns approval creation. Reconcile job status after pointer publication failures.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-hub-service --test submissions
cargo test --locked -p octosense-app-hub --test release_transactions
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Retries, restarts and duplicate uploads do not create conflicting releases.
- [ ] App code/source cannot run in the signer or change its own review evidence.
- [ ] Developers receive machine-readable status and errors throughout the lifecycle.

## Rollout, migration and recovery

Deploy privately with test keys and quotas; use a local blob adapter first and keep its interface suitable for object storage. Publish a versioned OpenAPI contract before shipping the CLI.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(publishing): add durable submission and artifact pipeline`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
