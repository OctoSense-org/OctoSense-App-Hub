# 26. Continuous integration and outside-developer launch qualification Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prove the whole developer-to-device lifecycle works for a person outside the organization, with measurable usability and recovery outcomes.

**Architecture:** Use a staging service/catalog/test anchor, packaged SDK and independently owned fixture apps. Keep headless conformance, native UI/device tests and human onboarding measurements as separate evidence, tied to exact release revisions.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P0 — Release gate. **Phase:** Cross-cutting qualification. **Relative size:** M (complexity, not a delivery-date estimate).

**Prerequisites:** [01 — Publisher key continuity and signed release identity](2026-09-24-store-01-publisher-key-continuity.md); [02 — Installed-version launch and precise revocation](2026-09-24-store-02-installed-version-launch.md); [03 — Runnable bundle admission and actionable validation](2026-09-24-store-03-bundle-admission.md); [04 — Atomic catalog publication, renewal and signing operations](2026-09-24-store-04-catalog-release-operations.md); [05 — Versioned runtime contracts and release compatibility](2026-09-24-store-05-runtime-compatibility.md); [06 — Card application lifecycle, state and effects](2026-09-24-store-06-card-application-runtime.md); [07 — Host services, capability matrix and structured network policy](2026-09-24-store-07-host-services.md); [09 — Installable SDK, pinned runtimes and runnable starters](2026-09-24-store-09-sdk-and-starters.md); [10 — Development loop, signed device preview and editor tooling](2026-09-24-store-10-development-and-testing.md); [11 — Publisher authentication, namespace ownership and registry](2026-09-24-store-11-publisher-accounts.md); [12 — Immutable artifact upload and submission API](2026-09-24-store-12-submission-api.md); [13 — Review decisions, publisher feedback and abuse handling](2026-09-24-store-13-review-and-moderation.md); [14 — One-command publishing and GitHub release action](2026-09-24-store-14-publish-cli-and-ci.md); [15 — Mobile uninstall, complete listings and user controls](2026-09-24-store-15-mobile-app-management.md)

**Review coverage:** R27, R29 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/.github/workflows/ci.yml](../../.github/workflows/ci.yml) |
| Create | [H/.github/workflows/ecosystem-staging.yml](../../.github/workflows/ecosystem-staging.yml) |
| Create | [H/tools/setup-ci-workspace.py](../../tools/setup-ci-workspace.py) |
| Create | [H/tools/ecosystem-smoke.py](../../tools/ecosystem-smoke.py) |
| Create | [H/docs/validation/launch-checklist.md](../../docs/validation/launch-checklist.md) |
| Create | [H/docs/validation/onboarding-study.md](../../docs/validation/onboarding-study.md) |
| Modify | [M/.github/workflows/runtime.yml](../../../OctoSense-mobile/.github/workflows/runtime.yml) |
| Create | [H/docs/validation/staging-config.example.json](../../docs/validation/staging-config.example.json) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
QualificationRecord {
  sdk_version, hub_revision, shell_revision, runtime_revisions,
  app_source_commit, reviewed_digest, device_os,
  author_role: "outside-publisher",
  scenarios, measured_setup_minutes, measured_submission_minutes,
  failures, evidence_paths
}
// Suggested targets: functioning preview <=15 minutes; first submission
// <=30 minutes excluding human review, measured on supported clean machines.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Establish repeatable CI immediately

**Touch:** `H/.github/workflows/ci.yml`, `H/tools/setup-ci-workspace.py`, `M/.github/workflows/runtime.yml`, `H/docs/validation/staging-config.example.json`.

1. **Write the regression/acceptance case** `baseline_and_new_core_suites_run_in_clean_workspace`: Assemble exact pinned sibling runtimes in disposable CI storage and run core suites without relying on a developer's working checkout. New packages join the matrix as created.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add a lockfile-aware setup helper and CI for policy/hub/runtime/service packages plus mobile integration. Do not execute untrusted app fixtures in jobs that hold signing secrets. Generate a local test configuration from staging-config.example.json with ephemeral test credentials/anchors, and document the prerequisite service/device setup before running the smoke command.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Exercise the complete free-app journey

**Touch:** `H/tools/ecosystem-smoke.py`, `H/.github/workflows/ecosystem-staging.yml`.

1. **Write the regression/acceptance case** `outside_publisher_can_release_update_and_uninstall`: Use a non-organization publisher: login, new notes app, dev, test, submit, review, install, save data, publish v2, open v1, update, restart and uninstall.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Build an end-to-end staging harness with real signed artifacts and public APIs. Capture runtime results and native screenshots/input evidence; never prepopulate the production catalog with test listings.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Drill failure and incident paths

**Touch:** `H/tools/ecosystem-smoke.py`, `H/docs/validation/launch-checklist.md`.

1. **Write the regression/acceptance case** `fault_matrix_preserves_trust_and_data`: Inject offline/stale catalog, bad signature, wrong publisher key, corrupt download, review timeout, missing adapter, interrupted update and withdrawn exact release.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Record expected recovery and user/developer messages for each fault. Extend public-beta qualification with migrations, rollout, key recovery, reporting and accessibility when their plans complete.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Measure onboarding with real outsiders

**Touch:** `H/docs/validation/onboarding-study.md`.

1. **Write the regression/acceptance case** `new_developer_completes_without_maintainer_shell_access`: Observe developers unfamiliar with the repositories on clean supported machines. Measure setup/submission time, undocumented steps, confusing errors and required maintainer intervention.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Use at least three independent developers and more than one host/device setup for the pilot exit assessment. Fix observed friction in the owning plans; targets are acceptance goals, not current performance claims.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Gate releases with evidence

**Touch:** `H/docs/validation/launch-checklist.md`.

1. **Write the regression/acceptance case** `release_gate_rejects_missing_runtime_device_evidence`: Attempt qualification with only unit tests, an untested advertised platform or stale fixture captures. The checklist remains incomplete.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Require documented evidence per advertised platform, named operational owner and passing stage-specific checklist. Public-beta exit additionally requires Plans 08 and 16–20 and 22; paid/native activation uses their separate qualification.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
python3 tools/ecosystem-smoke.py --profile free-pilot --config target/staging-test.json --output target/qualification
cargo test --locked -p octosense-app-hub -p octosense-app-policy
(cd ../OctoSense-mobile && cargo test --locked -p octosense-app-hub-app --lib)
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] A real outside developer completes the full app lifecycle using documented packaged tools and public APIs.
- [ ] Fault drills protect approval/signatures, existing app usability and user data.
- [ ] Launch scope matches actual device/runtime evidence and measured onboarding results.

## Rollout, migration and recovery

Write the harness and baseline CI early; prerequisites are needed to pass the full pilot gate, not to begin test work. Run private staging qualification, invited free pilot, then public free beta. Keep commerce/native gates independent.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `test(ecosystem): qualify the outside developer app lifecycle`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
