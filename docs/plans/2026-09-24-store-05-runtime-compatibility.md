# 05. Versioned runtime contracts and release compatibility Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Select installable releases by runtime, platform and capabilities while preserving verification of legacy signed catalogs.

**Architecture:** Introduce explicit wire-versioned types and separate installed-release identity from release ordering. Parse and verify v1 with its original canonical representation; introduce v2 catalog/manifest contracts and bootstrap compatible clients before exposing v2-only apps.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P0. **Phase:** B — Application platform. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [02 — Installed-version launch and precise revocation](2026-09-24-store-02-installed-version-launch.md)

**Review coverage:** R10, R11 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Modify | [H/crates/app-policy/src/manifest.rs](../../crates/app-policy/src/manifest.rs) |
| Modify | [H/crates/app-policy/src/policy.rs](../../crates/app-policy/src/policy.rs) |
| Modify | [H/crates/app-hub/src/index.rs](../../crates/app-hub/src/index.rs) |
| Modify | [H/crates/app-hub/src/client.rs](../../crates/app-hub/src/client.rs) |
| Create | [H/crates/app-policy/src/compatibility.rs](../../crates/app-policy/src/compatibility.rs) |
| Modify | [H/crates/app-policy/src/lib.rs](../../crates/app-policy/src/lib.rs) |
| Create | [H/crates/app-hub/tests/compatibility.rs](../../crates/app-hub/tests/compatibility.rs) |
| Create | [H/docs/reference/manifest-v2.md](../../docs/reference/manifest-v2.md) |
| Modify | [M/apps/app-hub/src/catalog.rs](../../../OctoSense-mobile/apps/app-hub/src/catalog.rs) |
| Modify | [M/apps/app-hub/src/view.rs](../../../OctoSense-mobile/apps/app-hub/src/view.rs) |
| Modify | [H/crates/app-policy/Cargo.toml](../../crates/app-policy/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
{
  "schema": 2,
  "id": "org.example.notes",
  "version": "1.2.0",
  "release_number": 12,
  "runtime": {"api": "1", "min_build": 3, "platforms": ["android","macos"]},
  "requires": ["storage.kv@1"],
  "entrypoints": {"ui": "page.card", "logic": "app.octoscript"},
  "data_schema": 1
}
// Abbreviated v2 example; existing integrity/listing/capability fields remain.
// Display version is SemVer for v2; monotonic release_number orders releases.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Freeze v1 signing compatibility

**Touch:** `H/crates/app-policy/src/manifest.rs`, `H/crates/app-hub/src/index.rs`, `H/crates/app-hub/tests/compatibility.rs`.

1. **Write the regression/acceptance case** `v1_golden_catalog_signature_still_verifies`: Capture canonical v1 manifests/catalogs, signatures and unsupported-schema cases. Deserializing then verifying must not add default fields to the original signed payload.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Use versioned wire structs rather than adding serde-default fields to an old signed type. Specify v1 and v2 signing bytes with golden fixtures.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Define deterministic compatibility

**Touch:** `H/crates/app-policy/src/compatibility.rs`, `H/crates/app-hub/src/client.rs`.

1. **Write the regression/acceptance case** `newest_compatible_release_is_selected`: Offer a newer incompatible release and an older compatible one; only the latter is installable. Test missing required features, wrong OS, too-old runtime and numeric release ordering.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add RuntimeDescriptor/CompatibilityResult and a resolver with reasons. Declare tested platforms separately from requirements; SDK constraints cannot self-grant unavailable capabilities.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Define the v2 upgrade path

**Touch:** `H/crates/app-hub/src/index.rs`, `H/docs/reference/manifest-v2.md`.

1. **Write the regression/acceptance case** `v1_client_receives_only_v1_catalog`: Exercise a legacy client against its original endpoint and a v2 client against a v2 endpoint. Neither may accept unknown semantics under a v1 signature.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Publish versioned catalogs concurrently; keep catalog.json as v1 until supported clients have migrated. Use separate sequence floors per trust domain/schema and never infer support from a listing label.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Expose compatibility to users and authors

**Touch:** `M/apps/app-hub/src/catalog.rs`, `M/apps/app-hub/src/view.rs`, `H/docs/reference/manifest-v2.md`.

1. **Write the regression/acceptance case** `incompatible_listing_has_reason_and_no_install`: Show an incompatible app as unavailable with a concrete runtime/platform requirement; installed approved v1 still opens. CLI validation reports the same reason.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Carry the compatibility result into the mobile model and hub check JSON report. Document the supported runtime matrix with release-managed values.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-hub --test compatibility
cargo test --locked -p octosense-app-policy
(cd ../OctoSense-mobile && cargo test --locked -p octosense-app-hub-app --lib)
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Adding v2 does not invalidate existing signatures or silently widen older clients' behavior.
- [ ] Release order is deterministic and downgrade selection requires an explicit authorized policy.
- [ ] Unsupported OS/runtime/features produce useful install refusals before download.

## Rollout, migration and recovery

Ship readers before writers. Keep legacy artifacts and catalogs available during migration; track installed-client adoption before retiring a schema.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(hub): version runtime and compatibility contracts`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
