//! What an app may and may not obtain by asking. Every test here is a rule
//! from ADR 0002; a failure means an app got more reach than it declared, or
//! a well-formed app was refused.
use octosense_app_policy::*;
use std::path::Path;

const BUNDLE: &[u8] = b"a card bundle";

fn manifest_with(body: &str) -> String {
    format!(r#"{{"schema":1,"id":"weather","version":"1.0.0","name":"Weather","integrity":{{"bundle_blake3":"{}"}}{}}}"#,
        bundle_digest(BUNDLE),
        if body.is_empty() { String::new() } else { format!(",{body}") })
}

fn open_limits() -> HostLimits {
    HostLimits { require_signature: false, ..HostLimits::default() }
}

fn resolve(body: &str) -> Result<AppPolicy, String> {
    admit_and_resolve(&manifest_with(body), BUNDLE, &open_limits(), &RefuseAllSignatures)
}

// --------------------------------------------------------------- admission

#[test]
fn a_changed_bundle_is_refused_before_anything_else() {
    let err = admit_and_resolve(&manifest_with(""), b"tampered", &open_limits(), &RefuseAllSignatures).unwrap_err();
    assert!(err.contains("does not match the manifest"), "{err}");
}

#[test]
fn an_unsigned_bundle_is_refused_when_the_host_requires_signing() {
    let err = admit_and_resolve(&manifest_with(""), BUNDLE, &HostLimits::default(), &RefuseAllSignatures).unwrap_err();
    assert!(err.contains("unsigned"), "{err}");
}

#[test]
fn a_signature_without_a_verifier_is_refused_rather_than_ignored() {
    let body = r#""#;
    let json = manifest_with(body).replace(
        &format!(r#""bundle_blake3":"{}""#, bundle_digest(BUNDLE)),
        &format!(r#""bundle_blake3":"{}","signature":{{"key_id":"release","value":"aabb"}}"#, bundle_digest(BUNDLE)),
    );
    let err = admit_and_resolve(&json, BUNDLE, &HostLimits::default(), &RefuseAllSignatures).unwrap_err();
    assert!(err.contains("no signature verifier"), "{err}");
}

#[test]
fn an_unknown_field_is_refused_so_a_newer_manifest_cannot_run_under_looser_rules() {
    let err = admit_and_resolve(
        &manifest_with(r#""sandbox":"off""#),
        BUNDLE,
        &open_limits(),
        &RefuseAllSignatures,
    )
    .unwrap_err();
    assert!(err.contains("manifest is not valid"), "{err}");
}

#[test]
fn a_foreign_schema_is_refused() {
    let json = manifest_with("").replace(r#""schema":1"#, r#""schema":2"#);
    let err = admit_and_resolve(&json, BUNDLE, &open_limits(), &RefuseAllSignatures).unwrap_err();
    assert!(err.contains("schema 2 is not 1"), "{err}");
}

// ------------------------------------------------------------ capabilities

#[test]
fn nothing_is_granted_by_default() {
    let policy = resolve("").unwrap();
    assert!(policy.capabilities.is_empty());
    assert!(!policy.allows("net"));
    assert!(!policy.may_prompt);
    assert!(policy.agent.is_none());
    assert!(!policy.isolate_settings(Path::new("/data")).allow_net);
}

#[test]
fn an_unknown_capability_is_refused() {
    let err = resolve(r#""capabilities":["root"]"#).unwrap_err();
    assert!(err.contains("unknown capability"), "{err}");
}

#[test]
fn prompting_is_a_capability_of_its_own() {
    assert!(!resolve(r#""capabilities":["storage"]"#).unwrap().may_prompt);
    assert!(resolve(r#""capabilities":["prompt"]"#).unwrap().may_prompt);
}

// ----------------------------------------------------------------- network

#[test]
fn only_listed_hosts_are_reachable() {
    let policy = resolve(r#""capabilities":["net"],"network":{"hosts":["api.weather.example"]}"#).unwrap();
    assert!(policy.allows_host("api.weather.example"));
    assert!(!policy.allows_host("evil.example"));
}

#[test]
fn the_net_capability_without_hosts_reaches_nothing_and_gets_no_network_module() {
    let policy = resolve(r#""capabilities":["net"]"#).unwrap();
    assert!(!policy.allows_host("api.weather.example"));
    assert!(!policy.isolate_settings(Path::new("/data")).allow_net);
}

#[test]
fn hosts_without_the_capability_are_a_refused_manifest_not_a_silent_grant() {
    let err = resolve(r#""network":{"hosts":["api.weather.example"]}"#).unwrap_err();
    assert!(err.contains("does not request the net capability"), "{err}");
}

#[test]
fn a_host_may_not_be_a_wildcard_a_url_a_port_or_a_scheme() {
    for host in ["*.example", "https://api.example", "api.example/path", "api.example:8443", "..", ".api.example"] {
        let body = format!(r#""capabilities":["net"],"network":{{"hosts":["{host}"]}}"#);
        assert!(resolve(&body).is_err(), "host {host:?} should be refused");
    }
}

// ------------------------------------------------------------------ quotas

#[test]
fn a_quota_above_the_hosts_ceiling_is_clamped_not_granted() {
    let body = r#""storage":{"max_bytes":999999999},"compute":{"instruction_budget":999999999999,"memory_bytes":999999999}"#;
    let limits = open_limits();
    let policy = admit_and_resolve(&manifest_with(body), BUNDLE, &limits, &RefuseAllSignatures).unwrap();
    assert_eq!(policy.storage_bytes, limits.max_storage_bytes);
    assert_eq!(policy.instruction_budget, limits.max_instruction_budget);
    assert_eq!(policy.memory_bytes, limits.max_memory_bytes);
}

#[test]
fn a_smaller_quota_is_honoured() {
    let policy = resolve(r#""storage":{"max_bytes":4096}"#).unwrap();
    assert_eq!(policy.storage_bytes, 4096);
}

// ---------------------------------------------------------------- identity

#[test]
fn an_id_may_not_navigate_the_filesystem() {
    for id in ["../escape", ".hidden", "Weather", "with space", ""] {
        let json = manifest_with("").replace(r#""id":"weather""#, &format!(r#""id":"{id}""#));
        assert!(
            admit_and_resolve(&json, BUNDLE, &open_limits(), &RefuseAllSignatures).is_err(),
            "id {id:?} should be refused"
        );
    }
}

#[test]
fn the_jail_is_one_directory_per_app() {
    let policy = resolve("").unwrap();
    assert_eq!(policy.jail_root(Path::new("/data/apps")), Path::new("/data/apps/weather"));
}

// ------------------------------------------------------------------- agent

#[test]
fn full_access_is_not_expressible_in_a_manifest() {
    let err = resolve(r#""agent":{"profile":"full-access"}"#).unwrap_err();
    assert!(err.contains("manifest is not valid"), "{err}");
}

#[test]
fn an_agent_may_only_use_tools_the_host_offers_contained_apps() {
    let err = resolve(r#""agent":{"profile":"read-only","tools":["shell"]}"#).unwrap_err();
    assert!(err.contains("does not offer contained apps"), "{err}");
}

#[test]
fn the_agents_workspace_is_the_apps_own_jail_and_nothing_else_is_readable() {
    let body = r#""capabilities":["net"],"network":{"hosts":["api.weather.example"]},
        "agent":{"profile":"workspace-write-never-ask","tools":["net.fetch"],"max_iterations":3}"#;
    let policy = resolve(body).unwrap();
    let session = policy.session_profile(Path::new("/data/apps")).unwrap();
    assert_eq!(session.workspace, Path::new("/data/apps/weather"));
    assert!(session.read_allow_paths.is_empty());
    assert_eq!(session.mode, "workspace-write-never");
    assert_eq!(session.max_iterations, 3);
    // The agent reaches exactly the hosts the app does.
    assert_eq!(session.hosts, vec!["api.weather.example".to_string()]);
}

#[test]
fn an_agents_budget_is_clamped_like_any_other_quota() {
    let limits = open_limits();
    let body = r#""agent":{"profile":"read-only","max_iterations":9999,"token_budget":99999999}"#;
    let policy = admit_and_resolve(&manifest_with(body), BUNDLE, &limits, &RefuseAllSignatures).unwrap();
    let agent = policy.agent.unwrap();
    assert_eq!(agent.max_iterations, limits.max_iterations);
    assert_eq!(agent.token_budget, limits.max_token_budget);
}

#[test]
fn every_agent_session_carries_the_app_it_acts_for() {
    let policy = resolve(r#""agent":{"profile":"read-only"}"#).unwrap();
    let session = policy.session_profile(Path::new("/data/apps")).unwrap();
    assert_eq!(session.session_id, "weather.agent");
    assert_eq!(session.provenance.app_id, "weather");
    assert_eq!(session.provenance.app_version, "1.0.0");
}

// ------------------------------------------------- the two containers agree

#[test]
fn the_isolate_and_the_session_come_from_the_same_declaration() {
    let body = r#""capabilities":["storage","net","prompt"],"network":{"hosts":["a.example","b.example"]},
        "storage":{"max_bytes":8192},"agent":{"profile":"workspace-write","tools":["storage.read","net.fetch"]}"#;
    let policy = resolve(body).unwrap();
    let isolate = policy.isolate_settings(Path::new("/data/apps"));
    let session = policy.session_profile(Path::new("/data/apps")).unwrap();
    assert_eq!(isolate.jail_root, session.workspace, "the agent writes where the app writes");
    assert_eq!(isolate.storage_quota, 8192);
    assert!(isolate.allow_net);
    assert!(isolate.host_prompts);
    assert_eq!(session.hosts.len(), 2);
    assert_eq!(isolate.capabilities, vec!["net".to_string(), "prompt".to_string(), "storage".to_string()]);
}

#[test]
fn a_directory_bundle_is_admitted_by_its_precomputed_digest() {
    let digest = bundle_digest(BUNDLE);
    let policy =
        admit_and_resolve_dir(&manifest_with(""), &digest, &open_limits(), &RefuseAllSignatures).unwrap();
    assert_eq!(policy.app_id, "weather");
    // And a digest for different content is still refused.
    let err = admit_and_resolve_dir(&manifest_with(""), &bundle_digest(b"other"), &open_limits(), &RefuseAllSignatures)
        .unwrap_err();
    assert!(err.contains("does not match the manifest"), "{err}");
}

// ------------------------------------------------------- host services

#[test]
fn each_published_assistant_service_is_admitted_by_its_exact_name() {
    for service in OCTOS_SERVICES {
        let policy = resolve(&format!(r#""capabilities":["{service}"]"#)).unwrap();
        assert!(policy.allows(service), "{service}");
        assert_eq!(policy.capabilities.len(), 1, "{service} grants nothing else");
    }
}

#[test]
fn an_assistant_prefix_or_invented_service_is_refused() {
    for name in ["octos.", "octos.session", "octos.admin", "octos.peer.prepare", "octos.session.open.all", "Octos.turn.start"] {
        let err = resolve(&format!(r#""capabilities":["{name}"]"#)).unwrap_err();
        assert!(err.contains("unknown capability"), "{name}: {err}");
    }
}

#[test]
fn reading_assistant_history_does_not_grant_starting_a_turn() {
    let policy = resolve(r#""capabilities":["octos.session.open","octos.session.history"]"#).unwrap();
    assert!(policy.allows("octos.session.history"));
    assert!(!policy.allows("octos.turn.start"));
    assert!(!policy.allows("octos.turn.interrupt"));
    assert!(policy.agent.is_none(), "a service grant is not an app agent");
}

#[test]
fn matrix_services_are_exact_and_reading_does_not_grant_sending() {
    let policy = resolve(r#""capabilities":["matrix.read_messages","matrix.profile"]"#).unwrap();
    assert!(policy.allows("matrix.read_messages"));
    assert!(!policy.allows("matrix.send_message"));
    let err = resolve(r#""capabilities":["matrix.admin"]"#).unwrap_err();
    assert!(err.contains("unknown capability"), "{err}");
}

#[test]
fn the_store_says_what_a_service_grant_allows() {
    let manifest = AppManifest::parse(&manifest_with(
        r#""capabilities":["matrix.read_messages","matrix.send_message","octos.session.history"]"#,
    ))
    .unwrap();
    let lines = privacy_summary(&manifest);
    assert!(lines.iter().any(|l| l.starts_with("Reads from your Matrix account")), "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("Acts on your Matrix account")), "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("cannot ask it to work")), "{lines:?}");
    let manifest = AppManifest::parse(&manifest_with(r#""capabilities":["octos.turn.start"]"#)).unwrap();
    assert!(privacy_summary(&manifest).iter().any(|l| l.contains("keys stay with the device")));
}

#[test]
fn glance_is_its_own_capability_and_implies_nothing_else() {
    let policy = resolve(r#""capabilities":["glance"]"#).unwrap();
    assert!(policy.allows("glance"));
    assert_eq!(policy.capabilities.len(), 1, "glance grants nothing else");
    assert!(!policy.allows("news") && !policy.allows("net") && !policy.allows("storage"));
    assert!(!policy.may_prompt);
    // Only the exact name: a method or a near name is not a grant.
    for name in ["glance.", "glance.publish", "glance.list", "Glance", "glances"] {
        let err = resolve(&format!(r#""capabilities":["{name}"]"#)).unwrap_err();
        assert!(err.contains("unknown capability"), "{name}: {err}");
    }
}

#[test]
fn model_is_its_own_capability_and_implies_nothing_else() {
    let policy = resolve(r#""capabilities":["model"]"#).unwrap();
    assert!(policy.allows("model"));
    assert_eq!(policy.capabilities.len(), 1, "model grants nothing else");
    // Not the provider manager, not the assistant, not the network.
    assert!(!policy.allows("llm") && !policy.allows("net") && !policy.allows("octos.turn.start"));
    assert!(!policy.may_prompt);
    for name in ["model.", "model.complete", "Model", "models"] {
        let err = resolve(&format!(r#""capabilities":["{name}"]"#)).unwrap_err();
        assert!(err.contains("unknown capability"), "{name}: {err}");
    }
    let manifest = AppManifest::parse(&manifest_with(r#""capabilities":["model"]"#)).unwrap();
    assert!(privacy_summary(&manifest).iter().any(|l| l.contains("AI provider you configured")));
}
