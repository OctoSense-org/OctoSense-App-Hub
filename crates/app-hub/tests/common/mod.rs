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
