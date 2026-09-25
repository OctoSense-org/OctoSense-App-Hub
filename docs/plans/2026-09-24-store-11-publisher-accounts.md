# 11. Publisher authentication, namespace ownership and registry Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Give outside developers a verified identity and exclusive app ownership without Hub repository write access.

**Architecture:** Create a small Rust control-plane service with a transactional SQLite store for the initial single-region deployment. Separate account identity, publisher ownership, key enrollment and app namespaces; scale the store behind a repository interface only when required.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P0. **Phase:** D — Self-service publishing. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [01 — Publisher key continuity and signed release identity](2026-09-24-store-01-publisher-key-continuity.md); [05 — Versioned runtime contracts and release compatibility](2026-09-24-store-05-runtime-compatibility.md)

**Review coverage:** R04, R12 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/hub-service/Cargo.toml](../../crates/hub-service/Cargo.toml) |
| Create | [H/crates/hub-service/src/main.rs](../../crates/hub-service/src/main.rs) |
| Create | [H/crates/hub-service/src/lib.rs](../../crates/hub-service/src/lib.rs) |
| Create | [H/crates/hub-service/src/auth.rs](../../crates/hub-service/src/auth.rs) |
| Create | [H/crates/hub-service/src/publishers.rs](../../crates/hub-service/src/publishers.rs) |
| Create | [H/crates/hub-service/migrations/001_identity.sql](../../crates/hub-service/migrations/001_identity.sql) |
| Create | [H/crates/hub-service/tests/publisher_accounts.rs](../../crates/hub-service/tests/publisher_accounts.rs) |
| Modify | [H/Cargo.toml](../../Cargo.toml) |
| Create | [H/docs/api/publishers.md](../../docs/api/publishers.md) |
| Modify | [H/crates/app-hub/src/publishers.rs](../../crates/app-hub/src/publishers.rs) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
POST /v1/login/device
POST /v1/login/token
GET  /v1/me
POST /v1/publishers
POST /v1/publishers/{publisher_id}/keys/challenge
POST /v1/publishers/{publisher_id}/keys
POST /v1/apps
# Key enrollment = verified identity + proof of possession.
# app_id UNIQUE; owner_id immutable except authorized transfer.
# Roles initially: publisher owner, submitter, reviewer, operator.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Build the service/database boundary

**Touch:** `H/crates/hub-service/src/lib.rs`, `H/crates/hub-service/src/main.rs`, `H/crates/hub-service/migrations/001_identity.sql`.

1. **Write the regression/acceptance case** `identity_migration_is_repeatable`: Start a service with a fresh temporary database, apply migrations twice and restart; identities persist and no privileged account is silently seeded.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add HTTP router, repository/clock/identity-provider interfaces, transactional migrations, health/readiness endpoints and typed errors. Pin chosen maintained dependencies in Cargo.lock; mock identity externally in tests.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Authenticate CLI and browser sessions

**Touch:** `H/crates/hub-service/src/auth.rs`, `H/crates/hub-service/tests/publisher_accounts.rs`.

1. **Write the regression/acceptance case** `expired_or_wrong_audience_token_is_refused`: Use a fake device/OAuth provider to test valid login, expiry, replay and token audience/scope mismatch. No private token appears in response logs.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Implement browser/device authorization via a provider adapter, short-lived scoped API tokens and explicit logout/revocation. Store stable provider IDs rather than relying on mutable account names.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Claim namespaces transactionally

**Touch:** `H/crates/hub-service/src/publishers.rs`, `H/crates/hub-service/migrations/001_identity.sql`.

1. **Write the regression/acceptance case** `concurrent_app_claim_has_one_owner`: Have two publishers concurrently claim the same normalized app ID. Exactly one succeeds; changing the publisher's display name does not transfer ownership.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add unique constraints and ownership checks on every app mutation. Default generated IDs use a claimed publisher namespace; importing an existing ID requires an operator-reviewed claim record.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Enroll trusted keys

**Touch:** `H/crates/hub-service/src/publishers.rs`, `H/crates/app-hub/src/publishers.rs`.

1. **Write the regression/acceptance case** `key_enrollment_requires_possession_and_owner`: Attempt to enroll another user's key, replay an enrollment challenge and overwrite an existing binding. Require owner authorization and a signed one-time challenge.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Back Plan 01's registry with durable publisher/app/key bindings and audit events. First local release keys use OS-keystore generation; managed/CI key custody is a separately enrolled adapter, never a changed mapping hidden in an upload.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Bound abusive access

**Touch:** `H/crates/hub-service/src/auth.rs`, `H/docs/api/publishers.md`.

1. **Write the regression/acceptance case** `scopes_and_limits_apply_per_publisher`: Exceed configured login/submission request limits and use a submitter token on operator routes. Responses are explicit, retryable where suitable, and reveal no other publisher's data.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add per-principal quotas, audit redaction, owner-visible key/session inventory and permission middleware. Document configuration and local mock-provider setup.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-hub-service --test publisher_accounts
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] An outside account can own an app and enroll a key without repository write access.
- [ ] Duplicate claims, cross-publisher mutation and spoofed identity/key enrollment fail.
- [ ] Credentials remain scoped, revocable and excluded from logs/bundles.

## Rollout, migration and recovery

Develop with a mock identity provider and localhost database. Before production, choose the provider registration, secret store, region and backup owner; these choices do not block schema/API implementation.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(publishing): add publisher identities and ownership registry`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
