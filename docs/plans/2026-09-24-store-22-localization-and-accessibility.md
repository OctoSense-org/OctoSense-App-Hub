# 22. Localization, accessibility and developer quality tooling Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the store and SDK usable across supported languages, input methods and accessibility needs.

**Architecture:** Add locale-aware listing resources through a versioned contract and reuse native accessibility semantics in store/kit widgets. SDK checks should validate semantic structure and real input journeys rather than infer accessibility from screenshots.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P1. **Phase:** E — Public free-store reliability. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [05 — Versioned runtime contracts and release compatibility](2026-09-24-store-05-runtime-compatibility.md); [06 — Card application lifecycle, state and effects](2026-09-24-store-06-card-application-runtime.md); [09 — Installable SDK, pinned runtimes and runnable starters](2026-09-24-store-09-sdk-and-starters.md); [10 — Development loop, signed device preview and editor tooling](2026-09-24-store-10-development-and-testing.md); [15 — Mobile uninstall, complete listings and user controls](2026-09-24-store-15-mobile-app-management.md)

**Review coverage:** R14, R24 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Modify | [H/crates/app-policy/src/listing.rs](../../crates/app-policy/src/listing.rs) |
| Create | [H/crates/app-policy/tests/localized_listing.rs](../../crates/app-policy/tests/localized_listing.rs) |
| Create | [H/crates/app-validator/src/accessibility.rs](../../crates/app-validator/src/accessibility.rs) |
| Create | [H/crates/app-validator/tests/accessibility.rs](../../crates/app-validator/tests/accessibility.rs) |
| Modify | [M/apps/app-hub/src/view.rs](../../../OctoSense-mobile/apps/app-hub/src/view.rs) |
| Modify | [M/apps/app-hub/src/catalog.rs](../../../OctoSense-mobile/apps/app-hub/src/catalog.rs) |
| Modify | [K/crates/octoscript-widgets/src/kit.rs](../../../octoscript-makepad/crates/octoscript-widgets/src/kit.rs) |
| Create | [H/templates/notes/locales/en.json](../../templates/notes/locales/en.json) |
| Create | [H/docs/ACCESSIBILITY.md](../../docs/ACCESSIBILITY.md) |
| Create | [H/docs/LOCALIZATION.md](../../docs/LOCALIZATION.md) |
| Modify | [H/crates/app-validator/src/lib.rs](../../crates/app-validator/src/lib.rs) |
| Modify | [H/crates/app-validator/Cargo.toml](../../crates/app-validator/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
{
  "default_locale": "en",
  "localizations": {
    "en": {"name":"Notes","description":"Keep notes","release_notes":"First release"},
    "zh-CN": {"name":"笔记","description":"记录笔记","release_notes":"首次发布"}
  }
}
// Locale selection has a deterministic default fallback.
// Controls carry role/name/state/action; text scale and direction are inputs.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Version listing localization safely

**Touch:** `H/crates/app-policy/src/listing.rs`, `H/crates/app-policy/tests/localized_listing.rs`.

1. **Write the regression/acceptance case** `localized_listing_falls_back_without_changing_signature`: Select supported/unsupported locales and verify default fallback. Preserve v1 signature golden fixtures and reject dangling localized asset references.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Introduce v2 localized fields with length/asset validation and explicit fallback rules; keep localized claims under the bundle/catalog signature.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Make native controls accessible

**Touch:** `M/apps/app-hub/src/view.rs`, `K/crates/octoscript-widgets/src/kit.rs`.

1. **Write the regression/acceptance case** `install_consent_and_uninstall_work_without_touch`: Traverse Today/Search/Details/Consent/Library/Uninstall with keyboard and platform accessibility tools; verify focus, labels, roles and announced progress/errors.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Expose semantics in native widgets and avoid color-only status. Preserve focus on reload and modal transitions; standardize minimum touch targets and readable permission text.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Handle text and locale rendering

**Touch:** `M/apps/app-hub/src/view.rs`, `H/templates/notes/locales/en.json`.

1. **Write the regression/acceptance case** `large_text_rtl_cjk_do_not_hide_primary_actions`: Render long translations, CJK, RTL and enlarged text on supported form factors; exercise input and action visibility, not only screenshot differences.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Bundle/license appropriate font coverage with the SDK, use direction-aware layouts and locale-aware formatting. Fix truncation and scrolling that blocks consent or deletion controls.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Give authors automated feedback

**Touch:** `H/crates/app-validator/src/accessibility.rs`, `H/docs/ACCESSIBILITY.md`, `H/docs/LOCALIZATION.md`.

1. **Write the regression/acceptance case** `missing_control_label_has_source_diagnostic`: Validate missing accessible names, focus traps and contrast issues with known positive/negative fixtures; require source locations and avoid false claims for checks needing human review.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add hub test accessibility checks and manual device/screen-reader checklist. Include localized starter examples and document responsibilities at app, kit and shell boundaries.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-policy --test localized_listing
cargo test --locked -p octosense-app-validator --test accessibility
(cd ../OctoSense-mobile && cargo test --locked -p octosense-app-hub-app --lib)
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Store install/update/uninstall journeys are usable with supported accessibility input.
- [ ] SDK examples render supported CJK/RTL/large-text cases without hiding critical actions.
- [ ] Localized listing selection preserves verified metadata and provides reliable fallbacks.

## Rollout, migration and recovery

Accessibility basics apply to earlier UI work immediately; this plan is the conformance gate before a broad free-store launch. Add locales only with tested translations/font coverage.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(store): add accessible localized app experiences`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
