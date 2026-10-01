# octosense-app-contract

The OctoSense app contract: the one small, versioned interface between App
Hub and every app ([OctoSense ADR 0005](https://github.com/OctoSense-org/OctoSense/blob/main/docs/adr/0005-app-contract.md)).
It holds only what an app or a host needs to read, check and run an app
package:

| Part | Items |
| --- | --- |
| Manifest | `AppManifest` and its parts, `SCHEMA`, `MANIFEST_FILE`, `parse` |
| Policy | `policy::resolve`, `HostLimits`, `AppPolicy` (capabilities, network hosts, storage, budgets, research scope) |
| Integrity | `digest_dir`, `bundle_digest`, `admit`, `admit_digest`, `SignatureVerifier`, `RefuseAllSignatures` |
| Running a package | `SCRIPT_ENTRY`, `script_source`, `ASSETS_PLACEHOLDER`, `AssetServer`, `StaticAssets`, `rewrite_assets` |

What an app may do is in the contract; how a host sandboxes it is not. Each
host builds its own sandbox (an isolate's settings, an agent session) from
`AppPolicy`, and **may restrict more than `AppPolicy` says, never less.**
App Hub's store, catalog, agents, listings and host services stay in
`octosense-app-policy` and `octosense-app-hub`, which depend on this crate.

```toml
[dependencies]
octosense-app-contract = "1"
```

```rust
use octosense_app_contract::{admit_digest, digest_dir, parse, policy, HostLimits, RefuseAllSignatures, MANIFEST_FILE};

fn open(package: &std::path::Path) -> Result<octosense_app_contract::AppPolicy, String> {
    let manifest = parse(&std::fs::read_to_string(package.join(MANIFEST_FILE)).map_err(|e| e.to_string())?)?;
    admit_digest(&manifest, &digest_dir(package)?, &RefuseAllSignatures)?;
    policy::resolve(&manifest, &HostLimits { require_signature: false, ..HostLimits::default() })
}
```

## Stability

Within `1.x` the contract only grows (ADR 0005 section 2):

- **Additive only.** New types, functions, optional manifest fields and enum
  variants. Nothing is removed or renamed, and no existing field, default
  or rule changes meaning. The manifest's structs and enums, the research
  scope and `AppPolicy` are `#[non_exhaustive]` so that they can grow.
  Anything else is `2.0`, decided in an ADR.
- **The parser stays strict.** Every manifest struct refuses fields it does
  not know, so a manifest written for a newer host never runs under looser
  rules on an older one.
- **Required features are named.** A field added in `1.x` that restricts or
  changes what an app gets is a *feature*. A manifest that uses one lists
  it in `requires`:

  ```json
  { "schema": 1, "schema_minor": 1, "requires": ["<feature>"], ... }
  ```

  A host whose `KNOWN_FEATURES` lacks a listed feature refuses the app:
  `app <id> needs a newer host: <feature>`. `1.0.0` knows no features.
- **`schema` stays `1`** for the whole `1.x` line. `schema_minor` (default
  0; `SCHEMA_MINOR` is the newest this build knows) records which additions
  a manifest uses. Both fields are left out of the canonical signing bytes
  when empty, so a manifest signed before they existed signs as it did.
- **Behaviour is pinned by fixtures.** [`tests/fixtures/`](tests/fixtures/README.md)
  holds real manifests and packages with their digest and resolved
  `AppPolicy`. Every `1.x` must reproduce all of them; the corpus is
  append-only.

## Checks and releases

App Hub's CI (`.github/workflows/app-contract.yml`) tests this crate on its
own, as crates.io builds it, and runs `cargo semver-checks` against the
latest published version: a non-additive change fails the pull request. It
skips the API diff, saying so, until a first version is published.

A release is a version bump here and in [CHANGELOG.md](CHANGELOG.md),
reviewed by App Hub and one app owner (Rinx), then the manual workflow
`publish-app-contract` (`.github/workflows/publish-app-contract.yml`), which
runs the tests and publishes with the `CARGO_REGISTRY_TOKEN` secret. It
refuses a version that is already on crates.io.

## License

Apache-2.0.
