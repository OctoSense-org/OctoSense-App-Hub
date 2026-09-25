# OctoSense app hub

The index of apps published for OctoSense, the signed catalog every OctoSense
store reads, the hub's own copy of each admitted bundle, and the code that
runs the hub and the store. No app code lives here; each app stays in its
publisher's repository.

| Path | What it is |
| --- | --- |
| `catalog.json` | The signed catalog. Stores verify it against the anchor below before showing anything. |
| `index/<app>-<version>.json` | One admitted entry per app version: its manifest, publisher, source and status. |
| `artifacts/<app>-<version>.bundle/` | The hub's copy of the bundle, exactly the bytes that were reviewed. |
| `artifacts/<app>-<version>.bundle.pack.json` | The same bundle as one file, which stores download. |
| `docs/FIRST-APP.md` | First-app walkthrough: author, package, run, capture, validate and submit. |
| `docs/PUBLISHING.md` | Shared bundle, listing, permissions and publication contract. |
| `docs/ICONS.md` | Canonical icon ownership, export constraints and visual review. |
| `docs/DEVELOPMENT.md` | Guide map for UI, data/state, runtime setup and testing. |
| `templates/app/` | App repository scaffold with metadata, example icon and linked agent instructions. |
| `crates/app-policy` | The signed manifest and listing, admission, and resolution into an isolate's settings and an agent session profile (ADR 0002). |
| `crates/app-hub` | The index, the signed catalog, the gate, the agent scan, the device client and the `hub` command (ADR 0003). |
| `crates/appstore` | The store as an OctoSense module, and the `card` module that runs an installed app as its own client. |
| `crates/appstore-app` | The store as a standalone app (`appstore`). |
| `crates/card-host` | The reference contained host for one card bundle (`card-host`). |
| `crates/app-host` | A one-window host that runs any OctoSense AppModule as a standalone app. |

The crates build against the pinned OctoSense forks of Makepad and Octoscript,
resolved from sibling checkouts (`../makepad`, `../octoscript-makepad`,
`../octoscript`) as the launcher workspace does. `cargo test --workspace`
runs the policy, gate, signing and store tests headless; `cargo run -p
octosense-app-hub --bin hub` is the publishing tool.

## Trust anchor

Stores trust this anchor and follow its certificate to the working key that
signs the catalog. Rotating the working key needs no store release.

```
6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11
```

## Pointing a store here

```sh
OCTOSENSE_HUB=https://raw.githubusercontent.com/OctoSense-org/OctoSense-App-Hub/main/ \
OCTOSENSE_HUB_ANCHOR=6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11 \
appstore
```

## Publishing an app

Start with [Build your first Hub app](docs/FIRST-APP.md) and the
[app starter](templates/app/README.md). Follow [Publishing](docs/PUBLISHING.md)
for the complete contract and [Icons](docs/ICONS.md) for artwork. The
[development guide map](docs/DEVELOPMENT.md) links the existing authoring and
testing guides. Build the bundle, run `hub check` and `hub test`, sign the manifest, and open
an entry here. The gate and the agent scan run on the
exact bytes; a passing submission from a publisher on record merges without a
person, a first submission waits for one. A version is withdrawn with
`hub withdraw`, and every store honours it on its next fetch.

## Apps

| App | Version | Category | Runs on | Publisher | Allowed to | Status |
| --- | --- | --- | --- | --- | --- | --- |
| _none yet_ | | | | | | |

The camera card that exercised the pipeline was removed on 20 Sep 2026:
Camera is a system app that ships with the ROM (like Calendar, News and
Photos), not a store app. Its repository stays at
[ymote/camera-card](https://github.com/ymote/camera-card) as a worked
example of a publishable bundle.
