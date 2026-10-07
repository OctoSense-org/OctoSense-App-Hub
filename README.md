# OctoSense app hub

English | [简体中文](README.zh-CN.md)

The index of apps published for OctoSense, the signed catalog every OctoSense
store reads, the hub's own copy of each admitted bundle, and the code that
runs the hub and the store. Publisher app source stays in each publisher's
repository; this repository also contains the store UI, runtime hosts,
templates and test fixtures.

| Looking for | Repository |
| --- | --- |
| How to build an app: quickstart, script API, script-app template, design flows, examples | [OctoScript-App-Design-Flow](https://github.com/OctoSense-org/OctoScript-App-Design-Flow) |
| The AppCard assistant runtime (opt-in in the shells, `--features app-appcard`) | [OctoSense `apps/appcard`](https://github.com/OctoSense-org/OctoSense/tree/main/apps/appcard) |
| The first-party system apps (News, Photos, Maps, Camera, Mail, AI providers) and their host services (`mail`, `llm`) | [OctoSense `apps/`](https://github.com/OctoSense-org/OctoSense/tree/main/apps) |
| The L0 parser/checker and its Makepad lowering/renderer | [Octoscript](https://github.com/OctoSense-org/Octoscript) and [Octoscript-Makepad](https://github.com/OctoSense-org/OctoScript-Makepad) |
| The bundle format, the gate, signing, submission and the store | this repository |

**Building an app?** Start at the [OctoSense-org profile](https://github.com/OctoSense-org)'s "read these first" list (OctoScript-App-Design-Flow's `AGENTS.md`, then its `docs/QUICKSTART.md`). You need this repository only as a sibling checkout to build `hub` and `card-host`, and to submit (an issue here, see [Submitting](docs/PUBLISHING.md#submitting)). Clone it; do not edit `catalog.json`, `index/` or `artifacts/`.

| Path | What it is |
| --- | --- |
| `catalog.json` | The signed catalog. Stores verify it against the anchor below before showing anything. |
| `index/<app>-<version>.json` | One admitted entry per app version: its manifest, publisher, source and status. A maintainer exports the admitted entry from the catalog after `hub publish`; absent while no app is published. |
| `artifacts/<app>-<version>.bundle/` | The hub's copy of the bundle, exactly the bytes that were reviewed. Created by `hub publish`. |
| `artifacts/<app>-<version>.bundle.pack.json` | The same bundle as one file, which stores download. |
| `docs/FIRST-APP.md` | First-app walkthrough for a card app or a script app: author, package, run, capture, validate and submit. |
| `docs/PUBLISHING.md` | The bundle, listing, capabilities, host services, gate rules, signing and submission contract. |
| `docs/ICONS.md` | Canonical icon ownership, export constraints and visual review. |
| `docs/DEVELOPMENT.md` | Where authoring lives, delivery paths, and `card-host` with its remote-control routes. |
| `templates/app/` | Card app repository scaffold with metadata, example icon and linked agent instructions. |
| `crates/app-contract` | The app contract, `octosense-app-contract` on crates.io (OctoSense ADR 0005): the manifest, the policy an app gets, package integrity and running a package. Apps and hosts depend on it by version; within `1.x` it only grows ([README](crates/app-contract/README.md)). |
| `crates/app-policy` | The signed manifest and listing, admission, and resolution into an isolate's settings and an agent session profile (ADR 0002); an app's own agent (`tools.json`, `AGENT.md`, skills) loaded as an `AgentBundle`; the `tools.json` parser and checks that native modules' tool manifests share (`ToolManifest::load`). Re-exports the app contract. |
| `crates/app-hub` | The index, the signed catalog, the gate, the agent scan, the device client and the `hub` command (ADR 0003). |
| `crates/appstore` | The store as an OctoSense module, the `card` module that runs an installed app as its own client, system apps (`os.` ids) and host services with their sheets. |
| `crates/appstore-app` | The store as a standalone app (`appstore`). |
| `crates/card-host` | The reference contained host for one bundle, card or script app (`card-host`). |
| `crates/card-studio` | Renders a card in a hidden `card-host --remote` at glance, phone and desktop sizes, runs the measured checks and prepares the vision critique (`card-studio`, ADR 0002 section 7). |
| `skills/card-studio` | The octos skill over `card-studio`: `card_render`, `card_critique_payload`. |
| `crates/app-host` | A one-window host that runs any OctoSense AppModule as a standalone app. |
| `crates/app-hub-app` | The shell integration every OctoSense shell links: the native store module, the `card` runner module, the system apps named by `OCTOSENSE_SYSTEM_APPS`, installed apps and icons ([README](crates/app-hub-app/README.md)). |

The crates build against the pinned OctoSense forks of Makepad and Octoscript,
resolved from sibling checkouts (`../makepad`, `../octoscript-makepad`,
`../octoscript`) as the launcher workspace does. The [native tools CI](.github/workflows/native-tools.yml)
builds `hub` and `card-host` against those unmodified pins and runs the
contract, policy, gate, signing, CLI and store tests. `cargo run -p
octosense-app-hub --bin hub` is the publishing tool.

`appstore` and `app-hub-app` enable `text-input-state-query` by default for OctoSense's IME
runtime overlay. Standalone `card-host` disables it. Use the walkthrough's
package-specific commands with the plain runtime: a workspace-wide build
unifies features from other shell consumers and requires that overlay. CI tests
both runner crates with `--no-default-features` against the plain runtime.

`hub keygen` creates a new file only, refusing existing files and symlinks; new
keys have mode `0600` on Unix. On other platforms, use a directory private to
your user. Help such as `hub check --help` or `hub keygen --help` never reads a
bundle or creates a key. `hub scan` permits unsigned development bundles, but a
signed bundle still needs `--publisher-key id=hex` or a trusted catalog key.

## Code walkthrough

Start with [the walkthrough's host and launch choices](docs/CODE-WALKTHROUGH.md#2-run-the-right-host),
then trace a bundle into its UI and a host-service reply back to its callback.
A saved-notes request connects app-agent conversations, account storage and
tool grants. The walkthrough distinguishes native `AppModule`, Splash and
L0 apps, with the crate inventory at the end. Contributor instructions are
in [AGENTS.md](AGENTS.md).

System apps with cross-app agent tools need an explicit host admission offer.
Before preparing the app, its shell calls `system::set_agent_tool_offer` for that
app id. Only requested names are eligible; owner sharing, caller grants and a
real executor remain separate shell checks. An offer neither starts an agent
nor gives script UI code raw host-service access. Store defaults are unchanged.

App contract 1.5 admits four additional, independent host-service capabilities:
`auth` for GitHub/Google connection management, `github` for repository operations,
`gcalendar` for Google Calendar, and `gmail` for Gmail. OAuth tokens belong to the
host; apps receive handles bound to their own identity and authorized scopes.
The declarations do not register services: standalone `card-host` has none of
these providers. The OctoSense host implementation and provider registrations
must be installed before a sample can sign in. No OctoSense account is required.

An ordinary app can bind its own agent tool to a reviewed shared service with
`"implemented_by":"host-service", "host_method":"github.read"` in `tools.json`.
The tool keeps its app namespace. Admission checks the target against
`SHARED_HOST_METHODS`, the declared service grant, private-data disclosure and
minimum risk; the shell must check the resolved grant again when executing it.
Omitting `host_method` preserves the original dispatch behavior. Authentication,
host sheets, direct saves and sends cannot be aliased. An admitted alias neither
installs its service nor grants approval for an external action.

Catalog tool summaries omit `host_method` so older stores can read the catalog.
They retain permission and risk metadata, but are never used for dispatch.
The signed bundle keeps the complete tools file; installation and launch still
verify its exact digest and publisher signature before loading those bindings.

The `calendar` capability (app contract 1.4) separately admits Calendar UI
requests. OctoSense registers the Calendar service and checks its owning app
identity. A declared capability alone does not provide a service, a calendar
account, or agent tools.

Host-owned account and review sheets are modal: text, keyboard, IME,
clipboard and pointer-release events reach only the visible sheet. Timer and
service callbacks continue to reach the app. The runner captures its own card
and sheet references before evaluating app widgets, so a bundle cannot replace
the host surface by reusing a widget ID. The mobile wrapper, standalone runner,
and shutdown path retain those same references for drawing, input, requests,
and cancellation. `services::is_sheet_input_event` supplies
the same input boundary to integrated foreground Glance hosts.

Store privacy text uses both the manifest and the catalog's reviewed tools.
Even with `agent: null`, a valid nonempty `tools.json` can offer the host's Ask
assistant after consent; conversation and tool results may reach the configured
AI provider. That does not declare background work or automatic triggers, start
a peer, or install an executor. Apps with neither agent nor tools still show
“Runs no assistant.”

The `photos` and `youtube` capabilities (app contract 1.5) admit requests to
the app-owned media services supplied by OctoSense. They do not grant Android
Gallery access, a YouTube account, or another agent's tools. Cross-app tools
still need an owner declaration, an explicit caller grant, host admission, and
an executor.

## Trust anchor

Stores trust this anchor and follow its certificate to the working key that
signs the catalog. Rotating the working key needs no store release.

```
6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11
```

## Pointing a store here

A store build reads this hub and trusts this anchor by default. The
variables override them, for a mirror or a development hub; spelled out,
the defaults are:

```sh
OCTOSENSE_HUB=https://raw.githubusercontent.com/OctoSense-org/OctoSense-App-Hub/main/ \
OCTOSENSE_HUB_ANCHOR=6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11 \
appstore
```

## Publishing an app

An app is a card app (`page.card`) or a script app (`main.splash`). Start
with [Build your first Hub app](docs/FIRST-APP.md). Follow
[Publishing](docs/PUBLISHING.md) for the complete contract and
[Icons](docs/ICONS.md) for artwork; the
[development guide map](docs/DEVELOPMENT.md) links the authoring and testing
guides in the other repositories.

Stamp the bundle, capture a screenshot, restamp, run `hub check` and
`hub scan`, sign the manifest, then submit by opening an issue here
([Submitting](docs/PUBLISHING.md#submitting)). There is no publish action or
separate index repository yet: a maintainer runs `hub publish` on the exact
bytes of your tagged commit and commits the signed catalog. A version is
withdrawn with `hub withdraw`, and every store honours it on its next fetch;
`hub remove` drops an entry that should never have been published.

## Apps

| App | Version | Category | Runs on | Publisher | Allowed to | Status |
| --- | --- | --- | --- | --- | --- | --- |
| [GitHub Notes](https://github.com/ymote/octosense-github-notes) | 0.1.0 | Productivity | macOS | ymote | Local drafts; host-managed GitHub login, reviewed commits and optional shell Ask | Preview |
| [Inbox Assistant](https://github.com/ymote/octosense-inbox-assistant) | 0.1.0 | Productivity | macOS | ymote | Gmail, local drafts, configured model and consented Glance/agent work | Preview |
| [Google Calendar](https://github.com/ymote/octosense-google-calendar) | 0.1.0 | Productivity | macOS | ymote | Google Calendar, local drafts, consented chat and Glance | Preview |

These are **macOS developer previews**, admitted in catalog sequence 7. Install
[OctoSense desktop 0.1.0-beta.2](https://github.com/OctoSense-org/OctoSense/releases/tag/desktop-v0.1.0-beta.2)
on an Apple Silicon Mac. Search App Hub for a name above, choose Get, review
permissions, then Install → Open. The [admission and acceptance record](reviews/connected-apps-0.1.0/README.md)
identifies the publisher commits and tested runtime. Legacy catalog parsing was
tested; running these apps on older desktop versions is unverified.

Provider sign-in requires a host OAuth registration, stored outside app bundles;
see [connected accounts](https://github.com/OctoSense-org/OctoSense/blob/desktop-v0.1.0-beta.2/crates/oauth-service/README.md).
Live GitHub/Google login and remote writes remain unverified. The Calendar
assistant is advisory; it does not book events. No OctoSense cloud account is
required.

The camera card that exercised the pipeline was removed on 20 Sep 2026:
Camera is a system app that ships with the shells (like News and Photos),
not a store app. Its repository stays at
[ymote/camera-card](https://github.com/ymote/camera-card) as a worked
example of a publishable bundle.
