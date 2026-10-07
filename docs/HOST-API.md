# Host API compatibility

English | [简体中文](HOST-API.zh-CN.md)

The 1.6 contract adds declarations and discovery; it does not give apps arbitrary
access to Rust functions or the operating system. These changes require a
compatible host build. Contract 1.6.0 is published. Until a compatible host build is released, use the
matching reviewed source revisions; older downloads do not gain new services.

An app may request the `runtime` capability and call `runtime.list` with `{}` or
`runtime.describe` with `{"method":"location.get"}`. The result distinguishes
method descriptors from `runtime_features` such as `app_tools.dispatch@1`.
A runtime ABI is not callable through `host.request`. Metadata includes schemas,
ABI versions, platform support and whether agents may call a method. Only
registered descriptors appear; undescribed legacy services are not an exhaustive
inventory. The result contains no account data or credentials.

Presence is separate from configuration and permission. `configured: null` means
unknown, and `authorization: "checked-on-call"` means every operation still
checks its own grants. Use the particular service's status or account methods.
A disabled OS permission or absent provider registration is not fixed by declaring
a requirement.

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

This is a manifest fragment. Required versions match exactly, not by minimum
host release. Missing optional methods need a usable fallback. `host-api-v1`
also requires the `app_policy.device_consent@1` runtime policy ABI; a plain
Makepad runner cannot claim it merely because it can parse these fields.
`backend-api-v1` requires `auth.backend.request@1`; `script-tools-v1` requires
`app_tools.dispatch@1`. Declare the relevant markers alongside `host-api-v1`
when using the manifest's new API/backend fields. App Hub refuses incompatible
installs and checks the installed release again on launch. A catalog update
that needs newer APIs does not take away an older, still-compatible install.

An optional `backend` block contains public OAuth client metadata and named
same-origin operations. It requires `auth`, account storage, and
`backend-api-v1`. The host resolves it only from the admitted signed bundle,
returns opaque connection handles, refuses redirects and token-bearing results,
and reviews writes in its native UI. Background reads cannot approve writes.
Endpoint or lifecycle changes invalidate backend connections.

Script tools run a signed `app_tool(name, call_id)` handler in the full app's
existing VM and storage jail. They do not load native libraries or Wasm, and a
closed app returns `app_not_running`. See the
[script tool ABI](PUBLISHING.md) and the
[developer guide](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/HOST-API-V1.md).

Platform limits remain explicit: the initial device broker covers Android and
macOS permissions; `location.get` exposes Android's last-known fix with unknown
freshness. Linux/Windows embedded `WebReader` is unsupported. The fonts renderer
fix selects the host-served resource loader for bundled fonts and retains a CJK
fallback; the gate validates both literal and resolved one-level token fonts.
These changes do not constitute Linux/Windows, phone, provider or model acceptance.
