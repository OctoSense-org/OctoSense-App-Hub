# OctoSense app hub

The index of apps published for OctoSense, the signed catalog every OctoSense
store reads, and the hub's own copy of each admitted bundle. No app code
lives here; each app stays in its publisher's repository.

| Path | What it is |
| --- | --- |
| `catalog.json` | The signed catalog. Stores verify it against the anchor below before showing anything. |
| `index/<app>-<version>.json` | One admitted entry per app version: its manifest, publisher, source and status. |
| `artifacts/<app>-<version>.bundle/` | The hub's copy of the bundle, exactly the bytes that were reviewed. |
| `artifacts/<app>-<version>.bundle.pack.json` | The same bundle as one file, which stores download. |
| `docs/PUBLISHING.md` | How to publish; also the `AGENTS.md` a developer copies into their repository. |

## Trust anchor

Stores trust this anchor and follow its certificate to the working key that
signs the catalog. Rotating the working key needs no store release.

```
6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11
```

## Pointing a store here

```sh
OCTOSENSE_HUB=https://raw.githubusercontent.com/ymote/octosense-app-hub/main/ \
OCTOSENSE_HUB_ANCHOR=6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11 \
appstore
```

## Publishing an app

Read `docs/PUBLISHING.md`. In short: build the bundle, run `hub check`, sign
the manifest, and open an entry here. The gate and the agent scan run on the
exact bytes; a passing submission from a publisher on record merges without a
person, a first submission waits for one. A version is withdrawn with
`hub withdraw`, and every store honours it on its next fetch.

## Apps

| App | Version | Publisher | Allowed to | Status |
| --- | --- | --- | --- | --- |
| [Camera](https://github.com/ymote/camera-card) | 1.0.0 | ymote | draw its screen only | offered |
