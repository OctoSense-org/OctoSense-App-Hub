# Changelog

`octosense-app-contract` follows the rules in [README.md](README.md#stability):
within `1.x` it only grows.

crates.io has 1.0.0, 1.1.0, 1.2.0, 1.5.0 and 1.6.0; 1.3.0 and 1.4.0 are
unpublished, and 1.5.0 includes their changes.
[README.md](README.md#versions-on-cratesio) shows how to update a lock file
that holds 1.2.0.

## 1.6.0

- Adds signed `host_api` requirements with exact ABI-major versions, implemented
  method descriptions, and the introspection-only `runtime` capability.
- Adds public, app-owned `backend` declarations: one HTTPS origin, fixed public
  client metadata, and a bounded named operation map. They contain no
  credentials.
- Adds the `host-api-v1`, `backend-api-v1` and `script-tools-v1` requirement
  markers. Older hosts reject the markers; newer hosts must also check that
  required methods and runtime policy/dispatch ABIs exist before installation.
- Old manifests preserve their serialized defaults and signature payloads.
  A declaration never grants OS access, authorizes a write or implements a service.

## 1.5.0

- `KNOWN_CAPABILITIES` gains the `auth`, `github`, `gcalendar` and `gmail`
  host-service capabilities. Each remains an independent grant. OAuth returns
  app-bound connection handles; provider tokens stay in the host. These
  declarations require corresponding OctoSense implementations and do not make
  a provider available by themselves.
- `KNOWN_CAPABILITIES` gains `photos` and `youtube`, so the contained Photos and
  YouTube UIs can call their owning host services. Neither grants Android
  Gallery access, a YouTube account or another app's agent tools; cross-app
  tools remain separate grants. When 1.5.0 was cut, OctoSense served neither
  family beyond its notice service, which answers only `photos.notify` and
  `youtube.notify` for its own system apps;
  [OctoSense#346](https://github.com/OctoSense-org/OctoSense/pull/346) tracks
  the services.
- The crate gains `portable_path`: a bundle-relative path written with `/` on
  every platform. `digest_dir` hashes these names, so a bundle with folders
  gets the same digest on Windows, Linux and macOS. Linux and macOS digests do
  not change, because the files keep their path-component order. Before 1.5.0,
  a Windows host hashed `\`, so its digests matched no other host's. A path
  that is not a plain UTF-8 name is refused instead of hashed lossily; the
  gate already refused such bundles.

## 1.4.0

- `KNOWN_CAPABILITIES` gains `calendar`, so a contained Calendar UI can call
  its owning host service. Agent tool sharing remains a separate grant. The
  capability grants no access to Android or Google Calendar accounts; the
  registered host service checks its owning app identity on every call.

## 1.3.0

- `RESERVED_NAMES` gains `octoscode`, reserved for OctosCode
  (octos-org/octoscode-app), a native app OctoSense planned for desktops and
  phones ([OctoSense#341](https://github.com/OctoSense-org/OctoSense/pull/341)).
  A store app with that id, or with it as its namespace
  (`com.example.octoscode`), is now refused as reserved. Nothing else changes:
  every other manifest 1.2.0 admitted is admitted unchanged.

## 1.2.0

Adds the operations of the Hagency/Rinx mini-app that administers Palpo. The
release is additive only: the schema stays at minor 0, and every manifest
1.1.0 admitted is admitted unchanged.

- `KNOWN_CAPABILITIES` gains 29 exact `palpo.*` operations:
  `palpo.intent.new`; `palpo.session.open` and `.disconnect`;
  `palpo.catalog.list`; `palpo.projects.list` and `.create`;
  `palpo.requests.list` and `.create`; `palpo.fleets.list`, `.register`,
  `.install`, `.set_state`, `.migrate`, `.queue`, `.export` and
  `.connect`; `palpo.agents.list`, `.register`, `.rename` and `.retire`;
  `palpo.activity.list`; `palpo.accounts.list`; `palpo.inbox.list`,
  `.submit`, `.get`, `.decide`, `.activate`, `.seen` and `.snooze`.
  Each is a grant of its own and implies no other. None changes Matrix
  admin, owner or membership authority, and credentials stay the host's.
- `palpo::SERVICES` lists them, and `palpo::words` gives each one's
  plain-language line for the consent sheet.
- `KNOWN_FEATURES` gains `palpo-admin-v1`; an app that uses these operations
  lists it in `requires`.

## 1.1.0

`RESERVED_NAMES` gains the native apps OctoSense shipped at the time (its
`native-apps.json`). The rule is unchanged: a store app whose id or namespace
is on the list is refused.

- `RESERVED_NAMES` gains `calculator`, `clock`, `notes`, `reminders` and
  `weather`: OctoSense shipped those Makepad apps as native apps and granted
  their read tools (`notes.search`, …) to its system agent by name. A store
  app with one of those ids, or one as its namespace (`com.example.notes`),
  is now refused as reserved. The tests' example app is `forecast` (it was
  `weather`).
- `RESERVED_NAMES` gains `browser` and `task`: OctoSense's desktop-only
  native Browser and Task Manager.

## 1.0.0

The contract as OctoSense ADR 0005 section 1 defines it, moved out of App
Hub's `octosense-app-policy` with its behavior unchanged:

- Manifest: `AppManifest` and its parts, `SCHEMA`, `MANIFEST_FILE`, `parse`.
- Policy: `policy::resolve`, `HostLimits`, `AppPolicy` (what the app may
  do: capabilities, hosts, storage, budgets, research scope). The app's
  agent is resolved by hosts that run agents, not here.
- Integrity: `digest_dir`, `bundle_digest`, `admit`, `admit_digest`,
  `SignatureVerifier`, `RefuseAllSignatures`.
- Running a package: `SCRIPT_ENTRY`, `script_source`, `ASSETS_PLACEHOLDER`,
  `AssetServer`, `StaticAssets`, `rewrite_assets`.

New in the contract:

- `requires` and `schema_minor` in the manifest, `KNOWN_FEATURES` (empty)
  and `SCHEMA_MINOR` (0): a manifest requiring an unknown feature is refused
  ("needs a newer host"); a manifest for a newer `1.x` is read with its
  unknown (optional) fields ignored and listed by
  `AppManifest::ignored_fields`.
- Every public struct and enum is `#[non_exhaustive]` (except the unit
  marker `RefuseAllSignatures`); `HostLimits` gains `with_*` builders and
  `Signature` gains `new`.
- `AppPolicy` carries the whole storage block (`StorageGrant`) and
  serializes.
- The fixture corpus (`tests/fixtures/`).
