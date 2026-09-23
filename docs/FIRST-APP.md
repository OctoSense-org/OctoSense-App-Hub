# Build your first Hub app

This walkthrough creates an app-owned **Card bundle**. Native binaries ship
with the shell and follow a different build process. See the
[development guide map](DEVELOPMENT.md) for the available authoring paths.

## 1. Prepare the tools and an app repository

Use a Rust toolchain and the shared Makepad/Octoscript checkouts described in
the [native workspace guide](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/docs/NATIVE-WORKSPACE.md).
The Hub workspace's Cargo manifests declare its expected revisions and sibling
source overrides. Check those revisions before building; do not update a dirty
shared checkout just to satisfy a guide.

From the Hub repository:

```sh
cargo build --release -p octosense-app-hub --bin hub
cargo build --release -p octosense-card-host --bin card-host
```

Then choose absolute paths and copy the starter into a **new** destination:

```sh
export HUB_REPO="/absolute/path/to/OctoSense-App-Hub"
export HUB_BIN="$HUB_REPO/target/release/hub"
export CARD_HOST_BIN="$HUB_REPO/target/release/card-host"
export APP_REPO="/absolute/path/to/my-app"
test ! -e "$APP_REPO" && cp -R "$HUB_REPO/templates/app" "$APP_REPO"
```

If Cargo uses a custom target directory, set the binary paths to its actual
outputs. For an existing app repository, merge the starter files deliberately;
do not overwrite its instructions or metadata.

The [starter](../templates/app/README.md) contains metadata and an example icon.
It deliberately omits UI, kit and screenshot output: generate and test those
for your app before publishing. Only `bundle/` is submitted. Keep `AGENTS.md`,
design prompts, source code, tools, logs, keys and test state outside it.

## 2. Define and build the experience

Describe the app's purpose, screens, actions, data sources, state and error
handling. Use the [Image-to-AppCard workflow](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/lab/image-to-appcard-flow/README.md)
to create and review the native Card output. Its `plan` command shows the steps
before running them. App-specific service logic and data binding still require
implementation; a screenshot or an atlas is not a running app.

Copy the reviewed runtime output into this shape:

```text
my-app/
  AGENTS.md
  README.md
  bundle/
    manifest.json
    listing.json
    page.card
    page.data.json       # optional; omit if the card needs no data
    kit/                 # all files required by this card's kit
    assets/              # local runtime artwork and your icon
    screenshots/
      01-main.png        # actual capture, added after native validation
```

Keep asset references relative to the bundle. Check exported kit/data files
for stale author-machine paths and development-server URLs. A service flow
that relies on an external Python or browser controller must be adapted to the
contained runtime before it is a Hub app. Use [L0](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/a2app-l0/framework/l0.md)
for data/state/event semantics and the host's supported capabilities for effects.

## 3. Complete identity, permissions and artwork

Edit `bundle/manifest.json`: choose a stable app ID, release version and name.
The starter requests no capabilities or assistant. Add only supported
capabilities the implemented app needs; network access also needs exact hosts.
The [publishing manifest reference](PUBLISHING.md#the-manifest) defines the fields.

Edit **all** placeholder values in `bundle/listing.json`: description, category,
publisher/support/privacy information, tested platforms, release notes and
license. Replace the example icon following [ICONS.md](ICONS.md). The icon's
declared path must match the exported asset. Declare only platforms you tested.
The starter's `example.com` URLs are placeholders, not your privacy policy.

## 4. Run the unsigned development bundle and capture it

Stamp after copying the card, data, kit and artwork:

```sh
"$HUB_BIN" stamp "$APP_REPO/bundle"
cd "$HUB_REPO"
"$CARD_HOST_BIN" --bundle "$APP_REPO/bundle" \
  --app-data "$APP_REPO/.local-state" --allow-unsigned --remote
```

Launch from the host's workspace so its resources resolve. Keep the bundle
unsigned for this reference-host check: the current `card-host` has no publisher
key option and refuses signed manifests even with `--allow-unsigned`.
Use an unsigned development copy when revisiting an already signed release.

Inspect the host log for successful admission **and** card evaluation. Test
actual interactions and app state with the [native instrument guide](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/lab/core/NATIVE-INSTRUMENT.md).
Use the endpoint printed by this process; do not attach to an unrelated app.
A hidden native window still requires the platform's graphical session.

In a second terminal, set the endpoint from that log and capture a frame:

```sh
export APP_ENDPOINT="http://127.0.0.1:PORT"
mkdir -p "$APP_REPO/bundle/screenshots"
curl --fail --silent --show-error "$APP_ENDPOINT/g?raw=1" \
  -o "$APP_REPO/bundle/screenshots/01-main.png"
curl --fail --silent --show-error "$APP_ENDPOINT/quit"
```

Set `APP_REPO` in that terminal too. Inspect the captured PNG before using it;
do not publish an error frame. Use the actual app content and current capture
dimensions instead of assuming a fixed artboard size. Review the icon at small
sizes and, during integration, in the target shell's launcher and Hub surfaces.

## 5. Check the final bytes

Adding a screenshot changes the bundle digest. Restamp, then run the gate and
produce a review packet **outside** the bundle:

```sh
mkdir -p "$APP_REPO/build"
"$HUB_BIN" stamp "$APP_REPO/bundle"
"$HUB_BIN" check "$APP_REPO/bundle" --allow-unsigned
"$HUB_BIN" scan "$APP_REPO/bundle" --packet "$APP_REPO/build/review.json"
```

An unsigned warning is expected for this development check. A gate pass means
the current admission rules passed; it does not run the app, decode all artwork,
approve placeholders or verify your privacy-policy contents. The untouched
starter is expected to fail because its declared screenshot is absent.

Confirm the granted permissions match the app's behavior and record native
checks separately. Keep the review packet out of the bundle: including it
changes the digest and can introduce development-only content.

## 6. Sign and submit

Follow [Signing](PUBLISHING.md#signing) and [Submitting](PUBLISHING.md#submitting)
for the final release. Stamp before signing, verify with the publisher's public
key, and change the version for each update. Any edit after signing requires
restamping and signing again. Check the currently available submission route;
the publishing guide marks its example release action as planned.

Submit the app bundle and source reference, not changes to the production
catalog or the Hub's signing keys. Keep source, authoring instructions and tests
in your app repository. Publication and review happen separately from this
local walkthrough.
