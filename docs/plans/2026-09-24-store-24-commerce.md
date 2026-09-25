# 24. Paid apps, entitlements, subscriptions and settlement Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add paid distribution only after the free-store workflow is reliable and the commercial operating model is approved.

**Architecture:** Keep billing behind a provider adapter and maintain an auditable entitlement ledger independent of catalog approval. Use hosted provider checkout and verified webhooks; no payment-card handling belongs in Card apps or Hub servers.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P2 — Optional. **Phase:** G — Optional product tracks. **Relative size:** XL (complexity, not a delivery-date estimate).

**Prerequisites:** [11 — Publisher authentication, namespace ownership and registry](2026-09-24-store-11-publisher-accounts.md); [13 — Review decisions, publisher feedback and abuse handling](2026-09-24-store-13-review-and-moderation.md); [16 — Private testing, release channels and controlled rollouts](2026-09-24-store-16-release-channels.md); [19 — Publisher teams, transfer, key recovery and trust revocation](2026-09-24-store-19-publisher-recovery-and-trust.md); [20 — Release diagnostics, developer console and operations health](2026-09-24-store-20-diagnostics-and-developer-console.md)

**Review coverage:** R25 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Create | [H/crates/hub-commerce/Cargo.toml](../../crates/hub-commerce/Cargo.toml) |
| Create | [H/crates/hub-commerce/src/lib.rs](../../crates/hub-commerce/src/lib.rs) |
| Create | [H/crates/hub-commerce/src/provider.rs](../../crates/hub-commerce/src/provider.rs) |
| Create | [H/crates/hub-commerce/src/entitlements.rs](../../crates/hub-commerce/src/entitlements.rs) |
| Create | [H/crates/hub-commerce/src/settlement.rs](../../crates/hub-commerce/src/settlement.rs) |
| Create | [H/crates/hub-commerce/tests/commerce_lifecycle.rs](../../crates/hub-commerce/tests/commerce_lifecycle.rs) |
| Create | [H/crates/hub-service/src/commerce.rs](../../crates/hub-service/src/commerce.rs) |
| Modify | [H/Cargo.toml](../../Cargo.toml) |
| Modify | [M/apps/app-hub/src/view.rs](../../../OctoSense-mobile/apps/app-hub/src/view.rs) |
| Create | [H/docs/commerce/design.md](../../docs/commerce/design.md) |
| Modify | [H/crates/hub-service/src/lib.rs](../../crates/hub-service/src/lib.rs) |
| Modify | [H/crates/hub-service/Cargo.toml](../../crates/hub-service/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |
| Create | [H/crates/hub-commerce/migrations/001_commerce.sql](../../crates/hub-commerce/migrations/001_commerce.sql) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
Product { app_id, product_id, kind: "paid-app" | "subscription", currency, price }
Entitlement { account_id, product_id, state, valid_until, event_version }
PaymentEvent { provider_event_id, transaction_id, type, verified_at }
# Idempotent provider events -> ledger -> entitlement.
# App approval never implies purchase; a claimed client receipt grants nothing.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Record commercial scope and provider contract

**Touch:** `H/docs/commerce/design.md`, `H/crates/hub-commerce/src/provider.rs`.

1. **Write the regression/acceptance case** `fake_provider_supports_complete_lifecycle`: Define paid-app vs in-app purchase/subscription scope, supported markets, seller/merchant responsibility and refund/support ownership. Use a deterministic fake provider for all subsequent tests.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Write a decision record comparing managed marketplace settlement with operator-managed settlement. Choose provider/business requirements before live integration; verify current provider/platform rules at implementation time.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Implement acquisition and entitlement

**Touch:** `H/crates/hub-commerce/src/entitlements.rs`, `H/crates/hub-commerce/tests/commerce_lifecycle.rs`.

1. **Write the regression/acceptance case** `forged_or_duplicate_payment_grants_nothing_extra`: Simulate successful/failed/cancelled checkout, forged receipt, duplicate/out-of-order webhooks and account restore. Entitlements follow only verified durable events.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add hosted checkout creation, webhook verification, idempotent event ledger and server-side entitlement query. Keep purchased-app access separate from catalog compatibility/revocation.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Handle renewals, refunds and chargebacks

**Touch:** `H/crates/hub-commerce/src/entitlements.rs`, `H/crates/hub-service/src/commerce.rs`.

1. **Write the regression/acceptance case** `refund_and_expiry_update_entitlement_consistently`: Exercise subscription renewal/grace/expiry, partial/full refund, chargeback and provider outage. Specify offline grace and restore behavior without silently destroying user data.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Build a versioned entitlement state machine and reconciliation job; user-visible states explain access and recovery. Repeated events and reordered delivery cannot double-grant or double-revoke.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Reconcile publisher settlement

**Touch:** `H/crates/hub-commerce/src/settlement.rs`, `H/docs/commerce/design.md`.

1. **Write the regression/acceptance case** `settlement_totals_reconcile_with_provider`: Use provider sandbox fixtures for gross receipts, fees, refunds, reserves and payout adjustments. Every reported amount traces to ledger/provider records.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Implement publisher payout onboarding through the provider, reporting and reconciliation; resolve tax/consumer/distribution obligations with the chosen provider and qualified owners before live launch rather than coding assumptions.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Expose purchasing and support

**Touch:** `M/apps/app-hub/src/view.rs`, `H/crates/hub-service/src/commerce.rs`.

1. **Write the regression/acceptance case** `price_and_terms_are_clear_before_checkout`: Show localized price/currency, subscription terms, restore, receipts and refund/support routes. Prevent double-tap duplicate checkout and purchases of unavailable products.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Integrate optional commerce screens without burdening free apps. Audit accessibility and error recovery; run the full provider sandbox journey before requesting any production payment activation.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-hub-commerce --test commerce_lifecycle
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Verified provider events are the sole authority for commercial entitlements.
- [ ] Purchase/restore/renewal/refund/chargeback flows are idempotent and auditable.
- [ ] Provider sandbox reconciliation and user-support ownership are complete before live charges.

## Rollout, migration and recovery

Explicitly excluded from the first free Card release. Implementation can begin with fake adapters; real provider accounts, money movement and public activation require a later concrete product/deployment decision.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(commerce): add provider-backed app entitlements`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
