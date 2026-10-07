# OctoSense App Hub

English | [简体中文](README.zh-CN.md)

The App Hub publishes apps for OctoSense. This repository holds:

- the signed catalog that every OctoSense store reads;
- App Hub's copy of each admitted bundle;
- the code for the gate (the admission checks every bundle must pass), signing,
  the store and the reference host.

Each app's source stays in its publisher's own repository.

| Looking for | Repository |
| --- | --- |
| How to build an app: quickstart, script API, script-app template, design flows, examples | [OctoScript-App-Design-Flow](https://github.com/OctoSense-org/OctoScript-App-Design-Flow) |
| The AppCard assistant runtime (opt-in in the shells, `--features app-appcard`) | [OctoSense `apps/appcard`](https://github.com/OctoSense-org/OctoSense/tree/main/apps/appcard) |
| The first-party system apps (AI providers, Calendar, Camera, Mail, Maps, News, Photos, YouTube) and their host services (`llm`, `model`, `calendar`, `mail`, `news`) | [OctoSense `apps/`](https://github.com/OctoSense-org/OctoSense/tree/main/apps) |
| The L0 parser and checker, and the Makepad lowering and renderer | [OctoScript](https://github.com/OctoSense-org/OctoScript) and [OctoScript-Makepad](https://github.com/OctoSense-org/OctoScript-Makepad) |
| The bundle format, the gate, signing, submission and the store | this repository |

**Building an app?** Start with the "read these first" list on the
[OctoSense-org profile](https://github.com/OctoSense-org): the `AGENTS.md` of
OctoScript-App-Design-Flow (Design Flow), then its `docs/QUICKSTART.md`. Next,
follow [Build your first Hub app](docs/FIRST-APP.md) and
[App icons and bundled artwork](docs/ICONS.md). Clone this repository beside
your Design Flow checkout to build `hub` and `card-host`. Do not edit
`catalog.json`, `index/` or `artifacts/`.

**Submitting an app?** Follow [Submit an app to the App Hub](docs/SUBMITTING.md):
you open an issue here for a signed bundle at a tagged commit. The catalog's
[three reference apps](docs/SUBMITTING.md#the-three-reference-apps) (GitHub
Notes, Inbox Assistant and Google Calendar) passed admission end to end; their
repositories show a complete submission. Look up rules, capabilities and
fields in the [publishing reference](docs/PUBLISHING.md).

New host API declarations and availability checks are described in [Host API compatibility](docs/HOST-API.md).

## Repository layout

| Path | What it is |
| --- | --- |
| `catalog.json` | The signed catalog. Stores verify it against the anchor below before showing anything. |
| `index/<app>-<version>.json` | One admitted entry per app version: its manifest, publisher, source and status. A maintainer exports it from the catalog after `hub publish`. |
| `artifacts/<app>-<version>.bundle/` | App Hub's copy of the bundle, exactly the bytes that were reviewed. `hub publish` creates it. |
| `artifacts/<app>-<version>.bundle.pack.json` | The same bundle as one file, which stores download. |
| `docs/FIRST-APP.md` | A first-app walkthrough for a card app or a script app: create, run, capture and check. |
| `docs/SUBMITTING.md` | The submission, step by step: repository, manifest, listing, screenshots, signing, release, issue and review. |
| `docs/PUBLISHING.md` | The reference: gate rules; capabilities and who serves them; manifest, listing and tool fields; host services; `hub` commands; signing. |
| `docs/ICONS.md` | Canonical icon ownership, export constraints and visual review. |
| `docs/DEVELOPMENT.md` | The guide map, delivery paths, `card-host` and its remote-control routes, and `card-studio`. |
| `templates/app/` | A card app repository scaffold with metadata, an example icon and linked agent instructions. |
| `crates/app-contract` | The app contract, `octosense-app-contract` ([OctoSense ADR 0005](https://github.com/OctoSense-org/OctoSense/blob/main/docs/adr/0005-app-contract.md)): the manifest, the policy an app gets, bundle integrity and running a bundle. Within `1.x` it only grows ([README](crates/app-contract/README.md)). Its latest version, 1.6.0, is on crates.io ([Versions on crates.io](crates/app-contract/README.md#versions-on-cratesio)). |
| `crates/app-policy` | The signed manifest and listing, admission, and resolution into an isolate's settings and an agent session profile ([OctoSense Home ADR 0002](https://github.com/OctoSense-org/OctoSense/blob/main/docs/adr/home/0002-agentic-app-security-model.md)); an app's own agent (`tools.json`, `AGENT.md`, skills) loaded as an `AgentBundle`; the `tools.json` parser and checks that native modules' tool manifests share (`ToolManifest::load`). It also re-exports the app contract. |
| `crates/app-hub` | The index, the signed catalog, the gate, the agent scan, the device client and the `hub` command ([OctoSense Home ADR 0003](https://github.com/OctoSense-org/OctoSense/blob/main/docs/adr/home/0003-app-hub-and-store.md)). |
| `crates/appstore` | The store as an OctoSense module, the `card` module that runs an installed app as its own client, system apps (`os.` ids) and host services with their sheets. |
| `crates/appstore-app` | The store as a standalone app (`appstore`). |
| `crates/card-host` | The reference contained host: it runs one bundle, a card app or a script app (`card-host`). |
| `crates/card-studio` | A tool that renders a card in a hidden `card-host --remote` at Glance, phone and desktop sizes, runs the measured checks and prepares the vision critique (`card-studio`, [OctoSense ADR 0002 section 7](https://github.com/OctoSense-org/OctoSense/blob/main/docs/adr/0002-event-driven-app-agents.md#7-cards-l0-grounded-rendered-and-critiqued-before-publishing)). |
| `crates/app-host` | A one-window host that runs any OctoSense `AppModule` as a standalone app. |
| `crates/app-hub-app` | The shell integration every OctoSense shell links: the native store module, the `card` runner module, the system apps named by `OCTOSENSE_SYSTEM_APPS`, installed apps and icons ([README](crates/app-hub-app/README.md)). |
| `skills/card-studio` | The octos skill over `card-studio`: `card_render`, `card_critique_payload`. |

## Build and test

The crates build against pinned revisions of OctoSense's Makepad fork,
OctoScript-Makepad and OctoScript, resolved from sibling checkouts
(`../makepad`, `../octoscript-makepad`, `../octoscript`);
[Build your first Hub app](docs/FIRST-APP.md#1-prepare-the-tools-and-an-app-repository)
sets them up. Build the two authoring tools:

```sh
cargo build --release -p octosense-card-host -p octosense-app-hub
```

To run `hub` from source without a release build, use
`cargo run -p octosense-app-hub --bin hub`. [Run the right host](docs/CODE-WALKTHROUGH.md#2-run-the-right-host) lists the
package tests. The [native tools CI](.github/workflows/native-tools.yml) runs
the release build above and those tests on macOS, against sibling checkouts without
OctoSense's runtime patches.

Build only the packages you need. A build that includes the store packages
with their default features, such as `cargo test --workspace`, also needs
OctoSense's runtime patches on `../makepad`. If a build fails with
`no variant … TextInputStateQuery`, see
[`card-host` fails to build](docs/DEVELOPMENT.md#card-host-fails-to-build).

The store's UI uses one set of controls for desktop and phone layouts. To
preview it, build its example and run it with an app-data directory of its own:

```sh
cargo build --locked --release -p octosense-app-hub-app --example preview
OCTOSENSE_PREVIEW_SIZE=1200x860 OCTOSENSE_APP_DATA="$(mktemp -d)" target/release/examples/preview
```

`OCTOSENSE_PREVIEW_SIZE` sets the window: `1200x860` for desktop, or
`406x820`, the default, for phone dimensions. A phone-size window is not a
test on a phone. For automated checks, also set `MAKEPAD_HIDE_WINDOWS=1` and
`MAKEPAD_REMOTE=<port>`. The preview builds the store with its default
features, so it needs the runtime patches as well. Installing an app from the
preview still goes through the store's consent and admission checks.

## Code walkthrough

Start with [Run the right host](docs/CODE-WALKTHROUGH.md#2-run-the-right-host).
The walkthrough then traces a bundle into its UI, a host-service request back
to its callback, and one app-agent request for saved notes. Contributor rules
are in [AGENTS.md](AGENTS.md).

## What hosts serve today

The gate admits 103 capability names, but a capability works only where a host
serves it. [Capabilities](docs/PUBLISHING.md#capabilities) lists who serves
each one today.

- `card-host` serves no host services and runs no agent.
- OctoSense serves `mail`, `model` and `glance` to any app granted them.
- OctoSense desktop 0.1.0-beta.2 serves the connected-account capabilities
  (`auth`, `github`, `gcalendar`, `gmail`) once the host has OAuth client
  registrations. Tokens stay with the host; apps get connection handles.
- OctoSense `main`, not yet in any release, also signs an app in to its own
  backend through `auth`, once the device's operator registers that backend ([Sign in to your own backend](docs/PUBLISHING.md#sign-in-to-your-own-backend)).
- OctoSense `main` also requires a physical press to approve a GitHub or
  Google Calendar save, refuses executable Splash (`script`) cards from app agents,
  and keeps Google Calendar events from 30 days back to 366 days ahead
  (in no
  release yet). OctoSense desktop 0.1.0-beta.2 does none of these.
- OctoSense serves `calendar`, `llm` and `news` only to its own system apps.
  For `photos` and `youtube` it serves only a `notify` method, to `os.photos`
  and `os.youtube`. Not yet: media services for store apps.
- OctoSense runs an app agent's granted `implemented_by: "host-service"`
  tools, including tools mapped to a reviewed shared service with
  `host_method`, and loads `AGENT.md` and skills as guidance. It refuses tools
  with `implemented_by: "app"`
  ([The app's agent and tools](docs/PUBLISHING.md#the-apps-agent-and-tools)).

## Trust anchor

Stores trust this anchor and follow its certificate to the working key that
signs the catalog. Rotating the working key needs no store release.

```text
6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11
```

## Point a store at another hub

A store build reads this hub and trusts this anchor by default. To read a
mirror, set `OCTOSENSE_HUB` to its directory or base URL. A development hub
signed under its own anchor also needs `OCTOSENSE_HUB_ANCHOR`. This command
sets both variables to their defaults:

```sh
OCTOSENSE_HUB=https://raw.githubusercontent.com/OctoSense-org/OctoSense-App-Hub/main/ \
OCTOSENSE_HUB_ANCHOR=6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11 \
appstore
```

## How a maintainer publishes an app

After you submit ([Submit an app to the App Hub](docs/SUBMITTING.md)), a
maintainer runs `hub publish` on the exact bytes of your tagged commit and
commits the signed catalog. Not yet: a publishing action or a separate index
repository.

A maintainer withdraws a version with `hub withdraw`, and every store honors
the withdrawal on its next catalog fetch. `hub remove` drops an entry that
should never have been published.

## Apps

| App | Version | Category | Runs on | Publisher | Uses | Status |
| --- | --- | --- | --- | --- | --- | --- |
| [GitHub Notes](https://github.com/ymote/octosense-github-notes) | 0.1.1 | Productivity | macOS | ymote | Local drafts, GitHub sign-in through the host, commits you review, and an optional app agent in the shell's Ask panel | Preview |
| [Inbox Assistant](https://github.com/ymote/octosense-inbox-assistant) | 0.1.1 | Productivity | macOS | ymote | Gmail, local drafts, the configured model, and, with your consent, Glance cards and agent work | Preview |
| [Google Calendar](https://github.com/ymote/octosense-google-calendar) | 0.1.1 | Productivity | macOS | ymote | Google Calendar, local drafts, and, with your consent, an app-agent chat and Glance cards | Preview |

These are **macOS developer previews**. Catalog sequence 10 offers version
0.1.1 of each and keeps its 0.1.0 entry. Install them from the store in
[OctoSense desktop 0.1.0-beta.2](https://github.com/OctoSense-org/OctoSense/releases/tag/desktop-v0.1.0-beta.2)
on an Apple silicon Mac. OctoSense desktop 0.1.0-beta.1 and Home 0.1.0-beta.1
list them but cannot install them: their stores refuse `auth` as an unknown
capability. No phone release can install them yet. The
[0.1.1 admission record](docs/admissions/connected-apps-0.1.1/README.md)
names the publisher commits and what was verified. The
[0.1.0 record](reviews/connected-apps-0.1.0/README.md) also covers a store
test in the desktop 0.1.0-beta.2 release.

Signing in to GitHub or Google needs OAuth client registrations on the host.
OctoSense desktop 0.1.0-beta.2 reads them only from the host's
`oauth/clients.json`, and the release ships none
([Connected accounts and App Hub samples](https://github.com/OctoSense-org/OctoSense/blob/desktop-v0.1.0-beta.2/crates/oauth-service/README.md)).
OctoSense `main` compiles in the registrations its distributor supplies, and
an optional `oauth/clients.json` replaces all of them; no release includes that
change yet.

Unverified on a release: live sign-in and remote writes. On development
builds, identity-only sign-in passed on macOS, and Google Calendar connected
and saved one event; GitHub commits and Gmail sends remain unverified
([current delivery boundary](https://github.com/OctoSense-org/OctoSense/blob/main/crates/oauth-service/README.md#current-delivery-boundary)).

The Calendar app agent only advises; it does not book events. None of the
three apps needs an OctoSense cloud account.
