# Publishing an app to the OctoSense app hub

This is the shared publication contract for app authors and their coding tools.
Start with [Build your first Hub app](FIRST-APP.md), follow the
[icon guidelines](ICONS.md), and use the [development guide map](DEVELOPMENT.md)
for UI, state, runtime setup and native testing.

The [app starter](../templates/app/README.md) includes a short
[`AGENTS.md`](../templates/app/AGENTS.md) that links to these guides. Merge it
into an existing repository's instructions. Keep any offline copy versioned
against a known Hub revision instead of maintaining independent rules.

The admission rules below are enforced by code and reported as refusals or
warnings. Authoring and visual-review recommendations are separate: a gate
pass does not prove the UI renders, the icon is readable, or the listing is
truthful. See [current icon enforcement](ICONS.md#technical-requirements-and-current-enforcement).

## What an app is

A **card app**: a directory of text and artwork that OctoSense renders in its own
sandboxed isolate. The Hub's Card bundle format contains no native code. An app
that needs new native runtime code must be integrated into a shell release;
see the [delivery paths](DEVELOPMENT.md#choose-the-appropriate-delivery-path).

```
my-app/
  manifest.json      what the app is and what it may do   (required)
  listing.json       what the store shows about it         (required)
  page.card          the L0 card, the app's screen          (required)
  page.data.json     the data bound into the card           (optional)
  kit/               the kit the card is lowered with       (required)
  assets/            icon and other local runtime artwork   (icon required)
  screenshots/       at least one PNG the listing names     (required)
```

Produce the card, data and kit with the
[image-to-appcard flow](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/lab/image-to-appcard-flow/README.md)
in the Octoscript-AppCard repository (`tools/image-to-appcard-flow.sh`). Do not
hand-write L0 unless asked. Keep developer instructions, source tools, keys,
test data directories and review packets outside the submitted bundle.

## Rules the gate enforces

| Rule | What is refused |
| --- | --- |
| Assets are local | Any `http://`, `https://`, `file://` or `../` in a card, data or text file. Ship the asset in the bundle and reference it by a bundle-relative path such as `assets/icon.svg`. |
| Allowed file types only | Anything other than `.card .json .l0 .octoscript .svg .png .jpg .jpeg .webp .ttf .otf .txt .md`. No scripts, archives or binaries. |
| Size | A bundle over 8 MB. |
| No symlinks | Any symlink in the bundle. |
| Digest matches | A manifest whose `integrity.bundle_blake3` does not match the directory. Run `hub stamp` after any change. |
| Manifest is exact | Unknown fields, an unknown capability, a `schema` other than 1, an id outside `[a-z0-9.-]{1,64}` not starting with `.`. |
| Hosts are bare | A host with a scheme, path, port, wildcard or credentials. `api.example.com` is right; `https://api.example.com/v1` and `*.example.com` are refused. |
| Hosts need `net` | Listing hosts without requesting the `net` capability. |
| Version is new | Re-publishing a version already in the catalog. |
| Listing present and complete | No `listing.json`; no icon or no screenshot; an unknown category, platform or age rating; a non-https privacy policy; or an icon or screenshot the listing names that is not in the bundle. |
| Publisher continuity | An update signed by a different key than the one on record for this app. |

## The manifest

```json
{
  "schema": 1,
  "id": "weather-card",
  "version": "1.0.0",
  "name": "Weather",
  "integrity": { "bundle_blake3": "<written by hub stamp>" },
  "capabilities": ["storage", "net"],
  "network": { "hosts": ["api.weather.example"] },
  "storage": { "max_bytes": 1048576 },
  "compute": { "instruction_budget": 5000000, "memory_bytes": 33554432 },
  "agent": {
    "profile": "workspace-write-never-ask",
    "tools": ["net.fetch", "storage.read"],
    "max_iterations": 4,
    "token_budget": 50000
  }
}
```

Ask for the least the app needs. Everything not requested is not granted, and
the store shows the person exactly what was requested, in plain words, before
they install.

**Capabilities** (closed list): `storage`, `net`, `prompt`, `ledger.read`,
`location`, `camera`, `clipboard`. Anything else is refused.

**Network**: `net` plus an exact host list. The list is enforced on every path
out of the isolate: the network module, artwork loading and data fetches. An
empty list with `net` reaches nothing.

**Quotas** are requests; the host clamps them to its ceilings (storage 16 MB,
20 000 000 instructions, 64 MB heap). Ask for less than the ceiling when you can.

**Agent** is optional; omit it and the app gets no assistant. `profile` is one
of `read-only`, `workspace-write`, `workspace-write-never-ask`. Full access does
not exist in this schema; do not add it. `tools` may name only what the host
offers contained apps: `ledger.read`, `ledger.write`, `net.fetch`,
`storage.read`, `storage.write`, `card.render`. Iterations clamp to 8, tokens
to 200 000. The agent's workspace is the app's own storage jail and its hosts are
the app's hosts; it cannot be given more than the app.

## The listing

`listing.json` is what a person sees in the store before installing. It is
reviewed with the bundle and travels in the signed catalog, so what a
reviewer read is what the store shows. The permissions shown beside it come
from the manifest, never from here: a listing cannot understate what the app
does.

```json
{
  "schema": 1,
  "subtitle": "One line under the name (80 characters)",
  "description": "What the app does, for a person deciding whether to install it (4000 characters).",
  "category": "photo-video",
  "keywords": ["camera", "viewfinder"],
  "screenshots": ["screenshots/01-photo-mode.png"],
  "icon": "assets/icon.svg",
  "platforms": ["macos", "android", "linux"],
  "publisher": {
    "name": "Your name or organisation",
    "support": "https://github.com/you/my-app/issues",
    "privacy_policy_url": "https://github.com/you/my-app/blob/main/PRIVACY.md"
  },
  "release_notes": "What changed in this version.",
  "age_rating": "all",
  "license": "Apache-2.0"
}
```

Rules the gate enforces: `category` is one of `productivity utilities
photo-video news weather travel finance health education entertainment games
social shopping lifestyle developer`; `platforms` names at least one of
`android ios macos windows linux openharmony web` (list what you tested; a
card app runs wherever the OctoSense shell does); `age_rating` is one of
`all 12+ 16+ 18+`; `privacy_policy_url` is an https URL; every screenshot
and the icon is a PNG or SVG inside the bundle; at most 10 keywords and 8
screenshots; unknown fields are refused. An icon and at least one screenshot
are required: the icon is what the launcher shows once the app is installed,
and a screenshot is the one claim a reviewer can check against the card.
Use one app-owned canonical icon across store and launcher surfaces; see
[ICONS.md](ICONS.md) for export limits, native rendering and small-size review.
The two-field icon declaration used by a built-in native app is not a complete
publishable listing.

To produce a screenshot, run an unsigned development bundle in the reference
host: `card-host --bundle my-app --allow-unsigned --remote`. Read its logged
endpoint; `/g` returns capture metadata and `/g?raw=1` returns PNG bytes.
Capture the actual app content at the host's current dimensions; do not assume
a fixed crop. The current reference host has no publisher-key option, so use
an unsigned development copy before final signing. See the
[first-app walkthrough](FIRST-APP.md#4-run-the-unsigned-development-bundle-and-capture-it)
and [native testing guide](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/lab/core/NATIVE-INSTRUMENT.md).

The store also shows a **privacy summary derived from the manifest**: what
the app stores, which hosts it contacts, which device features it uses,
whether it runs an assistant. Do not restate it in the description; make
the manifest right instead.

## Commands

The `hub` command is the same binary the hub itself runs, so the report you see
locally is the report the hub acts on.

```sh
hub stamp my-app                       # write the bundle digest into manifest.json; rerun after every change
hub check my-app --allow-unsigned      # current local gate; not a runtime or visual test
hub scan  my-app --packet review.json  # the agent-scan packet, to answer yourself or hand to a reviewer
```

`hub check` prints what the app will be granted. Read it back against the
manifest: if the grants are wider than the app visibly needs, reduce the manifest.
Use `--catalog <catalog.json>` when checking version and publisher continuity
against an existing catalog. Keep `review.json` outside `my-app/`; it is a
review artifact, not app content.

`hub scan` writes the questions a reviewer answers: does the app do what its
name claims, do its grants match what it draws, is anything deceptive, does any
text address an assistant rather than a person. Answer them honestly before
submitting; the hub's reviewer will ask the same ones.

## Signing

Signing is optional for a first submission and required for updates once a
key is on record. The example GitHub release workflow below is planned; use
manual signing until that action and submission route are available:

```sh
export APP_PUBLISHER_ID="your-publisher-id"
export APP_SIGNING_KEY="/absolute/private/path/publisher.key"
# Create the key once, outside the bundle; reuse it for future versions.
test ! -e "$APP_SIGNING_KEY" && hub keygen "$APP_SIGNING_KEY"
hub sign-manifest my-app --key "$APP_SIGNING_KEY" --key-id "$APP_PUBLISHER_ID"
APP_PUBLISHER_PUBLIC_KEY="$(hub pubkey "$APP_SIGNING_KEY")"
hub check my-app --publisher-key "$APP_PUBLISHER_ID=$APP_PUBLISHER_PUBLIC_KEY"
```

Sign after `hub stamp`, since the signature covers the digest. Once a bundle
is signed, `hub check` needs `--publisher-key <id>=<public key>` to verify it;
a signed bundle checked without the key is refused, by design.

## Submitting

Publishing is a pull request against the hub's index repository adding one
entry: your manifest, the bundle digest, your publisher id, the source
repository and commit, and a link to the bundle attached to your release. You
never fork the hub, and the app's code never enters it.

The planned release workflow has the following shape. Add it only after
confirming that the action and submission route are available:

```yaml
name: Publish to the OctoSense hub
on:
  release:
    types: [published]
permissions:
  id-token: write
  contents: read
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: octosense-org/publish-app@v1
        with:
          bundle: my-app
```

The action runs `hub check`, stamps, signs with the workflow's identity, packs
the bundle, and opens the submission on the hub. The hub re-runs the gate and
the scan on the exact bytes; a returning publisher's passing submission merges
without a person, a first submission or a human-review verdict waits for one.
(The action and the index repository are being set up; until they exist, run
the commands above and open the index entry by hand.)

## What happens after

- The hub keeps its own copy of the bundle and signs a new catalog. Every
  OctoSense store verifies that catalog against an anchor it ships with, so a
  catalog nobody signed is never shown.
- The app runs in its own isolate with exactly the manifest's grants; a
  request outside them fails with an error, and the person sees why.
- A version can be withdrawn with a reason. Installed copies stop running on
  the device's next catalog fetch. Publish a fixed version rather than
  arguing with a withdrawal.

## Do not

- Reference any server, CDN or local path from a card. Bundle the asset.
- Request `prompt`, `location`, `camera` or `clipboard` unless a screen needs
  it; each is shown to the person as a separate line.
- Put instructions to an assistant in card text or data. The scan treats text
  addressed to an assistant as a reason to reject.
- Edit `integrity.bundle_blake3` by hand. Run `hub stamp`.
- Reuse a version number.
