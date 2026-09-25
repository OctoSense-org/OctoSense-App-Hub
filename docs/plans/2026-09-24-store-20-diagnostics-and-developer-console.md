# 20. Release diagnostics, developer console and operations health Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Let developers and operators understand failed submissions, installs and app releases without collecting unnecessary personal data.

**Architecture:** Use structured release-correlated events and server-side job/audit metrics, with opt-in device diagnostics. Start with CLI/API and a small server-rendered developer console over the same authorization layer; avoid a separate analytics stack until measurements justify it.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P1. **Phase:** E — Public free-store reliability. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [11 — Publisher authentication, namespace ownership and registry](2026-09-24-store-11-publisher-accounts.md); [12 — Immutable artifact upload and submission API](2026-09-24-store-12-submission-api.md); [13 — Review decisions, publisher feedback and abuse handling](2026-09-24-store-13-review-and-moderation.md); [06 — Card application lifecycle, state and effects](2026-09-24-store-06-card-application-runtime.md)

**Review coverage:** R16, R21 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/hub-service/src/diagnostics.rs](../../crates/hub-service/src/diagnostics.rs) |
| Create | [H/crates/hub-service/src/console.rs](../../crates/hub-service/src/console.rs) |
| Create | [H/crates/hub-service/src/health.rs](../../crates/hub-service/src/health.rs) |
| Create | [H/crates/hub-service/tests/diagnostics.rs](../../crates/hub-service/tests/diagnostics.rs) |
| Create | [H/crates/app-runtime/src/diagnostics.rs](../../crates/app-runtime/src/diagnostics.rs) |
| Create | [H/crates/app-runtime/tests/diagnostic_redaction.rs](../../crates/app-runtime/tests/diagnostic_redaction.rs) |
| Modify | [M/apps/app-hub/src/view.rs](../../../OctoSense-mobile/apps/app-hub/src/view.rs) |
| Create | [H/docs/operations/observability.md](../../docs/operations/observability.md) |
| Modify | [H/crates/hub-service/src/lib.rs](../../crates/hub-service/src/lib.rs) |
| Modify | [H/crates/hub-service/Cargo.toml](../../crates/hub-service/Cargo.toml) |
| Modify | [H/crates/app-runtime/src/lib.rs](../../crates/app-runtime/src/lib.rs) |
| Modify | [H/crates/app-runtime/Cargo.toml](../../crates/app-runtime/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
Diagnostic {
  schema, app_id, release_id, runtime_build, platform,
  event: "install_failed" | "launch_failed" | "crash" | "migration_failed",
  error_code, consent_basis, occurred_at
}
// No note contents, prompts, credentials, full URLs or stable cross-app IDs.
// Developer queries are scoped to owned apps; small cohorts are suppressed.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Make local diagnostics useful

**Touch:** `H/crates/app-runtime/src/diagnostics.rs`, `H/crates/app-runtime/tests/diagnostic_redaction.rs`.

1. **Write the regression/acceptance case** `runtime_errors_include_source_and_release`: Throw in an app handler, fail a download and exceed a quota. Log a stable error code, release/runtime identity and source location where known.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add bounded local logs and an explicit export action, redact secrets/content, and retain source maps only for the owning publisher/release.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Collect only authorized telemetry

**Touch:** `H/crates/hub-service/src/diagnostics.rs`, `H/crates/app-runtime/src/diagnostics.rs`.

1. **Write the regression/acceptance case** `telemetry_disabled_sends_nothing`: Run with telemetry disabled, enabled, later revoked and with sensitive sample content in errors. Assert no unintended network calls or sensitive fields.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Define event allowlists, consent/preferences, sampling, retention/deletion and per-app aggregation. Queue bounded reports and avoid coupling app startup to reporting availability.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Give publishers one release view

**Touch:** `H/crates/hub-service/src/console.rs`, `H/crates/hub-service/tests/diagnostics.rs`.

1. **Write the regression/acceptance case** `publisher_dashboard_cannot_query_other_apps`: Log in as two publishers and inspect submissions, review feedback, active releases, health and support reports. Attempt cross-owner IDs and unsafe reflected text.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add accessible server-rendered pages and matching APIs/CLI JSON, with CSRF protections for browser mutations and output escaping. Keep initial views focused on next action and failures.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Monitor store operations

**Touch:** `H/crates/hub-service/src/health.rs`, `H/docs/operations/observability.md`.

1. **Write the regression/acceptance case** `catalog_freshness_and_queue_backlog_alert`: Simulate signer failure, stale catalogs, rising install failure, worker backlog and artifact unavailability; verify actionable alerts and deduplicated incidents.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Measure server/service metrics independently from opt-in client usage. Define SLOs and owners for publication/freshness/download success; avoid invented metrics from a zero-app baseline.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Feed rollout health conservatively

**Touch:** `H/crates/hub-service/src/diagnostics.rs`, `H/docs/operations/observability.md`.

1. **Write the regression/acceptance case** `small_sample_does_not_auto_halt_release`: Feed one failure and a statistically meaningful regression; only configured thresholds/sample windows allow automatic pause, and manual override remains available.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Expose aggregate release health to Plan 16. Keep automatic halt disabled until minimum coverage/baseline/error budgets are explicitly configured.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-hub-service --test diagnostics
cargo test --locked -p octosense-app-runtime --test diagnostic_redaction
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Developers can identify why a submission/install/release failed and act on it.
- [ ] Opt-out produces no client telemetry; permitted events exclude app content and credentials.
- [ ] Operators detect stalled signing, aging catalogs and release failures before widespread impact.

## Rollout, migration and recovery

Ship local diagnostics and server metrics first, then optional client reporting. Use a staging event sink and redaction fixtures before enabling external collection.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(hub): add developer diagnostics and release health`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
