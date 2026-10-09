# App Hub publishing reference

English | [简体中文](PUBLISHING.zh-CN.md)

| To | Read |
| --- | --- |
| Submit an app, step by step | [Submit an app to the App Hub](SUBMITTING.md) |
| Build and check a first app | [Build your first Hub app](FIRST-APP.md) |
| Set up the workspace, build `hub` and `card-host`, run and capture an app | [Quickstart](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/QUICKSTART.md) in OctoSense App Flow (formerly Design Flow), the app-development harness |
| Call a capability or a host service from a script | App Flow [Capabilities](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/CAPABILITIES.md) and [Script API](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/SCRIPT-API.md) |
| See which shell serves which host service | App Flow [Host services](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/HOST-SERVICES.md) |
| Prepare the icon | [App icons and bundled artwork](ICONS.md) |

The gate is the Hub's admission check. Run it yourself with `hub check`: it
reports each broken rule as a refusal or a warning. A pass does not show that
the app renders, that its icon reads at small sizes or that its listing is
true. Reviewers check those
([What reviewers check](SUBMITTING.md#8-what-reviewers-check)).

## What an app is

A bundle is a directory of text and artwork. A **host** runs it: the OctoSense
shell (the desktop or phone app) or `card-host`. Each app runs in an
**isolate**, a sandboxed script runtime of its own, under its **policy**: only
what its manifest requests and the host grants. A bundle holds no native code.
An app that needs new native code ships inside a shell release instead
([Choose a delivery path](DEVELOPMENT.md#choose-a-delivery-path)).

The entry file decides the kind of app:

| Kind | Entry | Runs as |
| --- | --- | --- |
| Script app | `main.splash` at the bundle root | A program in Splash, Makepad's UI script language, with its own state, handlers, storage and requests. |
| Card app | `page.card` at the bundle root | A card in L0, OctoSense's declarative card language, which the host renders as widgets. It holds no logic. |

A bundle with both entry files is a script app.

A script app's bundle holds these files:

```text
my-app/
  manifest.json      what the app is and what it may do       (required)
  listing.json       what the store shows about it             (required)
  main.splash        the program                               (required)
  assets/            the icon and other local artwork          (icon required)
  screenshots/       at least one PNG or SVG the listing names (required)
```

Build a script app with App Flow's
[script-app flow](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/flows/script-app/FLOW.md)
and [Script API](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/SCRIPT-API.md), starting from its
[`templates/script-app/`](https://github.com/OctoSense-org/OctoSense-App-Flow/tree/main/templates/script-app).

Name the bundle's own artwork through the `{{assets}}` placeholder, for example
`http_resource("{{assets}}/assets/logo.png")`. The host replaces the
placeholder with a loopback origin that serves this bundle and nothing else.

The first-party [system apps](https://github.com/OctoSense-org/OctoSense/tree/main/apps)
(`apps/<name>/bundle/`) are complete examples. Their `os.` ids are reserved,
so a copy needs an id of its own.

A card app's bundle holds these files:

```text
my-app/
  manifest.json      what the app is and what it may do       (required)
  listing.json       what the store shows about it             (required)
  page.card          the L0 card, the app's screen             (required)
  page.data.json     the data bound into the card              (optional)
  kit/               the widget kit that renders the card      (required)
  assets/            the icon and other local artwork          (icon required)
  screenshots/       at least one PNG or SVG the listing names (required)
```

Produce the card, its data and its kit with App Flow's
[image-to-card flow](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/flows/image-to-card/FLOW.md).
The [L0 specification](https://github.com/OctoSense-org/OctoSense/blob/main/apps/appcard/a2app-l0/framework/l0.md)
defines the card language.

Either kind may also ship its own agent next to `manifest.json`: `tools.json`,
`AGENT.md` and `skills/<name>/`
([The app's agent and tools](#the-apps-agent-and-tools)).

You submit only the bundle. Keep developer instructions, source tools, keys,
test data and review packets outside it
([Lay out the repository](SUBMITTING.md#1-lay-out-the-repository)).

## Rules the gate enforces

`hub check` runs in two stages: the structural limits first, then every other
rule.

### Structural limits

A bundle that breaks a structural limit gets no report. `hub check` prints a
single line, `hub: <reason>`, and exits 1. With `--json` it reports the same
reason as one refusal under the check name `bundle-invalid`.

| Limit | Reason printed |
| --- | --- |
| At most 8 MiB (8,388,608 bytes) of files, not counting `manifest.json` | `the bundle exceeds the size limit` |
| At most 2048 files and directories | `the bundle exceeds the file count limit` |
| Nesting at most 32 directories deep | `the bundle exceeds the directory depth limit` |
| A `manifest.json` at the bundle root, at most 64 KiB | `<bundle>/manifest.json: No such file or directory (os error 2)`, `manifest.json exceeds the size limit` |
| Regular files and directories only, no symlinks | `<path>: only regular files and directories are allowed` |
| Portable paths: UTF-8 names with no `\` or `:`, and no empty, `.` or `..` segment | `not a portable bundle path: "<path>"` |
| A manifest that parses: no unknown field or value, `schema` 1, every `requires` feature known | `manifest is not valid: …`, `manifest schema <n> is not 1`, `app <id> needs a newer host: …` |

### Findings

Every other broken rule adds a report line, a **finding**, that names its
check: `[refused] <check> (<path>): <detail>` or `[warning] <check>: <detail>`.
`(<path>)` appears only when one file or property is at fault. A refusal stops
admission. A warning does not.

| Check | Refused when | Warned when |
| --- | --- | --- |
| `digest` | `integrity.bundle_blake3` does not match the bundle. Stamp unsigned development bytes after edits; a sealed GitHub release needs a new proof. | |
| `publisher-signature` | The declared GitHub proof fails verification (even with `--allow-unsigned`); or the legacy signature does not verify against the key given with `--publisher-key` or recorded in the catalog; no key was given for a signed manifest (`publisher key "<id>" is not registered with this hub`); the manifest is unsigned and `--allow-unsigned` is absent. | The manifest is unsigned and `--allow-unsigned` is given. |
| `identity` | The id starts with `os.`; the id, or its last segment, is a reserved name ([Ids and reserved names](#ids-and-reserved-names)). | |
| `contents` | A file's extension is not one of `.card`, `.json`, `.l0`, `.octoscript`, `.splash`, `.svg`, `.png`, `.jpg`, `.jpeg`, `.webp`, `.ttf`, `.otf`, `.txt` or `.md`, and it is not a function module (`.wasm`, see `functions`). Files with no extension, such as `.DS_Store` and `LICENSE`, are refused too. | |
| `functions` | The bundle carries a `.wasm` module without the `wasm` capability, or more than 8 modules. A module that is not at `fns/<name>.wasm` (the name `[a-z0-9_-]`, at most 64 characters) or does not start with the 8-byte header of a WebAssembly core module, version 1, is refused as `contents-invalid`. The gate checks nothing else inside a module; the host checks its imports and exports when it loads the module. | The app declares `wasm` but carries no `fns/*.wasm`. |
| `contents-invalid` (text and images) | A text file (`.splash`, `.card`, `.json`, `.l0`, `.octoscript`, `.txt` or `.md`) is over 1 MiB or not UTF-8; JSON does not parse; a PNG, JPEG or WebP does not decode or is over 4096 px a side; the listing icon is not square, or is a bitmap over 1 MiB or 1024 px a side. | |
| `contents-invalid` (SVG) | An SVG does not parse; lacks a numeric `width` and `height` or a `viewBox`; is over 4096 px a side; or holds a script, a `foreignObject` or an `on…` handler. Its styling (a `style` attribute or a `<style>` block) imports a stylesheet, uses an escape, uses a comment or points `url()` anywhere but a `#fragment` in the same file. | |
| `entry` | The bundle has neither `main.splash` nor `page.card`; `page.card` is not valid L0; `page.data.json` is not JSON; neither `kit/native/<theme>/kit.json` nor every OctoScript kit module the card needs is in the bundle. | |
| `resource-invalid` | A card's image or font reference, or an SVG `href`, names a file that is not in the bundle. The finding names the JSON pointer. See [Fonts](#fonts). | |
| `assets` | A `.card`, `.json`, `.l0` or `.octoscript` file other than `manifest.json`, `listing.json` and the agent files contains `http://`, `https://`, `file://` or `../`. Plain documentation links are not asset loads. A `.splash` file or an agent file contains `http://`, `file://`, `../` or an `https://` host that is not in `network.hosts` (any public host is allowed when the app requests `images` or `web`). | |
| `secrets` | A `.card`, `.l0`, `.octoscript` or `.splash` file declares `is_password: true` or a `TextInputContentType` of `Password`, `NewPassword` or `OneTimeCode`. | |
| `storage` | | A `.splash` file calls `fs.*`, or the app requests `camera`, without the `storage` capability. The `grants:` line then says `storage none`. |
| `listing` | `listing.json` is missing or breaks a rule in [The listing](#the-listing); the listing names no screenshot or no icon; it names a screenshot or icon that is not in the bundle. | |
| `tools`, `agent`, `skills` | `tools.json`, `AGENT.md` or a skill breaks a rule in [Rules for agent files](#rules-for-agent-files). | A tool is destructive or outward (each call waits for approval); a tool says `confirm: "app"`; the app declares tools but `agent.model.needs` omits `tool_calling`; a background agent has destructive tools. |
| `policy` | A capability is unknown; the id breaks a rule in [Ids and reserved names](#ids-and-reserved-names); the version is empty; a host is not a bare host name, or hosts are listed without `net` ([Network hosts](#network-hosts)); the `research` scope or an `agent` field breaks its rules; `storage.cache_max_bytes` is 0. | |
| `version` (with `--catalog`) | The catalog already holds this version of the app. | |
| `continuity` (with `--catalog`) | The publisher rules in [Signing](#signing) are broken, or the catalog history disagrees with itself. | |

### Fonts

A card names a font with `font_src`, in the `page.data.json` placements or in
a native kit's component styles. The gate accepts a font file in the bundle,
or one of these exact built-in resource names from the pinned runtime:

```text
makepad_widgets:resources/Inter.ttf
makepad_widgets:resources/LXGWWenKaiRegular.ttf
makepad_widgets:resources/LXGWWenKaiBold.ttf
```

Use a bundle-relative path such as `"font_src": "assets/Body.ttf"` for a
bundled `.ttf` or `.otf`. The installed card runner and `card-host` evaluate
kit styles and font tokens first, then resolve the path and load the file
from the bundle's host-owned asset server. Keep HTTP URLs out of the bundle;
neither a network grant nor an external font URL is needed.

In a native kit, `font_src` may also be one token reference,
`{"$token": "<name>"}`, whose `value` in the kit's `tokens` is such a path. The
gate checks the path that the token resolves to. Any other object, or an
array, is refused with
`font_src must be a bundled font path, a supported built-in font, or one token resolving to a string`.

Other built-in names and crate paths are refused. Bundle a licensed font
subset when it is not one of these shipped resources.

Keep the font's required license and attribution with the bundle. Links in
plain `.txt` and `.md` documentation are not asset loads and grant no network
access. Agent instructions and skills still follow the declared-host checks;
card data, script code and SVG resources keep their own resource checks.

A bundled font counts toward the 8 MiB limit, so bundle a subset of a large
CJK font. The gate checks the font's path but does not decode the file, so
test the complete bundle in `card-host`. A host built with Makepad's
International font set, such as `card-host`, also draws Chinese with a
built-in CJK fallback font that it loads on first use.

OctoSense desktop 0.1.0-beta.2 predates bundled-font loading: it installs an
app with a bundled font, but its cards do not load the font. For Chinese text
on that release, build the card from the role kit and set no `font_src`. The
role kit is the OctoScript module set behind `Surface`, `TextTitle` and
`TextBody`: copy `_kit.octoscript`, `_derive.octoscript`,
`_derive_color.octoscript`, `_palette_light.octoscript` and
`_palette_dark.octoscript` from OctoScript-Makepad's `components/l0/` into the
bundle's `kit/`. It draws Chinese with Makepad's built-in CJK face, LXGW
WenKai.

By default, Makepad draws a glyph that the card's fonts lack with one of the
operating system's fonts, so on macOS a missing font can go unnoticed. Turn
that fallback off when you check a card:

```sh
MAKEPAD_SYSTEM_FONTS=0 tools/octo run ~/apps/my-app/bundle --port 8141 --detach
```

Each glyph that neither the card's fonts nor the renderer's fallback covers now
draws as a box.

A script app can bundle a font. Name the file through the `{{assets}}`
placeholder in a text style's `FontMember`; it counts toward the 8 MiB limit:

```splash
Label{text: "Hello" draw_text.text_style: TextStyle{font_family: FontFamily{latin := FontMember{res: http_resource("{{assets}}/fonts/<file>.ttf") asc: 0.0 desc: 0.0}} font_size: 20}}
```

Unverified: how the OctoSense shells draw these fonts.

## The manifest

```json
{
  "schema": 1,
  "id": "com.example.forecast",
  "version": "1.0.0",
  "name": "Forecast",
  "integrity": { "bundle_blake3": "<written by hub stamp>" },
  "capabilities": ["storage", "net"],
  "network": { "hosts": ["api.open-meteo.com"] },
  "storage": { "max_bytes": 1048576 },
  "compute": { "instruction_budget": 5000000, "memory_bytes": 33554432 }
}
```

| Field | Meaning | Rule |
| --- | --- | --- |
| `schema` | The manifest grammar. | `1`. |
| `id` | The app's identity. It names the app's storage folder, and its last segment is the namespace of the app's tools. | See [Ids and reserved names](#ids-and-reserved-names). Keep it the same in every version. |
| `version` | This release. | Not empty. Use a new value for every release; a published version is never replaced. |
| `name` | What the person sees. | |
| `integrity.bundle_blake3` | The bundle's digest. | Written by `hub stamp`. Never edit it by hand. |
| `integrity.github` | GitHub repository/owner IDs, workflow, tag, commit and attached Sigstore proof. | Requires `publisher-github-v1`; generated by `publisher-prepare` and `publisher-attach`. |
| `integrity.signature` | `{key_id, value}`, the publisher's signature over the manifest. | Written by `hub sign-manifest` ([Signing](#signing)). |
| `capabilities` | What the app may use. | Names from the closed list ([Capabilities](#capabilities)). |
| `network.hosts` | The hosts the app may reach. | Bare host names, and only with `net` ([Network hosts](#network-hosts)). |
| `storage` | `max_bytes`, `accounts`, `agent_workspace`, `cache_max_bytes`. | See [Storage and quotas](#storage-and-quotas). |
| `compute` | `instruction_budget`, `memory_bytes`. | Clamped to the host's ceilings. |
| `agent` | The app's own agent. | Optional ([The manifest's `agent`](#the-manifests-agent)). |
| `research` | The scope of `research` and `crawl`. | Required with `research` or `crawl`; refused when the manifest requests neither ([The research scope](#the-research-scope)). |
| `requires` | Host features the app needs. | Each must be a feature the host knows: `palpo-admin-v1`, `host-api-v1`, `backend-api-v1`, `script-tools-v1` or `publisher-github-v1`. GitHub publishing requires a real provenance verifier; the three API markers also need a host that implements their APIs ([Host API compatibility](HOST-API.md)). |
| `host_api` | The host API methods the app needs (`required`) or can use (`optional`), each with its ABI major version. | Optional; needs `host-api-v1` in `requires`. The store checks the `required` methods at install and at every launch ([Declare what the app needs](HOST-API.md#declare-what-the-app-needs)). |
| `backend` | The app's own backend: its public sign-in endpoints and the operations the app may call. | Optional; needs `backend-api-v1` in `requires`, the `auth` capability and `storage.accounts: true`. Holds no credentials ([Sign in to your own backend](#sign-in-to-your-own-backend)). |
| `schema_minor` | Which additions to schema 1 the manifest uses. | Leave it out. |

Any other field is refused. After `hub sign-manifest`, the manifest also holds
`null` for optional fields you left out, such as `"agent": null`. They change
nothing.

Ask for the least the app needs. The host grants nothing the manifest does not
request, and the store shows the person every request in plain words before
install.

### Capabilities

The gate knows 105 capability names: the 27 below and the 78 in
[Exact service names](#exact-service-names-octos-matrix-palpo). It refuses any
other name:

```text
[refused] policy: app com.example.forecast requests unknown capability "model.image"
```

A capability lets the app make requests; it does not provide a service to
answer them. A **host service**, code in the OctoSense shell that does what the
app may not do itself, answers them. **Served today** says what answers on
[desktop RC1](../README.md#download-a-compatible-host), within its stated
platform and provider limits; historical beta differences are explicit.

| Capability | Grants | The store says | Served today |
| --- | --- | --- | --- |
| `storage` | The app's own storage folder: `fs.*`, camera captures and local files a widget reads. Without it every `fs.*` call fails. | Keep its own data on this device | The runtime, in every host |
| `files` | Import and export files selected in the host's native dialog. Import/export also needs `storage`; the app receives an app-relative file, never general filesystem access. Require the specific `files.*` methods the app uses. | Import and export files you choose in the system file dialog | Contract 1.9, not yet published; requires a compatible host implementation |
| `net` | Requests to the hosts in `network.hosts`, and no others. | Reach only: *hosts* | The runtime, in every host |
| `images` | Pictures from any public https host, not only `network.hosts`. | Show pictures from any website | The runtime |
| `web` | Any public https page in the system web view, which has no way back into the app. | Open web pages in a browser view | The runtime on supported platforms, including RC1 Windows/WebView2 and Linux X11/XWayland/WebKitGTK; native Wayland embedding is unsupported ([requirements](../README.md#download-a-compatible-host)). |
| `location` | The device's location. On macOS in desktop RC1 and in compatible Android source builds, an app that declares `host-api-v1` must first ask with `location.permission.request`; on Android only, it can then read the last-known fix with `location.get` ([Host API compatibility](HOST-API.md)). | Use your location | The runtime, where the device has it |
| `camera` | The camera. A capture is saved in the app's storage, so the app also needs `storage`. On macOS in desktop RC1 and in compatible Android source builds, an app that declares `host-api-v1` must first ask with `camera.permission.request`. | Use the camera | The runtime, where the device has it |
| `microphone` | Sound with a camera video. On macOS in desktop RC1 and in compatible Android source builds, an app that declares `host-api-v1` must first ask with `microphone.permission.request`. | Use the microphone | The runtime, where the device has it |
| `library` | Offering captures to the system photo library, where other apps can see them. | Save to your photo library, where other apps can see it | The runtime, where the device has it |
| `clipboard` | The clipboard. | Use the clipboard | Not yet: no API uses it |
| `prompt` | Questions the app asks the person. | Ask you questions | Not yet: no host reads it. An app agent asks with `ask_user_question`. |
| `ledger.read` | Reading the shared ledger. | Read your shared data | Not yet: no `ledger` service |
| `mail` | Mail through the host's `mail` service, from accounts the person signs in to on a host [sheet](#sheets-apps-never-collect-secrets). | Read and send mail from accounts you sign in to on the device | OctoSense |
| `auth` | Connecting the app's own GitHub or Google accounts. In OctoSense desktop RC1, also signing in to the app's own backend and calling the operations the manifest declares ([Sign in to your own backend](#sign-in-to-your-own-backend)). | Connect its own GitHub or Google accounts, or sign in to its developer’s backend, through the host | OctoSense, with OAuth client registrations on the host ([Connected accounts](#connected-accounts)) |
| `github` | Reading repositories; each Markdown commit waits for the person's review. | Read authorized repositories and ask you to review Markdown commits | As `auth` |
| `gcalendar` | Reading Google calendars; each event change waits for the person's review. | Read authorized Google calendars and ask you to review event changes | As `auth` |
| `gmail` | Reading Gmail and keeping reply drafts; each send waits for the person's review. Separate from `mail`. | Read authorized Gmail messages, keep reply drafts and request native send review | As `auth` |
| `calendar` | Calendar's local event store and UI. Not Google Calendar. | Read and manage local events through the device's Calendar service | System app `os.calendar` only; store apps use `gcalendar` |
| `device_calendar` | Read selected native device calendars and request review of event changes. Separate app consent, OS permission and calendar selection are required. | Read device calendars you choose and ask you to review event changes | Contract 1.10.0 source, unpublished. Requires a compatible host adapter; not in desktop RC1 or Home beta.1. Declare exact required methods. |
| `llm` | Managing the device's AI providers through the `llm` service. | Manage the assistant's AI providers, whose keys stay with the device | System apps only |
| `news` | Items the device collects from its feeds and topic feeds. | Read news the device collects from its feeds and topics | System apps only |
| `photos` | Photos' own library and collections. | Read Photos's own library and publish collections | System apps only: OctoSense serves just `photos.notify`, to `os.photos` |
| `youtube` | YouTube search and music recommendations. | Search YouTube and manage music recommendations | System apps only: OctoSense serves just `youtube.notify`, to `os.youtube` |
| `glance` | Publishing Glance cards to the Glance screen (`glance.publish`, `glance.withdraw`, `glance.list`). The host checks, caps and expires the cards; a card opens only its own app. | Show cards on your glance screen | OctoSense |
| `model` | `model.complete` and `model.budget`, within a daily budget per app. `model.complete` takes a model class (`fast` or `strong`) and a JSON Schema. Image, audio, video and embedding methods are implemented in [OctoSense #368](https://github.com/OctoSense-org/OctoSense/pull/368), included in desktop RC1. | Send what you give it to the AI provider you configured, within a daily budget | OctoSense; media methods require the new implementation and a compatible configured provider |
| `research` | Searching through the system toolbox, within the manifest's research scope ([The research scope](#the-research-scope)). The host runs every search. | Search *what the scope allows* | System apps only, in phone builds |
| `crawl` | Crawling sites through the system toolbox, up to the scope's `max_depth` and `max_pages`, inside its domain lists. More reach than `research`. | Crawl websites, *within the scope*, which reaches more than searching | As `research` |
| `runtime` | Asking which APIs the host implements, with `runtime.list` and `runtime.describe` ([Host API compatibility](HOST-API.md)). It grants none of the APIs it lists. | Inspect available host APIs without gaining access to their data or permissions | Not on OctoSense desktop 0.1.0-beta.2, whose store refuses the name. App Hub's request dispatcher answers it in every host built from App Hub `main`, `card-host` included. |
| `wasm` | The app's own functions: WebAssembly modules in the bundle's `fns/` (at most 8), run by the host's `wasm` service in a sandbox with a deadline and a memory cap. A function gets only its input and reaches no file, network, clock or other app. An agent tool can run one with `host_method: "wasm.<function>"`. To write, build and call a function, see App Flow's [Run your own Rust code](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/RUST.md). | Run its own sandboxed functions on this device | Not in any release yet: standard desktop RC1 packages leave it disabled. Every standard desktop and Home build of OctoSense `main` serves it on macOS, Linux and Android, with limited support ([the service and its limits](https://github.com/OctoSense-org/OctoSense/blob/main/docs/wasm.md#the-service)). Builds for Windows, iOS and OpenHarmony leave it out: a call there answers `no service answers "wasm" on this device`. |

No capability implies another. Not yet: `photos` and `youtube` services for
store apps. For how a script calls each capability, see App Flow's
[Capabilities](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/CAPABILITIES.md).

Source: `KNOWN_CAPABILITIES` in `crates/app-contract/src/manifest.rs`.

### Exact service names: `octos.*`, `matrix.*`, `palpo.*`

Each of these 78 names is a separate capability, matched exactly. A prefix
such as `octos.` or `matrix.` is an unknown capability. Passing the gate is not enough: on every call, the host also
checks that it serves the name, that the app's policy includes it and that the
person granted it.

| Group | Names | Grants | Served today |
| --- | --- | --- | --- |
| `octos.*` | 4: `octos.session.open`, `octos.session.history`, `octos.turn.start`, `octos.turn.interrupt` | A conversation with the app's own agent, run by octos, the agent kernel OctoSense runs: open it, read its history, start a turn, stop a turn the app started. The app never names a provider, a model or a key. | OctoSense, once the person allows the app's agent. Until then a call answers `Waiting for the person to allow this app's agent (OctoSense asks the first time)`. |
| `matrix.*` | 45, such as `matrix.read_messages`, `matrix.room_members`, `matrix.send_message` | One Matrix operation each, on the person's current account, in the rooms they allow. | No OctoSense host service serves them. Unverified: Rinx, a native app OctoSense ships, serves them to bundles imported into it. |
| `palpo.*` | 29, such as `palpo.projects.list`, `palpo.inbox.decide` | One Palpo administration operation each. | Not yet: nothing in OctoSense serves them. |

Source: the store's words for each name are in
`crates/app-policy/src/services.rs` and `crates/app-contract/src/palpo.rs`.

### Ids and reserved names

An id is 1 to 64 characters of `[a-z0-9.-]`, does not start with `.` and never
contains `..`. The gate also refuses:

- **Every id under `os.`.** Those belong to system apps that ship with the
  device, and no store installs one.
- **An id that is a reserved name, or whose last segment is one.** The host
  keys an app's storage and consent by its id, and its tools by its namespace,
  so `com.example.notes` would stand in for the native Notes app. There are 23
  reserved names:

| Reserved because | Names |
| --- | --- |
| Apps OctoSense ships as native apps | `apphub`, `appcard`, `browser`, `calculator`, `clock`, `notes`, `octoscode`, `reference`, `reminders`, `rinx`, `sheets`, `task`, `terminal`, `weather` |
| Names the shell acts as or owns tools under | `agents`, `card`, `dev`, `octos`, `os`, `shell`, `system`, `toolbox`, `workflow` |

```text
[refused] identity: app id "com.example.notes" ends in "notes", which is reserved: its tools would be notes.*, a native app's or the host's
```

A system app's own namespace is not reserved: `com.example.news` is allowed.
An app that ships `tools.json` needs a namespace of `[a-z0-9_]{1,24}`:
`com.example.mynotes` can declare tools, and `com.example.my-notes` cannot.

Begin the id with a reverse-DNS prefix you control, such as your reversed
domain (`com.example` for `example.com`) or `io.github.<your-account>`; the
gate does not check that you control it.

Source: `RESERVED_NAMES` in `crates/app-contract/src/manifest.rs`.

### Network hosts

Apart from `images` and `web`, an app reaches the network only with `net` and
an exact host list. The runtime enforces the list on every path out of the
isolate: the network module, artwork loading and data fetches. `net` with an
empty list reaches nothing. Hosts match exactly: `example.com` does not allow
`api.example.com`. Write bare host names, such as `api.example.com`, with no
scheme, path, port or wildcard:

```text
[refused] policy: host "https://api.open-meteo.com/v1" must be a bare host name, with no scheme or path
```

The host list is part of each signed version, so list only stable hosts: a
published app cannot follow a tunnel's new name, and `localhost` is the
person's own device, not your server.

### Storage and quotas

| Field | Meaning |
| --- | --- |
| `storage.max_bytes` | The storage the app requests, in bytes. |
| `storage.accounts` | `true` gives each account its own data folder and agent. The default is one `device` folder. |
| `storage.agent_workspace` | `"account"` (the default): the agent reads the account's folder. `"none"`: the agent reads no files and works only through its tools. |
| `storage.cache_max_bytes` | The space the app requests for `cache/`, in bytes; above 0. |
| `compute.instruction_budget` | Script instructions per session, cumulative. |
| `compute.memory_bytes` | The isolate's heap. |

Quotas are requests. The host clamps each one to its ceiling, the host's
maximum, and an absent value gets the ceiling. The `grants:` line of
`hub check` shows the storage the app gets after clamping.

| Ceiling | Value |
| --- | --- |
| Storage | 16 MiB |
| Instructions per session | 20,000,000 |
| Heap | 64 MiB |

### The research scope

`research` and `crawl` share one scope, the manifest's top-level `research`
object. The gate checks it with octos's own rules, and the host hands the same
JSON to the system toolbox.

```json
{
  "capabilities": ["research", "crawl"],
  "research": {
    "langs": ["en", "zh"],
    "regions": ["US", "CN"],
    "domains_allow": [],
    "domains_deny": ["example-spam.com"],
    "max_age_days": 7,
    "categories": ["news"],
    "max_results": 20,
    "max_depth": 2,
    "max_pages": 20
  }
}
```

| Field | Meaning | Rule |
| --- | --- | --- |
| `langs` | BCP-47 languages the app may search in. | A language tag such as `en`, `zh-CN` or `zh-Hant`; `zh_cn` becomes `zh-CN`. |
| `regions` | ISO 3166-1 alpha-2 regions. | Two letters; upper-cased. |
| `domains_allow` | Only these domains and their subdomains. | A bare domain: `example.com`, `.example.com` or `*.example.com`; no scheme, path or port. |
| `domains_deny` | Never these domains. | As above. |
| `max_age_days` | The oldest material, in days back from now. | A whole number of days. |
| `categories` | Metasearch categories. | `news`, `general`, `science`, `it` or `social`. |
| `max_results` | The most results per search. | Above 0; default 20. |
| `max_depth`, `max_pages` | The `crawl` limits: link depth and pages of one crawl. | Both above 0 with `crawl`; both 0 or absent without it. |

An empty or absent list, or an absent `max_age_days`, means no limit. `{}`
keeps only the defaults: 20 results a search and no crawl. Unknown fields are
refused.

The store shows the scope in plain words. For `research` with
`{"langs":["en","zh"],"categories":["news"],"max_age_days":7}`, the store's
privacy summary ([The listing](#the-listing)) says "Searches news in English
and Chinese, from the last 7 days".

## The app's agent and tools

An app can ship an agent of its own: its tools in `tools.json`, its
instructions in `AGENT.md` and data-only skills under `skills/`, declared by
the manifest's `agent` block. Every file is under the bundle digest, so the
agent that runs is the one that was reviewed. The store calls this agent the
app's assistant.

### The manifest's `agent`

```json
"agent": {
  "profile": "workspace-write-never-ask",
  "tools": [],
  "max_iterations": 8,
  "token_budget": 120000,
  "model": {
    "needs": ["tool_calling", "long_context"],
    "tier": "standard",
    "per_task": { "triage": { "needs": ["tool_calling"], "tier": "fast" } }
  },
  "background": true,
  "triggers": { "schedule": ["0 7 * * *", "0 19 * * *"], "events": ["news.items.new"] },
  "instructions": "AGENT.md",
  "skills": ["news-digest"]
}
```

| Field | Meaning | Refused when |
| --- | --- | --- |
| `profile` | The agent session's permissions: `read-only` (every write asks first), `workspace-write` (reads and writes its workspace; anything else asks) or `workspace-write-never-ask` (the same, but anything else is refused without asking). No profile grants full access. | Any other value (`manifest is not valid`). |
| `tools` | Generic host tools (`ledger.read`, `ledger.write`, `net.fetch`, `storage.read`, `storage.write` and `card.render`) and one kernel tool, `ask_user_question`. The app's own tools come from `tools.json`; the agent gets both and nothing else. | A kernel tool other than `ask_user_question`, or a name the host does not offer. |
| `max_iterations`, `token_budget` | The most model iterations and tokens per request, clamped to 8 iterations and 200,000 tokens. | |
| `model` | What the agent needs from a model, never a provider or model name. `needs` lists any of `tool_calling`, `vision`, `long_context`, `reasoning`, `structured_output` and `multilingual`; `tier` is one of `fast`, `standard` or `strong` (default `standard`); `local_only` for data that must not leave the person's devices (app-wide; a task cannot relax it); `per_task` for named tasks that `AGENT.md` refers to. | An unknown need or tier (`manifest is not valid`); more than 8 `per_task` entries; a task name that is not `[a-z_]{1,32}`. |
| `background` | A request to run while the app is closed. The person grants it per app. | `true` without triggers. |
| `triggers.schedule` | Five-field cron, in local time. | Not five fields of digits and `* , / -`; more than 16 entries. |
| `triggers.events` | The app's own host-service events, in its namespace (`news.items.new`). | An event outside the app's namespace; more than 16 entries. |
| `instructions` | The agent's instructions, conventionally `AGENT.md`. | Not a `.md` path in the bundle. |
| `skills` | The skills to load, by directory name under `skills/`. | More than 16 skills, or one named twice. |

The agent's workspace is the app's own storage folder, and the agent reaches
only the app's hosts. It never gets more than the app has.

### An app with `tools.json` has an agent

OctoSense offers "Ask &lt;app&gt;" for any app that ships `tools.json`, even one with
no `agent` block. `hub check` still prints `agent none` for it, because the
`grants:` line reports only the manifest's `agent` block. What the store's
privacy summary says about such an app depends on the App Hub revision the
store was built from:

| Store | Privacy summary for `tools.json` without `agent` |
| --- | --- |
| A store built from App Hub `main` | "Offers the host's Ask assistant for its admitted tools, only after you consent. Your conversation and tool results may be sent to your configured AI provider." and "No app-declared background assistant or automatic triggers." |
| The store in OctoSense desktop 0.1.0-beta.2 | "Runs no assistant." |

Each OctoSense build shows the summary of the App Hub revision it pins. An app
with neither `agent` nor `tools.json` gets "Runs no assistant." in every store,
and an app that declares `agent` gets "Runs an assistant limited to this app's
own data."

The reference app GitHub Notes shows the difference. Version 0.1.0 ships three
tools with `"agent": null`: `hub check` prints `agent none`, and the store in
OctoSense desktop 0.1.0-beta.2 says "Runs no assistant." Version 0.1.1
declares a `read-only` `agent` block: `hub check` prints `agent read-only`,
and every store says "Runs an assistant limited to this app's own data."

If you ship `tools.json`, also declare `agent`, and disclose the agent in the
privacy policy. If the app should have no agent, ship no `tools.json`.

### `tools.json`: the app's tools

From the [News example](../crates/app-policy/tests/fixtures/news-agent),
abridged:

```json
{
  "schema": 1,
  "tools": [
    {
      "name": "news.list",
      "description": "List collected stories, newest first, optionally for one topic or since a time.",
      "input_schema": {
        "type": "object",
        "properties": {
          "topic": { "type": "string" },
          "since": { "type": "string", "format": "date-time" },
          "limit": { "type": "integer", "minimum": 1, "maximum": 100 }
        }
      },
      "output_schema": {
        "type": "object",
        "properties": { "items": { "type": "array", "items": { "type": "object" } } },
        "required": ["items"]
      },
      "risk": "read",
      "background": true,
      "shareable": true,
      "private_data": false,
      "implemented_by": "host-service"
    },
    { "name": "news.read",         "risk": "read", "…": "…" },
    { "name": "news.topics.get",   "risk": "read", "…": "…" },
    { "name": "news.topics.set",   "risk": "act",  "…": "…" },
    { "name": "news.digest.write", "risk": "act",  "…": "…" }
  ]
}
```

The gate admits these tools, but in a store app each call fails with
`not_granted`, because they have no `host_method`
([What OctoSense runs today](#what-octosense-runs-today)).

| Field | Meaning |
| --- | --- |
| `name` | `<namespace>.<tool>`. The namespace is the last segment of the app's id: `news` for both `os.news` and `dev.example.news`. Each segment is `[a-z0-9_]`. OctoSense's tool broker, which registers and dispatches app tools, spells the part after the namespace with underscores for dots (`news.topics.get` becomes `topics_get`). The gate refuses a broker name over 32 characters. |
| `description` | What the tool does and when to use it, for a model. 1 to 1024 characters. |
| `input_schema`, `output_schema` | JSON Schema, with only these keywords: `type`, `title`, `description`, `properties`, `required`, `items`, `enum`, `const`, `default`, `minimum`, `maximum`, `minLength`, `maxLength`, `minItems`, `maxItems`, `additionalProperties`, `format` and `pattern`. No `$ref`, no `anyOf`, `oneOf` or `allOf`, and no conditionals. At most 8 KiB and 8 levels deep. The input is an object. |
| `risk` | `read` (only reads), `act` (changes the app's own state) or `destructive` (sends, posts, shares, buys, deletes: anything past the app). Required. The tool broker's `Read`, `Act` and `Destructive` spellings are accepted. |
| `background` | The tool may run in a turn the person did not start. Default `false`. |
| `shareable` | Callers other than the app's own agent may be granted it, such as OctoSense's system agent (the device-wide agent the person talks to) and other apps' agents. Default `false`. |
| `private_data` | The result carries the person's private data. A shareable tool of a `local_only` app must say `false`; a `host_method` tool must say `true`, except one that runs `wasm.<function>`, which sees only its input. |
| `implemented_by` | `host-service`: a host service runs it. `app`: the app's own script runs it, in the open app ([Script tool execution](#script-tool-execution-script-tools-v1)). Required. |
| `host_method` | A reviewed shared-service method the tool runs on ([Map a tool to a shared service](#map-a-tool-to-a-shared-service-host_method)). Optional. |
| `outward` | Set it on an `act` tool whose call reaches outside the device (sends, posts, shares). Each call then waits for the person, as a destructive call does. Default `false`; refused on a `read` tool. |
| `auto_approvable` | A standing rule ("allow for an hour") may approve a call. Default `true`. Say `false` for deletion, payments, account or security changes and sharing outside the device, so the person approves each call as it happens. |
| `confirm` | Who asks the person before a destructive or outward call: `host` (the default, the host's approval path) or `app` (the app's own confirmation screen). `app` passes the gate only on a tool the app implements itself, and OctoSense refuses every destructive or outward call to a script tool that says `app` ([Who confirms a call](#who-confirms-a-call)). |

### Map a tool to a shared service: `host_method`

A `host_method` routes a tool to a reviewed method of a shared host service,
written `family.method`: the family is the service, such as `github` in
`github.read`. The tool keeps its own name in the app's namespace, and the
call runs as the app. The reference app [GitHub Notes](SUBMITTING.md#the-three-reference-apps)
maps its `githubnotes.read` tool to `github.read`:

```json
{
  "name": "githubnotes.read",
  "risk": "read",
  "private_data": true,
  "implemented_by": "host-service",
  "host_method": "github.read",
  "…": "…"
}
```

The gate refuses a `host_method` unless every rule holds:

| Rule | Refusal |
| --- | --- |
| The tool says `implemented_by: "host-service"`. | `host_method is only valid for implemented_by "host-service"` |
| The value is `family.method`, with `[a-z0-9_]` segments, at most 96 bytes. | `host_method must be family.method with nonempty [a-z0-9_] segments, at most 96 bytes` |
| No segment is `sheet`. | `host_method cannot target a host sheet or approve an action` |
| The method is in the table below, or is `wasm.<function>`: one of the app's own functions, one segment after `wasm.`. | `host_method "<m>" is not in the reviewed shared-service tool contract` |
| The tool's `risk` is at least the method's minimum (not for `wasm.<function>`). | `host_method "<m>" requires at least <risk> risk` |
| The tool says `"private_data": true` (not for `wasm.<function>`, which sees only its input). | `shared-service tools must declare private_data: true` |
| The manifest declares the method's family as a capability, or the exact method. | `host_method "<m>" requires the declared "<family>" service capability` |

| Family | Methods, minimum risk `read` | Methods, minimum risk `act` |
| --- | --- | --- |
| `github` | `github.repositories`, `github.files`, `github.read` | |
| `gcalendar` | `gcalendar.calendars`, `gcalendar.sync`, `gcalendar.refresh`, `gcalendar.cached`, `gcalendar.get`, `gcalendar.prepare` | |
| `device_calendar` | `device_calendar.permission.status`, `device_calendar.calendars.list`, `device_calendar.events.list`, `device_calendar.events.get` | |
| `mail` | `mail.compose_status` | `mail.compose` (local draft only) |
| `gmail` | `gmail.labels`, `gmail.messages`, `gmail.message`, `gmail.draft.get`, `gmail.event.status` | `gmail.draft.open`, `gmail.draft.edit`, `gmail.event.decide` |
| `glance` | `glance.list` | `glance.publish`, `glance.withdraw` |
| `auth` | `auth.backend.me`, `auth.backend.request` (declared `GET` operations only) | |
| `runtime` | `runtime.list`, `runtime.describe` | |
| `model` | `model.capabilities`, `model.video.status` | `model.image`, `model.audio`, `model.embeddings`, `model.video`, `model.video.cancel` |
| `camera` | `camera.permission.status` | |
| `microphone` | `microphone.permission.status` | |
| `location` | `location.permission.status`, `location.get` | |

The `device_calendar` aliases and `mail.compose` / `mail.compose_status` are
unreleased additions. All require `private_data: true` and the corresponding
capability. Calendar permission prompts, calendar selection and event writes
are foreground-only; so are `mail.review_send` and `mail.send`. Preparing a
draft does not send it. The compatible host must check authorization again
and show its own immutable review before a person approves an external write.

`wasm.<function>` runs one of the app's own functions (`fns/*.wasm`, the
`wasm` capability), only on hosts that serve `wasm`
([Capabilities](#capabilities)); standard desktop RC1 packages leave it
disabled. RC1 implements the `auth`, `runtime`, `camera`,
`microphone` and `location` methods above within the
[platform limits](HOST-API.md#limits).
The rules above apply to them too, `runtime.list` and `runtime.describe`
included. Admission does not configure an account, grant a permission or
add a missing API: check what the host implements with `runtime.describe`.
A tool mapped to `auth.backend.request` runs only the backend's declared
`GET` operations; a write still needs the app in the foreground and the
person's approval on the host's native review screen. Permission `request` and
`revoke`, account management and runtime ABIs such as `app_tools.dispatch@1`
have no `host_method`.

The seven media aliases above require [OctoSense #368](https://github.com/OctoSense-org/OctoSense/pull/368);
desktop RC1 includes them. They keep the `model` capability and
`private_data: true` requirements. Generation and embeddings may be billable,
so they and video cancellation require `act` risk and use the bounded model
service. Discovery does not prove provider entitlement; video cancellation
can fail when the remote job is already running. See the bilingual
[media contract](https://github.com/OctoSense-org/OctoSense/blob/main/apps/ai-providers/host-service/MEDIA.md)
for quotas and process-local job lifetime. Live provider/device validation is
not claimed by these policy aliases.

Account/business writes, sign-in, reviews and approvals have no `host_method`:
the person starts them from the app's own screen. The bounded model jobs above
are the explicit exception for model-provider submissions.

For a tool mapped to `glance.publish`, let `input_schema` accept only
`template` with `initial`, or an L0 `source` with `data`. Never accept
`script`. OctoSense desktop 0.1.0-beta.2 publishes an agent's script card
under the app's own policy, so a model-written `script` runs as your app.
OctoSense desktop RC1 refuses it with
`Agents cannot publish executable Splash; choose an admitted template with initial data, or L0 source`,
and refuses an L1 `source` too.

Source: `SHARED_HOST_METHODS` in `crates/app-policy/src/agent.rs`.

### Script tool execution (`script-tools-v1`)

A script tool is a tool with `"implemented_by": "app"`: the app's own Splash
code runs it, inside the open app. It needs a host that advertises
`app_tools.dispatch@1`, such as [desktop RC1](../README.md#download-a-compatible-host). OctoSense desktop 0.1.0-beta.2 refuses these tools with
`app_tool_unavailable`.

To add one:

1. Add `"requires": ["script-tools-v1"]` to the manifest.
2. Declare the tool in `tools.json` with its name, its schemas and
   `"implemented_by": "app"`.
3. Implement the `app_tool` hook in `main.splash`. In this example the app id
   is `dev.example.notebook`, so the tool namespace is `notebook`. An app
   cannot use `notes`, which is a reserved name
   ([Ids and reserved names](#ids-and-reserved-names)).

```splash
fn app_tool(name, call_id) {
    let request = mod.app_tools.request(call_id)
    if name == "notebook.read" {
        mod.app_tools.complete(call_id, {text: fs.read("note.txt")})
    } else {
        mod.app_tools.fail(call_id, "Unknown tool")
    }
}
```

`mod.app_tools.request(call_id)` returns `args`, the arguments after the host
has checked them against `input_schema`, and `context`, which holds `app`,
`account`, `caller` and `call_id`. The host fills in `context`; the script
cannot choose its values. Finish the call with `mod.app_tools.complete` or
`mod.app_tools.fail`. A result that does not match `output_schema` reaches the
caller as `invalid_result`. The caller gets `app_error` when the hook calls
`fail`, raises a script error or exceeds the VM's instruction limit. The host
does not enforce `pattern` or `format`.

The hook runs on the UI thread, in the same Splash VM and storage folder as
the app's screens, within the VM's instruction and memory limits. It can also
finish the call later, for example in a `host.request` callback:
`mod.app_tools.active(call_id)` says whether the call is still open. Another
app or a host sheet cannot use the `call_id` to read the arguments or finish
the call.

| Limit | Value |
| --- | --- |
| Arguments, result | 1 MiB each |
| Deadline | 60 s at most |
| Open calls | 16 per app, 128 per host process |

A call ends early when the app closes (`app_not_running`), the app's account
changes (`account_scope`), the deadline passes (`timeout`) or the caller
cancels. Ending a call neither stops a hook that is already running nor undoes
what it did, and the host ignores a `complete` or `fail` that comes after the
end.

A script tool cannot:

- Run while the app is closed. The host starts neither the app nor a background
  copy of it, even for a tool with `background: true`, so the call answers
  `app_not_running`.
- Run in a Glance copy of the app or in a second open copy. Only the full app
  opened first owns the tools; a second copy gets none and shows
  `App tools unavailable: app_busy: …`.
- Raise a host sheet, such as a permission request. While any of the app's
  calls is open, the app's own screens cannot raise one either.
- Be confirmed on the app's own screen. The host refuses a destructive or
  outward call to a script tool that says `confirm: "app"`, so keep the
  default, `confirm: "host"`.
- Use a host API that the app is not granted.

Unverified: script tools on a phone and with a real model.

### Who confirms a call

`risk` and `outward` decide whether a call needs the person; `confirm` decides
whose surface asks. Read calls, and act calls without `outward`, run
unattended. A destructive or outward call runs only after the person approves
it, live or through a standing rule:

| `risk: "destructive"` (or `outward: true`) with | Person present | Person absent |
| --- | --- | --- |
| `confirm: "host"` (default) | The host's approval path asks. | An approval request in the app's conversation. |
| `confirm: "app"` | The app's own confirmation screen is the only confirmation (for example Rinx's `send_message`). The host does not ask again. | An approval request in the app's conversation. |

The person is never asked twice for one call. A destructive tool may still say
`background: true`: the gate records a warning, and the tool runs only after
approval. `confirm: "app"` on a tool that is neither destructive nor outward
confirms nothing, and the gate warns about it.

Only a native app, such as Rinx, has a confirmation screen of its own today.
In a store app, the gate refuses `confirm: "app"` on a `host-service` tool,
and OctoSense refuses a destructive or outward call to a script tool that says
`confirm: "app"` ([Script tool execution](#script-tool-execution-script-tools-v1)).
Keep the default.

### `AGENT.md` and skills

`AGENT.md` is the agent's role and instructions: what to do on each trigger,
what matters in the app's data, the rubric for its output and its rules for
memory.

A skill is an octos skill directory with `SKILL.md` and `manifest.json`. For a
store app it is data only: the skill's `manifest.json` holds `name` (its directory),
`version`, `description`, `uses` (the tools it calls, each one of the app's
tools or in `agent.tools`) and optionally `prompts.include`.

```json
{
  "name": "news-digest",
  "version": "1.0.0",
  "description": "Write a cited morning or evening digest from collected stories.",
  "uses": ["news.list", "news.read", "news.digest.write"]
}
```

### Rules for agent files

The gate refuses these under the check `tools`, `agent` or `skills`:

| File | Refused when |
| --- | --- |
| `tools.json` | The file is over 64 KiB or declares no tools or more than 64; a tool is outside the app's namespace; the namespace is not `[a-z0-9_]{1,24}`; a name is duplicated; two names share a broker name; a broker name is over 32 characters; a tool lacks `name`, `description`, `input_schema`, `output_schema`, `risk` or `implemented_by`; a field is unknown; a schema uses a keyword the `input_schema` row does not list; a host-service tool says `confirm: "app"`; a `read` tool says `outward`; a shareable tool of a `local_only` app lacks `"private_data": false`; a `host_method` breaks its rules. |
| `AGENT.md` | `agent.instructions` does not name it; the text is empty, over 32 KiB, not UTF-8, holds control characters, starts with `#!` or contains `<script`, `<iframe`, `<object`, `<embed`, `javascript:`, `vbscript:` or `data:text/html`. |
| `skills/<name>/` | `agent.skills` does not name it; its manifest declares executable fields (`tools`, `binaries`, `sha256`, `mcp_servers`, `hooks`, `hardware_lifecycle`, `tool_discovery`, `actions` or `make_type`); it holds a file other than `.md`, `.json` or `.txt`, or a symlink; the manifest's `name` differs from the directory; a `uses` entry is neither one of the app's tools nor in `agent.tools`. |
| `AGENT.md` or `skills/` | The manifest has no `agent` block. |

### What the store shows

The store derives lines like these from the manifest and `tools.json`, and
shows them beside the other permissions before install:

- "Run an assistant for this app, only after you allow it" (`agent`)
- "Its assistant can use these app tools: news.list, news.read" (the tools in
  `tools.json` of an app that declares `agent`)
- "Its assistant requests these additional tools: net.fetch" (`agent.tools`
  other than `ask_user_question`)
- "Its assistant may work while the app is closed, on a schedule and when new
  data arrives; only if you allow it, and you can turn it off." (`background`)
- "Can ask to mail.send: nothing of this runs until you approve it." (a
  destructive or outward tool that the host confirms)
- "You approve every call of pay.transfer yourself: no standing rule can."
  (`auto_approvable: false`)
- "Offers news.list to other assistants you allow." (`shareable`)

The store in OctoSense desktop 0.1.0-beta.2 shows one line in place of the
first three: "Run an assistant for this app (&lt;tools&gt;), inside this
app's own data only", where &lt;tools&gt; lists only `agent.tools`. An app
whose tools are all in `tools.json` therefore gets "Run an assistant for this app
(no tools), inside this app's own data only".

### What OctoSense runs today

On OctoSense desktop 0.1.0-beta.2, each part works as follows:

| Part | Today |
| --- | --- |
| Talking to the agent | The shell's "Ask &lt;app&gt;" panel, once the person allows the app's agent. OctoSense asks at first use. `card-host` runs no agent. |
| Tools with `implemented_by: "host-service"` | Run as the app on the service that their `host_method` names, which the manifest must request. Without `host_method`, a tool calls the service named by its namespace. That works for a system app, such as `os.news`, but a store app's namespace is not a capability, so the call fails with `not_granted`. A tool call never raises a sheet. |
| Tools with `implemented_by: "app"` | Not yet: a call fails with `app_tool_unavailable`. |
| `AGENT.md` and skills | Loaded as guidance for every turn. They grant no tools. |
| `agent.tools` | `ask_user_question` works. Not yet: an executor for `ledger.read`, `ledger.write`, `net.fetch`, `storage.read`, `storage.write` or `card.render`. |
| `background` and `triggers.events` | Only the event `<namespace>.new_message`, for an app granted `gmail` and `auth`, with `background: true`, after the person allows its agent. |
| `triggers.schedule` and other events | Not yet. |
| `agent.model` | Not yet: OctoSense ignores it. |
| Glance cards from the agent | Any card the app may publish, a `script` card included. |

OctoSense desktop RC1 changes four things:

- Tools with `implemented_by: "app"` run in the open app
  ([Script tool execution](#script-tool-execution-script-tools-v1)).
- A tool call that publishes a Glance card (`glance.publish`, directly or
  through `host_method`) accepts only a template with an `initial` object, or
  L0 `source`. It refuses `script` and L1 source with the error kind
  `unsafe_card_source`.
- Approving a GitHub or Google Calendar save takes a physical press
  ([Connected accounts](#connected-accounts)).
- The host keeps Google Calendar events from 30 days back to 366 days ahead.

## The listing

`listing.json` is what a person sees in the store before installing. It is
reviewed with the bundle and travels in the signed catalog, so what a reviewer
read is what the store shows. The permissions shown beside it come from the
manifest, never from the listing, so a listing cannot understate what the app
does.

| Field | Rule |
| --- | --- |
| `schema` | `1`. |
| `subtitle` | At most 80 characters. |
| `description` | Not empty; at most 4000 characters. |
| `category` | One of `productivity`, `utilities`, `photo-video`, `news`, `weather`, `travel`, `finance`, `health`, `education`, `entertainment`, `games`, `social`, `shopping`, `lifestyle` or `developer`. |
| `keywords` | At most 10. |
| `screenshots` | 1 to 8 paths to PNG or SVG files in the bundle. Use real captures of the running app ([Capture the screenshots](SUBMITTING.md#4-capture-the-screenshots)). |
| `icon` | The path to a square PNG or SVG in the bundle ([App icons and bundled artwork](ICONS.md)). A PNG icon is at most 1 MiB and 1024 px a side. Use the same icon in the launcher. |
| `platforms` | At least one of `android`, `ios`, `macos`, `windows`, `linux`, `openharmony` or `web`. List only platforms you tested. |
| `publisher.name` | Not empty. |
| `publisher.support` | A URL or an email address. |
| `publisher.privacy_policy_url` | An `https://` URL. |
| `release_notes` | What changed in this version. |
| `age_rating` | `all`, `12+`, `16+` or `18+`. |
| `license` | An SPDX identifier, when the source is open. |

`subtitle`, `keywords`, `release_notes` and `license` are optional; every other
field is required. Unknown fields are refused. The listing is part of the
signed version: to change its text, publish a new version.

The store also shows a **privacy summary**, derived from the manifest and, in a
store built from App Hub `main`, the reviewed tools: what the app stores,
which hosts it contacts, which device features it uses and whether it runs an
agent ([An app with `tools.json` has an agent](#an-app-with-toolsjson-has-an-agent)).
Do not restate it in the description; make the manifest right instead. You
may still describe what your app agent sends, as GitHub Notes 0.1.1 does.

## Host services and sheets

A script app never holds a credential. For work that needs one, it calls a
host service:

```splash
host.request("mail.list", {…}, fn(r){ … })
```

The isolate refuses the call unless the app's policy grants the family (`mail`
for `mail.*`) or the exact service name. A granted call goes to the service
the host registered for that family. The service does the work and answers
with data, never with a credential. A connection handle it returns is opaque
and bound to the app. A call to a family that
no service answers fails at once with `no service answers "<family>" on this device`;
`card-host` registers no service; only App Hub's `runtime` discovery
answers there.
To see which shell serves which family, read
App Flow's
[Host services](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/HOST-SERVICES.md).

`card-host` also implements none of the APIs that `host-api-v1`,
`backend-api-v1` and `script-tools-v1` require, so it refuses an app whose
manifest requires one of them: test such an app in an OctoSense shell built
from `main`.

### Sheets: apps never collect secrets

Input that only the person should give, such as a password or an account
approval, goes on a **sheet**: a surface the host draws over the app, in an
isolate of its own under no app's policy. Only a service can open one. Service
methods that take a secret live under `<family>.sheet.` and are accepted only
from that sheet. An app's own password fields take no input at runtime, and
the gate refuses a bundle that declares one.

While a sheet is up, text, keys, IME input, the clipboard and pointer releases
go only to the sheet; timers and service replies still reach the app. A host
built from App Hub `main` keeps its own reference to the sheet, so an app
widget named `sheet` cannot hide or replace it.

See App Flow's
[Mail, the worked example](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/HOST-SERVICES.md#mail-the-worked-example).

### Connected accounts

An app that works with GitHub or Google declares `auth` plus the provider
capability it uses: `github`, `gcalendar` or `gmail`. The person signs in on a
host sheet, and the app gets a connection handle, never a token. Writes go
through a host review. Set `storage.accounts: true`, so that each account keeps
its own data. For the hosts that serve these apps, see
[Before you start](SUBMITTING.md#before-you-start); for the calls, see
App Flow's [Use a connected account](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/CAPABILITIES.md#use-a-connected-account).

What the host enforces depends on its build and platform. Protected writes
remain unsupported and fail closed on Windows/Linux; see
[platform limits](../README.md#download-a-compatible-host):

| | OctoSense desktop 0.1.0-beta.2 | OctoSense desktop RC1 |
| --- | --- | --- |
| Approving a Gmail send | A physical press on the native Approve & Send control | The same |
| Approving a GitHub or Google Calendar save | The host's review sheet, which does not check that the press is physical | A physical press on the native Approve & Save control; script and agent requests cannot approve |
| Google Calendar events the host keeps | The calendar's whole history | 30 days before today to 366 days after (UTC), with recurring events expanded |

To identify the person without reading their data, declare `auth` alone and
request only identity scopes: `read:user` for GitHub, or `openid`, `email` and
`profile` for Google. `auth.connect` then returns a connection with the
provider's `subject` and a display `label`, and grants no access to
repositories, mail or calendars.

Sign-in needs the provider's OAuth registration in the host:

| Host | Registrations come from | Without one, `auth.connect` fails with |
| --- | --- | --- |
| OctoSense desktop 0.1.0-beta.2 | The host's `oauth/clients.json`, which the release does not ship | `OAuth is not configured. Add provider registrations in the host's oauth/clients.json` |
| OctoSense desktop RC1 | Distributor build settings or a host `oauth/clients.json` override; the public RC1 packages include no registrations | `GitHub sign-in is unavailable in this build. Check for an OctoSense update or contact its distributor.`, or the same for Google |

### Sign in to your own backend

OctoSense desktop RC1 can sign an app in to its developer's
backend and call the backend operations that the app declares, within the
[platform limits](../README.md#download-a-compatible-host). Declare the
backend in the manifest. The declaration is public and holds no credential:

```json
{
  "capabilities": ["auth"],
  "storage": { "accounts": true },
  "requires": ["backend-api-v1"],
  "backend": {
    "id": "notes",
    "client_id": "<public-client-id>",
    "authorization_url": "https://login.example.com/authorize",
    "token_url": "https://login.example.com/token",
    "me_url": "https://login.example.com/me",
    "logout_url": "https://login.example.com/logout",
    "scopes": ["app.session"],
    "operations": {
      "notes.list": { "method": "GET", "path": "/api/notes", "query_keys": ["tag"] },
      "notes.create": { "method": "POST", "path": "/api/notes" }
    }
  }
}
```

| Field | Rule |
| --- | --- |
| `id` | 1 to 64 characters of `[A-Za-z0-9._-]`. |
| `client_id` | The backend's public client id: 1 to 256 characters, with no spaces or control characters. |
| `authorization_url`, `token_url`, `me_url`, `logout_url` | Four different `https://` URLs on one origin and port 443, with no user name, password, query, fragment, `%` escape or `\`, and no `.` or `..` segment. |
| `scopes` | Exactly `["app.session"]`. |
| `operations` | Up to 64, keyed by name (`[A-Za-z0-9._-]`). Each has a `method` (`GET`, `POST`, `PUT`, `PATCH` or `DELETE`), an exact `path` on the same origin that is not one of the four URLs' paths, and up to 32 distinct `query_keys`. |

The manifest must also request `auth`, set `storage.accounts: true` and require
`backend-api-v1`. The gate checks these rules as it parses the manifest, so a
broken declaration stops `hub check` with a single line, such as
`hub: backend requires auth and storage.accounts`.

At runtime, call the `auth` service:

1. `auth.connect` with `{"provider":"backend","scopes":["app.session"]}`. The
   person registers or signs in on the backend's own web page. On macOS and
   Android 9 or later, the host shows that page in its own web view, which
   stays on your login origin. On desktop, add `"presentation":"browser"` to
   use the system browser instead, as you must if the page sends the person to
   GitHub or another provider.
2. `auth.backend.me` with the returned connection handle. It answers with the
   backend's verified identity (`sub` and `label`).
3. `auth.backend.request` with the handle, an operation and its declared query
   keys, such as
   `{"connection":"<handle>","operation":"notes.list","query":{"tag":"work"}}`.
   A write also takes a JSON `body`. The host adds the session's token, calls
   the declared method and path, and answers with the backend's JSON.

A `GET` operation runs at once, even from a home-screen tile or an agent tool.
A `POST`, `PUT`, `PATCH` or `DELETE` operation runs only after the person
approves the exact request on the host's native review with a physical press,
so the app must be in the foreground. From a tile or an agent tool, a write
fails with `Open the app to review this backend change`. Bodies and answers are
at most 64 KiB of JSON, and the host refuses redirects. Changing or removing
the `backend` block, updating the app or withdrawing it ends the app's backend
sessions, and the person signs in again.

Where it works:

- Only a host that implements `auth.backend.request@1` installs the app:
  desktop RC1 advertises it on macOS, Windows and Linux; compatible Android
  source builds also implement it. Windows/Linux use external-browser login
  and declared reads; embedded login and protected writes are unsupported.
  A host missing the method refuses it with
  `app <id> needs a host implementing auth.backend.request@1`. iOS has no backend sign-in.
- An app without a `backend` block signs in only on devices whose operator
  registered its backend in the host's `oauth/backends.json`. Windows and Linux
  use the browser for that sign-in (unverified).
- Unverified: a live sign-in and an approved write on a device.

What an app cannot do:

- Reach its backend any other way. It cannot choose a URL, method, header or
  token, or send a query key it did not declare, and it never sees the
  backend's tokens.
- Approve its own writes. Script and agent requests cannot approve the host's
  review.
- Collect the password itself, or reuse the host's GitHub or Google tokens.

The backend's side of the protocol is in OctoSense's
[developer backend contract](https://github.com/OctoSense-org/OctoSense/blob/main/crates/oauth-service/README.md#developer-backend-contract).

### Limits on host-service calls

Each call gets exactly one answer: the service's data, or an error the app can
act on.

| Limit | Value | Otherwise the app gets |
| --- | --- | --- |
| Time to answer | 60 s, unless the service asks for more; paused while the service's sheet is up | `the host service timed out` |
| Calls waiting per app | 32 | `too many host requests are waiting; try again when some have answered` |
| Arguments | 1 MiB of valid JSON | `the request's arguments exceed 1 MiB`, or `the request's arguments are not valid JSON: …` |
| Answer | 4 MiB | `the service's answer exceeds 4 MiB` |

When the app closes, its waiting calls end with it, and no late answer reaches
the app's next session.

### Where a sheet may appear

A service may raise a sheet only over an app in the foreground. For an app
shown as a home-screen tile, or a tool call from an agent, the host sets
`may_prompt` to `false` and refuses any sheet with
`this surface cannot raise a prompt; open the app to continue`. The `prompt`
capability does not change this.

## Commands

`hub` runs the gate's own code, so its report is the one the Hub acts on.
Build it as in App Flow's
[Quickstart](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/QUICKSTART.md).
Run `hub`, `hub help` or any command with `--help` or `-h` for the usage;
none of them reads a bundle or writes a file. An unknown command fails with
``hub: unknown command "<name>"; run `hub help` for usage``.

| Command | What it does |
| --- | --- |
| `hub publisher-prepare/attach/verify/pack/unpack/entry` | Prepare a canonical GitHub subject, attach its proof, verify, deliver or build a review candidate; see [GitHub publisher provenance](#github-publisher-provenance) and `hub --help` for exact flags. No command publishes the catalog. |
| `hub stamp <bundle>` | Parses `manifest.json` with the gate's parser, then writes the bundle's digest into `integrity.bundle_blake3` and prints it. Refuses a manifest the gate cannot read or an already GitHub-attested release. |
| `hub check <bundle> [--allow-unsigned] [--publisher-key <id>=<hex>] [--catalog <file> [--anchor <hex>]] [--json] [--system-app]` | The gate. Prints `PASSED` or `REFUSED`, every finding and what the app will be granted. Exits 1 on a refusal. `--json` prints the report as JSON (`schema`, `stage`, `passed`, `app_id`, `version`, `digest`, `findings` and `resources`). |
| `hub scan <bundle> [--publisher-key <id>=<hex>] [--catalog <file> [--anchor <hex>]] [--packet <out.json>] [--reviewer <cmd>] [--system-app]` | Runs the gate, then writes the review packet and optionally hands it to a reviewer command. A bundle the gate refuses gets no scan. |
| `hub keygen <key-file>` | Creates a new key file, writes a signing key into it as hex and prints the public half. Refuses a path that already exists, including a symlink. On macOS and Linux the file is readable only by you (mode `0600`). |
| `hub pubkey <key-file>` | Prints a key's public half. |
| `hub sign-manifest <bundle> --key <key-file> --key-id <publisher-id>` | Signs the manifest, which covers the digest. |
| `hub verify <catalog> --anchor <hex>` | Verifies a catalog against a trust anchor. |

`--catalog <file>` adds the `version` and `continuity` checks against a
published catalog, such as this repository's `catalog.json`, and uses the
publisher identities it records. A missing file is refused. A v2 catalog must
verify against the compiled GitHub workflow policy; a legacy catalog must
verify against the Hub's anchor
([Trust anchor](../README.md#trust-anchor)), or against `--anchor <hex>` for a
development hub. Otherwise `hub check` stops with
`hub: could not authenticate base catalog: …`.

`hub publish`, `hub withdraw`, `hub remove` and `hub certify` need the Hub's own
keys. Only the Hub's maintainers run them.

### Read a `hub check` report

```text
my-notes 0.1.0 — PASSED
  [warning] publisher-signature: unsigned: accountability rests on the hub alone
  grants: capabilities {"storage"}, hosts {}, storage 16777216 bytes, agent none
```

The first line gives the app id, version and verdict. Each finding line uses
the format in [Findings](#findings). The `grants:` line is what the app will
get: its capabilities, its hosts, its storage quota in bytes (or `none`
without `storage`) and its agent profile (or `none` without an `agent` block). The
profile appears in the kernel's spelling: `workspace-write-never-ask` prints as
`workspace-write-never`.

### The scan questions

The review packet that `hub scan` writes holds the manifest, the listing, the
grants in the store's words, the entry file's source, the card data, the agent
files and the questions. It holds no screenshots. Keep it outside the bundle.

The questions, abridged from `crates/app-hub/src/scan.rs`:

1. Does the app do what its name, subtitle and description claim?
2. Do its platforms and category fit?
3. Do the granted capabilities, and every host, match what the app visibly
   does?
4. Is any part of the interface deceptive?
5. Does any text in the source or data address an assistant?
6. Is any wording abusive, or aimed at a private individual?
7. With `tools.json`, `AGENT.md` or skills only: do the agent files stay within
   the app, and does each tool's risk match what it does?
8. Route: pass, human-review or reject, with reasons a publisher can act on.

Reviewers ask the same questions.

### Developing a shipped system app

Use `hub check <bundle> --system-app` or `hub scan <bundle> --system-app`
for a local `os.*` bundle. This mode uses the same resource ceilings as
`card-host --system`, accepts an unsigned development bundle, and still
checks its digest, contents, tools and any supplied signature. Supply
`--publisher-key` when the bundle is signed. Store-app IDs are refused in
this mode, and `publish` refuses both `--system-app` and `os.*` bundles.
The mode does not install host services or add the shell's extra tool offers.
A passing system-development report cannot become a catalog entry.

The optional reviewer command runs in `sh -c` on Unix and `cmd.exe /D /S /C`
on Windows. Configure a command for the host platform; it receives the review
packet on stdin. Its failure or invalid output still requires human review.

## Signing

New apps can use GitHub-managed publisher provenance: developers do not
create, store or rotate a publisher private key. This source implementation
uses published contract **1.8.0** and `publisher-github-v1`;
[desktop RC1](../README.md#download-a-compatible-host) is a compatible released host. Two real tag-push releases passed the workflow and
native Store acceptance described below. The historical Ed25519 route
remains below for existing packages.

### GitHub publisher provenance

A public repository's GitHub-hosted workflow runs on a `v<manifest.version>`
tag push. The host verifies a full Sigstore v0.3 proof offline using its
embedded public-good trust snapshot: certificate, issuer, signature, signed
certificate timestamp and transparency-log evidence. It also requires exact
repository and owner IDs, repository URL, local workflow path, tag, source
commit, workflow commit and the canonical manifest subject. A GitHub identity
is authority over a repository workflow, not proof of a particular person's
real-world identity. No app receives GitHub credentials.

This route needs a public repository: GitHub signs a private repository's
attestations with its own Sigstore instance, which hosts do not trust.
OctoScript apps are open by default: each bundle already ships the app's
source as readable text, so a public repository reveals little more
([ADR 0002](adr/0002-github-attested-publisher-identity.md)).

The workflow calls these native commands:

```sh
hub publisher-prepare bundle --repository OWNER/REPO \
  --repository-id REPOSITORY_ID --owner-id OWNER_ID \
  --workflow .github/workflows/publish-app.yml \
  --tag v0.1.0 --commit IMMUTABLE_GIT_SHA \
  --out build/octosense-app-manifest.json
# actions/attest attests exactly build/octosense-app-manifest.json.
hub publisher-attach bundle --attestation build/publisher-attestation.sigstore.json
hub publisher-verify bundle
hub publisher-pack bundle --out build/app.bundle.pack.json
```

`build/` must exist outside `bundle/`. `publisher-prepare` adds
`requires: ["publisher-github-v1"]`, records the identity in `integrity.github`
and stamps the final bundle. Canonical signing bytes include that identity
and the bundle digest, but exclude `integrity.github.attestation`. The proof
is attached afterward; the manifest is excluded from the bundle digest, so
there is no hash cycle. The proof is at most 48 KiB and the complete manifest
at most 64 KiB. Prepare refuses an already sealed release. Attach and pack
verify the final bytes and never restamp; edits require a new release proof.
No `keygen`, `sign-manifest` or `--publisher-key` belongs in this workflow.

Validation used the public synthetic fixture's [v0.1.0 workflow](https://github.com/ymote/octosense-publisher-fixture/actions/runs/37736273522)
and [v0.1.1 workflow](https://github.com/ymote/octosense-publisher-fixture/actions/runs/37736765473),
without repository secrets. Both generated and verified real GitHub proofs.
The [native acceptance example](../crates/app-hub/examples/publisher_acceptance.rs)
then installed both release packs, prepared and validated launches, preserved
the full proofs, and refused content/proof/identity tampering, rollback,
unsigned ownership replacement and withdrawn releases. The [receipt](../reviews/github-publisher-v1/acceptance.json)
binds the input hashes and native source. Its Store catalog was an ephemeral
local test catalog; the fixture has no App Hub submission or catalog entry.
This does not establish app UI execution or phone publisher installation.

On macOS, [OctoSense desktop RC1](../README.md#download-a-compatible-host)
installs the GitHub-attested sample apps in catalog sequence 13 and checks
their attestations and publisher continuity at install and update. Store
installs on iOS, Windows and Linux remain unverified, and no released phone
build supports `publisher-github-v1`.

Download the **release pack**, which contains the generated attested manifest;
a source checkout alone does not contain those final bytes. A reviewer can
run `hub publisher-unpack app.bundle.pack.json --out review-bundle`, then
`hub publisher-verify review-bundle --catalog <authenticated-catalog>`.
Unpack requires a new directory, rejects traversal and removes its own output
on failure. `hub publisher-entry review-bundle --catalog <authenticated-catalog>
--out build/index.json` produces a review candidate, without publication.

Open an **App Hub submission issue** to request publication; it can precede
the release. Add the repository, immutable tag/commit, release-pack link and
verification evidence when ready. Creating a tag or GitHub Release
does not submit, approve or list the app. Admin catalog review remains the
[separate protected publication flow](GITHUB-PUBLISHING.md).

Catalog publisher identity is `github:<repository_id>`. Updates retain the
same repository, immutable repository/owner IDs and workflow path, and must
advance semantic-version precedence. A withdrawn release remains in ownership
history. Unsigned updates, replacing that identity, replaying a version and
adopting a historical Ed25519 app ID are refused. This change supplies no
migration or recovery route and changes no admitted artifacts.

Older hosts refuse the new marker; a host with no provenance verifier refuses
the proof even with unsigned development enabled. Standalone `card-host` has
no publisher verifier: use an unsigned development copy for its previews and
a compatible Store host for the sealed app. Admission is not app UX or device
acceptance. Publishing contract 1.8.0 alone does not ship a compatible host.

### Legacy Ed25519 publisher signatures

Catalog administrators can use [GitHub-managed catalog signing](GITHUB-PUBLISHING.md)
without a separate Hub private key. Existing Ed25519 publisher history and
its signature bytes remain unchanged:

`hub stamp` and `card-host --stamp` refuse existing signing metadata without
rewriting it. To edit a signed release, first create an unsigned development
copy; keep the original release intact, then stamp and sign the final new
version. An absent or null `integrity.signature` remains unsigned.

Your publisher id is your signature's key id. The key on record is the one in
the published catalog, never one a submission supplies:

| Case | Rule | Refusal |
| --- | --- | --- |
| A first submission | May be unsigned. | |
| An update of a published app | Must be signed under the publisher id on record. | Unsigned: `continuity: <app id> is already published by "<publisher-id>"; an update must carry that key`. Under another id: `continuity: <app id> was published by "<publisher-id>"; this version is signed by "<other-id>". Re-keying is a reviewed change.` |
| Any manifest signed under a publisher id that has a key on record | Must be signed by that key. | `continuity: not signed by the key on record for "<publisher-id>": …`; with another key in `--publisher-key`: `publisher-signature: conflicting public keys for publisher key "<publisher-id>"` |
| The first signed update of an app published unsigned | Puts its key on record. | |

There is no key-replacement flag. A lost or rotated key, or history that
disagrees with itself, needs a maintainer: ask in an issue.

> **Warning:** The key file is the only copy of your publisher key. Keep it
> outside every repository, back it up and never share it. `hub keygen`
> refuses to overwrite an existing file, and on macOS and Linux it creates the
> key readable only by you. On Windows, keep the key in a folder that only
> your user can read.

The signature covers the manifest, including `integrity.bundle_blake3`. Follow
these rules:

- **Sign last.** `card-host`, and any host without a signature verifier,
  refuses a signed bundle: `no signature verifier is installed, so the
  signature from key "<id>" cannot be checked`. Capture screenshots and test
  on the unsigned bundle, then stamp and sign.
- **Stamp, then sign.** Signing an unstamped manifest signs the wrong digest.
- **Prepare a new unsigned copy after an edit.** Changing bundle bytes
  invalidates their digest; changing signed manifest fields invalidates the
  signature. Preserve the original release. In the development copy, remove
  the old `integrity.signature`, then run `hub stamp`, `hub sign-manifest`
  and `hub check --publisher-key` for the final new version. `hub stamp`
  refuses the old signature instead of silently invalidating it.
- **Check signed bytes with the key.** Unless `--catalog` already records
  your key, `hub check` refuses a signed bundle without
  `--publisher-key <id>=<hex>`, even with `--allow-unsigned`. `hub scan` takes
  no `--allow-unsigned`: it accepts unsigned bundles, and a signed one needs a
  key the same way.
- **Keep the bytes exact.** The digest covers every file except
  `manifest.json`: its path, length and bytes, with `/` between path segments
  on every platform. Line-ending conversion changes it, so keep Git from
  converting the bundle
  ([Lay out the repository](SUBMITTING.md#1-lay-out-the-repository)).

## Submitting

Submit by opening an issue, never by a pull request that edits `catalog.json`,
`index/` or `artifacts/` ([Submit an app to the App Hub](SUBMITTING.md)).

## After publication

- **Versions.** An installed app runs the installed version, with that
  version's grants. A newer version is an update the person may take; until
  they do, the installed version keeps opening.
- **Integrity.** The host keeps the installed bundle outside the app's
  storage, so the app cannot write to it. Each launch checks it against the
  catalog: the manifest, the digest and the publisher signature. A bundle that
  no longer matches is refused until the person reinstalls the app.
- **Withdrawal.** A withdrawn version stops running on each device's next
  catalog fetch, and other versions keep working
  ([After you submit](SUBMITTING.md#9-after-you-submit)).
