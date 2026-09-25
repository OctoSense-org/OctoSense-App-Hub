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
