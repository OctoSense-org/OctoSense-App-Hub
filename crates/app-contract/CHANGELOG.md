# Changelog

`octosense-app-contract` follows the rules in [README.md](README.md#stability):
within `1.x` it only grows.

## 1.5.0

- Admit the `auth`, `github`, `gcalendar` and `gmail` host-service capabilities.
  Each remains an independent grant. OAuth returns app-bound connection handles;
  provider tokens stay in the host. These declarations require corresponding
  OctoSense implementations and do not make a provider available by themselves.
- Add the `photos` and `youtube` capabilities for the contained Photos and
  YouTube UIs to call their owning host services. Neither grants Android
  Gallery access, a YouTube account or another app's agent tools; cross-app
  tools remain separate grants.
- Add `portable_path`: a bundle-relative path written with `/` on every
  platform. `digest_dir` hashes these names, so a bundle with folders gets
  the same digest on Windows as on Linux and macOS, where its digest is
  unchanged: the files keep their path-component order. A Windows host used
  to hash `\`, which matched no other host. A path that is not a plain UTF-8
  name is refused instead of hashed lossily; the gate already refused such
  bundles.

## 1.4.0

- Add the `calendar` capability for a contained Calendar UI to call its
  owning host service. Agent tool sharing remains a separate grant. This
  grants no access to Android or Google Calendar accounts; the registered
  host service checks its owning app identity on every call.

## 1.3.0

- `RESERVED_NAMES` gains `octoscode`: OctoSense ships OctosCode
  (octos-org/octoscode-app) as a native app, on desktops and phones. A store
  app with that id, or with it as its namespace (`com.example.octoscode`),
  is now refused as reserved. Nothing else changes: every other manifest
  1.2.0 admitted is admitted unchanged.

## 1.2.0

Palpo mini-app operations (App Hub #72), for the Hagency/Rinx mini-app
that administers Palpo. Only additions: the schema stays at minor 0, and
every manifest 1.1.0 admitted is admitted unchanged.

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
- `KNOWN_FEATURES` gains `palpo-admin-v1`, the host feature an app that
  uses these operations requires.

## 1.1.0

`RESERVED_NAMES` lists the native apps OctoSense ships (its
`native-apps.json`), so it grows with them; the rule itself (an id or a
namespace on the list is refused) is unchanged. A manifest that 1.0.0
admitted with one of the new names as its id or namespace is refused.

- `RESERVED_NAMES` gains `calculator`, `clock`, `notes`, `reminders` and
  `weather`: OctoSense ships those Makepad apps as native apps, and grants
  their read tools (`notes.search`, …) to its system agent by name. A store
  app with one of those ids, or one as its namespace (`com.example.notes`),
  is now refused as reserved. The tests' example app is `forecast` (it was
  `weather`).
- `RESERVED_NAMES` gains `browser` and `task`: OctoSense's desktop-only
  native Browser and Task Manager.

## 1.0.0

The contract as OctoSense ADR 0005 section 1 defines it, moved out of App
Hub's `octosense-app-policy` with its behaviour unchanged:

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
  serialises.
- The fixture corpus (`tests/fixtures/`).
