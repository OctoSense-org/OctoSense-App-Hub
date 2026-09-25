# 21. Discovery, ratings, user reviews and publisher replies Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Help users find useful apps and give developers a trustworthy feedback channel.

**Architecture:** Keep signed release identity/permissions separate from mutable community content. Serve moderated reviews and aggregate discovery signals through the service, with deterministic ranking and optional authenticated acquisition eligibility.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P2. **Phase:** F — Marketplace expansion. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [11 — Publisher authentication, namespace ownership and registry](2026-09-24-store-11-publisher-accounts.md); [13 — Review decisions, publisher feedback and abuse handling](2026-09-24-store-13-review-and-moderation.md); [20 — Release diagnostics, developer console and operations health](2026-09-24-store-20-diagnostics-and-developer-console.md)

**Review coverage:** R19, R22 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/hub-service/src/discovery.rs](../../crates/hub-service/src/discovery.rs) |
| Create | [H/crates/hub-service/src/ratings.rs](../../crates/hub-service/src/ratings.rs) |
| Create | [H/crates/hub-service/migrations/006_reviews_discovery.sql](../../crates/hub-service/migrations/006_reviews_discovery.sql) |
| Create | [H/crates/hub-service/tests/discovery_reviews.rs](../../crates/hub-service/tests/discovery_reviews.rs) |
| Modify | [M/apps/app-hub/src/catalog.rs](../../../OctoSense-mobile/apps/app-hub/src/catalog.rs) |
| Modify | [M/apps/app-hub/src/view.rs](../../../OctoSense-mobile/apps/app-hub/src/view.rs) |
| Create | [H/docs/policies/community-reviews.md](../../docs/policies/community-reviews.md) |
| Modify | [H/crates/hub-service/src/lib.rs](../../crates/hub-service/src/lib.rs) |
| Modify | [H/crates/hub-service/Cargo.toml](../../crates/hub-service/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
GET /v1/discovery?q=notes&category=productivity&cursor=...
POST /v1/apps/{app_id}/reviews
PUT /v1/reviews/{review_id}
POST /v1/reviews/{review_id}/publisher-reply
# Review: account, app, rating, text, locale, acquired_release, moderation_state.
# Acquisition eligibility is not proof of real usage; label it accurately.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Make search complete and measurable

**Touch:** `H/crates/hub-service/src/discovery.rs`, `M/apps/app-hub/src/catalog.rs`.

1. **Write the regression/acceptance case** `search_matches_localized_title_keywords_and_category`: Search multiword queries, publisher names, keywords, categories and empty results; validate stable pagination and ranking with fixed fixtures.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Index only eligible public listings. Start with transparent lexical relevance plus explicit curation, exclude incompatible/private/withdrawn offers as appropriate, and preserve offline cached search.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Collect bounded authenticated reviews

**Touch:** `H/crates/hub-service/src/ratings.rs`, `H/crates/hub-service/tests/discovery_reviews.rs`.

1. **Write the regression/acceptance case** `one_account_has_one_current_review_per_app`: Submit duplicate ratings, edit/remove a review, forge an acquisition receipt and review a private app without access.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add consumer identity linkage, rate limits and an explicit eligibility policy; keep historical edits for moderation while counting only the current eligible rating.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Moderate and support replies

**Touch:** `H/crates/hub-service/src/ratings.rs`, `H/docs/policies/community-reviews.md`.

1. **Write the regression/acceptance case** `publisher_cannot_edit_user_review`: Attempt publisher deletion of criticism, cross-publisher replies and abusive content reports. Only authorized moderation alters visibility; publisher replies are attributed.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Reuse Plan 13's reporting/appeal system. Record moderation reasons and review changes; render user text safely and avoid disclosing account contact information.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Build honest discovery surfaces

**Touch:** `M/apps/app-hub/src/view.rs`, `H/crates/hub-service/src/discovery.rs`.

1. **Write the regression/acceptance case** `zero_reviews_shows_no_fabricated_rating`: Display apps with zero reviews/download history, withdrawn releases, small samples and reviewed replies. No invented star scores or popularity labels.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add ratings/replies screens, publisher pages and curated shelves. Label editorial placement; expose ranking rules and avoid using sparse crash telemetry as popularity.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-hub-service --test discovery_reviews
(cd ../OctoSense-mobile && cargo test --locked -p octosense-app-hub-app --lib)
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Search/discovery use real listings and transparent signals with no invented ratings.
- [ ] Reviews and replies have clear ownership, moderation and abuse controls.
- [ ] Community content cannot modify verified manifest permissions or impersonate publisher identity.

## Rollout, migration and recovery

Ship better search and publisher pages first. Enable ratings only after consumer identity, moderation staffing and sufficient app usage exist.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(store): add discovery and moderated app reviews`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
