# 18. Download queue, resume, cancellation and automatic updates Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make installation and updating reliable on slow or interrupted connections while keeping users in control.

**Architecture:** Persist a per-app operation queue and use immutable pack objects with transport digests for resumable downloads. Reuse existing staged verification and consent checks; a resumed transfer has no authority until the complete package is verified.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P1. **Phase:** E — Public free-store reliability. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [02 — Installed-version launch and precise revocation](2026-09-24-store-02-installed-version-launch.md); [04 — Atomic catalog publication, renewal and signing operations](2026-09-24-store-04-catalog-release-operations.md); [05 — Versioned runtime contracts and release compatibility](2026-09-24-store-05-runtime-compatibility.md); [15 — Mobile uninstall, complete listings and user controls](2026-09-24-store-15-mobile-app-management.md); [17 — App data migrations, recovery and functional rollback](2026-09-24-store-17-data-migrations.md)

**Review coverage:** R20, R23 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/app-hub/src/download.rs](../../crates/app-hub/src/download.rs) |
| Modify | [H/crates/app-hub/src/remote.rs](../../crates/app-hub/src/remote.rs) |
| Modify | [H/crates/app-hub/src/lib.rs](../../crates/app-hub/src/lib.rs) |
| Create | [H/crates/app-hub/tests/downloads.rs](../../crates/app-hub/tests/downloads.rs) |
| Modify | [H/crates/app-hub/src/index.rs](../../crates/app-hub/src/index.rs) |
| Create | [M/apps/app-hub/src/operations.rs](../../../OctoSense-mobile/apps/app-hub/src/operations.rs) |
| Modify | [M/apps/app-hub/src/lib.rs](../../../OctoSense-mobile/apps/app-hub/src/lib.rs) |
| Modify | [M/apps/app-hub/src/catalog.rs](../../../OctoSense-mobile/apps/app-hub/src/catalog.rs) |
| Modify | [M/apps/app-hub/src/view.rs](../../../OctoSense-mobile/apps/app-hub/src/view.rs) |
| Modify | [H/crates/app-hub/src/client.rs](../../crates/app-hub/src/client.rs) |
| Modify | [H/crates/app-hub/Cargo.toml](../../crates/app-hub/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
Operation {
  id, app_id, target_release, expected_transport_digest, received_bytes,
  state: "queued" | "downloading" | "verifying" | "installing" | "done"
}
// Pause/cancel are persisted. Auto-update settings: enabled, network policy,
// time window, charging preference; permission increases always require consent.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Queue and recover operations

**Touch:** `M/apps/app-hub/src/operations.rs`, `M/apps/app-hub/src/catalog.rs`.

1. **Write the regression/acceptance case** `restart_resumes_one_operation_per_app`: Queue multiple apps, restart during a transfer, and queue duplicate updates. Enforce ordering and a bounded concurrency limit without blocking catalog browsing.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Persist operations keyed by app/release and expose progress/error/retry status. Coordinate with uninstall tombstones and catalog refresh rather than holding one global lock through network I/O.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Resume immutable downloads safely

**Touch:** `H/crates/app-hub/src/download.rs`, `H/crates/app-hub/src/remote.rs`, `H/crates/app-hub/tests/downloads.rs`.

1. **Write the regression/acceptance case** `changed_etag_or_digest_restarts_download`: Use a test HTTP server with Range support, no Range support, changed ETag, wrong length, truncated body and timeout. Verify safe restart and final hash checking.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Download bytes to bounded temp files; require matching object identity before append. Fall back to full download when range is unsupported. Keep base64 pack compatibility initially; Plan 23 adds a more efficient transport.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Make cancellation predictable

**Touch:** `M/apps/app-hub/src/operations.rs`, `M/apps/app-hub/src/view.rs`.

1. **Write the regression/acceptance case** `cancel_before_commit_never_installs_app`: Cancel queued, downloading, verifying and committing operations. Before commit nothing installs; after commit report completed instead of pretending it was undone.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add operation cancellation tokens and cleanup/recovery markers; do not interrupt an atomic data migration in an inconsistent phase. Provide pause/resume/cancel UI with actual byte progress.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Schedule consent-aware updates

**Touch:** `M/apps/app-hub/src/operations.rs`, `H/crates/app-hub/src/client.rs`.

1. **Write the regression/acceptance case** `auto_update_pauses_on_new_permissions`: Offer an update with unchanged grants, widened hosts, added storage/tool access and a revoked target. Only eligible unchanged-grant releases auto-install under user settings.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Compute a semantic permission delta from resolved policies, not text equality. Honor metered network/battery/platform background limitations; schedule within supported host lifecycle rather than assuming an always-running daemon.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Recheck trust at commit

**Touch:** `M/apps/app-hub/src/catalog.rs`, `H/crates/app-hub/tests/downloads.rs`.

1. **Write the regression/acceptance case** `withdrawal_during_download_prevents_commit`: Withdraw/change target permissions while bytes arrive, then simulate offline/stale catalog before commit. Existing good installs remain intact.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Retain the mobile pre/post-download consent and revocation rechecks; invalidate the queued operation when its release assignment changes and require renewed consent.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-hub --test downloads
(cd ../OctoSense-mobile && cargo test --locked -p octosense-app-hub-app --lib)
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Downloads resume or restart safely and show real progress; cancellation survives restart.
- [ ] Auto-updates never silently widen grants and obey user connectivity settings.
- [ ] Failed/cancelled downloads leave working installed code and data intact.

## Rollout, migration and recovery

Start with manual queued downloads and cancellation; add resume and then opt-in automatic updates after device/network fault tests. Keep a full-download fallback.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(store): add resilient downloads and update scheduling`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
