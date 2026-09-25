# 19. Publisher teams, transfer, key recovery and trust revocation Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Support real publisher organizations and key incidents without weakening app ownership or replay protection.

**Architecture:** Record append-only ownership/key lifecycle events and authenticated role changes in the registry. Extend trust metadata with explicit key epochs/revocations, maintaining separately signed versioned client trust state.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P1. **Phase:** E — Public free-store reliability. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [01 — Publisher key continuity and signed release identity](2026-09-24-store-01-publisher-key-continuity.md); [04 — Atomic catalog publication, renewal and signing operations](2026-09-24-store-04-catalog-release-operations.md); [05 — Versioned runtime contracts and release compatibility](2026-09-24-store-05-runtime-compatibility.md); [11 — Publisher authentication, namespace ownership and registry](2026-09-24-store-11-publisher-accounts.md)

**Review coverage:** R01, R12, R16 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/hub-service/src/teams.rs](../../crates/hub-service/src/teams.rs) |
| Create | [H/crates/hub-service/src/key_lifecycle.rs](../../crates/hub-service/src/key_lifecycle.rs) |
| Create | [H/crates/hub-service/migrations/005_publisher_lifecycle.sql](../../crates/hub-service/migrations/005_publisher_lifecycle.sql) |
| Create | [H/crates/hub-service/tests/publisher_lifecycle.rs](../../crates/hub-service/tests/publisher_lifecycle.rs) |
| Create | [H/crates/app-hub/src/trust.rs](../../crates/app-hub/src/trust.rs) |
| Modify | [H/crates/app-hub/src/signing.rs](../../crates/app-hub/src/signing.rs) |
| Modify | [H/crates/app-hub/src/lib.rs](../../crates/app-hub/src/lib.rs) |
| Create | [H/crates/app-hub/tests/trust_lifecycle.rs](../../crates/app-hub/tests/trust_lifecycle.rs) |
| Create | [H/docs/operations/key-recovery.md](../../docs/operations/key-recovery.md) |
| Modify | [H/crates/hub-service/src/lib.rs](../../crates/hub-service/src/lib.rs) |
| Modify | [H/crates/hub-service/Cargo.toml](../../crates/hub-service/Cargo.toml) |
| Modify | [H/crates/app-hub/Cargo.toml](../../crates/app-hub/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
KeyEvent {
  publisher_id, old_key_fingerprint, new_key_fingerprint,
  kind: "rotate" | "recover" | "revoke",
  authority, effective_release, reason, timestamp
}
TrustState { schema, anchor_epoch, working_key_epoch, revoked_keys, sequence }
// Historical approved releases retain their original verification bindings.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Enforce team roles

**Touch:** `H/crates/hub-service/src/teams.rs`, `H/crates/hub-service/tests/publisher_lifecycle.rs`.

1. **Write the regression/acceptance case** `submitter_cannot_rotate_key_or_transfer_app`: Invite owner/admin/developer/reviewer users and exercise each route; remove a member and retry with a formerly valid session.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add app-scoped roles, invitation expiry, least-privilege authorization and immediate session revocation. Separate reviewer/operator powers from publisher powers.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Rotate without rewriting history

**Touch:** `H/crates/hub-service/src/key_lifecycle.rs`, `H/crates/app-hub/src/signing.rs`.

1. **Write the regression/acceptance case** `old_release_verifies_after_authorized_rotation`: Rotate with old/new key proof, publish a new release, verify old approved releases and reject an unrecorded replacement. Avoid last-entry-wins publisher key maps.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Store key fingerprints/epochs per release; require a valid rotation record and old-key proof where available. Apply it as an authorization event rather than editing catalog history.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Recover and transfer ownership explicitly

**Touch:** `H/crates/hub-service/src/key_lifecycle.rs`, `H/docs/operations/key-recovery.md`.

1. **Write the regression/acceptance case** `lost_key_recovery_requires_independent_review`: Attempt recovery using only a logged-in account and transfer an app without both parties' acknowledgment. Both must fail; an audited recovery can bind a new key.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Define verified recovery evidence, separate reviewer/operator authorization, notification and a configurable hold. Transfers preserve app ID/data/update continuity and record both publishers; handle managed custody explicitly if offered.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Revoke compromised working keys

**Touch:** `H/crates/app-hub/src/trust.rs`, `H/crates/app-hub/tests/trust_lifecycle.rs`.

1. **Write the regression/acceptance case** `older_certified_key_cannot_resume_signing`: Accept a newer trust epoch revoking K1, then replay a K1-signed catalog with larger sequence. The client must reject it despite its old anchor certificate.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add trusted key-epoch/revocation checks and durable high-water marks. Design root compromise separately: an offline recovery authority or client release is required; never assert an already-compromised root can securely revoke itself unaided.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Run incident drills

**Touch:** `H/docs/operations/key-recovery.md`, `H/crates/app-hub/tests/trust_lifecycle.rs`.

1. **Write the regression/acceptance case** `recovery_restore_keeps_revocations`: Restore old database/catalog backups after key revocation and verify newer client trust state is not reset. Test lost publisher key and compromised service signer scenarios.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Write runbooks with scope, operator ownership, recovery artifacts and user/developer communication templates. Rehearse on test anchors before production enablement.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-hub-service --test publisher_lifecycle
cargo test --locked -p octosense-app-hub --test trust_lifecycle
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Teams can collaborate without every developer gaining key/ownership control.
- [ ] Authorized recovery/transfer preserves user update continuity and immutable history.
- [ ] Clients enforce key revocations across restarts and catalog replays.

## Rollout, migration and recovery

Introduce registry roles first; ship new trust readers before emitting key-epoch metadata. Legacy clients need an explicit migration or retirement policy for compromise handling.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(publishing): add publisher lifecycle and trust recovery`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
