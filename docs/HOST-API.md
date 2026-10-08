# Host API compatibility

English | [简体中文](HOST-API.zh-CN.md)

Contract 1.6 lets an app declare the host APIs it needs and discover the ones
a host implements. It adds declarations and discovery, not access: an app can
call only the Rust services compiled into its host, and every call still needs
the app's grants. The same contract lets a bundle declare its own backend
([Sign in to your own backend](PUBLISHING.md#sign-in-to-your-own-backend)) and
run its own agent tools
([Script tool execution](PUBLISHING.md#script-tool-execution-script-tools-v1)).

Contract 1.6.0 is on crates.io, but no released OctoSense build implements
these APIs yet. Until one does, test in an OctoSense shell built from `main`.
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
  unknown age. `WebReader` opens embedded pages only on macOS, iOS and
  Android; on other platforms, OctoSense answers `open` with `false`.
- **Permission requests.** Only an app in the foreground can request a
  permission. A request from an agent or a background card fails with
  `<method> is unavailable to agents/background surfaces`. A request still
  waiting when the host goes to the background fails with
  `authorization_required`, as does `location.get` until the person grants
  the app location access.
- **Device widgets.** With `host-api-v1`, `CameraPreview`,
  `sys.request_location`, `sys.gps` and map GPS reads need the app's device
  consent too. After the host starts, they stay closed until the app calls a
  permission method, such as `camera.permission.status`, which loads the
  saved consent. Call it when the app opens.
- **`card-host`.** It implements none of the APIs that the three markers need,
  so it refuses apps that require them
  ([Run a bundle locally](DEVELOPMENT.md#run-a-bundle-locally-card-host)).
- **Unverified:** a physical permission approval, camera capture, live
  providers, real models, and host API acceptance on Linux, Windows and phones.

For the calls, see Design Flow's
[Discover and use host APIs](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/HOST-API-V1.md).
