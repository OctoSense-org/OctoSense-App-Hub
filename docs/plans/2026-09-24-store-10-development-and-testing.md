# 10. Development loop, signed device preview and editor tooling Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Give developers fast reload, useful errors and tests against the same signed artifact path used in production.

**Architecture:** Make hub dev/test thin clients of the shared runtime/validator. A separate developer-mode trust store accepts explicit development identities without changing the production anchor or catalog.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P0. **Phase:** C — Developer workflow. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [03 — Runnable bundle admission and actionable validation](2026-09-24-store-03-bundle-admission.md); [06 — Card application lifecycle, state and effects](2026-09-24-store-06-card-application-runtime.md); [07 — Host services, capability matrix and structured network policy](2026-09-24-store-07-host-services.md); [09 — Installable SDK, pinned runtimes and runnable starters](2026-09-24-store-09-sdk-and-starters.md)

**Review coverage:** R05, R26, R27 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/app-hub/src/dev.rs](../../crates/app-hub/src/dev.rs) |
| Create | [H/crates/app-hub/tests/dev_workflow.rs](../../crates/app-hub/tests/dev_workflow.rs) |
| Modify | [H/crates/app-hub/src/bin/hub.rs](../../crates/app-hub/src/bin/hub.rs) |
| Modify | [H/crates/card-host/src/main.rs](../../crates/card-host/src/main.rs) |
| Create | [H/crates/card-host/tests/signed_preview.rs](../../crates/card-host/tests/signed_preview.rs) |
| Create | [H/schemas/app.schema.json](../../schemas/app.schema.json) |
| Create | [H/schemas/listing.schema.json](../../schemas/listing.schema.json) |
| Create | [H/templates/notes/.vscode/settings.json](../../templates/notes/.vscode/settings.json) |
| Create | [M/apps/app-hub/src/developer_preview.rs](../../../OctoSense-mobile/apps/app-hub/src/developer_preview.rs) |
| Modify | [M/apps/app-hub/src/lib.rs](../../../OctoSense-mobile/apps/app-hub/src/lib.rs) |
| Create | [H/docs/TESTING.md](../../docs/TESTING.md) |
| Modify | [H/crates/app-hub/src/lib.rs](../../crates/app-hub/src/lib.rs) |
| Modify | [H/crates/app-hub/Cargo.toml](../../crates/app-hub/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
hub dev --device <paired-device>
hub test --target android --json
hub preview --bundle build/app.bundle --publisher-key <id=public-key>
# Diagnostics: code, source_path, line/column, severity, suggested_fix.
# Test reports: bundle_digest, runtime_build, platform, assertions, captures.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Add reload without destroying state

**Touch:** `H/crates/app-hub/src/dev.rs`, `H/crates/app-hub/tests/dev_workflow.rs`.

1. **Write the regression/acceptance case** `valid_edit_reloads_invalid_edit_keeps_last_good`: Change a UI label then introduce a syntax error. Reload the valid edit, keep user state and retain the last working app for the invalid edit with an actionable diagnostic.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Watch author sources only, debounce builds, stage the next bundle and reload on successful validation. Reset state requires an explicit separate command.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Run a signed release locally

**Touch:** `H/crates/card-host/src/main.rs`, `H/crates/card-host/tests/signed_preview.rs`.

1. **Write the regression/acceptance case** `signed_preview_uses_supplied_trusted_key`: Supply a correct, missing and wrong publisher public key to card-host. Correct signed bundles run unchanged; wrong/missing trust is refused.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add explicit publisher-key/trust-store input to reference host; share verification with installed runtime. Keep production trust roots immutable when using development mode.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Provide reproducible app tests

**Touch:** `H/crates/app-hub/src/dev.rs`, `H/docs/TESTING.md`.

1. **Write the regression/acceptance case** `interaction_tests_capture_state_and_screen`: Run declared app interactions for save/restart, denied permission, offline behavior and layout on desktop/device. A screenshot alone must not satisfy interaction assertions.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Expose hub test using validator and native instrumentation. Store reports/captures outside bundle content and provide deterministic fixtures/clock/service mocks.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Pair a device without publishing

**Touch:** `M/apps/app-hub/src/developer_preview.rs`, `H/crates/app-hub/src/dev.rs`.

1. **Write the regression/acceptance case** `development_bundle_never_appears_in_public_library`: Pair a local developer, install a preview bundle in a separate namespace/data root, expire/revoke pairing and verify production apps/catalog remain unaffected.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Use an expiring pairing token and a developer-only preview screen; provide USB first with an explicit network preview option. Show preview identity and reset/remove controls.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Add author-time feedback

**Touch:** `H/schemas/app.schema.json`, `H/schemas/listing.schema.json`, `H/templates/notes/.vscode/settings.json`.

1. **Write the regression/acceptance case** `schema_completion_matches_manifest_contract`: Validate schema fixtures with unknown permissions, wrong types and obsolete runtime versions; errors should match CLI validation categories.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Generate JSON schemas from the contract or verify drift against fixtures, supply editor settings and concise code examples. Avoid a bespoke IDE until this baseline is measured.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-hub --test dev_workflow
cargo test --locked -p octosense-card-host --test signed_preview
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Edit, reload, inspect and run tests work from an external app directory.
- [ ] The exact signed release bundle is previewable with explicit publisher trust.
- [ ] Device preview is isolated from production identities, data and anchors.

## Rollout, migration and recovery

Release desktop dev loop first, then USB preview. Document supported platforms and manual native/device verification commands before declaring parity.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(sdk): add signed previews and developer test loop`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
