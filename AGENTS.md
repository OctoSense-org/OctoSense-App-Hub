# Working on App Hub

Read [README.md](README.md), the
[code walkthrough](docs/CODE-WALKTHROUGH.md), and the relevant crate before
editing. This repository owns the app contract, admission, the catalog and its
signing, the store, the contained runner and the shared host-service
transport. OctoSense owns the shells, first-party services and live agent
integration; OctoSense App Flow (formerly Design Flow) owns authoring
workflows.

Do not confuse this file with a bundle's `AGENT.md` (singular), which
instructs an app's agent at runtime and follows the contract's rules
([`AGENT.md` and skills](docs/PUBLISHING.md#agentmd-and-skills)).

## Where each topic is documented

One document owns each topic; the others link to it. Change a fact in its
owner, and update the owner's `.zh-CN.md` twin, if it has one, in the same
change, with the same sections, in the same order and with the same content.

| Topic | Owner |
| --- | --- |
| Gate rules, capability names and who serves them, reserved ids, manifest, listing and tool fields, `hub` commands, signing | [docs/PUBLISHING.md](docs/PUBLISHING.md), the reference, and its `.zh-CN.md` twin |
| GitHub-admin catalog publication, environment protection and v2 migration | [docs/GITHUB-PUBLISHING.md](docs/GITHUB-PUBLISHING.md) and its `.zh-CN.md` twin |
| The rationale for GitHub publisher provenance over publisher keys, and for public app repositories | [docs/adr/0002-github-attested-publisher-identity.md](docs/adr/0002-github-attested-publisher-identity.md) and its `.zh-CN.md` twin |
| Host API declarations, `runtime` discovery and which hosts implement them | [docs/HOST-API.md](docs/HOST-API.md) and its `.zh-CN.md` twin |
| The submission's four stages and its steps: repository layout, release, issue fields, what reviewers check, approval and publication, common refusals, and who runs each `hub` command, in which step | [docs/SUBMITTING.md](docs/SUBMITTING.md) and its `.zh-CN.md` twin |
| A first app, from a template to an unsigned bundle that passes the gate | [docs/FIRST-APP.md](docs/FIRST-APP.md) and its `.zh-CN.md` twin |
| The contract's versions and crates.io status | [crates/app-contract/README.md](crates/app-contract/README.md#versions-on-cratesio) |
| Icons | [docs/ICONS.md](docs/ICONS.md) and its `.zh-CN.md` twin |
| Delivery paths, `card-host` flags and remote routes, `card-studio`, the `card-host` build failure | [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) and its `.zh-CN.md` twin |
| Code paths and the crate inventory | [docs/CODE-WALKTHROUGH.md](docs/CODE-WALKTHROUGH.md) |
| Workspace setup, building `hub` and `card-host`, `tools/octo`, running and capturing, the script API, how to use each capability | App Flow [docs/QUICKSTART.md](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/QUICKSTART.md), [docs/SCRIPT-API.md](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/SCRIPT-API.md), [docs/CAPABILITIES.md](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/CAPABILITIES.md) |
| Which shell serves which host service | App Flow [docs/HOST-SERVICES.md](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/HOST-SERVICES.md) |
| The three reference apps in detail | App Flow [examples/connected-apps/README.md](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/examples/connected-apps/README.md); [docs/SUBMITTING.md](docs/SUBMITTING.md) summarizes them |

`docs/FIRST-APP.md` repeats App Flow's setup, build, run and capture
commands. When those change, update it too.

The publisher-facing documents are `docs/FIRST-APP.md` (the first app),
`docs/SUBMITTING.md` (the steps) and `docs/PUBLISHING.md` (the reference).
Never state review times or hackathon rules in them; each event's own page
owns those.

## Code rules

- Trace behavior to executable code. Distinguish manifest validation,
  resolved grants, shell registration and the code that executes a call.
  `AgentBundle::load` alone starts no peer. OctoSense loads `AGENT.md` and
  skills as per-turn guidance, not as kernel skills.
- Keep native `AppModule`, Splash script bundles and OctoScript L0 cards
  distinct. `tools/octo` is App Flow's Python CLI; octos is the agent
  kernel. `card-host` serves no host service except `runtime` discovery,
  and runs no agent.
- For agent or storage changes, also inspect OctoSense `crates/shell/src/host_tools`,
  `crates/shell/src/app_storage`, `crates/ai-host` and `crates/app-peers` at
  the revision the consumer pins. Do not claim a tool runs because
  `tools.json` passes the gate: OctoSense runs granted
  `implemented_by: "host-service"` tools on their services. A host that
  advertises `app_tools.dispatch@1` runs `implemented_by: "app"` tools in the
  admitted full app's existing Splash isolate; a closed app fails with
  `app_not_running`. Never create a second state store, and never register a
  Glance copy as a tool owner.
- Preserve contract compatibility and refusal behavior. Consult
  [app-contract's compatibility rules](crates/app-contract/README.md#stability).
  Do not widen an app's resolved grants in a mounting path.
- Keep new routing fields out of catalog entries: older stores parse catalog
  tool summaries strictly, so `entry_for` drops `host_method` from them. Hosts
  dispatch from the signed bundle's `tools.json`.
- Keep host sheets modal. `services::is_sheet_input_event` decides which input
  events reach only the visible sheet; an integrated foreground Glance host
  must apply it too.
- Capture the runner's card, sheet and notice references before any app widget
  exists, and use those references for drawing, input, service pumping and
  shutdown. Never look a host surface up by widget id after the app mounts:
  an app may name its own widget `sheet`, `card` or `notice`.
- Build the store's privacy text with `privacy_summary_with_tools` wherever
  the reviewed tools are available. A nonempty `tools.json` offers the host's
  Ask even when the manifest's `agent` is `null`, and `privacy_summary`, which
  reads only the manifest, says "Runs no assistant." for such an app.
- In the store's permission lines (`Entry::permissions_summary`), name an
  agent app's own tools from `tools.json` apart from the additional tools in
  `agent.tools`. An empty `agent.tools` does not mean the assistant has no
  tools.

## Build and test

- Build the authoring tools with
  `cargo build --release -p octosense-card-host -p octosense-app-hub`, against
  plain sibling checkouts without OctoSense's runtime patches.
- Behind its default feature `text-input-state-query`, `appstore` matches
  `Event::TextInputStateQuery`, which only OctoSense's patched Makepad
  defines. `app-hub-app` turns the same feature on by default and passes it
  to `appstore`; `card-host` turns it off. Put any other patch-only event
  behind this feature. On a plain Makepad checkout, run only the package
  commands in [Run the right host](docs/CODE-WALKTHROUGH.md#2-run-the-right-host);
  [`card-host` fails to build](docs/DEVELOPMENT.md#card-host-fails-to-build)
  lists the builds that need the patches.
- [Versions on crates.io](crates/app-contract/README.md#versions-on-cratesio)
  lists the `octosense-app-contract` releases; source 1.10.0 includes `files`, `device_calendar` and `audio` but is
  unpublished. App Policy requires 1.10, so source consumers need the exact
  git patch until it is released. 1.8.0 is published on crates.io but does
  not satisfy this tree's App Policy minimum version. Do
  not take a published contract as proof that a released host implements its
  APIs. This workspace's `[patch.crates-io]` points the crate at
  `crates/app-contract`. Never tell readers that a lock file holding 1.2.0 or
  older admits `calendar`, `auth`, `github`, `gcalendar`, `gmail`, `photos`
  or `youtube`; `cargo update -p octosense-app-contract` moves an
  unconstrained 1.x consumer to 1.8.0. When the tree gets ahead of crates.io
  again, or a new version is published, update that section.
- Do not change pins or regenerate `Cargo.lock` in a documentation edit.
- Match validation to the change:

  | Change | Validate with |
  | --- | --- |
  | Admission | `cargo test -p octosense-app-contract -p octosense-app-policy -p octosense-app-hub` |
  | `hub` commands | `cargo test -p octosense-app-hub --test cli` |
  | `card-host` arguments | `cargo test -p octosense-card-host --test cli` |
  | Host sheets and modal input | `cargo test -p octosense-appstore -p octosense-app-hub-app --no-default-features --lib` and `cargo test -p octosense-card-host --bin card-host` |
  | Service lifecycle | The changed service's tests |
  | Documentation only | Source, link and command checks; report builds or devices you could not use |

- For automated GUI checks, set `MAKEPAD_HIDE_WINDOWS=1`, use a distinct
  `MAKEPAD_REMOTE` port and app-data directory per instance, and quit only
  instances you started.

## Documentation rules

- In walkthroughs, lead with a launch path, then trace one request to its
  reply. Define routing terms at first use and keep source inventories after
  the main path. Preserve prerequisites and capability limits in examples.
- Label source-reviewed commands separately from commands you ran. Mark a
  command nobody ran "(not run)" or **unverified**.
- Never edit evidence records such as `reviews/**/*.json`.
- Before you rename or remove a heading, search App Hub, App Flow,
  OctoSense and the three reference apps' repositories for links to it, and
  keep linked heading text. App Flow and `docs/SUBMITTING.md` link
  `docs/PUBLISHING.md` headings such as `#submitting`, `#the-manifest` and
  `#rules-the-gate-enforces`.

## Submissions

Apply these rules when you help a developer submit an app, or help a
reviewer check one:

- After you build `hub` and `card-host`, run App Flow's `tools/octo doctor`
  before you build or check an app. Run `hub help` for the exact commands, and
  use only the commands it lists.
- The submission issue is the developer's request to publish. Never treat a
  tag or a GitHub release as a submission or an approval.
- Check the exact bytes at the submitted tag and in the release pack with
  `hub check`, `hub publisher-unpack` and `hub publisher-verify`, and post
  findings in the issue.
  No bot reviews submission issues; never claim that one did.
- A GitHub publisher app needs no developer signing key: its workflow attests
  each release. An app published with an Ed25519 key keeps that key for
  every update. Never ask a developer for a private key.

## Catalog and publication

- GitHub catalog publication uses the protected `app-hub-catalog` environment,
  exact admin-approved digest and native proof verification. Never execute a
  candidate bundle or weaken existing publisher continuity to publish it.
- In documentation and development work, never change `catalog.json`,
  `index/`, `artifacts/`, publisher keys or publication status, and never run
  `hub publish`, `hub withdraw` or `hub remove`.
- Test installs against a temporary local catalog.
- Publish only a submission that an App Hub admin has approved, within the
  user's authorized scope, after the checks in
  [What reviewers check](docs/SUBMITTING.md#8-what-reviewers-check).
