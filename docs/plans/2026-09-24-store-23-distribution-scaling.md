# 23. Efficient artifacts and scalable signed catalog delivery Implementation Plan

> **For implementers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Scale catalog browsing and artifact downloads while preserving verification, revocation and offline behavior.

**Architecture:** Measure the existing static delivery first, then introduce immutable compressed transport and a signed catalog root referencing content-addressed pages. Keep v1 delivery available and preserve independent revocation/installed-release verification.

**Tech Stack:** Rust, serde/JSON, the existing Hub policy/client, Makepad/Octoscript where applicable; additional service/storage adapters follow the [shared design](2026-09-24-app-store-design.md).

**Status:** Planned; no feature implementation is claimed. **Priority:** P2. **Phase:** F — Marketplace expansion. **Relative size:** L (complexity, not a delivery-date estimate).

**Prerequisites:** [04 — Atomic catalog publication, renewal and signing operations](2026-09-24-store-04-catalog-release-operations.md); [05 — Versioned runtime contracts and release compatibility](2026-09-24-store-05-runtime-compatibility.md); [18 — Download queue, resume, cancellation and automatic updates](2026-09-24-store-18-download-and-update-management.md); [20 — Release diagnostics, developer console and operations health](2026-09-24-store-20-diagnostics-and-developer-console.md)

**Review coverage:** R23 in the [roadmap coverage matrix](2026-09-24-app-store-roadmap.md#review-coverage).

Read the shared design first for repository aliases, wire-compatibility rules, isolated development, meaningful test requirements and coordinated revision-pin updates. File paths below are exact relative to their named repository. “Create” means new code; “Modify” may refer to a file introduced by a prerequisite plan.

## Files

| Action | Path |
| --- | --- |
| Modify | [H/crates/app-hub/src/pack.rs](../../crates/app-hub/src/pack.rs) |
| Modify | [H/crates/app-hub/src/remote.rs](../../crates/app-hub/src/remote.rs) |
| Create | [H/crates/app-hub/src/catalog_pages.rs](../../crates/app-hub/src/catalog_pages.rs) |
| Modify | [H/crates/app-hub/src/lib.rs](../../crates/app-hub/src/lib.rs) |
| Create | [H/crates/app-hub/tests/distribution_scaling.rs](../../crates/app-hub/tests/distribution_scaling.rs) |
| Create | [H/crates/hub-service/src/blob_store.rs](../../crates/hub-service/src/blob_store.rs) |
| Modify | [H/crates/hub-service/src/artifacts.rs](../../crates/hub-service/src/artifacts.rs) |
| Create | [H/tools/benchmark-distribution.py](../../tools/benchmark-distribution.py) |
| Create | [H/docs/operations/distribution.md](../../docs/operations/distribution.md) |
| Modify | [H/crates/app-hub/Cargo.toml](../../crates/app-hub/Cargo.toml) |
| Modify | [H/crates/hub-service/src/lib.rs](../../crates/hub-service/src/lib.rs) |
| Modify | [H/crates/hub-service/Cargo.toml](../../crates/hub-service/Cargo.toml) |
| Modify | [H/Cargo.lock](../../Cargo.lock) |

## Contract

The following is a proposed implementation contract, not an already-supported API:

```text
SignedCatalogRoot {
  schema, sequence, published, expires, pages: [{digest, length, path}],
  revocations_digest, release_lookup_root, signature
}
ArtifactTransport {
  format, compressed_length, uncompressed_length, transport_digest,
  directory_digest, immutable_url
}
// A page alone has no authority; verify it against the accepted signed root.
```

## Implementation tasks

Each task is a small reviewable slice. Apply the five-step test/implementation cycle in the shared design to each scenario below; split a slice further when it cannot be reviewed independently. Preserve already passing behavior and commit each completed slice with only its own files.

### Task 1: Set a measured scaling trigger

**Touch:** `H/tools/benchmark-distribution.py`, `H/docs/operations/distribution.md`.

1. **Write the regression/acceptance case** `catalog_benchmark_records_latency_memory_and_bytes`: Benchmark 10/1000/10000 app fixtures on target devices for cold/cached browse, memory and bytes. Record actual baselines and choose release budgets before redesigning.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Add a reproducible benchmark harness using fake immutable assets. Make paging/compression activation depend on observed size/latency/cost thresholds documented in the runbook.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 2: Add compact bounded transport

**Touch:** `H/crates/app-hub/src/pack.rs`, `H/crates/app-hub/tests/distribution_scaling.rs`.

1. **Write the regression/acceptance case** `compressed_pack_bombs_and_path_escapes_fail`: Round-trip valid bundles and reject decompression bombs, symlinks, duplicated paths, huge file counts and truncated streams before unbounded writes.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Introduce a versioned streaming transport with explicit file metadata/limits and transport hash while retaining the canonical directory digest. Select a maintained archive/codec implementation after format review; never invent unchecked unpacking.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 3: Page catalogs without losing trust

**Touch:** `H/crates/app-hub/src/catalog_pages.rs`, `H/crates/app-hub/src/remote.rs`.

1. **Write the regression/acceptance case** `missing_page_does_not_unrevoke_or_forget_installed_app`: Mix pages from two catalog generations, replay old revocation data and omit the page holding an installed release. Browsing may degrade, but installation/launch authorization cannot use incomplete trust evidence.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Verify root then page digests; keep installed-release proofs and required current revocation data independently of browse pagination. Fail closed on unavailable authorization metadata.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 4: Add object storage/CDN adapter

**Touch:** `H/crates/hub-service/src/blob_store.rs`, `H/docs/operations/distribution.md`.

1. **Write the regression/acceptance case** `origin_mirror_mismatch_is_detected`: Serve identical objects from origin/mirror, inject stale/corrupt cached content and drop the primary origin. Verification detects corruption; fallback does not weaken policy.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Implement BlobStore object adapter, immutable cache headers and availability probes. Pick provider/region only after benchmarks and operational requirements; retain local adapter for tests.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

### Task 5: Evaluate optional delta delivery

**Touch:** `H/crates/app-hub/src/pack.rs`, `H/crates/app-hub/tests/distribution_scaling.rs`.

1. **Write the regression/acceptance case** `failed_delta_falls_back_to_full_verified_bundle`: Apply a delta to the exact required base, then try a wrong/corrupt base or interrupted application. Never mutate the live bundle or trust the patch result without full verification.
2. **Run the relevant suite below before implementation.** Filter to that case where supported. Expected: the new behavior fails for the identified reason; if it already passes, inspect whether the scenario truly reaches the implementation and retain it only if it protects an uncovered contract.
3. **Implement:** Only add deltas if measured savings justify complexity. Stage against verified base bytes and check the complete reconstructed directory digest; otherwise record the decision to keep full compressed downloads.
4. **Verify:** rerun the new case and its containing suite. Expected: the scenario and existing affected behavior pass; record actual output. For device/UI scenarios, collect native evidence as well as headless assertions.
5. **Review and checkpoint:** inspect the diff for this slice, update its documentation and record the validation. Commit only the owning repository's files; do not stage unrelated work.

## Feature validation

Run from the Hub root unless a command changes directory. New packages/test targets are created by this plan or prerequisites. Use `--offline` only when dependencies are already cached; generate/update lockfiles once when intentionally adding dependencies, then use locked commands.

```sh
cargo test --locked -p octosense-app-hub --test distribution_scaling
python3 tools/benchmark-distribution.py --fixtures 10,1000,10000 --output target/distribution-benchmark.json
```

Expected after implementation: all listed suites pass with zero failures. These commands have **not** been run to claim completion of the proposed feature. Native/device checks described in the tasks are additional acceptance evidence; a host-only test is not platform coverage.

## Acceptance criteria

- [ ] Performance gains are measured against documented baselines on representative devices.
- [ ] Compressed/paged/delta transport preserves all signature/digest/path/revocation checks.
- [ ] Legacy clients retain a supported path during rollout; failures have a full-download fallback.

## Rollout, migration and recovery

Optimize only after measurements trigger the need. Roll out readers before writers and shadow-check new root/page/transport outputs before changing the default endpoint.

Keep the previous release/artifacts available while validating the new behavior. A catalog rollback publishes a newer signed sequence; never restore an older sequence to production. Preserve user data and report recovery failures rather than silently recreating it.

## Delivery checkpoint

Suggested commit subject after verified slices: `feat(distribution): add scalable verified artifact delivery`.

Use @superpowers:verification-before-completion before reporting success. Link the final test/native evidence and record updated dependency revisions in the owning pull requests. This planning document does not itself authorize deployment, credential creation, payments or public publication.
