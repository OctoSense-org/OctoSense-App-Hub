mod common;
use common::*;
use std::fs;

#[test]
fn runnable_card_fixture_passes() {
    let f = Fixture::new();
    let report = f.report(None);
    assert!(report.passed(), "{}", report.render());
}

#[test]
fn missing_card_is_refused() {
    let mut f = Fixture::new();
    fs::remove_file(f.bundle.join("page.card")).unwrap();
    f.sign();
    assert!(!f.report(None).passed(), "a listing without an app must not be admitted");
}

#[test]
fn missing_kit_is_refused() {
    let mut f = Fixture::new();
    fs::remove_dir_all(f.bundle.join("kit")).unwrap();
    f.sign();
    assert!(!f.report(None).passed(), "the runtime requires the kit closure");
}

#[test]
fn malformed_card_data_and_utf8_are_refused() {
    for (name, bytes) in [("page.card", b"not a card".as_slice()), ("page.data.json", b"{".as_slice()), ("page.card", &[0xff])] {
        let mut f = Fixture::new();
        fs::write(f.bundle.join(name), bytes).unwrap();
        f.sign();
        assert!(!f.report(None).passed(), "{name} must be valid input to the runtime");
    }
}

#[test]
fn undeclared_kit_component_is_refused() {
    let mut f = Fixture::new();
    let path = f.bundle.join("kit/native/light/kit.json");
    let mut pack: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    pack["components"].as_object_mut().unwrap().remove("title");
    fs::write(path, serde_json::to_vec(&pack).unwrap()).unwrap();
    f.sign();
    assert!(!f.report(None).passed());
}

#[test]
fn fake_or_truncated_png_is_refused() {
    for bytes in [b"not a screenshot".as_slice(), b"\x89PNG\r\n\x1a\n".as_slice()] {
        let mut f = Fixture::new();
        fs::write(f.bundle.join("screen.png"), bytes).unwrap();
        f.sign();
        assert!(!f.report(None).passed(), "an extension is not image validation");
    }
}

#[test]
fn svg_external_resources_are_refused() {
    let mut f = Fixture::new();
    fs::write(f.bundle.join("icon.svg"), r#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><image href="https://example.test/image.png"/></svg>"#).unwrap();
    f.sign();
    assert!(!f.report(None).passed());
}

#[test]
fn malformed_svg_is_refused() {
    let mut f = Fixture::new();
    fs::write(f.bundle.join("icon.svg"), "not svg").unwrap();
    f.sign();
    assert!(!f.report(None).passed());
}

#[test]
fn svg_mixed_local_and_external_css_urls_are_refused() {
    for value in ["fill:url(#local);filter:url(https://example.test/a)", "filter:URL (https://example.test/a)", r"filter:u\72l(https://example.test/a)"] {
        let mut f = Fixture::new();
        fs::write(f.bundle.join("icon.svg"), format!(r#"<svg width="64" height="64"><rect style="{value}"/></svg>"#)).unwrap();
        f.sign();
        assert!(!f.report(None).passed(), "{value}");
    }
}

#[test]
fn kit_expansion_is_bounded_before_cloning_component_styles() {
    let mut f = Fixture::new();
    let path = f.bundle.join("kit/native/light/kit.json");
    let mut pack: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    // Each token is small, but the style resolves it repeatedly. Keeping this
    // fixture modest tests the budget without exhausting the test process.
    pack["tokens"]["large"] = serde_json::json!({"value": "x".repeat(200_000)});
    for index in 0..90 { pack["components"]["title"]["style"][format!("p{index}")] = serde_json::json!({"$token":"large"}); }
    fs::write(path, serde_json::to_vec(&pack).unwrap()).unwrap();
    f.sign();
    assert!(!f.report(None).passed(), "small kit source must not allow unbounded expanded output");
}

#[test]
fn missing_local_resource_has_a_stable_code_and_path() {
    let mut f = Fixture::new();
    let path = f.bundle.join("page.data.json");
    let mut data: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    data["$kit"]["placements"]["panel"]["layout"]["src"] = "missing.png".into();
    fs::write(path, serde_json::to_vec(&data).unwrap()).unwrap();
    f.sign();
    let report = f.report(None);
    assert!(report.findings.iter().any(|e| e.check == "resource-invalid" && e.path.as_deref() == Some("page.data.json/$kit/placements/panel/layout/src")), "{}", report.render());
}

#[test]
fn displayed_url_is_not_a_resource_reference_but_remains_conservatively_denied() {
    let mut f = Fixture::new();
    fs::write(f.bundle.join("extra.json"), r#"{"message":"Visit https://example.test"}"#).unwrap();
    f.sign();
    let report = f.report(None);
    assert!(report.resources.iter().any(|r| matches!(r.kind, octosense_app_hub::admission::ResourceKind::DisplayUrl)));
    assert!(!report.findings.iter().any(|r| r.check == "resource-invalid"));
    assert!(report.findings.iter().any(|r| r.check == "assets"), "network-policy conformance is still required before relaxing URL checks");
}

#[test]
fn image_dimensions_are_bounded_before_decode() {
    let mut f = Fixture::new();
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(4097, 1).write_to(&mut bytes, image::ImageFormat::Png).unwrap();
    fs::write(f.bundle.join("screen.png"), bytes.into_inner()).unwrap();
    f.sign();
    assert!(!f.report(None).passed());
}

#[test]
fn launcher_icon_must_be_square() {
    let mut f = Fixture::new();
    let icon = f.bundle.join("icon.svg");
    fs::write(&icon, fs::read_to_string(&icon).unwrap().replace("height=\"64\"", "height=\"32\"")).unwrap();
    f.sign();
    assert!(!f.report(None).passed());
}

#[test]
fn metadata_limits_reject_large_files_many_entries_and_deep_directories() {
    use octosense_app_hub::admission::*;
    let f = Fixture::new();
    let huge = f.bundle.join("large.png");
    fs::File::create(&huge).unwrap().set_len(9 * 1024 * 1024).unwrap();
    assert!(inventory(&f.bundle).unwrap_err().contains("size limit"));
    fs::remove_file(huge).unwrap();
    let deep = f.bundle.join("deep");
    fs::create_dir_all((0..MAX_DEPTH+1).fold(deep.clone(), |p, _| p.join("d"))).unwrap();
    assert!(inventory(&f.bundle).unwrap_err().contains("depth limit"));
    fs::remove_dir_all(deep).unwrap();
    let many = f.bundle.join("many");
    fs::create_dir(&many).unwrap();
    for i in 0..MAX_ENTRIES { fs::write(many.join(format!("{i}.txt")), "").unwrap(); }
    assert!(inventory(&f.bundle).unwrap_err().contains("file count limit"));
}

#[test]
fn local_server_reports_match() {
    let mut f = Fixture::new();
    fs::write(f.bundle.join("screen.png"), "fake").unwrap();
    f.sign();
    let before = octosense_app_policy::digest_dir(&f.bundle).unwrap();
    let report = f.report(None);
    let cli = std::process::Command::new(env!("CARGO_BIN_EXE_hub")).args([
        "check", f.bundle.to_str().unwrap(), "--json", "--publisher-key", &format!("publisher-one={}", f.publisher.public_hex()),
    ]).output().unwrap();
    assert!(!cli.status.success());
    let json: serde_json::Value = serde_json::from_slice(&cli.stdout).expect("machine-readable gate output");
    assert_eq!(json["findings"], serde_json::to_value(&report.findings).unwrap());
    assert_eq!(before, octosense_app_policy::digest_dir(&f.bundle).unwrap(), "checks must not write into the bundle");
}

#[test]
fn pack_limits_apply_before_writing_staged_files() {
    use octosense_app_hub::{Pack, pack_dir, unpack};
    use base64::Engine;
    let f = Fixture::new();
    let encoded = base64::engine::general_purpose::STANDARD.encode(vec![0; 9 * 1024 * 1024]);
    let pack = Pack { schema: 1, files: [("large.png".into(), encoded)].into() };
    let out = f.root.join("staged");
    assert!(unpack(&pack, &out).is_err());
    assert!(!out.join("large.png").exists());
    fs::File::create(f.bundle.join("large.png")).unwrap().set_len(9 * 1024 * 1024).unwrap();
    assert!(pack_dir(&f.bundle).is_err());
}

#[cfg(unix)]
#[test]
fn unpack_cannot_follow_an_existing_staging_symlink() {
    use octosense_app_hub::{Pack, unpack};
    use base64::Engine;
    let f = Fixture::new();
    let out = f.root.join("staged");
    fs::create_dir(&out).unwrap();
    std::os::unix::fs::symlink(&f.bundle, out.join("escape")).unwrap();
    let pack = Pack { schema: 1, files: [("escape/owned.txt".into(), base64::engine::general_purpose::STANDARD.encode(b"x"))].into() };
    assert!(unpack(&pack, &out).is_err());
    assert!(!f.bundle.join("owned.txt").exists());
}

#[test]
fn data_fields_and_kit_prop_names_are_not_resource_loads() {
    let mut f = Fixture::new();
    fs::write(f.bundle.join("extra.json"), r#"{"src":"database column","image":"description"}"#).unwrap();
    let path = f.bundle.join("kit/native/light/kit.json");
    let mut pack: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    pack["components"]["image-example"] = serde_json::json!({"props":{"src":"src"},"slot":false,"style":{"t":"image"}});
    fs::write(path, serde_json::to_vec(&pack).unwrap()).unwrap();
    f.sign();
    assert!(f.report(None).passed(), "schema property names and arbitrary data are not asset paths");
}

#[test]
fn native_theme_overlay_does_not_require_native_placements() {
    let mut f = Fixture::new();
    // Semantic Card shape used by the renderer's own l0_packs test. A native
    // theme overlay may coexist with legacy kit modules without placements.
    fs::write(f.bundle.join("page.card"), "theme light\nview root Surface { TextBody(text: \"Source type\") }").unwrap();
    fs::write(f.bundle.join("page.data.json"), "{}").unwrap();
    f.sign();
    assert!(f.report(None).passed(), "the runtime worker selects and verifies the actual rendering branch");
}

#[test]
fn native_cli_failures_preserve_machine_readable_findings() {
    for structural in [true, false] {
        let mut f = Fixture::new();
        if structural { fs::remove_file(f.bundle.join("page.card")).unwrap(); f.sign(); }
        let result = std::process::Command::new(env!("CARGO_BIN_EXE_hub")).args([
            "test", f.bundle.to_str().unwrap(), "--json", "--publisher-key", &format!("publisher-one={}", f.publisher.public_hex()),
            "--validator", f.root.join("missing-worker").to_str().unwrap(),
        ]).output().unwrap();
        assert!(!result.status.success());
        let json: serde_json::Value = serde_json::from_slice(&result.stdout).expect("failed validation must produce JSON");
        assert_eq!(json["passed"], false);
        if structural { assert_eq!(json["findings"], serde_json::to_value(f.report(None).findings).unwrap()); }
        else { assert_eq!(json["findings"][0]["check"], "runtime-validation-failed"); assert!(json["findings"][0]["detail"].as_str().unwrap().contains("unavailable")); }
    }
}
