# Review progress

- Bootstrapped requested Superpowers instructions and loaded planning-with-files.
- Located Hub code, documentation and mobile integration; inspected repository status and recent history.
- Started outside-developer journey review. No application changes made.
- Read CLI publish/gate/policy/client and mobile installation/catalog paths; identified update policy and publisher continuity concerns for reproduction.
- Started locked offline policy and hub tests.
- Consulted official store workflow documentation for comparison.
- Completed 78 existing tests successfully; isolated probe reproduced three release-blocking gaps without changing application source.
- Verified live repository/catalog and publishing workflow availability through read-only requests.
- Completed source review of current mobile UX, runtime host wiring and reference-host limitations; preparing final report.
- Wrote docs/reviews/2026-09-24-store-readiness.md with capability coverage, six launch blockers, concrete evidence, release phases and acceptance criteria.
- Verified all 29 local evidence links resolve. Hub tracked source has no diff; only review documents/planning notes are untracked. Probe remains ignored under target/. Mobile's pre-existing planning changes are unchanged.

## Implementation planning

- User requested a prioritized plan addressing every review issue plus implementation plans for individual features.
- Loaded brainstorming and writing-plans skills; retained planning-with-files workflow.
- Asked optional first-release scope preference while continuing with dependency and source mapping; recommended free Card apps first.
- Planning only: no application implementation, publication or deployment authorized by this follow-up.
- Confirmed the user's free-Card-first scope and wrote all 26 feature plans plus the shared architecture/execution guide.
- Plans explicitly cover signing custody, legacy signature preservation, immutable code/data separation, private artifacts and release recovery.
- Building the master priority table, dependency graph and review-to-plan coverage matrix, then checking file/dependency consistency.
- Finalized planning on 2026-09-25, retaining 2026-09-24 filenames for the review/planning session.
- Created 28 documents: ranked roadmap, shared design/execution guide and 26 feature implementation plans. They contain 122 task slices and 78 acceptance criteria covering all 29 review items.
- Final consistency verifier passed: all review coverage/cross-references, dependency graph, planned/existing file ownership, document links, task counts and code fences; zero errors.
- Confirmed no tracked application-source diff in Hub; preserved the mobile repository's existing planning-file changes. No feature implementation, commits, issue tracker writes or deployment performed.

## 2026-09-25 — Implementation begins

- User authorized implementing each item; free Card apps remain first-release scope.
- Created isolated `feat/app-store-foundations` branch/worktree beside existing repositories to preserve relative Cargo patches.
- Copied review and implementation plans into the worktree; existing shared checkouts preserved.
- Starting plan 01 with regression tests for publisher identity and key continuity.

## Plan 01 verified — 2026-09-25

- Twelve publisher/CLI regressions pass, with seven intended failures observed before the fix. Existing 42 Hub/policy tests remain green.
- Authenticated catalog history now anchors actual publisher keys; unsigned public releases, ambiguous legacy mappings, mismatched owners and reused reports after edits are refused.
- Independent code review found no important issues; corrected a PNG fixture CRC.
- Plan 02 regression suite reproduces all seven launch/manifest defects after correcting a test field name (`version`). No plan 02 production changes yet.

## Plan 02 implementation — 2026-09-25

- Exact installed-release lookup now verifies the complete admitted manifest, canonical bytes and catalog-bound publisher signature. Open and Update are independent in shared and mobile store UI.
- Launch workers create bounded owned snapshots, then recheck current catalog metadata before startup. Snapshots survive concurrent updates and disappear on normal shutdown. Active mobile instances close only for their exact revoked release; pending approvals are invalidated by a catalog refresh.
- Reviewer found a verify/start race and delayed Remove/Update race; both were fixed with owned code snapshots and pending-launch cancellation. Independent follow-up review found no remaining important issues.
- Verification: 68 Hub/policy tests; 41 mobile App Hub tests; one real native shared-store action regression, all passed. Full mobile shell checked successfully against its pinned framework/runtime. New cancellation test failed when its fix was removed, then passed when restored.
- Build preparation initially used the older shared Makepad checkout and hit an existing windows API mismatch. Prepared separate runtime checkouts with tools/setup-native.py; no shared runtime source was changed. Validated Makepad 1d3d383, Octoscript-Makepad c4c9682, Octoscript 68f6a9d.
- Native widget evaluation and shell compilation are host evidence; no Android/device rollout claimed. Coordinated Hub revision-pin verification is the remaining integration checkpoint.

- Coordinated integration passed: mobile pins published Hub commit `0269a85b0b2166de3866464716f06dce3e69ab1a`; full shell check and 41 mobile tests passed again using fetched Git dependencies, without a local Hub patch. Framework overrides point to exact pinned revisions in isolated checkouts. Hub feature branch pushed; no main merge or deployment.
- Beginning plan 03: bounded structural admission, shared runtime preparation and isolated validation evidence.

## Plan 03 structural slice — 2026-09-25

- Added bounded inventory, UTF-8/JSON/Card syntax and kit closure checks; full PNG/JPEG/WebP decode, SVG resource checks, square icons, structured diagnostics and typed resource inventories. Public `hub check --json` and library reports match.
- Packing and unpacking now enforce size/entry/depth limits before writes and refuse nonempty/symlink staging. Tests reproduced both oversized unpacking and staging-symlink escapes before fixes.
- Twenty admission tests pass, including seven initial structural failures and reviewer regressions. Retained conservative text URL denial pending plan 07.
- Review found Card/kit expansion was not bounded by input size. Removed realization from the headless Hub process; full preparation must occur in the upcoming constrained runtime worker. Cached token weights, checked every SVG CSS URL, and avoided interpreting kit property declarations as resource paths or theme overlays as native-only Cards.
- Added dependencies from cached pinned releases: existing image 0.25.10 with PNG/JPEG/WebP codecs, roxmltree 0.21.1 and the already-used Octoscript L0 revision. Policy core remains free of graphical dependencies. Image maximum dimensions are strict; decoded output bytes are checked explicitly before allocation because decoder max_alloc is advisory (https://docs.rs/image/0.25.10/image/struct.Limits.html).
- Plan 03 remains in progress: shared native preparation, isolated runtime evidence, publication binding and native end-to-end verification are still required.

## Plan 03 native admission slice — 2026-09-25

- Added the native worker, shared Card preparation, owned validation snapshots, full manifest/payload/runtime binary binding and mandatory publish checks. Reference and installed hosts use the same preparation function.
- Native smoke uses actual Splash + the existing policy adapter, temporary storage, startup and shutdown. Review found fixed evaluator budgets diverged from installed policies; regressions for tiny memory/instruction budgets now pass using the actual host.
- Worker timeout/crash/output/environment checks pass; reviewer subprocesses retain their configured cwd/environment after a compatibility regression was fixed. Worker Rust allocations are capped at 256 MiB before Card expansion, with a real expansion regression.
- JSON refusal regressions failed before fixes; hub test now preserves structural findings and returns a stable runtime failure envelope, also emitted by the worker.
- Current full suite: 101 tests pass (Hub/policy/native validator, including doctest). A native CLI signed-fixture/publication/catalog check and consumer integration are being verified before checkpoint.
- Evidence scope is macOS native widget construction and lifecycle. The static fixture has no declared interactions; GPU rendering, Android and application-state conformance remain separate (plans 06/26). Initial compile needed closures to discard script_mod return values; corrected with no framework edits.
- Native end-to-end CLI journey passed: ephemeral publisher/anchor/working keys, signed real Card fixture, native hub test, publication, catalog signature verification and identical runtime sidecar. No production publication. Independent follow-up review found no important issues.
- Mobile local-consumer integration passed: 41 App Hub tests and the native shared-store action regression, using isolated exact mobile runtime revisions.
- Exact mobile runtime verification also passed all seven native-validator cases from the Hub workspace, plus the full mobile shell check. Cargo cannot test dev-dependencies of a non-member dependency; reran from the owning Hub workspace with runtime overrides, then restored its normal lockfile. No framework source changes.
