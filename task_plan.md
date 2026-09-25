# App Hub store-readiness implementation

Goal: implement the reviewed App Hub roadmap in dependency order, prioritizing free Card apps; native distribution and payments follow later.

## Phases
1. Inventory current architecture, instructions and author documentation — complete.
2. Trace publisher admission, distribution, runtime and mobile lifecycle — complete.
3. Validate core behavior and compare with official store developer workflows — complete.
4. Produce an evidence-backed readiness report and prioritized roadmap — complete.
5. Convert all review findings into a ranked, dependency-aware delivery roadmap — complete.
6. Write a separate implementation plan for every feature/workstream — complete.
7. Validate issue coverage, file paths, plan dependencies and acceptance gates — complete.

8. Implement and verify publisher key continuity (plan 01) — verified locally.
9. Implement and verify installed-version launch (plan 02) — verified locally.
10. Implement runnable bundle admission (plan 03) — verified locally; consumer pin checkpoint in progress.
11. Continue remaining plans in dependency order — pending.

Implementation worktree: `/Users/guofoo/git/octosense/app-hub-store-work`, branch `feat/app-store-foundations`.

## Constraints
- Implementation authorized by the user on 2026-09-25. Work on isolated branches; no production deployment or commercial activation is implied.
- Preserve existing work and outer-workspace planning files from unrelated tasks.
- Distinguish implemented code, documented intent, live deployment and recommendations.

## Errors
- Optional .github inventory returned missing-directory status; git tracked-file inventory confirms no workflow directory. This is evidence, not a build failure.
- An exploratory search assumed a kit directory; runtime kit implementation is a file. Corrected by searching actual runtime tree.
- Planned publish-app repository API returned 404; report as unavailable to this session, without asserting no private implementation exists.
- Plan inventory confirmed docs/plans and Hub service/workflow directories do not exist yet; new paths will be marked Create. Corrected a cross-repository inventory command to use owning repository directories.
- First plan consistency check found three nonreciprocal coverage references; corrected them and reran successfully with zero errors.
