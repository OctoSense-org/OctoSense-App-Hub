# 09. Installable SDK, pinned runtimes and runnable starters Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Let an outside developer create and run a useful app without checking out the framework repositories.

**Architecture:** Distribute the hub CLI, compatible reference runtime and kit as versioned SDK artifacts. A single project config and lockfile describe inputs; generated release metadata stays derived and deterministic.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P0. **Phase:** C — Developer workflow. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [05 — Versioned runtime contracts and release compatibility](2026-09-24-store-05-runtime-compatibility.md); [06 — Card application lifecycle, state and effects](2026-09-24-store-06-card-application-runtime.md); [07 — Host services, capability matrix and structured network policy](2026-09-24-store-07-host-services.md)

**Review coverage:** R05, R27 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/app-hub/src/sdk.rs](../../crates/app-hub/src/sdk.rs) |
| Create | [H/crates/app-hub/src/project.rs](../../crates/app-hub/src/project.rs) |
| Modify | [H/crates/app-hub/src/lib.rs](../../crates/app-hub/src/lib.rs) |
| Modify | [H/crates/app-hub/src/bin/hub.rs](../../crates/app-hub/src/bin/hub.rs) |
| Create | [H/crates/app-hub/tests/sdk_project.rs](../../crates/app-hub/tests/sdk_project.rs) |
| Create | [H/templates/notes/app.toml](../../templates/notes/app.toml) |
| Create | [H/templates/notes/src/page.card](../../templates/notes/src/page.card) |
| Create | [H/templates/notes/src/app.octoscript](../../templates/notes/src/app.octoscript) |
| Create | [H/templates/api-reader/app.toml](../../templates/api-reader/app.toml) |
| Create | [H/templates/api-reader/src/app.octoscript](../../templates/api-reader/src/app.octoscript) |
| Create | [H/sdk/releases/schema.json](../../sdk/releases/schema.json) |
| Create | [H/.github/workflows/sdk-release.yml](../../.github/workflows/sdk-release.yml) |
| Modify | [H/docs/FIRST-APP.md](../../docs/FIRST-APP.md) |
| Modify | [H/docs/DEVELOPMENT.md](../../docs/DEVELOPMENT.md) |
| Modify | [H/crates/app-hub/Cargo.toml](../../crates/app-hub/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |
| Create | [H/templates/notes/assets/icon.svg](../../templates/notes/assets/icon.svg) |
| Create | [H/templates/api-reader/src/page.card](../../templates/api-reader/src/page.card) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
# app.toml — proposed sole author configuration
[app]
id = "org.example.notes"
name = "Notes"
version = "0.1.0"
template = "notes"
[sdk]
channel = "stable"
[permissions]
capabilities = ["storage"]
# hub.lock pins SDK, runtime build, kit digest and compiler/schema versions.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Create a genuinely runnable project

**Touch:** `H/crates/app-hub/src/project.rs`, `H/templates/notes/app.toml`, `H/templates/notes/src/page.card`, `H/templates/notes/src/app.octoscript`.

1. **Write the regression/acceptance case** `new_notes_project_runs_without_framework_checkout`: Generate notes into a fresh empty directory, start the packaged runtime, type/save a note and reopen it. Reject overwriting an existing project.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add hub new with a complete Card/logic/kit source template and real default artwork; template generation expands the kit from the SDK cache rather than requiring sibling paths.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Resolve and lock SDK assets

**Touch:** `H/crates/app-hub/src/sdk.rs`, `H/sdk/releases/schema.json`, `H/crates/app-hub/tests/sdk_project.rs`.

1. **Write the regression/acceptance case** `sdk_digest_mismatch_is_refused`: Download a fake versioned SDK, corrupt one asset, interrupt download and run offline with a valid cache. Verify digest checks, atomic cache publication and readable offline errors.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Implement SDK manifests with supported OS/architecture, API range, asset hashes and trusted release signatures. hub doctor reports exact missing tool/runtime mismatch and fixes only its managed cache.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Derive reproducible metadata

**Touch:** `H/crates/app-hub/src/project.rs`, `H/crates/app-hub/src/bin/hub.rs`.

1. **Write the regression/acceptance case** `same_sources_produce_identical_bundle`: Build twice in unrelated checkout paths; bundle digests must match. Edit one permission or source byte and require a new digest. Secrets and local-state paths must be excluded.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Generate manifest/listing from one config plus release inputs, with explicit asset allowlist. Pin all tool versions in hub.lock; preserve source config rather than reverse-syncing generated files.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Ship supported binaries

**Touch:** `H/.github/workflows/sdk-release.yml`, `H/crates/app-hub/src/sdk.rs`.

1. **Write the regression/acceptance case** `clean_machine_sdk_installs_and_runs`: Use clean macOS and Linux runners and a supported Android device preview route. Verify checksums/signatures and native resource lookup outside the repository.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Create SDK build/package/release workflow, release manifest and host resource packaging. Publish only tested host/architecture combinations; Windows support follows passing packaging tests rather than an untested promise.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Teach complete examples

**Touch:** `H/templates/api-reader/app.toml`, `H/templates/api-reader/src/app.octoscript`, `H/docs/FIRST-APP.md`, `H/docs/DEVELOPMENT.md`.

1. **Write the regression/acceptance case** `api_reader_works_with_test_endpoint`: Run notes offline and API-reader against a local test service under development policy, then a listed HTTPS host for release validation. Add the AI example only when Plan 08 is ready.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Rewrite FIRST-APP around installed tools; document limits/errors and provide a troubleshooting ladder. Include support/privacy placeholders that block publish until completed.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-hub --test sdk_project
cargo test --locked -p octosense-app-runtime --test app_lifecycle
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] A clean machine reaches an interactive persistent notes app with hub new and the packaged reference host; Plan 10 supplies the one-command dev loop.
- [ ] Developers do not need Rust or sibling framework repositories to use a supported packaged SDK.
- [ ] Releases pin runtime/kit/schema versions and exclude private/development files.

## Rollout, migration and recovery

Offer a preview SDK channel first; keep source-build instructions as an advanced path. Do not silently install platform tools or mutate an unrelated checkout.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(sdk): add packaged tools and runnable app starters`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
