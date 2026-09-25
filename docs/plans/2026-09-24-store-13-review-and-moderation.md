# 13. Review decisions, publisher feedback and abuse handling Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make admission decisions reproducible, attributable and understandable to publishers and users.

**Architecture:** Use deterministic validation as a prerequisite and store human/automated decisions separately from catalog releases. First-publisher approval and any exception are authenticated records bound to immutable evidence, never a boolean CLI bypass.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P0. **Phase:** D — Self-service publishing. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [03 — Runnable bundle admission and actionable validation](2026-09-24-store-03-bundle-admission.md); [11 — Publisher authentication, namespace ownership and registry](2026-09-24-store-11-publisher-accounts.md); [12 — Immutable artifact upload and submission API](2026-09-24-store-12-submission-api.md)

**Review coverage:** R08, R19 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/hub-service/src/reviews.rs](../../crates/hub-service/src/reviews.rs) |
| Create | [H/crates/hub-service/src/reports.rs](../../crates/hub-service/src/reports.rs) |
| Create | [H/crates/hub-service/src/notifications.rs](../../crates/hub-service/src/notifications.rs) |
| Create | [H/crates/hub-service/migrations/003_reviews.sql](../../crates/hub-service/migrations/003_reviews.sql) |
| Create | [H/crates/hub-service/tests/review_workflow.rs](../../crates/hub-service/tests/review_workflow.rs) |
| Modify | [H/crates/app-hub/src/scan.rs](../../crates/app-hub/src/scan.rs) |
| Modify | [H/crates/app-hub/src/bin/hub.rs](../../crates/app-hub/src/bin/hub.rs) |
| Create | [H/docs/policies/review.md](../../docs/policies/review.md) |
| Create | [H/docs/policies/reporting.md](../../docs/policies/reporting.md) |
| Modify | [H/crates/hub-service/src/lib.rs](../../crates/hub-service/src/lib.rs) |
| Modify | [H/crates/hub-service/Cargo.toml](../../crates/hub-service/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
ReviewDecision {
  submission_id, bundle_digest, runtime_validation_id, policy_revision,
  route: "approve" | "request_changes" | "reject",
  reviewer_principal, reasons: [{code, path, message, remedy}], decided_at
}
# Separate transitions: resubmit(new digest), appeal(decision_id),
# report(app_id, release_id, category), resolve_report(action + reason).
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Require auditable decisions

**Touch:** `H/crates/hub-service/src/reviews.rs`, `H/crates/app-hub/src/bin/hub.rs`.

1. **Write the regression/acceptance case** `reviewed_flag_cannot_authorize_public_publish`: Try --reviewed and a forged decision JSON against the service. First publisher requires a recorded reviewer decision; a returning publisher follows explicit policy rules.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Replace public-path overrides with authenticated review transitions. Keep local scan output advisory; only trusted server actors can approve immutable submissions.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Make reviewer execution bounded

**Touch:** `H/crates/app-hub/src/scan.rs`, `H/crates/hub-service/src/reviews.rs`.

1. **Write the regression/acceptance case** `hung_reviewer_routes_to_pending_human_review`: Run a reviewer that hangs, returns malformed output, requests wider permissions or exits midway. It cannot pass or hold a worker forever.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add timeout/output caps and kill/reap behavior; reviewers run in isolated workers without credentials. Store card/kit/resource evidence and actual screenshot references, not an empty placeholder list.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Support feedback and resubmission

**Touch:** `H/crates/hub-service/src/reviews.rs`, `H/crates/hub-service/src/notifications.rs`.

1. **Write the regression/acceptance case** `resubmission_needs_new_evidence_for_new_bytes`: Reject a bundle with actionable reasons, resubmit corrected bytes and attempt to reuse the old approval. The corrected release is revalidated and preserves review history.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add publisher-readable reasons, policy links, stable status, resubmission links and a minimal authenticated reviewer queue. Use in-app notifications first; external notification delivery is opt-in and idempotent.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Handle reports and appeals

**Touch:** `H/crates/hub-service/src/reports.rs`, `H/crates/hub-service/migrations/003_reviews.sql`.

1. **Write the regression/acceptance case** `report_cannot_unilaterally_revoke_an_app`: Submit duplicate/spam reports, a publisher appeal and a reviewed emergency takedown. A report alone must not change catalog status.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add report categories, rate limits, moderation roles, evidence retention and decision records. A takedown invokes an explicit release/app revocation transaction and records scope/reason.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Publish policies and operational targets

**Touch:** `H/docs/policies/review.md`, `H/docs/policies/reporting.md`.

1. **Write the regression/acceptance case** `review_history_records_policy_revision`: Change a review policy and re-evaluate a submission; retain which version governed the original decision.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Write concise eligibility, privacy, permissions, impersonation, content, appeal and takedown policies. Define an initial review response target and responsible queue owner; do not promise an SLA until staffed.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-hub-service --test review_workflow
cargo test --locked -p octosense-app-hub scan
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Every publication is tied to validation and an authenticated approval of the exact bytes.
- [ ] Publishers can understand rejection, resubmit and appeal; users can report abuse.
- [ ] Reviewer failures and timeouts route safely without blocking the service indefinitely.

## Rollout, migration and recovery

Launch with human approval and deterministic validation; enable returning-publisher automation only after policy/evidence quality is demonstrated. A model reviewer is optional, not a launch dependency.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(publishing): add auditable review and moderation workflow`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
