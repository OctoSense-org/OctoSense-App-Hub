mod common;
use common::{Fixture, write_catalog};
use octosense_app_hub::*;
use std::{fs, process::Command};

fn renew(f: &Fixture, anchor: &HubKey, working: &HubKey, expected: u64, retry: &str) -> std::process::Output {
    let key = f.root.join("working.key");
    fs::write(&key, hex::encode(working.to_bytes())).unwrap();
    Command::new(env!("CARGO_BIN_EXE_hub")).args([
        "admin", "renew", "--catalog", f.root.join("catalog.json").to_str().unwrap(),
        "--state-dir", f.root.join("release-state").to_str().unwrap(),
        "--anchor", &anchor.public_hex(), "--key", key.to_str().unwrap(),
        "--anchor-cert", &anchor.certify(&working.public_hex()).unwrap(),
        "--expected-sequence", &expected.to_string(), "--idempotency-key", retry,
    ]).output().unwrap()
}

#[test]
fn renewal_advances_sequence_and_date_without_changing_releases() {
    for populated in [false, true] {
        let f = Fixture::new();
        let anchor = HubKey::generate();
        let working = HubKey::generate();
        let entries = if populated { vec![f.entry()] } else { vec![] };
        let mut before = Catalog::new(7, "2026-09-01", entries);
        working.sign_catalog(&mut before, &anchor.certify(&working.public_hex()).unwrap()).unwrap();
        let path = f.root.join("catalog.json");
        write_catalog(&path, &before);
        let output = renew(&f, &anchor, &working, 7, "daily-renewal");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let after: Catalog = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(after.sequence, 8);
        assert_eq!(after.published, today());
        assert_eq!(serde_json::to_value(&after.entries).unwrap(), serde_json::to_value(before.entries).unwrap());
        verify_catalog(&after, &anchor.public_hex()).unwrap();
        assert!(renew(&f, &anchor, &working, 7, "daily-renewal").status.success(), "retry must return the same result");
        assert!(!renew(&f, &anchor, &working, 7, "different-request").status.success(), "stale compare-and-swap must fail");
        assert!(!renew(&f, &anchor, &working, 8, "daily-renewal").status.success(), "an idempotency key cannot change meaning");
    }
}

#[test]
fn renewal_refuses_bad_future_dates_and_invalid_signatures() {
    for (date, invalid) in [("2026-02-30", false), ("9999-01-01", false), ("2026-09-01", true)] {
        let f = Fixture::new();
        let anchor = HubKey::generate();
        let working = HubKey::generate();
        let mut catalog = Catalog::new(1, date, vec![]);
        working.sign_catalog(&mut catalog, &anchor.certify(&working.public_hex()).unwrap()).unwrap();
        if invalid { catalog.sequence += 1; }
        let path = f.root.join("catalog.json");
        write_catalog(&path, &catalog);
        let before = fs::read(&path).unwrap();
        assert!(!renew(&f, &anchor, &working, catalog.sequence, "bad").status.success());
        assert_eq!(fs::read(path).unwrap(), before);
    }
}

#[test]
fn restore_never_replays_an_old_sequence_and_working_keys_can_rotate() {
    let f = Fixture::new();
    let anchor = HubKey::generate();
    let working = HubKey::generate();
    let rotated = HubKey::generate();
    let mut old = Catalog::new(12, "2026-09-01", vec![]);
    working.sign_catalog(&mut old, &anchor.certify(&working.public_hex()).unwrap()).unwrap();
    let path = f.root.join("catalog.json");
    write_catalog(&path, &old);
    assert!(renew(&f, &anchor, &rotated, 12, "rotation").status.success());
    let latest: Catalog = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(latest.sequence, 13);
    assert_eq!(latest.key.unwrap().public, rotated.public_hex());
    write_catalog(&path, &old);
    assert!(!renew(&f, &anchor, &working, 12, "restored").status.success(), "restored catalog must reconcile with durable generation history");
    assert_eq!(serde_json::from_slice::<Catalog>(&fs::read(path).unwrap()).unwrap().sequence, 12);
}
