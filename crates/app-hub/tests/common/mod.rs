#![allow(dead_code)]
use octosense_app_hub::*;
use octosense_app_policy::{AppManifest, HostLimits};
use std::{fs, path::{Path, PathBuf}, sync::atomic::{AtomicU64, Ordering}};

pub struct Fixture {
    pub root: PathBuf,
    pub bundle: PathBuf,
    pub publisher: HubKey,
    pub manifest: AppManifest,
}

impl Fixture {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("hub-regression-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&root).unwrap();
        let bundle = root.join("bundle");
        fs::create_dir(&bundle).unwrap();
        fs::write(bundle.join("icon.svg"), r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64"><rect width="64" height="64" fill="#146"/></svg>"##).unwrap();
        // A real, decodable 1x1 PNG; visual quality is tested separately.
        use base64::Engine;
        fs::write(bundle.join("screen.png"), base64::engine::general_purpose::STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=").unwrap()).unwrap();
        fs::write(bundle.join("listing.json"), serde_json::to_vec(&serde_json::json!({
            "schema": 1, "description": "Publisher regression fixture", "category": "utilities",
            "screenshots": ["screen.png"], "icon": "icon.svg", "platforms": ["android"],
            "publisher": {"name": "Example", "support": "https://example.test/support", "privacy_policy_url": "https://example.test/privacy"},
            "age_rating": "all"
        })).unwrap()).unwrap();
        fs::create_dir_all(bundle.join("kit/native/light")).unwrap();
        fs::write(bundle.join("page.card"), include_str!("../fixtures/card/page.card")).unwrap();
        fs::write(bundle.join("page.data.json"), include_str!("../fixtures/card/page.data.json")).unwrap();
        fs::write(bundle.join("kit/native/light/kit.json"), include_str!("../fixtures/card/kit/native/light/kit.json")).unwrap();
        let publisher = HubKey::generate();
        let manifest = AppManifest::parse(r#"{"schema":1,"id":"example-app","version":"1.0.0","name":"Example","integrity":{"bundle_blake3":""}}"#).unwrap();
        let mut fixture = Self { root, bundle, publisher, manifest };
        fixture.sign();
        fixture
    }

    pub fn write_manifest(&self) {
        fs::write(self.bundle.join("manifest.json"), serde_json::to_vec_pretty(&self.manifest).unwrap()).unwrap();
    }

    pub fn sign(&mut self) {
        self.manifest.integrity.bundle_blake3 = octosense_app_policy::digest_dir(&self.bundle).unwrap();
        sign_manifest(&self.publisher, &mut self.manifest, "publisher-one").unwrap();
        self.write_manifest();
    }

    pub fn keys(&self) -> PublisherKeys {
        PublisherKeys::new().with("publisher-one", &self.publisher.public_hex())
    }

    /// Trusted test worker for CLI ownership tests. Real native execution is
    /// covered separately by app-validator/tests/runtime_validation.rs.
    #[cfg(unix)]
    pub fn protocol_worker(&self) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let (digest, manifest_digest) = runtime::identity(&self.bundle).unwrap();
        let report = runtime::RuntimeReport {
            schema: 1, check_version: runtime::CHECK_VERSION, runtime: runtime::CARD_RUNTIME.into(),
            target: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH), digest, manifest_digest,
            checks: vec!["structural".into(), "card-preparation".into(), "native-widget-load".into(), "startup-shutdown".into()],
        };
        let path = self.root.join("protocol-worker");
        fs::write(&path, format!("#!/bin/sh\ncat <<'VALIDATION_JSON'\n{}\nVALIDATION_JSON\n", serde_json::to_string(&report).unwrap())).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }

    pub fn report(&self, previous: Option<&Catalog>) -> GateReport {
        check_bundle(&self.bundle, &HostLimits::default(), &self.keys(), previous).unwrap()
    }

    pub fn entry(&self) -> Entry {
        let report = self.report(None);
        assert!(report.passed(), "{}", report.render());
        entry_for(&self.bundle, &report, "publisher-one", &self.publisher.public_hex(), "", "", "2026-09-25").unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); }
}

pub fn write_catalog(path: &Path, catalog: &Catalog) {
    fs::write(path, serde_json::to_vec_pretty(catalog).unwrap()).unwrap();
}
