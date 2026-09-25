# 03. Runnable bundle admission and actionable validation Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prevent incomplete, malformed or non-runnable bundles from passing public admission.

**Architecture:** Keep cheap structural checks in the headless Hub crate and run Card preparation/render checks in a separate validator process. The signer will require a digest-bound validation result; the core policy crate must not acquire a graphical runtime dependency.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** In progress (2026-09-25); structural admission is the first slice. **Priority:** P0. **Phase:** A — Correctness. **Relative size:** M (complexity, not a delivery-date estimate).

**Prerequisites:** [01 — Publisher key continuity and signed release identity](2026-09-24-store-01-publisher-key-continuity.md)

**Review coverage:** R03, R08, R09 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Modify | [H/crates/app-hub/src/gate.rs](../../crates/app-hub/src/gate.rs) |
| Modify | [H/crates/app-hub/src/scan.rs](../../crates/app-hub/src/scan.rs) |
| Modify | [H/crates/app-hub/src/pack.rs](../../crates/app-hub/src/pack.rs) |
| Modify | [H/Cargo.toml](../../Cargo.toml) |
| Create | [H/crates/app-validator/Cargo.toml](../../crates/app-validator/Cargo.toml) |
| Create | [H/crates/app-validator/src/main.rs](../../crates/app-validator/src/main.rs) |
| Create | [H/crates/app-validator/src/lib.rs](../../crates/app-validator/src/lib.rs) |
| Create | [H/crates/app-hub/tests/bundle_admission.rs](../../crates/app-hub/tests/bundle_admission.rs) |
| Create | [H/crates/app-validator/tests/runtime_validation.rs](../../crates/app-validator/tests/runtime_validation.rs) |
| Modify | [H/docs/ICONS.md](../../docs/ICONS.md) |
| Modify | [H/docs/PUBLISHING.md](../../docs/PUBLISHING.md) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
{
  "schema": 1,
  "bundle_digest": "<blake3>",
  "runtime_build": "<pinned-runtime>",
  "checks": [
    {"code":"entrypoint.missing","path":"page.card","severity":"error",
     "message":"Add the app entrypoint before publishing."}
  ],
  "passed": false
}
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Reject the known incomplete fixture

**Touch:** `H/crates/app-hub/tests/bundle_admission.rs`, `H/crates/app-hub/src/gate.rs`.

1. **Write the regression/acceptance case** `missing_card_kit_and_fake_png_fail`: Promote the review probe's no-page.card/no-kit/fake-PNG fixture into a regression case. Independently remove each required file and corrupt screenshot bytes so diagnostics identify the actual cause.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Validate required shape, regular UTF-8 text files, Card/data syntax, kit dependency closure and decoded image type/dimensions. Bound file count, nesting, input and decoded image sizes before allocating.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Share rendering preparation

**Touch:** `H/crates/app-validator/src/lib.rs`, `H/crates/app-validator/src/main.rs`, `H/crates/app-validator/tests/runtime_validation.rs`.

1. **Write the regression/acceptance case** `valid_template_prepares_with_pinned_runtime`: Run a genuine Card fixture through the exact preparation path used by card-host; introduce missing kit imports and unresolved asset paths and require specific failures.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Create app-validator with a library API and a process wrapper around the shared preparation function. Return machine-readable diagnostics and runtime identity, with timeout/resource limits supplied by its caller.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Make smoke evidence bind to reviewed bytes

**Touch:** `H/crates/app-hub/src/scan.rs`, `H/crates/app-validator/src/main.rs`.

1. **Write the regression/acceptance case** `changed_bundle_invalidates_validation`: Change one file after validation, or return a result for another runtime. Publication must not accept the stale report. Test hung/crashed validators and reject automatic pass.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Bind validation evidence to digest/runtime/check version; run a native launch and minimal declared interaction in an isolated runner. The later review service decides on that evidence, not a caller's passed boolean.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Make resource diagnostics structural

**Touch:** `H/crates/app-hub/src/gate.rs`, `H/crates/app-validator/src/lib.rs`.

1. **Write the regression/acceptance case** `displayed_url_is_not_a_resource_reference`: Classify literal displayed URLs separately from asset/resource/effect references. Tests still reject escaping paths, remote asset loads and unresolved imports.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add a typed resource inventory and structured error codes. Retain existing deny behavior where the runtime cannot yet enforce the distinction; Plan 07 activates the safer relaxed policy after network-path conformance passes.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Align local and server checks

**Touch:** `H/crates/app-hub/src/gate.rs`, `H/docs/ICONS.md`, `H/docs/PUBLISHING.md`.

1. **Write the regression/acceptance case** `local_server_reports_match`: Validate the same immutable fixture through the CLI and worker interfaces; check stable codes/paths, valid screenshot decode and no generated files inside the bundle.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Expose the same report format to hub check/test and submission validation. Correct documentation so package, runtime and visual-review guarantees remain distinct.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-hub --test bundle_admission
cargo test --locked -p octosense-app-validator --test runtime_validation
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Missing entrypoints, missing kits, corrupt images and unresolvable local resources are refused with useful diagnostics.
- [ ] A real signed starter passes both structural and native checks using the same reviewed bytes.
- [ ] A failed, stale or missing runtime validation report never becomes an automatic approval.

## Rollout, migration and recovery

Add required shape/image checks first. Introduce the runtime validator as a separate CI worker; v2 app entrypoints extend this validator in Plan 06. Resource-policy relaxation remains disabled until Plan 07 proves containment.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(hub): validate runnable bundles and artwork`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
