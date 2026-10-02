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
| `index/<app>-<version>.json` | One admitted entry per app version: its manifest, publisher, source and status. Created by `hub publish`; absent while no app is published. |
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
`../octoscript`) as the launcher workspace does. `cargo test --workspace`
runs the policy, gate, signing and store tests headless; `cargo run -p
octosense-app-hub --bin hub` is the publishing tool.

## Code walkthrough

Start with [the walkthrough's host and launch choices](docs/CODE-WALKTHROUGH.md#2-run-the-right-host),
then trace a bundle into its UI and a host-service reply back to its callback.
A saved-notes request connects app-agent conversations, account storage and
tool grants. The walkthrough distinguishes native `AppModule`, Splash and
L0 apps, with the crate inventory at the end. Contributor instructions are
in [AGENTS.md](AGENTS.md).

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
| _none yet_ | | | | | | |

The camera card that exercised the pipeline was removed on 20 Sep 2026:
Camera is a system app that ships with the shells (like News and Photos),
not a store app. Its repository stays at
[ymote/camera-card](https://github.com/ymote/camera-card) as a worked
example of a publishable bundle.
