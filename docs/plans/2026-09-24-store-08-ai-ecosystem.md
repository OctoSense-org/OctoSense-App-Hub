# 08. Contained app tools and AI ecosystem integration Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Let a Card app expose approved tools and request a bounded agent session without inheriting shell-wide privileges.

**Architecture:** Use the existing AI service bus and per-instance executor instead of a new agent framework. Derive session/tool access from the signed app policy and route responses to the requesting live app instance.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P1. **Phase:** B — Application platform. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [05 — Versioned runtime contracts and release compatibility](2026-09-24-store-05-runtime-compatibility.md); [06 — Card application lifecycle, state and effects](2026-09-24-store-06-card-application-runtime.md); [07 — Host services, capability matrix and structured network policy](2026-09-24-store-07-host-services.md)

**Review coverage:** R07, R27 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/app-runtime/src/agent.rs](../../crates/app-runtime/src/agent.rs) |
| Create | [H/crates/app-runtime/tests/agent_integration.rs](../../crates/app-runtime/tests/agent_integration.rs) |
| Modify | [H/crates/app-policy/src/containers.rs](../../crates/app-policy/src/containers.rs) |
| Modify | [H/crates/app-policy/src/manifest.rs](../../crates/app-policy/src/manifest.rs) |
| Modify | [H/crates/appstore/src/cardapp.rs](../../crates/appstore/src/cardapp.rs) |
| Modify | [H/crates/card-host/src/main.rs](../../crates/card-host/src/main.rs) |
| Modify | [M/src/ai_bus.rs](../../../OctoSense-mobile/src/ai_bus.rs) |
| Modify | [F/libs/ai/services/src/engine/core.rs](../../../makepad/libs/ai/services/src/engine/core.rs) |
| Create | [H/docs/reference/app-tools.md](../../docs/reference/app-tools.md) |
| Modify | [H/crates/app-runtime/src/lib.rs](../../crates/app-runtime/src/lib.rs) |
| Modify | [H/crates/app-runtime/Cargo.toml](../../crates/app-runtime/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
{
  "tool": "notes.search",
  "input_schema": {"type":"object","properties":{"query":{"type":"string"}},"required":["query"]},
  "effect": "read",
  "required_capabilities": ["storage"],
  "timeout_ms": 3000
}
// Signed tool declarations define the ceiling. Runtime consent and OS grants
// may reduce it. Agent session.workspace is the app data jail, never code.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Define tool schema and policy intersection

**Touch:** `H/crates/app-policy/src/manifest.rs`, `H/crates/app-policy/src/containers.rs`.

1. **Write the regression/acceptance case** `tool_cannot_exceed_app_grants`: Declare a storage read tool without storage, a write tool under read-only policy and an unknown input field. All must fail without invoking app code.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Validate tool definitions in the v2 manifest. Intersect app capabilities, offered host tools, session profile and current consent rather than accepting tools independently.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Connect the installed executor

**Touch:** `H/crates/appstore/src/cardapp.rs`, `M/src/ai_bus.rs`, `H/crates/app-runtime/src/agent.rs`.

1. **Write the regression/acceptance case** `installed_app_tool_is_routed_to_its_instance`: Register two instances/apps with similar tool names and call one. Only the addressed authorized instance receives the call; closed apps unregister.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Replace the CardExecutor unavailable placeholder with a runtime adapter using the existing bus. Bind registration to host identity and serialize bounded JSON results.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Start and stop bounded sessions

**Touch:** `H/crates/app-runtime/src/agent.rs`, `F/libs/ai/services/src/engine/core.rs`, `H/crates/card-host/src/main.rs`.

1. **Write the regression/acceptance case** `agent_session_respects_jail_hosts_and_budget`: Use a deterministic fake model/engine to request tool calls beyond host limits, exhaust iterations/tokens, cancel, and close the app. Assert denial/cancellation and no residual session.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Create the actual session from SessionProfile through the existing engine adapter. Add a narrow trait if an engine entrypoint is absent; first implement fake and local engine adapters, without choosing a paid model provider.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Expose meaningful user controls

**Touch:** `H/crates/app-runtime/src/agent.rs`, `H/docs/reference/app-tools.md`.

1. **Write the regression/acceptance case** `assistant_disabled_preserves_core_app_behavior`: Run the example with no AI provider or with assistant permission disabled. Notes remain usable and AI actions explain availability/cost scope.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add app-level session status, cancellation and budget feedback; shared-data writes require explicit mediated authorization and provenance. Reject opaque prompt text as authority.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-runtime --test agent_integration
(cd ../OctoSense-mobile && cargo test --locked --bin octosense --features mobile-only,app-hub ai_bus)
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Installed app tools work through the existing assistant bus with validated inputs.
- [ ] An app's agent cannot use broader hosts, files or tools than its signed policy and current consent.
- [ ] Reference host executes the session path rather than only logging the profile.

## Rollout, migration and recovery

Ship disabled unless an engine is available; use deterministic mock integration in CI and opt-in real-provider checks. Do not label AI integration supported before this plan is complete.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(runtime): integrate scoped app tools and agent sessions`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
