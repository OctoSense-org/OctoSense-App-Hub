//! The manifest's growth rules within 1.x (ADR 0005 §2): unknown fields are
//! refused at this build's minor and ignored (and reported) in a manifest
//! written for a newer 1.x, a required feature this build does not know
//! refuses the app at every minor, and the two new fields leave older
//! manifests' signing bytes alone.
use octosense_app_contract::*;

fn manifest_with(body: &str) -> String {
    format!(
        r#"{{"schema":1,"id":"forecast","version":"1.0.0","name":"Forecast","integrity":{{"bundle_blake3":"00"}}{}}}"#,
        if body.is_empty() { String::new() } else { format!(",{body}") }
    )
}

#[test]
fn palpo_extension_preserves_schema_and_declares_its_host_feature() {
    assert_eq!(SCHEMA, 1);
    assert_eq!(SCHEMA_MINOR, 0);
    assert_eq!(
        KNOWN_FEATURES,
        &["palpo-admin-v1", "host-api-v1", "backend-api-v1", "script-tools-v1", "publisher-github-v1", "wasm-components-v1"]
    );
}

#[test]
fn wasm_components_need_the_wasm_capability() {
    // ADR 0014: a bundle with a component in fns/ requires the feature, and
    // the feature means nothing without the wasm capability.
    assert!(parse(&manifest_with(r#""requires":["wasm-components-v1"]"#)).is_err());
    assert!(parse(&manifest_with(r#""requires":["wasm-components-v1"],"capabilities":["storage"]"#)).is_err());
    let manifest = parse(&manifest_with(r#""requires":["wasm-components-v1"],"capabilities":["wasm","storage"]"#)).unwrap();
    assert_eq!(manifest.requires, ["wasm-components-v1"]);
}

#[test]
fn host_api_requirements_need_the_feature_and_do_not_grant_access() {
    assert!(parse(&manifest_with(r#""host_api":{"required":{"camera.permission.status":1}}"#)).is_err());
    let manifest = parse(&manifest_with(r#""requires":["host-api-v1"],"host_api":{"required":{"camera.permission.status":1}},"capabilities":["runtime"]"#)).unwrap();
    let policy = resolve(&manifest, &HostLimits::default().with_require_signature(false)).unwrap();
    assert!(policy.allows("runtime"));
    assert!(!policy.allows("camera"));
}

#[test]
fn selected_file_access_needs_its_own_grant_and_an_implemented_host() {
    let limits = HostLimits::default().with_require_signature(false);
    let manifest = parse(&manifest_with(r#""requires":["host-api-v1"],"capabilities":["files"],"host_api":{"required":{"files.import":1}}"#)).unwrap();
    let policy = resolve(&manifest, &limits).unwrap();
    assert!(policy.allows("files"));
    for other in ["storage", "photos", "library", "camera", "net"] {
        assert!(!policy.allows(other), "files must not imply {other}");
    }
    let storage = resolve(&parse(&manifest_with(r#""capabilities":["storage"]"#)).unwrap(), &limits).unwrap();
    assert!(!storage.allows("files"));
    let mut versions = std::collections::BTreeMap::from([("app_policy.device_consent".into(), 1)]);
    assert!(manifest.check_host_apis(&versions).unwrap_err().contains("files.import@1"));
    versions.insert("files.import".into(), 1);
    assert!(manifest.check_host_apis(&versions).is_ok());
    for near in ["files.*", "files.import", "FILES"] {
        let manifest = parse(&manifest_with(&format!(r#""capabilities":["{near}"]"#))).unwrap();
        assert!(resolve(&manifest, &limits).is_err(), "{near} must be unknown");
    }
}

#[test]
fn host_api_marker_requires_the_policy_abi_even_without_a_device_method() {
    let manifest = parse(&manifest_with(r#""requires":["host-api-v1"]"#)).unwrap();
    let mut versions = std::collections::BTreeMap::new();
    assert!(manifest.check_host_apis(&versions).unwrap_err().contains("app_policy.device_consent@1"));
    versions.insert("camera.permission.status".into(), 1);
    assert!(manifest.check_host_apis(&versions).is_err(), "a method is not the policy ABI");
    versions.insert("app_policy.device_consent".into(), 2);
    assert!(manifest.check_host_apis(&versions).is_err(), "ABI majors match exactly");
    versions.insert("app_policy.device_consent".into(), 1);
    assert!(manifest.check_host_apis(&versions).is_ok());
}

#[test]
fn palpo_uses_exact_service_grants() {
    let services = serde_json::to_string(octosense_app_contract::palpo::SERVICES).unwrap();
    let manifest = parse(&manifest_with(&format!(
        r#""requires":["palpo-admin-v1"],"capabilities":{services}"#
    ))).unwrap();
    let limits = HostLimits::system().with_require_signature(false);
    let resolved = resolve(&manifest, &limits).unwrap();
    for service in octosense_app_contract::palpo::SERVICES {
        assert!(resolved.allows(service));
    }
    assert!(!resolved.allows("palpo.*"));
    assert!(!resolved.allows("palpo.users.delete"));
    let unknown = parse(&manifest_with(r#""capabilities":["palpo.users.delete"]"#)).unwrap();
    assert!(resolve(&unknown, &limits).is_err());
}

#[test]
fn calendar_ui_access_is_explicit_and_does_not_grant_other_services() {
    let limits = HostLimits::system();
    let old = resolve(&parse(&manifest_with("")).unwrap(), &limits).unwrap();
    assert!(!old.allows("calendar"));
    let calendar = resolve(&parse(&manifest_with(r#""capabilities":["calendar"]"#)).unwrap(), &limits).unwrap();
    assert!(calendar.allows("calendar"));
    assert!(!calendar.allows("mail") && !calendar.allows("calendar.*"));
    assert!(!calendar.allows("glance"));
    assert!(resolve(&parse(&manifest_with(r#""capabilities":["calendar.*"]"#)).unwrap(), &limits).is_err());
}

#[test]
fn media_services_require_exact_separate_grants() {
    let limits = HostLimits::system();
    for capability in ["photos", "youtube"] {
        let manifest = parse(&manifest_with(&format!(r#""capabilities":["{capability}"]"#))).unwrap();
        let policy = resolve(&manifest, &limits).unwrap();
        assert!(policy.allows(capability));
        assert!(!policy.allows("mail") && !policy.allows("glance") && !policy.allows("net"));
        assert!(!policy.allows(if capability == "photos" { "youtube" } else { "photos" }));
        let wildcard = parse(&manifest_with(&format!(r#""capabilities":["{capability}.*"]"#))).unwrap();
        assert!(resolve(&wildcard, &limits).is_err());
    }
}

#[test]
fn native_calendar_grant_is_separate_and_required_methods_must_exist() {
    let manifest = parse(&manifest_with(r#""capabilities":["device_calendar"],"requires":["host-api-v1"],"host_api":{"required":{"device_calendar.events.list":1}}"#)).unwrap();
    let policy = resolve(&manifest, &HostLimits::system()).unwrap();
    assert!(policy.allows("device_calendar"));
    for unrelated in ["calendar", "gcalendar", "auth", "mail", "location", "device_calendar.*"] {
        assert!(!policy.allows(unrelated), "native calendar must not imply {unrelated}");
    }
    let mut versions = std::collections::BTreeMap::from([("app_policy.device_consent".into(), 1)]);
    assert!(manifest.check_host_apis(&versions).unwrap_err().contains("device_calendar.events.list"));
    versions.insert("device_calendar.events.list".into(), 2);
    assert!(manifest.check_host_apis(&versions).is_err(), "ABI 2 does not silently satisfy ABI 1");
    versions.insert("device_calendar.events.list".into(), 1);
    manifest.check_host_apis(&versions).unwrap();
    let old = resolve(&parse(&manifest_with(r#""capabilities":["calendar","gcalendar"]"#)).unwrap(), &HostLimits::system()).unwrap();
    assert!(!old.allows("device_calendar"));
}

#[test]
fn playback_never_grants_recording_storage_or_background_authority() {
    let manifest = parse(&manifest_with(r#""capabilities":["audio"],"requires":["host-api-v1"],"host_api":{"required":{"audio.play":1}}"#)).unwrap();
    let policy = resolve(&manifest, &HostLimits::system()).unwrap();
    assert!(policy.allows("audio"));
    for unrelated in ["microphone", "storage", "camera", "net", "audio.*"] {
        assert!(!policy.allows(unrelated));
    }
    let mut versions = std::collections::BTreeMap::from([("app_policy.device_consent".into(), 1)]);
    assert!(manifest.check_host_apis(&versions).is_err());
    versions.insert("audio.play".into(), 1);
    manifest.check_host_apis(&versions).unwrap();
    let capture = resolve(&parse(&manifest_with(r#""capabilities":["microphone","storage"]"#)).unwrap(), &HostLimits::system()).unwrap();
    assert!(!capture.allows("audio"));
}

#[test]
fn an_apps_own_functions_are_a_grant_of_their_own() {
    let limits = HostLimits::system();
    let manifest = parse(&manifest_with(r#""capabilities":["wasm"]"#)).unwrap();
    let policy = resolve(&manifest, &limits).unwrap();
    assert!(policy.allows("wasm"));
    for other in ["net", "storage", "model", "glance", "mail", "runtime"] {
        assert!(!policy.allows(other), "wasm must not imply {other}");
    }
    for near in ["wasm.*", "wasm.find_slots", "WASM"] {
        let manifest = parse(&manifest_with(&format!(r#""capabilities":["{near}"]"#))).unwrap();
        assert!(resolve(&manifest, &limits).is_err(), "{near} must be unknown");
    }
}

#[test]
fn an_unknown_required_feature_needs_a_newer_host() {
    let err = parse(&manifest_with(r#""requires":["storage.encrypted"]"#)).unwrap_err();
    assert_eq!(err, "app forecast needs a newer host: storage.encrypted");
    let err = parse(&manifest_with(r#""requires":["a","b"],"schema_minor":2"#)).unwrap_err();
    assert!(err.ends_with("needs a newer host: a, b"), "{err}");
}

#[test]
fn oauth_provider_services_are_independent_grants() {
    let limits = HostLimits::default().with_require_signature(false);
    let capabilities = ["auth", "github", "gcalendar", "gmail"];
    for granted in capabilities {
        let manifest = parse(&manifest_with(&format!(r#""capabilities":["{granted}"]"#))).unwrap();
        let policy = resolve(&manifest, &limits).unwrap();
        for candidate in capabilities {
            assert_eq!(policy.allows(candidate), candidate == granted);
        }
        for unrelated in ["mail", "calendar", "net", "glance", "auth.*"] {
            assert!(!policy.allows(unrelated));
        }
        let wildcard = parse(&manifest_with(&format!(r#""capabilities":["{granted}.*"]"#))).unwrap();
        assert!(resolve(&wildcard, &limits).is_err());
    }
}

#[test]
fn resolve_checks_requires_even_for_a_manifest_not_read_by_parse() {
    let manifest: AppManifest = serde_json::from_str(&manifest_with(r#""requires":["x"]"#)).unwrap();
    let limits = HostLimits::default().with_require_signature(false);
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
    // At this build's own minor, said or not, the read is strict.
    let err = parse(&manifest_with(r#""schema_minor":0,"sandbox":"off""#)).unwrap_err();
    assert!(err.contains("unknown field"), "{err}");
    let err = parse(&manifest_with(r#""schema_minor":0,"network":{"hosts":[],"proxy":"x"}"#)).unwrap_err();
    assert!(err.contains("unknown field `proxy`"), "{err}");
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

#[test]
fn a_newer_minor_ignores_an_unknown_optional_field_and_reports_it() {
    let manifest = parse(&manifest_with(r#""schema_minor":1,"capabilities":["storage"],"theme_color":"slate""#)).unwrap();
    assert_eq!(manifest.ignored_fields(), ["theme_color"]);
    assert_eq!(manifest.capabilities, ["storage"]);
    let limits = HostLimits::default().with_require_signature(false);
    assert!(resolve(&manifest, &limits).unwrap().allows("storage"));
}

#[test]
fn a_newer_minor_with_an_unknown_required_feature_is_refused() {
    let err = parse(&manifest_with(r#""schema_minor":2,"requires":["net.proxy"],"network":{"hosts":[],"proxy":"p.example"}"#))
        .unwrap_err();
    assert_eq!(err, "app forecast needs a newer host: net.proxy");
    // Without unknown fields, the strict read reaches the same refusal.
    let err = parse(&manifest_with(r#""schema_minor":2,"requires":["net.proxy"]"#)).unwrap_err();
    assert_eq!(err, "app forecast needs a newer host: net.proxy");
}

#[test]
fn a_newer_minor_ignores_unknown_fields_at_every_level() {
    let manifest = parse(&manifest_with(
        r#""schema_minor":3,
           "capabilities":["net","research"],
           "network":{"hosts":["api.weather.example"],"retry":2},
           "storage":{"max_bytes":4096,"backup":"never"},
           "compute":{"memory_bytes":1024,"gpu":false},
           "research":{"langs":["en"],"safe_search":true},
           "integrity_extra":1,
           "agent":{"profile":"read-only","voice":"calm",
                    "model":{"needs":["tool_calling"],"temperature":0.2,
                             "per_task":{"triage":{"tier":"fast","budget":5}}},
                    "triggers":{"schedule":["0 7 * * *"],"geofence":"home"}}"#,
    ))
    .unwrap();
    assert_eq!(
        manifest.ignored_fields(),
        [
            "agent.model.per_task.triage.budget",
            "agent.model.temperature",
            "agent.triggers.geofence",
            "agent.voice",
            "compute.gpu",
            "integrity_extra",
            "network.retry",
            "research.safe_search",
            "storage.backup",
        ]
    );
    // What the build knows is read as written.
    assert_eq!(manifest.network.hosts, ["api.weather.example"]);
    assert_eq!(manifest.storage.max_bytes, Some(4096));
    assert_eq!(manifest.research.as_ref().unwrap().langs, ["en"]);
    assert_eq!(manifest.agent.as_ref().unwrap().model.as_ref().unwrap().per_task["triage"].tier, ModelTier::Fast);
}

#[test]
fn a_newer_minor_still_refuses_what_it_knows_to_be_wrong() {
    // Ignoring is for unknown fields only: a known field with a bad value,
    // a foreign schema and the old research shape are refused as before.
    let err = parse(&manifest_with(r#""schema_minor":1,"storage":{"agent_workspace":"everything"}"#)).unwrap_err();
    assert!(err.contains("manifest is not valid"), "{err}");
    let err = parse(&manifest_with(r#""schema_minor":1,"x":1"#).replace(r#""schema":1"#, r#""schema":2"#)).unwrap_err();
    assert!(err.contains("manifest is not valid"), "{err}");
    let err = parse(&manifest_with(r#""schema_minor":1,"capabilities":["research"],"research":{"languages":["en"]}"#)).unwrap_err();
    assert!(err.contains("old toolbox shape"), "{err}");
}

#[test]
fn a_newer_manifest_signs_with_the_fields_it_was_signed_with() {
    let manifest = parse(&manifest_with(r#""schema_minor":1,"theme_color":"blue","network":{"hosts":[],"retry":2}"#)).unwrap();
    let bytes = String::from_utf8(manifest.signing_bytes().unwrap()).unwrap();
    assert!(bytes.contains(r#""theme_color":"blue""#), "{bytes}");
    assert!(bytes.contains(r#""network":{"hosts":[],"retry":2}"#), "{bytes}");
}

#[test]
fn hostlimits_builders_set_each_ceiling() {
    let limits = HostLimits::system()
        .with_require_signature(true)
        .with_max_storage_bytes(1)
        .with_max_instruction_budget(2)
        .with_max_memory_bytes(3)
        .with_max_iterations(4)
        .with_max_token_budget(5)
        .with_offered_tools(["net.fetch"]);
    assert!(limits.require_signature);
    assert_eq!(
        (limits.max_storage_bytes, limits.max_instruction_budget, limits.max_memory_bytes, limits.max_iterations, limits.max_token_budget),
        (1, 2, 3, 4, 5)
    );
    assert_eq!(limits.offered_tools, ["net.fetch"]);
}
