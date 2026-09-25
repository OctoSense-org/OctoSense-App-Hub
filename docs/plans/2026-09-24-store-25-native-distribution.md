# 25. Native application distribution feasibility and implementation Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Determine and implement supported native-package delivery without weakening the Card runtime or assuming all platforms permit the same mechanism.

**Architecture:** Use an explicit package-kind discriminator and separate validators/installers. Native packages are launched through the operating system's package/application model; the plan does not add downloadable Rust libraries to the shell process.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P2 — Optional. **Phase:** G — Optional product tracks. **Relative size:** XL (complexity, not a delivery-date estimate).

**Prerequisites:** [04 — Atomic catalog publication, renewal and signing operations](2026-09-24-store-04-catalog-release-operations.md); [05 — Versioned runtime contracts and release compatibility](2026-09-24-store-05-runtime-compatibility.md); [11 — Publisher authentication, namespace ownership and registry](2026-09-24-store-11-publisher-accounts.md); [12 — Immutable artifact upload and submission API](2026-09-24-store-12-submission-api.md); [13 — Review decisions, publisher feedback and abuse handling](2026-09-24-store-13-review-and-moderation.md); [19 — Publisher teams, transfer, key recovery and trust revocation](2026-09-24-store-19-publisher-recovery-and-trust.md)

**Review coverage:** R28 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/docs/native-distribution/decision.md](../../docs/native-distribution/decision.md) |
| Create | [H/crates/app-native/Cargo.toml](../../crates/app-native/Cargo.toml) |
| Create | [H/crates/app-native/src/lib.rs](../../crates/app-native/src/lib.rs) |
| Create | [H/crates/app-native/src/validator.rs](../../crates/app-native/src/validator.rs) |
| Create | [H/crates/app-native/tests/native_distribution.rs](../../crates/app-native/tests/native_distribution.rs) |
| Modify | [H/Cargo.toml](../../Cargo.toml) |
| Modify | [H/crates/app-policy/src/manifest.rs](../../crates/app-policy/src/manifest.rs) |
| Modify | [H/crates/hub-service/src/submissions.rs](../../crates/hub-service/src/submissions.rs) |
| Modify | [M/src/android_integration.rs](../../../OctoSense-mobile/src/android_integration.rs) |
| Create | [M/apps/app-hub/src/native_install.rs](../../../OctoSense-mobile/apps/app-hub/src/native_install.rs) |
| Modify | [M/apps/app-hub/src/lib.rs](../../../OctoSense-mobile/apps/app-hub/src/lib.rs) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
PackageKind =
  Card { runtime_api, directory_digest }
  | AndroidPackage { package_name, version_code, signing_certificate_digest }
  | DesktopPackage { platform, architecture, package_format, signing_identity }
// Each kind has its own reviewed installer/validator and permissions UX.
// Unsupported native platforms remain unavailable, never treated as Cards.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Make the per-platform decision

**Touch:** `H/docs/native-distribution/decision.md`.

1. **Write the regression/acceptance case** `unsupported_platform_has_no_native_install_path`: Evaluate Android/ROM, desktop and iOS distribution constraints with current official documentation and a real target-device spike. Record allowed channel, installer privileges, signing and rollback limitations.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Compare continued source integration, OS-package distribution and web/Card adaptation. Select Android as the first spike only if product demand justifies it; no universal native-delivery commitment is implied.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Add package-kind admission

**Touch:** `H/crates/app-policy/src/manifest.rs`, `H/crates/app-native/src/validator.rs`.

1. **Write the regression/acceptance case** `apk_cannot_pass_card_validation`: Submit a mislabeled binary, wrong architecture/OS, mismatched signing certificate and incompatible update package. Reject before installation.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Introduce a new wire version/kind with dedicated metadata/signature/provenance requirements and native validation workers. Keep native code out of the Card artifact/parser path.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Install through the OS

**Touch:** `M/apps/app-hub/src/native_install.rs`, `M/src/android_integration.rs`.

1. **Write the regression/acceptance case** `os_install_denial_leaves_store_state_consistent`: Exercise user cancellation, OS permission denial, interrupted installation, signing mismatch and successful install/update/uninstall on a supported device.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Implement a PackageInstaller-style Android adapter using documented OS callbacks; desktop adapters are separate later slices. Respect system prompts and do not silently grant OS privileges.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Integrate identity and lifecycle

**Touch:** `H/crates/app-native/src/lib.rs`, `M/apps/app-hub/src/native_install.rs`.

1. **Write the regression/acceptance case** `native_app_id_maps_to_correct_os_package`: Install two native packages, launch each, reconcile external OS uninstall and query update state. Hub identity must not collide with Card/built-in identities.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Map Hub app/release IDs to OS package identity and signing lineage. Use brokered IPC for optional OctoSense integrations; do not assume Card permissions contain an OS process.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Qualify update and recovery semantics

**Touch:** `H/crates/app-native/tests/native_distribution.rs`, `H/docs/native-distribution/decision.md`.

1. **Write the regression/acceptance case** `native_update_failure_preserves_previous_app`: Test version/signature continuity, changed permissions, missing OS rollback support and store revocation. Specify what revocation can actually enforce on each OS.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Document platform-specific residual behavior, recovery/support flows and review policy. Publish only certified OS/package combinations and keep others disabled.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-native --test native_distribution
(cd ../OctoSense-mobile && cargo test --locked -p octosense-app-hub-app --lib)
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] A recorded per-platform feasibility decision precedes any public native promise.
- [ ] Native packages use dedicated OS installation and signing validation, isolated from Card execution.
- [ ] Unsupported platforms and OS-enforced limitations are visible to publishers and users.

## Rollout, migration and recovery

Explicitly deferred beyond the free Card launch. First deliver the feasibility record/device spike; proceed to each native adapter only when its supported distribution model is selected. Use current Android PackageInstaller and Apple distribution/review documentation rather than historical ADR assumptions.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(native): add explicit OS package distribution adapters`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
