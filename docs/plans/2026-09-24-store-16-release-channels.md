# 16. Private testing, release channels and controlled rollouts Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Test and release the same approved artifact progressively while preserving safe installed versions and private audiences.

**Architecture:** Represent releases independently from channel assignments and rollout policy. Public stable catalogs stay static; private beta catalogs/artifacts require authenticated membership and have separate trust/cache namespaces.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P1. **Phase:** E — Public free-store reliability. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [02 — Installed-version launch and precise revocation](2026-09-24-store-02-installed-version-launch.md); [04 — Atomic catalog publication, renewal and signing operations](2026-09-24-store-04-catalog-release-operations.md); [05 — Versioned runtime contracts and release compatibility](2026-09-24-store-05-runtime-compatibility.md); [12 — Immutable artifact upload and submission API](2026-09-24-store-12-submission-api.md); [13 — Review decisions, publisher feedback and abuse handling](2026-09-24-store-13-review-and-moderation.md); [14 — One-command publishing and GitHub release action](2026-09-24-store-14-publish-cli-and-ci.md); [17 — App data migrations, recovery and functional rollback](2026-09-24-store-17-data-migrations.md)

**Review coverage:** R11, R17, R26 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/hub-service/src/channels.rs](../../crates/hub-service/src/channels.rs) |
| Create | [H/crates/hub-service/src/rollouts.rs](../../crates/hub-service/src/rollouts.rs) |
| Create | [H/crates/hub-service/migrations/004_channels.sql](../../crates/hub-service/migrations/004_channels.sql) |
| Create | [H/crates/hub-service/tests/release_channels.rs](../../crates/hub-service/tests/release_channels.rs) |
| Modify | [H/crates/app-hub/src/index.rs](../../crates/app-hub/src/index.rs) |
| Modify | [H/crates/app-hub/src/client.rs](../../crates/app-hub/src/client.rs) |
| Modify | [H/crates/app-hub/src/publish.rs](../../crates/app-hub/src/publish.rs) |
| Modify | [M/apps/app-hub/src/catalog.rs](../../../OctoSense-mobile/apps/app-hub/src/catalog.rs) |
| Modify | [M/apps/app-hub/src/view.rs](../../../OctoSense-mobile/apps/app-hub/src/view.rs) |
| Create | [H/docs/RELEASING.md](../../docs/RELEASING.md) |
| Modify | [H/crates/hub-service/src/lib.rs](../../crates/hub-service/src/lib.rs) |
| Modify | [H/crates/hub-service/Cargo.toml](../../crates/hub-service/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
hub publish --channel beta
hub testers invite <app> <account>
hub promote <app>@<version> --to stable --rollout 10
hub releases halt <release>
hub releases rollback <release> --to <approved-release>
# Assignment: channel + audience + compatible target + rollout fraction.
# Promotion reuses an immutable approved digest; it never rebuilds.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Keep beta private

**Touch:** `H/crates/hub-service/src/channels.rs`, `H/crates/hub-service/tests/release_channels.rs`.

1. **Write the regression/acceptance case** `nonmember_cannot_fetch_beta_catalog_or_artifact`: Request beta metadata/artifacts as a member, nonmember, removed member and expired token. Verify stable caches and discovery never include beta data.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Store tester memberships/invitations and serve private artifacts through authenticated access. Define beta entitlement expiry/offline behavior separately from normal installed-app availability.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Promote an immutable reviewed artifact

**Touch:** `H/crates/hub-service/src/channels.rs`, `H/crates/app-hub/src/publish.rs`.

1. **Write the regression/acceptance case** `promotion_preserves_digest_and_review`: Promote beta to stable and compare artifact hash, manifest and validation evidence. A changed bundle cannot reuse that review or version.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add channel-assignment transactions and CLI promotion/status. Preserve public old-version records needed by installed clients and require compatibility checks per target.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Make rollout cohorts stable

**Touch:** `H/crates/hub-service/src/rollouts.rs`, `H/crates/app-hub/src/client.rs`.

1. **Write the regression/acceptance case** `same_device_stays_in_same_rollout_cohort`: Run fixed installation identifiers through 1%, 10%, 50%,100% allocation; increasing fractions only adds eligible devices. Reboots and refreshes do not reshuffle cohorts.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Use a stable local random install ID and signed rollout seed/policy, without a cross-app tracking identifier. Support scheduled release with server clock and explicit timezone; allow immediate halt.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Rollback behavior and data safely

**Touch:** `H/crates/hub-service/src/rollouts.rs`, `M/apps/app-hub/src/catalog.rs`.

1. **Write the regression/acceptance case** `bad_release_rolls_back_without_catalog_replay`: Promote a bad v2, halt it, and offer an approved prior build through a new signed catalog sequence. Verify Plan 17's data compatibility determines whether rollback is safe.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Rollback changes target policy, not app bytes or signature history. Retain the installed app if data cannot be safely rolled back and provide repair/export instructions. Automatic halt based on health is enabled only after Plan 20 supplies trustworthy signals.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Expose channel and release state

**Touch:** `M/apps/app-hub/src/view.rs`, `H/docs/RELEASING.md`.

1. **Write the regression/acceptance case** `user_can_leave_beta_without_losing_data`: Join/leave beta and handle a stable channel behind the installed beta version. Explain migration/downgrade constraints and require deliberate consent where needed.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add channel controls, release status, rollout progress and release-note selection; document version retirement versus explicit revocation.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-hub-service --test release_channels
cargo test --locked -p octosense-app-hub --test compatibility
(cd ../OctoSense-mobile && cargo test --locked -p octosense-app-hub-app --lib)
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Private beta artifacts and metadata are inaccessible to nonmembers.
- [ ] Promotion uses the reviewed bytes; rollout/halt/rollback preserve monotonic catalog sequences.
- [ ] Rollback cannot corrupt data or silently change permissions.

## Rollout, migration and recovery

Launch closed beta and manual promotion first; then deterministic staged rollout. Enable automated health-based decisions only after the diagnostics plan establishes sufficient samples and a manual override.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(releases): add beta channels and staged promotion`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
