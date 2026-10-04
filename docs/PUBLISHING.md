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

A bundle is a directory of text and artwork that OctoSense runs in its own
sandboxed isolate, under the policy its manifest resolves to. It contains no
native code. An app that needs new native runtime code must be integrated
into a shell release; see the
[delivery paths](DEVELOPMENT.md#choose-the-appropriate-delivery-path).

The entry file decides the kind (`crates/app-contract/src/entry.rs`): a bundle
with `main.splash` at its root is a **script app**; otherwise it is a
**card app** and runs `page.card`.

A **card app** is an L0 card the host lowers to widgets: presentation, no logic.

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
[image-to-card flow](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/flows/image-to-card/FLOW.md)
in OctoScript-App-Design-Flow. Do not hand-write L0 unless asked; the language
is specified in [L0](https://github.com/OctoSense-org/OctoSense/blob/main/apps/appcard/a2app-l0/framework/l0.md).

A **script app** is a Splash program with its own state, handlers, storage and
requests, evaluated as it is.

```
my-app/
  manifest.json      what the app is and what it may do   (required)
  listing.json       what the store shows about it         (required)
  main.splash        the program                            (required)
  assets/            icon and other local artwork           (icon required)
  screenshots/       at least one PNG the listing names     (required)
```

Either kind may also ship **its own agent** (ADR 0002): a tool manifest, the
agent's instructions and skills, next to `manifest.json`. See
[The app's agent and tools](#the-apps-agent-and-tools).

```
my-app/
  tools.json                     the app's tools, typed, with risk levels   (optional)
  AGENT.md                       the agent's instructions                   (optional)
  skills/<name>/SKILL.md         an octos skill, data only                  (optional)
  skills/<name>/manifest.json    its manifest                               (with SKILL.md)
```

Write it with the
[script-app flow](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/flows/script-app/FLOW.md)
and the [script API](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/SCRIPT-API.md),
starting from its [`templates/script-app/`](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/tree/main/templates/script-app).
The program names its own artwork through the `{{assets}}` placeholder
(`http_resource("{{assets}}/assets/logo.png")`): the host replaces it with a
loopback origin that serves this bundle and nothing else, and adds only that
origin to the app's hosts. The first-party
[system apps](https://github.com/OctoSense-org/OctoSense/tree/main/apps)
(`apps/<name>/bundle/`) are complete examples; their `os.` ids are reserved,
so a store copy needs an id of its own.

Keep developer instructions, source tools, keys, test data directories and
review packets outside the submitted bundle.

## Rules the gate enforces

| Rule | What is refused |
| --- | --- |
| Assets are local | Any `http://`, `https://`, `file://` or `../` in a `.card`, `.l0`, `.octoscript`, `.json`, `.txt` or `.md` file (`manifest.json`, `listing.json` and the agent files excepted). Ship the asset in the bundle and reference it by a bundle-relative path such as `assets/icon.svg`. |
| Script apps and agent files reach only declared hosts | In a `.splash` file, `tools.json`, the agent's instructions or anything under `skills/`: any `http://`, `file://` or `../`; and any `https://` address whose host is not in `network.hosts`, unless the app requests `images` or `web`, which allow any public https host. Name bundle artwork through `{{assets}}`. |
| Allowed file types only | Anything other than `.card .json .l0 .octoscript .splash .svg .png .jpg .jpeg .webp .ttf .otf .txt .md`. No other scripts, archives or binaries. |
| No secrets | A `.card`, `.l0`, `.octoscript` or `.splash` file declaring `is_password: true` or a `TextInputContentType` of `Password`, `NewPassword` or `OneTimeCode`. Apps never collect secrets; see [host services](#host-services-and-sheets). |
| System ids are reserved | An id starting with `os.`. Those belong to system apps that ship with the device, and no device installs one from a store. |
| Native and host names are reserved | An id, or an id's last segment (its tool namespace), that is a native app's id or a name the host acts under: `agents`, `apphub`, `appcard`, `card`, `dev`, `octos`, `os`, `reference`, `rinx`, `sheets`, `shell`, `system`, `terminal`, `toolbox`, `workflow`. The device keys an app's storage, tools and consent by its id, so `terminal` or `com.example.terminal` would stand in for the Terminal. |
| Size | A bundle over 8 MB. |
| No symlinks | Any symlink in the bundle. |
| Within the structural limits | More than 2048 files and directories, nesting deeper than 32, a manifest over 64 KiB, or a path that is not portable (a `\`, `:`, `.` or `..` segment, or not UTF-8). |
| An entry the runtime can start | Neither `main.splash` (a script app) nor `page.card` (a card). A card's source must be valid L0, its `page.data.json` valid JSON, and its kit complete: the `kit/native/<theme>/kit.json` its theme names, or the Octoscript kit modules it needs. |
| Well-formed contents (`contents-invalid`) | A text file (`.splash .card .json .l0 .octoscript .txt .md`) over 1 MiB or not UTF-8, or JSON that does not parse; a PNG, JPEG or WebP that does not decode, or is wider or taller than 4096 pixels; an SVG that does not parse, has no size, holds a script or foreign content, or whose styling (a `style` attribute or a `<style>` block) imports a stylesheet, uses escapes or comments, or points `url()` anywhere but `#a-fragment` in the same file. |
| Listing icon | An icon that is not square, or a PNG icon over 1 MiB or 1024 pixels a side. |
| Local resources (`resource-invalid`) | A card's image or font reference, or an SVG `href`, to a file not in the bundle. Each finding names the property (`page.data.json/$kit/placements/…/src`). |
| Digest matches | A manifest whose `integrity.bundle_blake3` does not match the directory. Run `hub stamp` after any change. |
| Manifest is exact | Unknown fields, an unknown capability, a `schema` other than 1, an id outside `[a-z0-9.-]{1,64}` not starting with `.`. |
| Hosts are bare | A host with a scheme, path, port, wildcard or credentials. `api.example.com` is right; `https://api.example.com/v1` and `*.example.com` are refused. |
| Hosts need `net` | Listing hosts without requesting the `net` capability. |
| Research has a scope | `research` or `crawl` without a top-level `research` scope; a scope without either capability; a scope that octos's `Scope::from_grant` would refuse (a language that is not a BCP-47 tag, a region that is not two letters, a category outside `news`, `general`, `science`, `it`, `social`, `max_results` of 0, an unknown field); a domain pattern that is not a bare domain; the toolbox's old field names (`languages`, `allowed_domains`, `denied_domains`, `recency_hours`). |
| Crawl limits need `crawl` | `crawl` without `max_depth` and `max_pages` above 0, or crawl limits without `crawl`. |
| Version is new | Re-publishing a version already in the catalog. |
| Listing present and complete | No `listing.json`; no icon or no screenshot; an unknown category, platform or age rating; a non-https privacy policy; or an icon or screenshot the listing names that is not in the bundle. |
| Publisher continuity | An update, or a new app from a publisher already on record, not signed by the key the catalog records for that publisher; a `--publisher-key` that disagrees with the recorded key; catalog history that disagrees with itself. |
| Tools are the app's own | In `tools.json`: a tool outside the app's namespace (the last segment of its id); a namespace that is not `[a-z0-9_]{1,24}`; a duplicate name, or two names the broker would spell the same; a missing `risk` or `implemented_by`; an unknown field; a schema outside the supported subset, over 8 KB or nested deeper than 8; an input that is not an object; more than 64 tools or a file over 64 KB; `confirm: "app"` on a tool the host service implements. |
| Local-only data stays local | A `shareable` tool of an app whose `agent.model.local_only` is true, unless it declares `"private_data": false`. |
| Agent files are declared text | An `AGENT.md` that `agent.instructions` does not name, or a `skills/<name>/` that `agent.skills` does not; instructions over 32 KB, not UTF-8, holding control characters, a leading `#!`, `<script`, `<iframe>`, `javascript:` or similar; agent files without an `agent`. |
| Skills are data only | A skill manifest declaring `tools`, `binaries`, `sha256`, `mcp_servers`, `hooks` or other executable fields; a file in a skill other than `.md`, `.json`, `.txt`; a manifest `name` other than its directory; a `uses` entry that is neither one of the app's tools nor in `agent.tools`. |
| Background needs triggers | `agent.background: true` without a schedule or an event; a schedule that is not five cron fields; an event outside the app's namespace; an unknown model need or tier. |

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

**Capabilities** (closed list, `KNOWN_CAPABILITIES` in
`crates/app-contract/src/manifest.rs`, the app contract). Anything else is
refused.

| Capability | Grants | The store says |
| --- | --- | --- |
| `storage` | The app's own storage jail: `fs.*`, camera captures, and the local files a widget reads (a map archive). Without it the app has no storage: every `fs` call errors, and `hub check` warns. | Keep its own data on this device |
| `net` | Requests to the hosts in `network.hosts`, and no others. | Reach only: *hosts* |
| `prompt` | Ask the person questions of its own. A service's sheet (Mail's sign-in) does not need it. | Ask you questions |
| `ledger.read` | Read the shared ledger; writing is always the app's own rows. | Read your shared data |
| `location` | The device's location. | Use your location |
| `camera` | The camera. A capture is saved in the app's storage, so it needs `storage` too; without it the preview shows and a capture saves nothing. | Use the camera |
| `clipboard` | The clipboard. | Use the clipboard |
| `images` | Show pictures from any public https host, not only `network.hosts` (a feed's thumbnails). | Show pictures from any website |
| `web` | Open any public https page in the system web view, which has no way back into the app. | Open web pages in a browser view |
| `microphone` | Record sound with a camera video. | Use the microphone |
| `library` | Offer captures to the system photo library, where other apps can see them; without it, captures stay in the app's storage. | Save to your photo library, where other apps can see it |
| `mail` | Read and send mail through the host's mail service, from accounts the person signs in to on the host's sheet. | Read and send mail from accounts you sign in to on the device |
| `llm` | See and arrange the assistant's LLM providers through the host's `llm` service; keys are typed, shown and scanned only on the host's sheets. The service answers only `os.` system apps (AI providers), so a store app gains nothing from it. | Manage the assistant's AI providers, whose keys stay with the device |
| `news` | Read the host's `news` service: items the device collects on a schedule from its feeds and topic feeds, and their text. The app does not fetch arbitrary sites through it. | Read news the device collects from its feeds and topics |
| `glance` | Publish cards to the glance screen through the host's `glance` service (`glance.publish`, `glance.withdraw`, `glance.list`). A card is an L0 card the host checks and lowers before storing it; the host caps its size, rate-limits publishing, keeps a few cards per app and expires them. The publisher is always the calling app: it sees, replaces and withdraws only its own cards, and a card opens only that app. | Show cards on your glance screen |
| `research` | Search through the system toolbox (`search`, `deep_research`) within the manifest's [`research` scope](#the-research-scope): languages, regions, domains, recency, categories and results per search. The host runs each search and refuses or narrows a call outside the scope; the app never fetches the sites itself, so it needs no `net` for it. | Search *what the scope allows*, for example "Search news in English and Chinese, from the last 7 days" |
| `crawl` | Crawl a site through the system toolbox (`deep_crawl`): follow links up to the scope's `max_depth` and read up to its `max_pages` a crawl, inside its domain lists. **More reach than `research`**, which reads only search results: ask for it only when a screen needs whole sites, and prefer a `domains_allow` list. Neither capability implies the other. | Crawl websites, following links up to *depth* deep and reading up to *pages* pages a crawl, *on which sites*, which reaches more than searching |
| `model` | Make bounded one-shot model calls through the host's `model` service (`model.complete`). The app names a model class (`fast` or `strong`) and a JSON Schema; the host picks the model from the person's own AI providers, sends the app's inputs there, checks the reply against the schema (URLs refused unless the app asks for them) and keeps a per-app daily rate and token budget. No tools, memory or history; the app never sees the provider, model id or key. Not `llm`, which only manages providers. | Send what you give it to the AI provider you configured, within a daily budget |

Location, camera and clipboard are each a separate consent; none implies
another.

### The research scope

`research` and `crawl` share one scope, the manifest's top-level `research`
object. Its schema is exactly octos's `octos_research::toolbox::Scope`
(octos `crates/octos-research/src/toolbox.rs`), the single source of truth for
an app's research permission: the gate checks it with the same rules as
`Scope::from_grant`, pins it, and the host hands the same JSON to the toolbox
(OctoSense `crates/toolbox/src/scope.rs` parses it). App Hub mirrors the
struct in `crates/app-contract/src/research.rs` because the app contract links
no octos code; the two change together.

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
| `langs` | BCP-47 languages the app may search in | each a language tag (`en`, `zh-CN`, `zh-Hant`); normalised |
| `regions` | ISO 3166-1 alpha-2 regions | two letters; upper-cased |
| `domains_allow` | only these domains (and their subdomains) | a bare domain: `example.com`, `.example.com` or `*.example.com`; no scheme, path or port |
| `domains_deny` | never these domains | as above |
| `max_age_days` | the oldest material, in days back from now | a whole number of days |
| `categories` | metasearch categories | `news`, `general`, `science`, `it`, `social` |
| `max_results` | most results per search | above 0; default 20 |
| `max_depth`, `max_pages` | the `crawl` limits: link depth and pages of one crawl | both above 0 with `crawl`; both 0 (or absent) without it |

An empty list or an absent field means no limit, and `{}` is a scope with no
limits (the store then says "Search the web in any language, from any time").
Unknown fields are refused. The bare-domain rule is App Hub's and is stricter
than octos, which compares patterns as strings: `https://example.com/` would
match no site, so in `domains_deny` it would deny nothing the person was told
it denies. A scope in the toolbox's old shape (`languages`,
`allowed_domains`, `denied_domains`, `recency_hours`, and `max_pages` as
articles per run) is refused with the fields to rename; it is not converted,
because rounding hours up to days would widen the grant.

The store shows the scope in plain words, derived from the manifest:

| Scope | The store's privacy summary says |
| --- | --- |
| `research`, `{"langs":["en","zh"],"categories":["news"],"max_age_days":7}` | Searches news in English and Chinese, from the last 7 days |
| `research`, `{}` | Searches the web in any language, from any time |
| `research`, `{"langs":["zh-TW"],"regions":["TW"],"categories":["news","it"],"max_age_days":1,"domains_allow":["cna.com.tw"]}` | Searches news and technology in Chinese (TW), region TW, from the last day, only on cna.com.tw |
| `crawl`, `{"max_depth":2,"max_pages":50}` | Crawls websites, following links up to 2 deep and reading up to 50 pages a crawl, on any site: this reaches more of the web than searching |
| `crawl`, `{"domains_allow":["docs.rs"],"max_depth":1,"max_pages":10}` | Crawls websites, following links up to 1 deep and reading up to 10 pages a crawl, only on docs.rs: this reaches more of the web than searching |

The person or the store may grant a narrower scope than the one requested;
never a wider one.

**Where these are served (2026-09-30, OctoSense `main` `7082ff5`, which pins
App Hub `0f332112`):** the OctoSense shells serve `mail` and `model` to any
app granted them (`model` since [OctoSense#95](https://github.com/OctoSense-org/OctoSense/pull/95), within the host's
per-app budget), `glance` to any contained app granted it
([OctoSense#86](https://github.com/OctoSense-org/OctoSense/pull/86)), and `llm` and `news` only to `os.` system apps.
`card-host` serves none of them: a call there answers `no service answers
"<family>" on this device`. No shell serves `research` or `crawl` to an app's
agent yet: OctoSense links its toolbox (`crates/toolbox`) only with the
`toolbox-peers` feature, which the shipped shells leave off
([OctoSense#64](https://github.com/OctoSense-org/OctoSense/issues/64)).
OctoSense pins this App Hub itself. Rinx, pinned by tagged release, no longer
depends on App Hub (since v1.1.0): App Hub, the shells and Rinx all take the
app contract from crates.io by version (`octosense-app-contract = "1"`,
OctoSense [ADR 0005](https://github.com/OctoSense-org/OctoSense/blob/main/docs/adr/0005-app-contract.md)),
so a build links one copy of it and needs no `[patch]` to line them up.

**Host services by exact name** (`crates/app-policy/src/services.rs`). A host
that offers the Matrix account (Rinx) or the device's assistant (Octos) serves
these to a bundle that requests them. Each name is its own consent, checked
exactly: a prefix such as `octos.` or `matrix.`, or any name not listed here,
is an unknown capability and the manifest is refused. Admission is not
dispatch: the host also intersects the request with the services it supports,
its policy and the person's per-instance grant (for Matrix, the rooms they
allow), and checks that lease on every call. A host that does not offer a
requested service shows the app as unavailable with the reason.

**Where these are served (2026-09-30):** the OctoSense shells serve `octos.*`
to contained apps where the shell hosts a kernel (not iOS). The first call
waits for the person to allow the app's agent (`Waiting for the person to
allow this app's agent (OctoSense asks the first time)`); after that the app
talks to its own peer, `card.<app id>` ([OctoSense#106](https://github.com/OctoSense-org/OctoSense/pull/106),
[#120](https://github.com/OctoSense-org/OctoSense/pull/120), [#184](https://github.com/OctoSense-org/OctoSense/pull/184); `OCTOSENSE_CONTAINED_APPS=1` asks nobody,
`0` turns it off). Rinx's mini-app host serves `octos.*` and `matrix.*` to
bundles a person imports into Rinx; no OctoSense shell serves `matrix.*` to a
contained app. `card-host` serves neither: every call answers `no service
answers "octos" on this device` (the dispatch in
`crates/appstore/src/services.rs`). See OctoSense
[architecture § Agents](https://github.com/OctoSense-org/OctoSense/blob/main/docs/architecture.md#2-agents).

| Capability | Grants | The store says |
| --- | --- | --- |
| `octos.session.open` | Open the app's own conversation with the host's assistant. The host binds it to this app and the current account; the app never names a profile, provider or workspace. | Open its own conversation with the assistant |
| `octos.session.history` | Read that conversation's history. Does not allow starting a turn. | Read its own conversations with the assistant |
| `octos.turn.start` | Send a request the assistant works on, under the host's AI settings and tool limits. The model provider and its keys stay with the host. | Ask the assistant to work for it, using the device's AI settings |
| `octos.turn.interrupt` | Stop a turn this app started. | Stop assistant work it started |
| `matrix.*` (45 names) | One Matrix operation each, on the person's current account, in the rooms they allow: reads such as `matrix.read_messages`, `matrix.room_members`, `matrix.profile`; actions such as `matrix.send_message`, `matrix.react`, `matrix.join`. The exact list is `KNOWN_CAPABILITIES`. | One plain line per name, for example "Read messages in rooms you allow" |

Sending room data to the assistant needs both the Matrix read grant and the
assistant grant. No `octos.*` service chooses a model provider, submits a key
or reaches the kernel's raw protocol.


**Network**: `net` plus an exact host list. The list is enforced on every path
out of the isolate: the network module, artwork loading and data fetches. An
empty list with `net` reaches nothing. Hosts match exactly: listing
`example.com` does not allow `api.example.com`.

**Id**: `[a-z0-9.-]{1,64}`, not starting with `.`, never containing `..`.
Ids under `os.` are reserved for system apps. Neither the id nor its last
segment may be a native app's id or one of the host's own names (the list is
in the gate table above, and in app-policy's `RESERVED_NAMES`).

**Storage** (the same block as OctoSense's `native-apps.json`, ADR 0004 §11):
`max_bytes` (the jail's ceiling), `accounts` (`true`: data and one agent per
account; default one `device` folder), `agent_workspace` (`account`, the
default: the agent reads the account's folder; `none`: tools only) and
`cache_max_bytes` (a positive ceiling for `cache/`). `external` (a path
outside the jail) is for reviewed native apps only and is refused here, like
any unknown field.

**Quotas** are requests; the host clamps them to its ceilings (storage 16 MB,
20 000 000 instructions, 64 MB heap). Ask for less than the ceiling when you can.

**Agent** is optional; omit it and the app gets no assistant. `profile` is one
of `read-only`, `workspace-write`, `workspace-write-never-ask`. Full access does
not exist in this schema; do not add it. `tools` may name only what the host
offers contained apps: `ledger.read`, `ledger.write`, `net.fetch`,
`storage.read`, `storage.write`, `card.render`, and one octos kernel tool,
`ask_user_question` (the agent asks the person a question on the device's own
surfaces; it has no side effect of its own; the store says "Ask you
questions"). No other kernel tool (shell, files, the web, memory, peers) may
be named, whatever a host offers (`KERNEL_TOOLS` in
`crates/app-policy/src/policy.rs`). Iterations clamp to 8, tokens
to 200 000. The agent's workspace is the app's own storage jail and its hosts are
the app's hosts; it cannot be given more than the app. The agent's own
tools, instructions, skills, model requirements and triggers are below.

## The app's agent and tools

> **Status (2026-09-30):** the gate admits, checks and pins everything in this
> section, and the store shows its lines. The OctoSense shells (`main`
> `7082ff5`, which pins App Hub `0f332112`) give an app that declares an
> `agent` or `octos.*`, or ships `tools.json`, its own peer once the person
> allows it: they load the bundle with `AgentBundle::load`, register its
> `tools.json` tools with the peer (the kernel side is
> [octos#2567](https://github.com/octos-org/octos/pull/2567), merged), keep
> the generic `agent.tools` names this gate admits (`ask_user_question`), run
> `implemented_by: "host-service"` tools on the app's own host services, and
> let the person talk to the agent in the shell's "Ask <app>" panel
> ([OctoSense#145](https://github.com/OctoSense-org/OctoSense/pull/145), [#184](https://github.com/OctoSense-org/OctoSense/pull/184)). Not yet: `AGENT.md` and
> `skills/` are not installed into the peer, `model` and `tier` choose no
> model, triggers and `background` do not fire, and `implemented_by: "app"`
> tools are refused (OctoSense
> [ADR 0002](https://github.com/OctoSense-org/OctoSense/blob/main/docs/adr/0002-event-driven-app-agents.md)
> is still Proposed, amended by ADR 0004). A field newer than the shells' App
> Hub pin is refused there until the pin moves (the "host older than them"
> below).

An app that wants an assistant of its own (ADR 0002 §3, §4) ships it in the
bundle. Every file is under the bundle digest, so the agent that runs is the
one that was reviewed; a shell loads it with `AgentBundle::load`
(`crates/app-policy/src/agent.rs`), which refuses a bundle whose digest does
not match and everything the gate refuses.

`tools.json` is the one tool manifest for every app. A native module ships the
same file as a module resource, pinned by the shell build. The app-peers broker
(OctoSense `crates/app-peers`) loads it with
`ToolManifest::load(json, <module id>, ToolHost::Native, local_only)`, which
runs the same checks, and builds its `ToolDef`s from it. The module's Rust code
only implements executors keyed by tool name. For a native module the
namespace is the module id; for a contained app it is the last segment of the
app id. The risk levels are the broker's, so one approval gate serves both.

The News example, complete, is
[`crates/app-policy/tests/fixtures/news-agent`](../crates/app-policy/tests/fixtures/news-agent).

### `tools.json`: the app's tools

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

| Field | Meaning |
| --- | --- |
| `name` | `<namespace>.<tool>`, the namespace being the last segment of the app's id (`os.news` and `dev.example.news` are both `news`). Segments are `[a-z0-9_]`; the broker sees the rest with dots as underscores (`topics_get`, at most 32 characters). |
| `description` | What it does and when to use it, for a model; at most 1024 characters. |
| `input_schema`, `output_schema` | JSON Schema, this subset only: `type title description properties required items enum const default minimum maximum minLength maxLength minItems maxItems additionalProperties format pattern`. No `$ref`, no `anyOf`/`oneOf`/`allOf`, no conditionals. The input is an object. |
| `risk` | `read` (looks), `act` (changes the app's own state) or `destructive` (sends, posts, shares, buys, deletes: anything past the app). Required. The broker's `Read`/`Act`/`Destructive` spelling is accepted. |
| `background` | May run in a run the person did not start. Default false. |
| `shareable` | Other callers (the system agent, other apps' agents, the person's assistant) may be granted it. Default false. |
| `private_data` | The result carries the person's private data. A shareable tool of a `local_only` app must say `false`. |
| `implemented_by` | `host-service` (the app's native host service, which holds data, devices, network or secrets) or `app` (the app's own script, for tools that only reshape its data). |
| `outward` | The call reaches outside the device (sends, posts, shares). It then waits for the person like a destructive one. Default false; refused on a `read` tool. |
| `auto_approvable` | A standing rule ("allow for an hour") may approve a call. Say `false` for permanent deletion, payments, sharing outside the device, account and security changes: each call then needs the person live. Default true. |
| `confirm` | Who asks the person before a destructive or outward call: `host` (the default: the host's approval path) or `app` (the app's own confirmation sheet). Independent of `risk`, and never inferred from it. `app` is allowed only for a tool the app implements itself (`implemented_by: "app"`) or a native module's tool. |

**Whether a call needs the person comes from the risk and `outward`; whose
surface asks comes from `confirm`.** Read and act run unattended. A destructive
tool, and an outward one, always waits for the person:

| `risk: "destructive"` (or `outward: true`) with | Person present | Person absent |
| --- | --- | --- |
| `confirm: "host"` (default) | The host's approval path asks. | An approval request in the app's conversation. |
| `confirm: "app"` | The app's own confirmation sheet is the only confirmation (for example Rinx's `send_message`); the host does not ask again. | An approval request in the app's conversation. |

The person is never asked twice for one call. A destructive tool may still say
`background: true`: the gate records a warning, and the tool only runs after
approval. `confirm: "app"` on a tool that is neither destructive nor outward confirms nothing,
and the gate warns about it. The gate report, the review packet and the store
lines all state whose confirmation each destructive tool uses.

An app may ship `tools.json` without an agent: its tools then serve other
callers, such as the person's assistant, but no agent of its own.

### The manifest's `agent`

```json
"agent": {
  "profile": "workspace-write-never-ask",
  "tools": [],
  "max_iterations": 8,
  "token_budget": 120000,
  "model": {
    "needs": ["tool_calling", "long_context", "multilingual"],
    "tier": "standard",
    "local_only": false,
    "per_task": {
      "triage": { "needs": ["tool_calling"], "tier": "fast" },
      "synthesis": { "needs": ["tool_calling", "reasoning", "long_context"], "tier": "strong" }
    }
  },
  "background": true,
  "triggers": { "schedule": ["0 7 * * *", "0 19 * * *"], "events": ["news.items.new"] },
  "instructions": "AGENT.md",
  "skills": ["news-digest"]
}
```

- `tools` stays the generic host tools (above). The app's own tools come from
  `tools.json`; the agent gets both and nothing else.
- `model` states needs, never a provider or model name: `needs` from
  `tool_calling vision long_context reasoning structured_output multilingual`,
  `tier` one of `fast standard strong` (default `standard`), `local_only` for
  data that must not leave the person's devices (app-wide; a task cannot relax
  it), and `per_task` for named tasks (`[a-z_]{1,32}`, at most 8) that
  `AGENT.md` refers to. The host picks a model from the person's providers.
- `background` asks to run while the app is closed. It is a request: the
  person grants it per app, and it requires `triggers`.
- `triggers.schedule` is five-field cron in local time; `triggers.events` are
  the app's own host-service events, in its namespace (`news.items.new`).
- `instructions` names the agent's instructions (`AGENT.md`); `skills` names
  each `skills/<name>/` directory. Undeclared agent files are refused.

These fields are optional additions to schema 1. A manifest without them
reads, and signs, exactly as before; a host older than them refuses a manifest
that uses them (unknown fields are refused), which is the safe direction.

### `AGENT.md` and skills

`AGENT.md` is the agent's role and instructions: what to do on each trigger,
what matters in the app's data, the rubric for its output, and its rules for
memory. Text only (32 KB, UTF-8, no HTML scripts or `#!`); the system agent may
add a local overlay but never edits it.

A skill is an octos skill directory with `SKILL.md` and `manifest.json`,
installed into this app's peer workspace only. For a contained app it is data
only: its manifest holds `name` (its directory), `version`, `description`,
`uses` (the tools it calls, each one of the app's tools or in `agent.tools`)
and optionally `prompts.include`; `.md`, `.json` and `.txt` files only.

```json
{
  "name": "news-digest",
  "version": "1.0.0",
  "description": "Write a cited morning or evening digest from collected stories.",
  "uses": ["news.list", "news.read", "news.digest.write"]
}
```

### What the store shows

Derived from the manifest and `tools.json`, beside the other permissions:

- "Its assistant may work while the app is closed, on a schedule and when new
  data arrives; only if you allow it, and you can turn it off."
- "Can ask to mail.send: nothing of this runs until you approve it." (host
  confirmation)
- "Asks you on its own screen before rinx.send_message; when you are away, it
  waits for your approval in the app's conversation." (`confirm: "app"`)
- "You approve every call of pay.transfer yourself: no standing rule can."
  (`auto_approvable: false`)
- "Offers news.list to other assistants you allow." (and, for a shareable tool
  with `private_data: true`, that it can pass private data)
- "Its assistant uses only models that run on your own devices."

The catalog entry carries the reviewed `tools.json` so a store can show these
before install; the review packet carries the agent files with a question on
them.

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
bundle runs wherever the OctoSense shell does); `age_rating` is one of
`all 12+ 16+ 18+`; `privacy_policy_url` is an https URL; every screenshot
and the icon is a PNG or SVG inside the bundle; at most 10 keywords and 8
screenshots; unknown fields are refused. An icon and at least one screenshot
are required: the icon is what the launcher shows once the app is installed,
and a screenshot is the one claim a reviewer can check against the running app.
Use one app-owned canonical icon across store and launcher surfaces; see
[ICONS.md](ICONS.md) for export limits, native rendering and small-size review.
The two-field icon declaration used by a built-in native app is not a complete
publishable listing.

To produce a screenshot, run an unsigned development bundle in the reference
host, `MAKEPAD_REMOTE=8151 card-host --bundle my-app --allow-unsigned &`, then
`curl 127.0.0.1:8151/g` (capture metadata, with the PNG's path) or
`curl -o 01-main.png '127.0.0.1:8151/g?raw=1'` (the PNG bytes), and
`curl 127.0.0.1:8151/quit`. Capture the actual app content at the host's
current dimensions; do not assume a fixed crop. `card-host` verifies no
publisher keys, so use an unsigned development copy before final signing. See
the [card-host reference](DEVELOPMENT.md#running-a-bundle-locally-card-host),
the [first-app walkthrough](FIRST-APP.md#4-run-the-unsigned-development-bundle-and-capture-it)
and the [native testing guide](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/flows/core/NATIVE-INSTRUMENT.md).

The store also shows a **privacy summary derived from the manifest**: what
the app stores, which hosts it contacts, which device features it uses,
whether it runs an assistant. Do not restate it in the description; make
the manifest right instead.

## Host services and sheets

A script app does not hold sockets, credentials or devices it does not need.
For work that needs them it calls a **host service**
(`crates/appstore/src/services.rs`):

```
host.request("mail.list", {…}, fn(r){ … })
```

The isolate refuses the call unless the app's policy grants the family
(`mail` for `mail.*`). A granted call goes to the Rust service registered for
that family, which does the work and answers the app with data, never with
the means. A family no service answers fails at once with
`no service answers "<family>" on this device`; the reference `card-host`
registers none.

**Apps never collect secrets.** Input only the person should give (a
password, an account approval) is collected on a **sheet**: a host-owned
surface the runner draws over the app, in an isolate of its own under no
app's policy. Only a service can open one. Service methods that take a secret
live under `<family>.sheet.` and are accepted only from that sheet, before any
service sees the call. An app's own password fields take no input at runtime,
and the gate refuses a bundle that declares one. Service state (accounts,
secrets, caches) lives in `<app data>/.host`, outside every app's jail.

Mail is the worked example: the
[Mail bundle](https://github.com/OctoSense-org/OctoSense/tree/main/apps/mail/bundle)
requests `mail`, and its
[host service](https://github.com/OctoSense-org/OctoSense/tree/main/apps/mail/host-service)
signs in on its own sheet. A store app can request `mail` only where the
shell links a mail service.

**Every request settles.** A call answers exactly once, with the service's
data or an error the app can act on:

- **Timeouts.** A call that waits longer than its service allows (60 seconds
  unless the service asks for more) answers `the host service timed out`. The
  clock stops while the service's sheet is up, because the person is still
  typing, and it starts again when the sheet closes.
- **Limits.** At most 32 calls from one app wait at a time. Further calls
  answer `too many host requests are waiting` until some have answered.
- **Sizes.** Arguments over 1 MiB are refused, and so is arguments text that
  is not valid JSON. An answer over 4 MiB becomes an error.
- **Closing.** When the app closes, its waiting calls end with it, and no late
  answer reaches the next copy of the app.

**The surface decides whether a sheet may appear.** An app in the foreground
may have a service raise a sheet over it. The same app shown as a home-screen
tile, or a call an assistant makes as a tool, may not: the service is told
(`may_prompt` is false), and a sheet it raises anyway is refused with
`this surface cannot raise a prompt; open the app to continue`. This does not
depend on the `prompt` capability, which is about the app's own questions.

## Commands

The `hub` command is the same binary the hub itself runs, so the report you see
locally is the report the hub acts on. Build it from this repository with
`cargo build --release -p octosense-app-hub --bin hub`.

| Command | What it does |
| --- | --- |
| `hub stamp <bundle>` | Write the bundle's digest into `manifest.json`. Rerun after every change. |
| `hub check <bundle> [--allow-unsigned] [--publisher-key <id>=<hex>] [--catalog <file> [--anchor <hex>]] [--json]` | The gate. Prints PASSED or REFUSED, each finding (with the file or property at fault), and what the app will be granted; exits non-zero on a refusal. `--json` prints the same report as JSON (`schema`, `passed`, `findings`, `resources`) for tools. |
| `hub scan <bundle> [--packet <out.json>] [--reviewer <cmd>]` | Stage two: the review packet (manifest, listing, grants, the entry's source, and the reviewer's questions), optionally handed to a reviewer command. Runs only on a bundle the gate passes. |
| `hub keygen <key file>` | Make a signing key; prints its public half. |
| `hub pubkey <key file>` | Print a key's public half. |
| `hub sign-manifest <bundle> --key <key file> --key-id <publisher id>` | Sign the manifest, which covers the digest. |

`hub check` prints what the app will be granted. Read it back against the
manifest: if the grants are wider than the app visibly needs, reduce the
manifest. Use `--catalog <catalog.json>` to check version and publisher
continuity against a published catalog (this repository's `catalog.json`).
The catalog counts as history only once it verifies against the hub's anchor
(the one in the [README](../README.md#trust-anchor) unless `--anchor <hex>`
names another, for a development hub).
Keep `review.json` outside the bundle; it is a review artifact, not app
content.

`hub scan` writes the questions a reviewer answers: does the app do what its
name claims, do its grants (and, for a script app, the hosts it requests) match
what it visibly does, is anything deceptive, does any text address an
assistant rather than a person. Answer them honestly before submitting; the
hub's reviewer asks the same ones.

## The full sequence

From an app repository whose bundle is `bundle/`, with `hub` and `card-host`
built:

```sh
# 1. Digest the bundle as it is.
hub stamp bundle

# 2. Run it and capture a real screenshot (unsigned, see card-host above).
MAKEPAD_REMOTE=8151 card-host --bundle bundle --allow-unsigned --app-data .local-state &
sleep 7
mkdir -p bundle/screenshots
curl -s -o bundle/screenshots/01-main.png '127.0.0.1:8151/g?raw=1'
curl -s 127.0.0.1:8151/quit

# 3. The screenshot changed the bytes: restamp.
hub stamp bundle

# 4. The gate and the review packet, on the final unsigned bytes.
hub check bundle --allow-unsigned
mkdir -p build && hub scan bundle --packet build/review.json

# 5. A publisher key, made once and kept outside the repository.
export APP_PUBLISHER_ID="your-publisher-id"
export APP_SIGNING_KEY="/absolute/private/path/publisher.key"
test -e "$APP_SIGNING_KEY" || hub keygen "$APP_SIGNING_KEY"
APP_PUBLISHER_PUBLIC_KEY="$(hub pubkey "$APP_SIGNING_KEY")"

# 6. Sign, and check the signed bytes the way the hub will.
hub sign-manifest bundle --key "$APP_SIGNING_KEY" --key-id "$APP_PUBLISHER_ID"
hub check bundle --publisher-key "$APP_PUBLISHER_ID=$APP_PUBLISHER_PUBLIC_KEY"

# 7. Commit and tag the signed bundle in your repository, then submit (below).
```

## Signing

Signing is optional for a first submission and required for updates once a
key is on record: the hub refuses an update signed by a different key, or not
signed, once a key is on record for the app.

The key on record is the one in the published catalog, never one a submission
supplies. Once a release signed by your key is published, every later version
of that app, and every new app you publish under the same publisher id, must
be signed by that key: a `--publisher-key` with another key under your id is
refused, and the hub does not need you to supply your key again. Your
publisher id is your signature's key id (`hub publish --publisher` must equal
it). An app first published unsigned gets its key on record with its first
signed update. There is no key-replacement flag: a lost or rotated key, or
history that disagrees with itself, needs the hub's maintainer. Keep your
signing key safe.

The signature covers the manifest, including `integrity.bundle_blake3`, so:

- **Stamp, then sign.** Signing an unstamped manifest signs the wrong digest.
- **Any edit after signing means restamp and re-sign.** Changing any file
  (a screenshot, the listing, one character of `main.splash`) makes
  `hub check` refuse with `digest: the bundle hashes to …, the manifest
  claims …`. Restamping alone then fails with `publisher-signature: the
  signature from key "<id>" does not match the manifest`. Run `hub stamp`,
  then `hub sign-manifest`, then `hub check --publisher-key` again.
- A signed bundle checked without `--publisher-key` is refused, by design.
- A new version needs a new `version` in the manifest; a published version is
  never replaced.

## Submitting

**What exists today.** The hub's published state is this repository,
[OctoSense-org/OctoSense-App-Hub](https://github.com/OctoSense-org/OctoSense-App-Hub):
`catalog.json` (the signed catalog stores read), `artifacts/` (the hub's
copy of each admitted bundle) and `index/` (one entry per admitted version).
The catalog and the artifacts are written by `hub publish`, which re-runs the
gate (and the scan, with a reviewer), copies the bundle and signs a new
catalog with the hub's working key; the maintainer adds the `index/` entry in
the same commit. Publishers do not hold that key.

**Not yet available.** There is no separate index repository and no
`octosense-org/publish-app` GitHub action. Do not add a release workflow that
uses them. When they exist, this section will say so and give the workflow.

**The route maintainers accept now:**

1. Push the signed bundle to your app's public repository and tag the commit
   (for example `v1.0.0`).
2. Open an issue in
   [OctoSense-org/OctoSense-App-Hub](https://github.com/OctoSense-org/OctoSense-App-Hub/issues)
   titled `Submit <app id> <version>`, giving:
   - the repository URL, the tag and the full commit SHA;
   - the bundle's path in that repository (usually `bundle/`);
   - your publisher id and public key (`hub pubkey`), or "unsigned" for an
     unsigned first submission;
   - the complete output of `hub check` on that commit (with
     `--publisher-key`), and your answers to the `hub scan` questions.
3. Do not open a pull request that edits `catalog.json`, `index/` or
   `artifacts/`. A catalog not signed by the hub's key is refused by every
   store, and the admitted bytes must be the ones the gate checked.

A maintainer checks out that commit, runs `hub check` and `hub scan` on the
exact bytes, and, if both pass, runs
`hub publish <bundle> --catalog catalog.json --publisher <id> --publisher-key <id>=<hex> --repo <url> --commit <sha> …`
and commits the result. A first submission, or a scan that asks for human
review, waits for a person. The issue is closed with the catalog sequence the
app appeared in, or with the findings to fix.

## What happens after

- The hub keeps its own copy of the bundle and signs a new catalog. Every
  OctoSense store verifies that catalog against an anchor it ships with, so a
  catalog nobody signed is never shown.
- The app runs in its own isolate with exactly the manifest's grants; a
  request outside them fails with an error, and the person sees why.
- An installed app runs as the release it is: the version installed, with
  that version's grants. A newer version on offer is an update the person
  may take; until they do, the installed version keeps opening.
- The installed bundle is kept outside the app's storage
  (`<app data>/.bundles/<id>/`, beside the app's own `<app data>/<id>/`),
  so an app cannot write to it. Each launch still checks it against the
  catalog (the manifest and the digest) and runs a copy of it; a bundle
  that no longer matches is refused until the app is reinstalled.
- A version can be withdrawn with a reason. Installed copies of that version
  stop running on the device's next catalog fetch; other versions are not
  affected. Publish a fixed version rather than arguing with a withdrawal.

## Do not

- Reference any server, CDN or local path from a card, or an undeclared host
  from a script app. Bundle the asset.
- Request `prompt`, `location`, `camera`, `microphone`, `clipboard`, `library`,
  `images`, `web`, `mail`, `news`, `glance`, `model`, `research` or `crawl` unless a screen needs it (and never `llm`, which
  serves only system apps); each is shown to the
  person as a separate line.
- Ask for a password, PIN or code in the app. A host service asks on its own
  sheet.
- Put instructions to an assistant in card text or data. The scan treats text
  addressed to an assistant as a reason to reject.
- Edit `integrity.bundle_blake3` by hand. Run `hub stamp`.
- Reuse a version number.
