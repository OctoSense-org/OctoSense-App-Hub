# Changelog

`octosense-app-contract` follows the rules in [README.md](README.md#stability):
within `1.x` it only grows.

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
