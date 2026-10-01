//! The manifest's growth rules within 1.x (ADR 0005 §2): unknown fields are
//! still refused, a required feature this build does not know refuses the
//! app, and the two new fields leave older manifests' signing bytes alone.
use octosense_app_contract::*;

fn manifest_with(body: &str) -> String {
    format!(
        r#"{{"schema":1,"id":"weather","version":"1.0.0","name":"Weather","integrity":{{"bundle_blake3":"00"}}{}}}"#,
        if body.is_empty() { String::new() } else { format!(",{body}") }
    )
}

#[test]
fn version_one_point_zero_knows_no_features_and_minor_zero() {
    assert_eq!(SCHEMA, 1);
    assert_eq!(SCHEMA_MINOR, 0);
    assert!(KNOWN_FEATURES.is_empty());
}

#[test]
fn an_unknown_required_feature_needs_a_newer_host() {
    let err = parse(&manifest_with(r#""requires":["storage.encrypted"]"#)).unwrap_err();
    assert_eq!(err, "app weather needs a newer host: storage.encrypted");
    let err = parse(&manifest_with(r#""requires":["a","b"],"schema_minor":2"#)).unwrap_err();
    assert!(err.ends_with("needs a newer host: a, b"), "{err}");
}

#[test]
fn resolve_checks_requires_even_for_a_manifest_not_read_by_parse() {
    let manifest: AppManifest = serde_json::from_str(&manifest_with(r#""requires":["x"]"#)).unwrap();
    let limits = HostLimits { require_signature: false, ..HostLimits::default() };
    let err = resolve(&manifest, &limits).unwrap_err();
    assert!(err.contains("needs a newer host: x"), "{err}");
}

#[test]
fn an_empty_requires_and_a_newer_minor_are_accepted() {
    let manifest = parse(&manifest_with(r#""requires":[],"schema_minor":3"#)).unwrap();
    assert!(manifest.requires.is_empty());
    assert_eq!(manifest.schema_minor, 3);
}

#[test]
fn unknown_fields_are_still_refused() {
    let err = parse(&manifest_with(r#""sandbox":"off""#)).unwrap_err();
    assert!(err.contains("unknown field"), "{err}");
    let err = parse(&manifest_with(r#""schema_minor":1,"sandbox":"off""#)).unwrap_err();
    assert!(err.contains("unknown field"), "{err}");
}

#[test]
fn the_new_fields_do_not_change_an_older_manifests_signing_bytes() {
    let plain = parse(&manifest_with("")).unwrap();
    let explicit = parse(&manifest_with(r#""requires":[],"schema_minor":0"#)).unwrap();
    assert_eq!(plain.signing_bytes().unwrap(), explicit.signing_bytes().unwrap());
    let bytes = String::from_utf8(plain.signing_bytes().unwrap()).unwrap();
    assert!(!bytes.contains("requires") && !bytes.contains("schema_minor"), "{bytes}");
    let minor = parse(&manifest_with(r#""schema_minor":1"#)).unwrap();
    assert!(String::from_utf8(minor.signing_bytes().unwrap()).unwrap().contains(r#""schema_minor":1"#));
}
