# 02. Installed-version launch and precise revocation Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Allow an approved installed version to keep running when a newer version is published, while enforcing revocation of the exact installed release.

**Architecture:** Separate installed-release lookup from offered-update lookup in the shared Store. The mobile model must represent can_open and update_available independently, and both reference and installed hosts must use the same launch decision.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P0. **Phase:** A — Correctness. **Relative size:** M (complexity, not a delivery-date estimate).

**Prerequisites:** None; can start against the reviewed baseline.

**Review coverage:** R02, R17 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Modify | [H/crates/app-hub/src/client.rs](../../crates/app-hub/src/client.rs) |
| Modify | [H/crates/app-hub/src/index.rs](../../crates/app-hub/src/index.rs) |
| Modify | [H/crates/appstore/src/cardapp.rs](../../crates/appstore/src/cardapp.rs) |
| Modify | [H/crates/appstore/src/lib.rs](../../crates/appstore/src/lib.rs) |
| Modify | [H/crates/appstore/src/ui.rs](../../crates/appstore/src/ui.rs) |
| Create | [H/crates/app-hub/tests/installed_release.rs](../../crates/app-hub/tests/installed_release.rs) |
| Modify | [M/apps/app-hub/src/catalog.rs](../../../OctoSense-mobile/apps/app-hub/src/catalog.rs) |
| Modify | [M/apps/app-hub/src/view.rs](../../../OctoSense-mobile/apps/app-hub/src/view.rs) |
| Modify | [M/src/main.rs](../../../OctoSense-mobile/src/main.rs) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
pub struct AppAvailability {
    pub installed_version: Option<String>,
    pub can_open: bool,
    pub update_version: Option<String>,
    pub unavailable_reason: Option<String>,
}
// Catalog release lookup: (app_id, installed_version, reviewed digest).
// Revocation and permission resolution use that exact release.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Lock in version-specific behavior

**Touch:** `H/crates/app-hub/tests/installed_release.rs`, `H/crates/app-hub/src/client.rs`.

1. **Write the regression/acceptance case** `offered_v1_runs_after_v2_publish`: Install signed v1; accept a catalog with offered v1 and v2; assert v1 still opens with v1 permissions. Withdraw v1 only and assert denial; withdraw v2 only and assert v1 still opens.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add release lookup by installed version and verify its installed manifest/content identity against the admitted release before returning policy. Keep missing catalog/release errors explicit.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Represent update and launch independently

**Touch:** `M/apps/app-hub/src/catalog.rs`, `M/apps/app-hub/src/view.rs`, `H/crates/appstore/src/ui.rs`.

1. **Write the regression/acceptance case** `update_available_does_not_hide_open`: Present an installed v1 with v2 offered, then inspect view actions. Both Open and Update must be possible; install/update consent must still bind to v2.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Replace the mutually exclusive Installed/UpdateAvailable presentation assumption with a lifecycle value that carries both facts. Adapt legacy and current UI actions.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Enforce changed-byte and withdrawal behavior

**Touch:** `H/crates/appstore/src/cardapp.rs`, `M/apps/app-hub/src/catalog.rs`, `M/src/main.rs`.

1. **Write the regression/acceptance case** `launch_rejects_modified_installed_bytes`: Modify installed manifest permissions or content after installation; opening must fail. Revoke an exact active version and verify that a refresh prevents further execution according to the declared revocation policy.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Verify outside the UI thread where hashing is needed; retain a bounded launch check and invalidate verified results on replacement. Notify the shell to close instances of a newly revoked release, without closing unaffected versions/apps.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Cover offline and missing-release semantics

**Touch:** `H/crates/app-hub/src/client.rs`, `M/apps/app-hub/src/catalog.rs`.

1. **Write the regression/acceptance case** `offline_approved_version_remains_openable`: Use a cached verified catalog with an approved installed release while the origin is unreachable/stale. It remains runnable; new installs stay subject to freshness. Missing or explicitly revoked versions show actionable reasons.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Document offline revocation limits. Do not silently treat a missing release as approved or use the latest release's broader permissions.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-hub --test installed_release
(cd ../OctoSense-mobile && cargo test --locked -p octosense-app-hub-app --lib)
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Normal v2 publication does not disable approved v1 or silently grant v2 permissions.
- [ ] Explicit withdrawal targets the correct release; running revoked instances are handled consistently.
- [ ] Mobile install staging/data-preservation and consent tests remain green.

## Rollout, migration and recovery

Ship as a compatible client fix before introducing channels. Release shared Hub changes and update the mobile pinned Hub revision before claiming device coverage.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `fix(store): separate installed release launch from updates`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.

