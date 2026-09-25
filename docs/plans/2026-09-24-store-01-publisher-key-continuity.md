# 01. Publisher key continuity and signed release identity Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reject replacement signing keys and mismatched publisher identities before a release can enter the catalog.

**Architecture:** Keep the existing v1 catalog usable. Resolve update verification from previously trusted key material rather than a submission-provided name-to-key mapping; introduce an explicit registry interface that the later publishing service can implement.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Implemented and verified locally on 2026-09-25; not deployed. **Priority:** P0. **Phase:** A — Correctness. **Relative size:** S (complexity, not a delivery-date estimate).

**Prerequisites:** None; can start against the reviewed baseline.

**Review coverage:** R01, R04 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Modify | [H/crates/app-hub/src/gate.rs](../../crates/app-hub/src/gate.rs) |
| Modify | [H/crates/app-hub/src/signing.rs](../../crates/app-hub/src/signing.rs) |
| Modify | [H/crates/app-hub/src/bin/hub.rs](../../crates/app-hub/src/bin/hub.rs) |
| Create | [H/crates/app-hub/src/publishers.rs](../../crates/app-hub/src/publishers.rs) |
| Modify | [H/crates/app-hub/src/lib.rs](../../crates/app-hub/src/lib.rs) |
| Create | [H/crates/app-hub/tests/publisher_continuity.rs](../../crates/app-hub/tests/publisher_continuity.rs) |
| Modify | [H/docs/PUBLISHING.md](../../docs/PUBLISHING.md) |
| Modify | [H/crates/app-hub/Cargo.toml](../../crates/app-hub/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
pub struct PublisherBinding {
    pub publisher_id: String,
    pub key_id: String,
    pub public_key_hex: String,
}
// For an existing app, derive this binding from trusted state.
// A submission may name a key; it must never redefine that binding.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Reject the reproduced substitution

**Touch:** `H/crates/app-hub/tests/publisher_continuity.rs`, `H/crates/app-hub/src/gate.rs`, `H/crates/app-hub/src/signing.rs`.

1. **Write the regression/acceptance case** `replacement_key_same_id_is_refused`: Copy the review probe's fresh-key fixture into an integration test: admit v1 with K1, then sign v2 with K2 using the same key ID. The update must fail for key continuity, while v2 signed with K1 succeeds.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Build the verifier for existing apps from the trusted prior binding. Reject a supplied mapping that disagrees with it before signature verification.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Bind all publisher fields

**Touch:** `H/crates/app-hub/src/publishers.rs`, `H/crates/app-hub/src/gate.rs`, `H/crates/app-hub/src/bin/hub.rs`.

1. **Write the regression/acceptance case** `entry_publisher_must_match_signature_owner`: Exercise different --publisher, signature.key_id, registry owner and stored publisher values; include two apps using the same trusted publisher key.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Make entry creation validate owner/key association, rather than accepting independent strings. Keep human publisher display names outside the key identity namespace.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Make signing mandatory on the public path

**Touch:** `H/crates/app-hub/src/bin/hub.rs`, `H/docs/PUBLISHING.md`.

1. **Write the regression/acceptance case** `public_publish_rejects_unsigned_first_release`: Attempt first publication and update without signatures. Explicit local unsigned development remains usable; the production path must reject both.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Separate development check flags from the public admission profile. Correct the guide's optional first-release signing advice; preserve --allow-unsigned only for local checks/fixtures.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Prepare explicit rotation without enabling it

**Touch:** `H/crates/app-hub/src/publishers.rs`, `H/crates/app-hub/src/signing.rs`.

1. **Write the regression/acceptance case** `unknown_rotation_requires_registry_authorization`: An unknown/new key is denied even when the account or key label matches. Existing legacy entries with conflicting bindings require operator reconciliation instead of choosing the last entry.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Expose a read-only trusted binding interface and reject ambiguous catalog mappings. Plan 19 supplies authenticated rotation records later; do not add an override bypass now.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-hub --test publisher_continuity
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [x] The exact different-key/same-ID reproduction fails; a same-key update passes.
- [x] No submission field can overwrite the trusted verification key; public first releases are signed.
- [x] Existing signed v1 catalogs continue to verify without reserialization changes.

## Rollout, migration and recovery

Deploy the gate fix before any automation accepts publisher-supplied keys. Inventory conflicting legacy mappings offline; the currently reviewed catalog is empty, but do not build that assumption into the code.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `fix(hub): enforce trusted publisher key continuity`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.

## Implementation evidence — 2026-09-25

- Added a read-only catalog publisher registry; conflicting historical keys/owners fail closed. No rotation override.
- Gate verifies updates and new apps from an existing publisher against the historical public key; entry creation binds publisher, signature, full manifest and content to the gate report.
- `hub publish` requires signatures, and checking/publishing against an existing catalog requires `--anchor` authentication. Local unsigned checks remain supported.
- Observed seven intended regression failures before implementation. All 12 publisher tests and the 42 preexisting Hub/policy tests then passed. Existing v1 catalog verifies unchanged.
- Independent code review found no important issues. Corrected the tiny PNG fixture CRC found during review.
- No new dependencies or wire format changes. First publisher enrollment and authorized rotation remain plans 11/19.
