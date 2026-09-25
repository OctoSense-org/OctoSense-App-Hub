# 14. One-command publishing and GitHub release action Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make hub publish the complete developer path from checked source to submitted release.

**Architecture:** Keep the CLI and GitHub Action thin clients of the same versioned submission API. Preserve operator commands under hub admin and keep catalog signing authority out of developer environments.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P0. **Phase:** D — Self-service publishing. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [09 — Installable SDK, pinned runtimes and runnable starters](2026-09-24-store-09-sdk-and-starters.md); [10 — Development loop, signed device preview and editor tooling](2026-09-24-store-10-development-and-testing.md); [11 — Publisher authentication, namespace ownership and registry](2026-09-24-store-11-publisher-accounts.md); [12 — Immutable artifact upload and submission API](2026-09-24-store-12-submission-api.md); [13 — Review decisions, publisher feedback and abuse handling](2026-09-24-store-13-review-and-moderation.md)

**Review coverage:** R04, R05, R26 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/app-hub/src/publish.rs](../../crates/app-hub/src/publish.rs) |
| Modify | [H/crates/app-hub/src/bin/hub.rs](../../crates/app-hub/src/bin/hub.rs) |
| Modify | [H/crates/app-hub/src/bin/hub-usage.txt](../../crates/app-hub/src/bin/hub-usage.txt) |
| Create | [H/crates/app-hub/tests/publish_cli.rs](../../crates/app-hub/tests/publish_cli.rs) |
| Create | [H/actions/publish-app/action.yml](../../actions/publish-app/action.yml) |
| Create | [H/actions/publish-app/README.md](../../actions/publish-app/README.md) |
| Create | [H/crates/hub-service/src/ci_identity.rs](../../crates/hub-service/src/ci_identity.rs) |
| Create | [H/crates/hub-service/tests/ci_identity.rs](../../crates/hub-service/tests/ci_identity.rs) |
| Modify | [H/docs/PUBLISHING.md](../../docs/PUBLISHING.md) |
| Create | [H/templates/notes/.github/workflows/publish.yml](../../templates/notes/.github/workflows/publish.yml) |
| Modify | [H/crates/app-hub/src/lib.rs](../../crates/app-hub/src/lib.rs) |
| Modify | [H/crates/app-hub/Cargo.toml](../../crates/app-hub/Cargo.toml) |
| Modify | [H/crates/hub-service/src/lib.rs](../../crates/hub-service/src/lib.rs) |
| Modify | [H/crates/hub-service/Cargo.toml](../../crates/hub-service/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
hub login
hub publish --channel stable --wait --json
hub submissions status <id>
hub admin publish ...            # private operator command
# Public publish stages: resolve -> build -> test -> metadata preflight
# -> stamp -> publisher-sign -> immutable upload -> submit -> status.
# GitHub OIDC authenticates a job; it does not authorize key substitution.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Implement a resumable CLI pipeline

**Touch:** `H/crates/app-hub/src/publish.rs`, `H/crates/app-hub/tests/publish_cli.rs`.

1. **Write the regression/acceptance case** `publish_failure_resumes_without_duplicate_release`: Interrupt after build, upload and submit, then retry. Invalid metadata, missing privacy URL and incorrect signing identity fail before release submission.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add persisted local operation IDs, deterministic outputs, a concise progress stream, JSON mode and remedy-oriented errors. Render the final permissions and app/release identity before submission without requiring repetitive prompts in configured CI.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Hide routine signing mechanics

**Touch:** `H/crates/app-hub/src/publish.rs`, `H/docs/PUBLISHING.md`.

1. **Write the regression/acceptance case** `keychain_signs_without_exposing_private_key`: Login/enroll once, sign a release through a keystore adapter, and test absent/locked keys. Private key data must never appear in stdout, files under bundle or CI logs.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Use OS keystore for local publisher keys and a documented self-managed path. For CI, support an explicitly enrolled protected signing adapter/key; managed per-publisher custody is optional and must never reuse the Hub catalog key.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Authenticate CI narrowly

**Touch:** `H/crates/hub-service/src/ci_identity.rs`, `H/crates/hub-service/tests/ci_identity.rs`.

1. **Write the regression/acceptance case** `fork_oidc_token_cannot_publish_owner_app`: Validate fake OIDC tokens for wrong issuer/audience/repository ID/ref/environment/expiry and unauthorized workflow. Only the enrolled repository/release identity can submit for the app.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Implement OIDC verification with trusted discovery/JWKS caching and explicit allowed claims. The action gets short-lived publish scope; a repository-name match alone is insufficient.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Ship a real action

**Touch:** `H/actions/publish-app/action.yml`, `H/actions/publish-app/README.md`, `H/templates/notes/.github/workflows/publish.yml`.

1. **Write the regression/acceptance case** `release_action_publishes_test_bundle_end_to_end`: Run the action in a fixture repository against staging, then repeat the release. Verify signed artifact identity, status URL and no duplicate version.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Ship a composite action in this repository first and use its actual pinned reference in docs/templates. A dedicated publish-app repository is an optional packaging step, not a nonexistent dependency.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Replace planned documentation with a working guide

**Touch:** `H/docs/PUBLISHING.md`, `H/crates/app-hub/src/bin/hub-usage.txt`.

1. **Write the regression/acceptance case** `clean_account_can_submit_using_only_documented_steps`: Use a non-member account/repository and follow the guide without operator instructions. Record all undocumented steps and remove them.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Update README/publishing/first-app links and provide CLI/API examples with explicit local development vs stable/beta availability. Plan 16 enables real private beta; until then refuse --channel beta rather than silently making it public.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-hub --test publish_cli
cargo test --locked -p octosense-hub-service --test ci_identity
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] A first outside publisher can submit with hub login/publish and receives an actionable status reference.
- [ ] The action works from a repository without Hub write permission or Hub signing credentials.
- [ ] Documented commands/actions exist and enforce the same checks as the service.

## Rollout, migration and recovery

Test with a staging origin/test anchor. Rename the old operator command with a transition error/help message, not a silent semantic change; pin action releases by commit in sensitive workflows.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(cli): add self-service publishing and release action`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
