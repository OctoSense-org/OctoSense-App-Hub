mod common;
use common::*;
use octosense_app_hub::*;
use octosense_app_policy::{HostLimits, SignatureVerifier};
use std::{fs, process::Command};

#[test]
fn replacement_key_same_id_is_refused() {
    let mut f = Fixture::new();
    let catalog = Catalog::new(1, "2026-09-25", vec![f.entry()]);
    f.manifest.version = "2.0.0".into();
    f.sign();
    assert!(f.report(Some(&catalog)).passed(), "a same-key update must pass");
    f.publisher = HubKey::generate();
    f.sign();
    let report = f.report(Some(&catalog));
    assert!(!report.passed(), "replacement key passed: {}", report.render());
    assert!(report.findings.iter().any(|f| f.check == "continuity"));
}

#[test]
fn entry_publisher_must_match_signature_owner() {
    let f = Fixture::new();
    let report = f.report(None);
    assert!(entry_for(&f.bundle, &report, "impostor", &f.publisher.public_hex(), "", "", "2026-09-25").is_err());
    assert!(entry_for(&f.bundle, &report, "publisher-one", &HubKey::generate().public_hex(), "", "", "2026-09-25").is_err());
}

#[test]
fn one_publisher_can_publish_two_apps_but_cannot_redefine_its_key() {
    let mut f = Fixture::new();
    let catalog = Catalog::new(1, "2026-09-25", vec![f.entry()]);
    f.manifest.id = "second-app".into();
    f.sign();
    assert!(f.report(Some(&catalog)).passed());
    f.publisher = HubKey::generate();
    f.sign();
    assert!(!f.report(Some(&catalog)).passed(), "new app must not redefine an existing publisher");
}

#[test]
fn unknown_rotation_requires_registry_authorization() {
    let mut f = Fixture::new();
    let first = f.entry();
    f.manifest.version = "2.0.0".into();
    f.publisher = HubKey::generate();
    f.sign();
    let second = f.entry();
    let catalog = Catalog::new(2, "2026-09-25", vec![first, second]);
    f.manifest.version = "3.0.0".into();
    f.sign();
    let report = f.report(Some(&catalog));
    assert!(!report.passed(), "ambiguous history must require reconciliation");
    assert!(report.render().contains("conflicting"), "{}", report.render());
}

#[test]
fn duplicate_key_mappings_do_not_choose_the_first_key() {
    let f = Fixture::new();
    let keys = f.keys().with("publisher-one", &HubKey::generate().public_hex());
    let sig = f.manifest.integrity.signature.as_ref().unwrap();
    assert!(keys.verify(&sig.key_id, &sig.value, &f.manifest.signing_bytes().unwrap()).is_err());
    // Repetition of the same public bytes is harmless, including hex case.
    let same = f.keys().with("publisher-one", &f.publisher.public_hex().to_uppercase());
    same.verify(&sig.key_id, &sig.value, &f.manifest.signing_bytes().unwrap()).unwrap();
}

#[test]
fn public_publish_rejects_unsigned_first_release() {
    let mut f = Fixture::new();
    f.manifest.integrity.signature = None;
    f.write_manifest();
    let local = Command::new(env!("CARGO_BIN_EXE_hub")).args(["check", f.bundle.to_str().unwrap(), "--allow-unsigned"]).output().unwrap();
    assert!(local.status.success(), "{}", String::from_utf8_lossy(&local.stderr));
    let key_path = f.root.join("working.key");
    let key = HubKey::generate();
    fs::write(&key_path, hex::encode(key.to_bytes())).unwrap();
    let cert = HubKey::generate().certify(&key.public_hex()).unwrap();
    let catalog_path = f.root.join("public/catalog.json");
    let result = Command::new(env!("CARGO_BIN_EXE_hub")).args([
        "publish", f.bundle.to_str().unwrap(), "--allow-unsigned", "--publisher", "publisher-one",
        "--catalog", catalog_path.to_str().unwrap(), "--key", key_path.to_str().unwrap(),
        "--anchor-cert", &cert, "--out", f.root.join("public").to_str().unwrap(),
    ]).output().unwrap();
    assert!(!result.status.success(), "unsigned public release was admitted");
    assert!(!catalog_path.exists(), "a refused release must not create a catalog");
}

#[test]
fn publication_requires_runtime_evidence_even_when_reviewed() {
    let f = Fixture::new();
    let working = HubKey::generate();
    let key_path = f.root.join("working.key");
    fs::write(&key_path, hex::encode(working.to_bytes())).unwrap();
    let anchor = HubKey::generate();
    let certificate = anchor.certify(&working.public_hex()).unwrap();
    let catalog = f.root.join("public/catalog.json");
    let result = Command::new(env!("CARGO_BIN_EXE_hub")).args([
        "publish", f.bundle.to_str().unwrap(), "--publisher", "publisher-one",
        "--publisher-key", &format!("publisher-one={}", f.publisher.public_hex()),
        "--catalog", catalog.to_str().unwrap(), "--key", key_path.to_str().unwrap(),
        "--anchor-cert", &certificate, "--out", f.root.join("public").to_str().unwrap(),
        "--reviewed-by", "test-operator", "--review-id", "test-review", "--validator", "/missing/app-validator",
        "--anchor", &anchor.public_hex(), "--state-dir", f.root.join("state").to_str().unwrap(),
        "--expected-sequence", "0", "--idempotency-key", "runtime-required",
    ]).output().unwrap();
    assert!(!result.status.success(), "a missing validator must refuse publication");
    assert!(!catalog.exists());
    assert!(!f.root.join("public/artifacts").exists());
}

#[test]
fn unsigned_update_is_refused_even_in_local_check() {
    let mut f = Fixture::new();
    let catalog = Catalog::new(1, "2026-09-25", vec![f.entry()]);
    f.manifest.version = "2.0.0".into();
    f.manifest.integrity.signature = None;
    f.write_manifest();
    let limits = HostLimits { require_signature: false, ..HostLimits::default() };
    assert!(!check_bundle(&f.bundle, &limits, &f.keys(), Some(&catalog)).unwrap().passed());
}

#[test]
fn entry_cannot_reuse_a_gate_report_after_manifest_changes() {
    let mut f = Fixture::new();
    let report = f.report(None);
    f.manifest.name = "Changed after review".into();
    f.sign();
    assert!(entry_for(&f.bundle, &report, "publisher-one", &f.publisher.public_hex(), "", "", "2026-09-25").is_err());
}

#[test]
fn existing_signed_v1_catalog_still_verifies() {
    let catalog: Catalog = serde_json::from_str(include_str!("../../../catalog.json")).unwrap();
    verify_catalog(&catalog, "6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11").unwrap();
}

#[test]
fn publish_authenticates_history_and_preserves_the_registered_key() {
    let mut f = Fixture::new();
    let working = HubKey::generate();
    let anchor = HubKey::generate();
    let certificate = anchor.certify(&working.public_hex()).unwrap();
    let key_path = f.root.join("working.key");
    fs::write(&key_path, hex::encode(working.to_bytes())).unwrap();
    let catalog_path = f.root.join("public/catalog.json");
    let publish = |f: &Fixture, with_anchor: bool| {
        let validator = f.protocol_worker();
        let mut command = Command::new(env!("CARGO_BIN_EXE_hub"));
        command.args(["publish", f.bundle.to_str().unwrap(), "--publisher", "publisher-one",
            "--publisher-key", &format!("publisher-one={}", f.publisher.public_hex()),
            "--catalog", catalog_path.to_str().unwrap(), "--key", key_path.to_str().unwrap(),
            "--anchor-cert", &certificate, "--out", f.root.join("public").to_str().unwrap(), "--validator", validator.to_str().unwrap(),
            "--state-dir", f.root.join("state").to_str().unwrap(), "--expected-sequence", if catalog_path.exists() { "1" } else { "0" },
            "--idempotency-key", &f.manifest.version, "--reviewed-by", "test-operator", "--review-id", "test-review"]);
        if with_anchor { command.args(["--anchor", &anchor.public_hex()]); }
        command.output().unwrap()
    };
    let first = publish(&f, true);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let before = fs::read(&catalog_path).unwrap();
    f.manifest.version = "2.0.0".into();
    f.sign();
    let unsigned_history = publish(&f, false);
    assert!(!unsigned_history.status.success());
    assert!(String::from_utf8_lossy(&unsigned_history.stderr).contains("authenticate"));
    assert_eq!(before, fs::read(&catalog_path).unwrap());
    let original = f.publisher.to_bytes();
    f.publisher = HubKey::generate();
    f.sign();
    let substitution = publish(&f, true);
    assert!(!substitution.status.success());
    assert!(String::from_utf8_lossy(&substitution.stderr).contains("continuity"));
    assert_eq!(before, fs::read(&catalog_path).unwrap());
    f.publisher = HubKey::from_bytes(&original);
    f.sign();
    let second = publish(&f, true);
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    let mut catalog: Catalog = serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
    verify_catalog(&catalog, &anchor.public_hex()).unwrap();
    assert_eq!(catalog.entries.len(), 2);
    catalog.entries[0].publisher_key = HubKey::generate().public_hex();
    write_catalog(&catalog_path, &catalog);
    f.manifest.version = "3.0.0".into();
    f.sign();
    let tampered_history = publish(&f, true);
    assert!(!tampered_history.status.success());
    assert!(String::from_utf8_lossy(&tampered_history.stderr).contains("signature"));
}

#[test]
fn conflicting_catalog_owner_and_missing_legacy_key_require_reconciliation() {
    let mut f = Fixture::new();
    let entry = f.entry();
    f.manifest.version = "2.0.0".into();
    f.sign();
    for bad_entry in [
        Entry { publisher: "someone-else".into(), ..entry.clone() },
        Entry { publisher_key: String::new(), ..entry },
    ] {
        let catalog = Catalog::new(1, "2026-09-25", vec![bad_entry]);
        assert!(!f.report(Some(&catalog)).passed());
    }
}

#[test]
fn refused_report_or_changed_content_cannot_be_published() {
    let f = Fixture::new();
    let report = f.report(None);
    fs::write(f.bundle.join("payload.txt"), "changed after gate").unwrap();
    assert!(entry_for(&f.bundle, &report, "publisher-one", &f.publisher.public_hex(), "", "", "2026-09-25").is_err());
    let refused = f.report(None);
    assert!(!refused.passed());
    assert!(entry_for(&f.bundle, &refused, "publisher-one", &f.publisher.public_hex(), "", "", "2026-09-25").is_err());
}
