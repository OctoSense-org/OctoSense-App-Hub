# 17. App data migrations, recovery and functional rollback Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Preserve usable user data across app upgrades, failures and explicit rollback.

**Architecture:** Extend the existing bundle replacement transaction to include a versioned app-data migration journal. Migrate a private copy or snapshot and atomically commit the matching bundle/data pair; downgrade only when a declared path is safe.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P1. **Phase:** E — Public free-store reliability. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [02 — Installed-version launch and precise revocation](2026-09-24-store-02-installed-version-launch.md); [05 — Versioned runtime contracts and release compatibility](2026-09-24-store-05-runtime-compatibility.md); [06 — Card application lifecycle, state and effects](2026-09-24-store-06-card-application-runtime.md)

**Review coverage:** R18 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/app-runtime/src/migrations.rs](../../crates/app-runtime/src/migrations.rs) |
| Create | [H/crates/app-runtime/tests/data_migrations.rs](../../crates/app-runtime/tests/data_migrations.rs) |
| Modify | [H/crates/app-policy/src/manifest.rs](../../crates/app-policy/src/manifest.rs) |
| Modify | [M/apps/app-hub/src/catalog.rs](../../../OctoSense-mobile/apps/app-hub/src/catalog.rs) |
| Create | [M/apps/app-hub/src/install_transaction.rs](../../../OctoSense-mobile/apps/app-hub/src/install_transaction.rs) |
| Modify | [M/apps/app-hub/src/lib.rs](../../../OctoSense-mobile/apps/app-hub/src/lib.rs) |
| Create | [H/docs/reference/data-migrations.md](../../docs/reference/data-migrations.md) |
| Modify | [H/crates/app-runtime/src/lib.rs](../../crates/app-runtime/src/lib.rs) |
| Modify | [H/crates/app-runtime/Cargo.toml](../../crates/app-runtime/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
Migration {
  from_schema: 1, to_schema: 2, entrypoint: "migrations/1-to-2.octoscript",
  reversible: false
}
InstallJournal {
  previous_release, target_release, previous_data_schema, target_data_schema,
  phase: "prepared" | "migrated" | "committed", backup_ref
}
// Migration code uses bounded local data APIs; no network/agent/device effects.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Declare valid migration paths

**Touch:** `H/crates/app-policy/src/manifest.rs`, `H/crates/app-runtime/src/migrations.rs`.

1. **Write the regression/acceptance case** `missing_or_cyclic_migration_path_is_refused`: Try schema upgrades with no path, multiple ambiguous paths, cycles and a supported sequential path. Validate before replacing any live data.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add schema/migration metadata and deterministic path selection to v2; ensure referenced code ships under the bundle digest and validation enforces method limits.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Commit bundle and data together

**Touch:** `M/apps/app-hub/src/install_transaction.rs`, `M/apps/app-hub/src/catalog.rs`.

1. **Write the regression/acceptance case** `crash_at_every_phase_restores_consistent_pair`: Interrupt before copy, during migration, after data staging and between final pointer writes. After restart there must be a valid old pair or valid new pair, never new code with half-migrated data.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Introduce a durable installation journal and recoverable swap of code/data references. Reuse the existing staging/rollback behavior rather than replacing it wholesale.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Run bounded migration code

**Touch:** `H/crates/app-runtime/src/migrations.rs`, `H/crates/app-runtime/tests/data_migrations.rs`.

1. **Write the regression/acceptance case** `migration_timeout_keeps_old_app_and_data`: Use a migration that loops, exceeds quota, throws or requests network. Preserve the old release/data and show a useful error.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Run migrations in a restricted runtime with fixed budget and no external effects; reserve/check disk space including backup cost and respect quota policy.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Define rollback and retention

**Touch:** `M/apps/app-hub/src/install_transaction.rs`, `H/docs/reference/data-migrations.md`.

1. **Write the regression/acceptance case** `irreversible_schema_prevents_unsafe_downgrade`: Attempt an old binary against newer irreversible data; require explicit safe recovery instead of loading it. Test a reversible migration and snapshot restore.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Offer compatible rollback or restore a documented pre-update snapshot with clear possible loss of post-update edits. Retain bounded backups, support export, and erase them on uninstall.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-runtime --test data_migrations
(cd ../OctoSense-mobile && cargo test --locked -p octosense-app-hub-app --lib)
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Interrupted or failed migrations preserve a runnable app/data pair.
- [ ] Functional rollback explicitly respects data-schema compatibility.
- [ ] Backup storage is bounded, visible, exportable where appropriate and deleted on uninstall.

## Rollout, migration and recovery

Add schema tracking for existing apps as a legacy schema without modifying their data. Enable migrations only for v2 releases and test with copied data before any device rollout.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(runtime): add transactional app data migrations`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
