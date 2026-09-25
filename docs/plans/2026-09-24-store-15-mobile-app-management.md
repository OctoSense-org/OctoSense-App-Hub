# 15. Mobile uninstall, complete listings and user controls Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make installed-app management complete in the current mobile App Hub and expose listing information already supplied by developers.

**Architecture:** Reuse Store removal and the shell's per-app lifecycle identities. Serialize uninstall with install/update and make removal recoverable; carry existing listing fields to mobile without changing the signed schema.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P0. **Phase:** C — Developer workflow. **Relative size:** M (complexity, not a delivery-date estimate).

**Prerequisites:** [02 — Installed-version launch and precise revocation](2026-09-24-store-02-installed-version-launch.md)

**Review coverage:** R13, R14, R19 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Modify | [M/apps/app-hub/src/catalog.rs](../../../OctoSense-mobile/apps/app-hub/src/catalog.rs) |
| Modify | [M/apps/app-hub/src/view.rs](../../../OctoSense-mobile/apps/app-hub/src/view.rs) |
| Modify | [M/apps/app-hub/src/lib.rs](../../../OctoSense-mobile/apps/app-hub/src/lib.rs) |
| Modify | [M/src/main.rs](../../../OctoSense-mobile/src/main.rs) |
| Modify | [M/src/apps.rs](../../../OctoSense-mobile/src/apps.rs) |
| Modify | [H/crates/app-hub/src/client.rs](../../crates/app-hub/src/client.rs) |
| Create | [H/crates/app-hub/tests/removal.rs](../../crates/app-hub/tests/removal.rs) |
| Modify | [M/apps/app-hub/README.md](../../../OctoSense-mobile/apps/app-hub/README.md) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
AppManagementAction =
    Uninstall { app_id, delete_local_data: true }
  | OpenSupport { app_id }
  | OpenPrivacyPolicy { app_id }
  | ReportApp { app_id, release_id }
// Show: publisher, support/privacy links, age rating, license,
// tested platforms, compatible status and release notes.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Remove from the current UI

**Touch:** `M/apps/app-hub/src/view.rs`, `M/apps/app-hub/src/catalog.rs`, `M/src/main.rs`.

1. **Write the regression/acceptance case** `uninstall_removes_app_data_and_launcher_identity`: Install a real fixture, save state, uninstall from Library, and verify bundle/data/icon/launcher/recents entries disappear. Built-in preview apps are not uninstall candidates.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add a plainly worded confirmation stating data deletion, then dispatch backend removal and close only that app's instances. Distinguish closing a window from uninstalling.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Resolve install/remove races

**Touch:** `H/crates/app-hub/src/client.rs`, `M/apps/app-hub/src/catalog.rs`, `H/crates/app-hub/tests/removal.rs`.

1. **Write the regression/acceptance case** `uninstall_cannot_be_undone_by_late_install_reply`: Start an update and request uninstall; restart between close, directory removal and shell notification. No completion callback resurrects the app.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Use per-app operation identity/tombstone plus existing I/O coordination. Return busy/queued state consistently and recover partial removal on startup; wipe keys/tokens owned by the app once their SDK exists.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Expose existing listing fields

**Touch:** `M/apps/app-hub/src/catalog.rs`, `M/apps/app-hub/src/view.rs`.

1. **Write the regression/acceptance case** `listing_links_and_rating_survive_presentation`: Present all listing fields including empty/invalid optional links. Verify safe URL handling and platform/age/license visibility in the detail view.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Carry metadata into Entry; open external links through a controlled user action, not automatic navigation. Preserve manifest-derived permissions/privacy summary alongside publisher privacy terms.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Connect the reporting action

**Touch:** `M/apps/app-hub/src/view.rs`, `M/apps/app-hub/README.md`.

1. **Write the regression/acceptance case** `report_action_preserves_app_release_context`: Activate Report for installed/withdrawn apps; pass app/release identity to the reporting flow once Plan 13 exists. Before that, provide a documented support route rather than a dead control.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add accessible labels, keyboard/back behavior and success/failure feedback. Report service integration is a follow-up activation after Plan 13, independent of shipping uninstall.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-hub --test removal
(cd ../OctoSense-mobile && cargo test --locked -p octosense-app-hub-app --lib)
(cd ../OctoSense-mobile && cargo test --locked --bin octosense --features mobile-only,app-hub app_hub_lifecycle_tests)
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Users can uninstall a downloaded app and knowingly delete its local state.
- [ ] Install/update races cannot restore an uninstalled app or delete another app's data.
- [ ] Support/privacy links and existing publisher metadata are usable in mobile details.

## Rollout, migration and recovery

Ship alongside the early client correctness fixes; this does not need the submission service. Link reporting when Plan 13 is deployed, keeping the UI useful meanwhile.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(app-hub): add mobile uninstall and complete listing details`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.

