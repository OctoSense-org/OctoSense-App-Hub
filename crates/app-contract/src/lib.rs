//! The OctoSense app contract: what an app or a host needs to read, check
//! and run an app package, and nothing else (OctoSense ADR 0005).
//!
//! App Hub, the OctoSense shell and every app that runs other apps (Rinx's
//! mini apps) depend on this crate by version, `octosense-app-contract =
//! "1"`, so Cargo links one `1.x` for the whole build and App Hub can change
//! its store, catalog and runner without forcing an app release.
//!
//! - **Manifest:** [`AppManifest`] and its parts, [`SCHEMA`],
//!   [`MANIFEST_FILE`], [`parse`].
//! - **Policy:** [`policy::resolve`], [`HostLimits`], [`AppPolicy`]: the
//!   capabilities, network hosts, storage and budgets an app gets.
//! - **Integrity:** [`digest_dir`], [`bundle_digest`], [`admit`],
//!   [`admit_digest`], [`SignatureVerifier`], [`RefuseAllSignatures`].
//! - **Running a package:** [`SCRIPT_ENTRY`], [`script_source`],
//!   [`ASSETS_PLACEHOLDER`], [`AssetServer`], [`StaticAssets`],
//!   [`rewrite_assets`].
//!
//! What an app may do is in the contract; how a host sandboxes it is not.
//! Each host turns an [`AppPolicy`] into its own sandbox settings under one
//! rule: **a host may restrict more than the policy says, never less.**
//!
//! The order is always: [`parse`] → [`admit`] (or [`admit_digest`]) →
//! [`policy::resolve`]. Skipping a step is the bug this crate exists to make
//! hard.
//!
//! ```
//! use octosense_app_contract::{admit, bundle_digest, parse, policy, HostLimits, RefuseAllSignatures};
//! let bundle = b"the card bundle bytes";
//! let manifest = format!(
//!     r#"{{"schema":1,"id":"weather","version":"1.0.0","name":"Weather",
//!         "integrity":{{"bundle_blake3":"{}"}},
//!         "capabilities":["storage","net"],
//!         "network":{{"hosts":["api.weather.example"]}}}}"#,
//!     bundle_digest(bundle)
//! );
//! let manifest = parse(&manifest).unwrap();
//! admit(&manifest, bundle, &RefuseAllSignatures).unwrap();
//! let limits = HostLimits { require_signature: false, ..HostLimits::default() };
//! let policy = policy::resolve(&manifest, &limits).unwrap();
//! assert!(policy.allows_host("api.weather.example"));
//! assert!(!policy.allows_host("example.com"));
//! ```
//!
//! # Stability
//!
//! Within `1.x` the contract only grows (ADR 0005 §2):
//!
//! - **Additive only.** New types, functions, optional manifest fields and
//!   enum variants; nothing is removed or renamed, and no existing field,
//!   default or rule changes meaning. The manifest's structs and enums, the
//!   research scope and [`AppPolicy`] are `#[non_exhaustive]` so that they
//!   can grow. Anything else is `2.0`, decided in an ADR. CI runs `cargo
//!   semver-checks` against the latest published `1.x` on every change.
//! - **The parser stays strict.** Every manifest struct refuses fields it
//!   does not know (`deny_unknown_fields`), so a manifest written for a
//!   newer host never runs under looser rules on an older one.
//! - **Required features are named.** A field added in `1.x` that restricts
//!   or changes what an app gets is a *feature*: a manifest that uses it
//!   lists it in `requires`, and a host whose [`KNOWN_FEATURES`] lacks it
//!   refuses the app with "needs a newer host: <feature>". `1.0.0` knows no
//!   features.
//! - **`schema` stays `1`** for the whole `1.x` line; `schema_minor`
//!   (default 0, [`SCHEMA_MINOR`] in this build) records which additions a
//!   manifest uses. Both new fields are left out of a manifest's canonical
//!   signing bytes when empty, so every manifest signed before them signs
//!   exactly as it did.
//! - **Behaviour is pinned by fixtures.** `tests/fixtures/` holds real
//!   manifests and packages with their digest and resolved [`AppPolicy`];
//!   every `1.x` release must accept all of them with the same results, and
//!   the corpus is append-only within `1.x`.
pub mod assets;
pub mod bundle;
pub mod entry;
pub mod manifest;
pub mod policy;
pub mod research;
pub mod verify;

pub use assets::{rewrite_assets, AssetServer, StaticAssets};
pub use bundle::{digest_dir, MANIFEST_FILE};
pub use entry::{script_source, ASSETS_PLACEHOLDER, SCRIPT_ENTRY};
pub use manifest::{
    check_reserved_id, parse, short_id, AgentSpec, AgentWorkspace, AppManifest, Compute, Integrity, ModelNeed, ModelSpec,
    ModelTier, Network, ProfileMode, Signature, Storage, TaskModel, Triggers, KNOWN_CAPABILITIES, KNOWN_FEATURES,
    KNOWN_MODEL_NEEDS, RESERVED_NAMES, SCHEMA, SCHEMA_MINOR,
};
pub use policy::{resolve, AppPolicy, HostLimits};
pub use research::ResearchScope;
pub use verify::{admit, admit_digest, bundle_digest, RefuseAllSignatures, SignatureVerifier};
