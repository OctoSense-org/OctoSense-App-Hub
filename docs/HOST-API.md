# Host API compatibility

English | [简体中文](HOST-API.zh-CN.md)

Contract 1.6 lets an app declare the host APIs it needs and discover the ones
a host implements. It adds declarations and discovery, not access: an app can
call only the Rust services compiled into its host, and every call still needs
the app's grants. The same contract lets a bundle declare its own backend
([Sign in to your own backend](PUBLISHING.md#sign-in-to-your-own-backend)) and
run its own agent tools
([Script tool execution](PUBLISHING.md#script-tool-execution-script-tools-v1)).

These declarations are included in contract 1.8.0. The
[RC1 release](../README.md#download-a-compatible-host), source `933abbcf`,
implements them within the platform limits below; its download status is
tracked in that guide.
An older host, such as OctoSense desktop 0.1.0-beta.2, serves none of these
APIs and refuses an app that requires them.

## Declare what the app needs

This manifest fragment requires `runtime.list` and uses `location.get` when
the host has it:

```json
{
  "requires": ["host-api-v1"],
  "capabilities": ["runtime", "location"],
  "host_api": {
    "required": {"runtime.list": 1},
    "optional": {"location.get": 1}
  }
}
```

| Field or marker | Meaning |
| --- | --- |
| `host_api.required` | Methods the app cannot run without, each with its exact ABI major version. Version 2 does not satisfy a request for version 1, and a version is not a host release number. |
| `host_api.optional` | Methods the app uses when the host has them. Give each one a fallback. |
| `host-api-v1` | Needed for `host_api`. It also puts camera, microphone and location behind a per-app consent, so the host must implement the `app_policy.device_consent@1` runtime ABI. A runner that only parses these fields does not satisfy it. |
| `backend-api-v1` | Needed for `backend`. The host must implement `auth.backend.request@1`. |
| `script-tools-v1` | Needed for `implemented_by: "app"` tools. The host must implement the `app_tools.dispatch@1` runtime ABI. |

The store checks these requirements when it installs the app and again each
time the app opens. It refuses an app that the host cannot serve, with
`app <id> needs a host implementing <method>@1` or
`this host does not implement required APIs: <method>@<version>`. A catalog
update that needs newer APIs does not take away an installed version that the
host still serves.

## Discover what the host implements

### Native device calendars (unreleased contract 1.10)

The new `device_calendar` capability is separate from `calendar` (the system
Calendar app's own store) and `gcalendar` (Google's connector). It does not
grant either of them, or an OS permission. An app that needs native events
declares `device_calendar` and `host-api-v1`, with
`host_api.required: {"device_calendar.events.list": 1}` and any other methods
its workflow requires. A host without those methods refuses installation.

This source change admits the capability and read tool aliases. It does not
install a native adapter into desktop RC1 or Home beta.1. The adapter must
also enforce app/account scope, OS permission, selected calendars and trusted
review of mutations. Runtime and device acceptance of the new adapter are
separate from these contract tests. The service's method descriptions remain
the authority for supported platforms and agent access.

### Reviewed Mail drafts (unreleased)

With the existing `mail` capability, a compatible host exposes `mail.compose`
for app/account-bound local draft changes and `mail.compose_status` for their
status. Agent aliases require minimum risk `act` and `read`, respectively,
with `private_data: true`. They do not send mail. The foreground app requests
`mail.review_send` (or the reviewed `mail.send` compatibility entry point);
the host shows the exact draft and sends only after physical user approval.
Declare required method versions. Desktop RC1 does not implement this public
compose flow; the older system Mail draft methods remain system-app-only.

### Runtime inventory

With the `runtime` capability, call `runtime.list` with `{}` for every
described method, or `runtime.describe` with `{"method":"location.get"}` for
one:

- A method description gives its schemas, ABI version, capability, platforms
  and whether agents may call it.
- `runtime_features` lists runtime ABIs, such as `app_tools.dispatch@1`.
  `host.request` cannot call a runtime ABI.
- Only methods with a registered description appear. Some older services have
  none, so the list is not a complete inventory.
- The result holds no account data or credentials.

Availability is not configuration or permission. `configured: null` means the
host does not know, and `authorization: "checked-on-call"` means the host
checks the app's grants on every call. To learn whether a service is
configured and authorized, call its status or account methods. Declaring a
requirement neither turns on an OS permission nor adds a provider
registration.

## Selected-file transfer (contract 1.9, unreleased)

The `files` capability grants access to host-owned file dialogs, not arbitrary
host paths. Import/export also need `storage`; neither grant implies the other.
Require `files.import@1` or `files.export@1` through `host_api.required`, or declare
them optional and inspect `runtime.list`. Contract support alone does not mean a
released host implements these APIs. `files.status` reports adapter availability.

The matching OctoSense implementation imports one selected file into a new
app-relative destination, and exports a snapshot of an existing app file. It
returns a relative path and byte count, never an OS path or Android provider URI.
Transfers are foreground-only and cannot be initiated by an agent/background
turn. Native `fs.write_bytes` is identified by the `storage.binary_write@1`
runtime ABI, not a callable `host.request` method.

Imports keep the current 1 MiB file limit and app storage quota; they refuse
existing destinations. A cancelled dialog returns `{"cancelled":true}`. Current
adapters cover macOS, Windows, Android and Linux with a native dialog helper;
iOS, OpenHarmony, web and direct-framebuffer Linux are unsupported. Check the
host's status and release notes for build and device validation.

### Photo selection and text sharing (unreleased host extension)

A matching host adds `files.pick_photo({"path":"photos/chosen.jpg"})`, using
`files` plus `storage`. It opens an image chooser and validates PNG, JPEG or
WebP signatures before importing into a new app-relative path. It returns
`path`, `bytes` and `mime`, or `{"cancelled":true}`. The 1 MiB limit still
applies: larger originals fail explicitly; no image is silently resized.

`files.share({"text":"A short note"})` uses `files` only and is Android-only
in this batch. Text must contain 1–8192 UTF-8 bytes. The foreground call opens
the native chooser and returns
`{"handoff":"chooser_opened","delivery":"unknown"}` only when that handoff
succeeds. It does not prove delivery or support attachments. Neither method
has an agent alias. Declare each method version and check the host's status;
these source APIs are not included in desktop RC1. Device validation of the
new adapters remains pending.

## Foreground audio sessions (contract 1.10, unreleased)

A compatible macOS or Android host exposes these methods; declare the method
versions rather than inferring support from the capability name:

| Methods | Required capabilities | Behavior |
| --- | --- | --- |
| `microphone.record_start({path,max_duration_ms})` | `microphone`, `storage` | After app and OS consent, record at most 30 seconds of mono 16 kHz PCM WAV into a new app file. |
| `microphone.record_status/record_stop/record_cancel({session})` | `microphone` | Inspect, stop and save, or discard this live app's recording. |
| `audio.play({path})` | `audio`, `storage` | Play a bounded local WAV, MP3, FLAC or Ogg file. |
| `audio.status/audio.stop({session})` | `audio` | Inspect or stop this live app's playback. |

Each response contains an opaque `session`, `status`, `path`, `error`, `frames`
and `format`. A start response means `starting`, not successful recording or
playback. Poll status until the first native callback reports `recording` or
`playing`, and observe terminal failures. Stopping a recording initially
returns `stopping`; use its path only after `saved`. Device frame counts do not
prove audibility. File input/output remains within the current 1 MiB limit;
decoded playback also has separate duration/memory limits.

These APIs are foreground-only with no agent aliases. Playback does not grant
microphone access; recording still needs `microphone.permission.request` and
OS approval. Sessions bind to the live app isolate, stop when it loses its
foreground surface or permission, and never resume automatically. Hosts must
arbitrate microphone ownership with dictation. Embedded store previews without
a dedicated active app identity refuse recording; open the installed app.
Source and synthetic tests do not establish real microphone/speaker acceptance.

## Limits

- **Agent tools.** A `host-service` tool can map with `host_method` to
  `auth.backend.me`, `auth.backend.request`, `runtime.list`,
  `runtime.describe`, the three `*.permission.status` methods or
  `location.get`, at minimum risk `read`, with the method's capability and
  `private_data: true`
  ([Map a tool to a shared service](PUBLISHING.md#map-a-tool-to-a-shared-service-host_method)).
  An agent's call never prompts, so `auth.backend.request` runs only declared
  `GET` operations and refuses a write before any HTTP request; a write still
  needs the foreground app and the host's native review. Permission requests
  and revocation, account management and sheet controls have no `host_method`.
  Admission does not replace the host's account, consent and platform checks.
- **Backends.** The host reads the `backend` block only from the admitted,
  signed bundle. It returns opaque connection handles, refuses redirects and
  any backend answer that carries a token, and asks the person to approve
  every write on its native review screen. A background or agent call
  cannot approve a write. Changing the endpoints, updating the app or
  withdrawing it ends the app's backend sessions.
- **Script tools.** The signed `app_tool(name, call_id)` handler runs in the
  open full app's own VM and storage folder. No native library or Wasm loads,
  and a closed app answers `app_not_running`.
- **Platforms.** The device-permission methods cover Android and macOS, and
  `location.get` covers Android only. It returns the last-known fix, of
  unknown age. The RC also embeds ordinary `WebReader` pages on Windows
  (WebView2) and Linux X11/XWayland (WebKitGTK); native Wayland is unsupported.
  Linux/Windows advertise `auth.backend.request@1` for declared reads and
  implement external-browser backend login; live sign-in is not validated here.
  Embedded backend login and protected writes remain unsupported and fail closed.
- **Permission requests.** Only an app in the foreground can request a
  permission. A request from an agent or a background card fails with
  `<method> is unavailable to agents/background surfaces`. A request still
  waiting when the host goes to the background fails with
  `authorization_required`, as does `location.get` until the person grants
  the app location access.
- **Device widgets.** With `host-api-v1`, `CameraPreview`,
  `sys.request_location`, `sys.gps` and map GPS reads need the app's device
  consent too. After the host starts, each stays closed until the app calls
  its capability's permission method, which loads that saved consent:
  `camera.permission.status` before `CameraPreview`, and
  `location.permission.status` before a GPS read. Call these when the app
  opens.
- **`card-host`.** It implements none of the APIs that the three markers need,
  so it refuses apps that require them
  ([Run a bundle locally](DEVELOPMENT.md#run-a-bundle-locally-card-host)).
- **Unverified:** a physical permission approval, camera capture, live
  providers, real models, and host API acceptance on Linux, Windows and phones.

For the calls, see
[Discover and use host APIs](https://github.com/OctoSense-org/OctoSense-App-Flow/blob/main/docs/HOST-API-V1.md)
in OctoSense App Flow (formerly Design Flow).
