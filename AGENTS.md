# Working on App Hub

Read [README.md](README.md), the
[code walkthrough](docs/CODE-WALKTHROUGH.md), and the relevant crate before
editing. This repository owns the app contract, admission, catalog/signing,
store, contained runner and shared host-service transport. OctoSense owns
the shells, first-party services and live agent integration; Design Flow
owns authoring workflows.

- Trace behavior to executable code. Distinguish manifest validation,
  resolved grants, shell registration and an actual executor. `AgentBundle`
  loading alone does not start a peer or install instructions/skills.
- Keep native `AppModule`, Splash script bundles and Octoscript L0 cards
  distinct. `tools/octo` is Design Flow's Python CLI; octos is the agent
  kernel. `card-host` supplies no registered host services or agent runtime.
- For agent/storage changes, also inspect OctoSense `crates/shell/src/host_tools`,
  `crates/shell/src/app_storage`, `crates/ai-host` and `crates/app-peers` at
  the consumer's actual pin. Do not claim store script tools run merely
  because `tools.json` passes the gate.
- Preserve contract compatibility and refusal behavior. Consult
  [app-contract's compatibility rules](crates/app-contract/README.md).
  Do not widen an app's resolved grants in a mounting path.
- Use the prepared sibling source layout from `Cargo.toml`; do not replace
  pins or regenerate locks as a side effect of a documentation edit.
- Match validation to the change: contract/policy/hub tests for admission,
  `card-host` CLI tests for arguments, relevant service tests for lifecycle
  changes. Documentation-only changes need source/link/command checks;
  report unavailable builds or devices explicitly.
- In walkthroughs, lead with a launch path, then trace one request to its
  reply. Define routing terms at first use and keep source inventories after
  the main path. Preserve prerequisites and capability limits in examples.
- Keep English and Chinese README summaries aligned. Preserve historical
  evidence and label source-reviewed commands separately from commands run.
- Use hidden windows for automated GUI checks, a distinct bridge port and
  app-data directory per instance, and quit only instances you started.
- Do not change real `catalog.json`, `index/`, `artifacts/`, publisher keys
  or publication status as part of ordinary documentation/development work.
  Use a temporary local fixture for install tests; publishing follows
  [PUBLISHING.md](docs/PUBLISHING.md) and the user's authorized scope.

`AGENTS.md` instructs repository contributors. A bundle's singular
`AGENT.md` is runtime app-agent material with separate contract rules.
