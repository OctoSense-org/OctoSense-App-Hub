mod common;

use common::Fixture;
use std::{fs, process::Command};

fn hub(fixture: &Fixture, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_hub"))
        .current_dir(&fixture.root)
        .args(args)
        .output()
        .expect("hub runs")
}

#[test]
fn help_is_read_only_for_commands_that_would_read_or_create_files() {
    let fixture = Fixture::new();
    let before = fs::read_dir(&fixture.root).unwrap().count();
    for args in [vec![], vec!["help"], vec!["--help"], vec!["check", "--help"],
                 vec!["keygen", "--help"], vec!["keygen", "not-created.key", "-h"]] {
        let result = hub(&fixture, &args);
        assert!(result.status.success(), "{args:?}: {}", String::from_utf8_lossy(&result.stderr));
        let text = String::from_utf8_lossy(&result.stdout);
        assert!(text.contains("OctoSense app hub"));
        assert!(text.contains("scan permits unsigned") && text.contains("--publisher-key"));
    }
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), before);
}

#[test]
fn keygen_creates_a_valid_key_and_never_replaces_an_existing_file() {
    let fixture = Fixture::new();
    let created = hub(&fixture, &["keygen", "new.key"]);
    assert!(created.status.success());
    let private = fs::read(fixture.root.join("new.key")).unwrap();
    assert_eq!(private.len(), 64);
    let public = hub(&fixture, &["pubkey", "new.key"]);
    assert!(public.status.success());
    assert_eq!(public.stdout, created.stdout);
    let refused = hub(&fixture, &["keygen", "new.key"]);
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
    assert_eq!(fs::read(fixture.root.join("new.key")).unwrap(), private);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(fixture.root.join("new.key")).unwrap().permissions().mode() & 0o777, 0o600);
    }
}

#[cfg(unix)]
#[test]
fn keygen_refuses_symlinks_without_changing_the_target() {
    let fixture = Fixture::new();
    let target = fixture.root.join("existing.key");
    fs::write(&target, "existing private material").unwrap();
    std::os::unix::fs::symlink(&target, fixture.root.join("alias.key")).unwrap();
    let result = hub(&fixture, &["keygen", "alias.key"]);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert_eq!(fs::read_to_string(target).unwrap(), "existing private material");
}

#[test]
fn invalid_commands_and_missing_positionals_fail_without_writing() {
    let fixture = Fixture::new();
    let before = fs::read_dir(&fixture.root).unwrap().count();
    for args in [vec!["unknown"], vec!["keygen"], vec!["keygen", "--bad"], vec!["check"]] {
        let result = hub(&fixture, &args);
        assert!(!result.status.success(), "{args:?}");
        assert!(result.stdout.is_empty());
    }
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), before);
}

#[test]
fn signed_scan_needs_the_documented_publisher_key() {
    let fixture = Fixture::new();
    assert!(!hub(&fixture, &["scan", "bundle"]).status.success());
    let key = format!("publisher-one={}", fixture.publisher.public_hex());
    let result = hub(&fixture, &["scan", "bundle", "--publisher-key", &key]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert!(String::from_utf8_lossy(&result.stdout).contains("questions for one"));
}
