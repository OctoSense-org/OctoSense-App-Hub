//! Operator release transactions. The state directory is private durable
//! signing history, backed up separately from the public catalog/artifacts.
use crate::{Catalog, verify_catalog, signing::CatalogSigner};
use serde::{Deserialize, Serialize};
use std::{fs::{self, File, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}};

/// Only release operations can construct a transaction for the signer.
/// Renewal preserves releases exactly; publication must add validated evidence.
pub struct ValidatedCatalog { catalog: Catalog }
impl ValidatedCatalog { pub fn catalog(&self) -> &Catalog { &self.catalog } }

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    request_hash: String,
    previous_hash: String,
    previous_sequence: u64,
    catalog: Catalog,
}

const MAX_RECORD_BYTES: usize = 64 * 1024 * 1024;
// Leave room for signature, key certificate and transaction metadata in records.
const MAX_CATALOG_BYTES: usize = MAX_RECORD_BYTES - 4096;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    idempotency_hash: String,
    request_hash: String,
    previous_hash: String,
    previous_sequence: u64,
    catalog: Catalog,
}

enum Change<'a> {
    Renew,
    Publish(&'a crate::approval::ApprovedRelease),
    Withdraw { app: &'a str, version: &'a str, reason: &'a str },
}
impl Change<'_> {
    fn identity(&self) -> serde_json::Value {
        match self {
            Self::Renew => serde_json::json!({"operation":"renew"}),
            Self::Publish(release) => serde_json::json!({"operation":"publish","approval":release.id}),
            Self::Withdraw { app, version, reason } => serde_json::json!({"operation":"withdraw","app":app,"version":version,"reason":reason}),
        }
    }
    fn candidate(&self, current: &Catalog, now: &str) -> Result<Catalog, String> {
        let mut next = current.clone();
        match self {
            Self::Renew => { if current.published.is_empty() { return Err("renewal requires an existing signed catalog".into()); } }
            Self::Publish(release) => {
                if current.entries.iter().any(|entry| entry.app_id() == release.entry.app_id() && entry.version() == release.entry.version()) {
                    return Err("this app version is already in the catalog".into());
                }
                let registry = crate::publishers::CatalogPublishers::from_catalog(current)?;
                crate::publishers::verify_continuity(&release.entry.manifest, &registry)?;
                let mut entry = release.entry.clone();
                entry.admitted = now.into();
                next.entries.push(entry);
            }
            Self::Withdraw { app, version, reason } => {
                if reason.trim().is_empty() || reason.len() > 4096 { return Err("withdrawal needs a reason of 1 to 4096 bytes".into()); }
                let entry = next.entries.iter_mut().find(|entry| entry.app_id() == *app && entry.version() == *version)
                    .ok_or("withdrawal target is not in the catalog")?;
                entry.status = crate::Status::Withdrawn((*reason).into());
            }
        }
        next.sequence = next.sequence.checked_add(1).ok_or("catalog sequence exhausted")?;
        next.published = now.into();
        next.signature = None;
        next.key = None;
        Ok(next)
    }
    fn stage(&self, store: &ReleaseStore, hook: &impl Fn(&str) -> Result<(), String>) -> Result<(), String> {
        if let Self::Publish(release) = self { release.stage(store.catalog_path.parent().unwrap(), &store.state, hook)?; }
        Ok(())
    }
}

pub struct ReleaseStore {
    catalog_path: PathBuf,
    state: PathBuf,
    anchor: String,
    catalog: Catalog,
    _lock: File,
}

impl ReleaseStore {
    /// Locks the catalog's canonical sibling lock for the entire transaction.
    /// All writers must use this boundary; manual catalog edits are unsupported.
    pub fn open(catalog_path: &Path, state: &Path, anchor: &str) -> Result<Self, String> {
        let parent = catalog_path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
        let parent = parent.canonicalize().map_err(|e| e.to_string())?;
        let catalog_path = parent.join(catalog_path.file_name().ok_or("catalog filename is missing")?);
        if fs::symlink_metadata(&catalog_path).is_ok_and(|meta| meta.file_type().is_symlink()) {
            return Err("catalog pointer may not be a symlink".into());
        }
        let lock_path = catalog_path.with_file_name(format!("{}.release.lock", catalog_path.file_name().unwrap().to_string_lossy()));
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        #[cfg(unix)] {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW).mode(0o600);
        }
        let lock = options.open(&lock_path).map_err(|e| e.to_string())?;
        #[cfg(unix)] {
            use std::os::fd::AsRawFd;
            if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) } != 0 { return Err(std::io::Error::last_os_error().to_string()); }
        }
        #[cfg(not(unix))] { return Err("release transactions require a supported Unix operator runner".into()); }
        let catalog: Catalog = if catalog_path.exists() {
            let catalog = read_json(&catalog_path)?;
            verify_catalog(&catalog, anchor)?;
            catalog
        } else { Catalog::new(0, "", vec![]) };
        if catalog.schema != crate::CATALOG_SCHEMA { return Err("unsupported catalog schema".into()); }
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)] { use std::os::unix::fs::DirBuilderExt; builder.mode(0o700); }
        builder.create(state).map_err(|e| e.to_string())?;
        if fs::symlink_metadata(state).map_err(|e| e.to_string())?.file_type().is_symlink() { return Err("release state may not be a symlink".into()); }
        let state = state.canonicalize().map_err(|e| e.to_string())?;
        if state.starts_with(&parent) { return Err("release state must be outside the public catalog directory".into()); }
        fs::create_dir_all(state.join("requests")).map_err(|e| e.to_string())?;
        sync_parent(&state)?;
        sync_parent(&state.join("requests"))?;
        Ok(Self { catalog_path, state, anchor: anchor.into(), catalog, _lock: lock })
    }

    pub fn catalog(&self) -> &Catalog { &self.catalog }

    /// `now` is supplied by the trusted operator clock, never by an app.
    pub fn renew(self, expected: u64, idempotency_key: &str, now: &str, signer: &impl CatalogSigner) -> Result<Catalog, String> {
        self.renew_with_hook(expected, idempotency_key, now, signer, |_| Ok(()))
    }

    pub fn publish(self, expected: u64, idempotency_key: &str, now: &str, signer: &impl CatalogSigner,
        release: &crate::approval::ApprovedRelease) -> Result<Catalog, String> {
        self.transact(expected, idempotency_key, now, signer, Change::Publish(release), |_| Ok(()))
    }

    pub fn withdraw(self, expected: u64, idempotency_key: &str, now: &str, signer: &impl CatalogSigner,
        app: &str, version: &str, reason: &str) -> Result<Catalog, String> {
        self.transact(expected, idempotency_key, now, signer, Change::Withdraw { app, version, reason }, |_| Ok(()))
    }

    fn renew_with_hook(self, expected: u64, idempotency_key: &str, now: &str, signer: &impl CatalogSigner,
        hook: impl Fn(&str) -> Result<(), String>) -> Result<Catalog, String> {
        self.transact(expected, idempotency_key, now, signer, Change::Renew, hook)
    }

    fn transact(mut self, expected: u64, idempotency_key: &str, now: &str, signer: &impl CatalogSigner,
        change: Change<'_>, hook: impl Fn(&str) -> Result<(), String>) -> Result<Catalog, String> {
        date(now)?;
        if !self.catalog.published.is_empty() { valid_date_at(&self.catalog.published, now)?; }
        if idempotency_key.is_empty() || idempotency_key.len() > 256 { return Err("idempotency key must be 1 to 256 bytes".into()); }
        let request_hash = blake3::hash(&serde_json::to_vec(&serde_json::json!({"change":change.identity(),"anchor":self.anchor,"expected":expected})).map_err(|e| e.to_string())?).to_hex().to_string();
        let idempotency_hash = blake3::hash(idempotency_key.as_bytes()).to_hex().to_string();
        let receipt_path = self.state.join("requests").join(format!("{idempotency_hash}.json"));
        let head_path = self.state.join("head.json");
        let intent_path = self.state.join("pending.json");
        let current_hash = hash(&self.catalog)?;
        let head: Option<Record> = if head_path.exists() { Some(read_json(&head_path)?) } else { None };
        if let Some(head) = &head { verify_catalog(&head.catalog, &self.anchor)?; }
        let mut pending: Option<Intent> = if intent_path.exists() { Some(read_json(&intent_path)?) } else { None };
        if let Some(intent) = &pending {
            // A crash after the pointer swap only left housekeeping. Confirm
            // both the visible and durable generation before clearing it.
            if intent.catalog.sequence == self.catalog.sequence && intent.catalog.signing_bytes()? == self.catalog.signing_bytes()? {
                self.check_head(head.as_ref(), &current_hash)?;
                clear_pending(&intent_path)?;
                pending = None;
            }
        }
        if let Some(intent) = &pending {
            valid_date_at(&intent.catalog.published, now)?;
            if intent.idempotency_hash != idempotency_hash || intent.request_hash != request_hash
                || intent.previous_hash != current_hash || intent.previous_sequence != expected {
                return Err("a prepared generation reserves this sequence; retry its original request before another release".into());
            }
            let intended = change.candidate(&self.catalog, &intent.catalog.published)?;
            if intended.signing_bytes()? != intent.catalog.signing_bytes()? { return Err("prepared transaction does not match this release change".into()); }
        }
        if receipt_path.exists() {
            let record: Record = read_json(&receipt_path)?;
            verify_catalog(&record.catalog, &self.anchor)?;
            valid_date_at(&record.catalog.published, now)?;
            if record.request_hash != request_hash { return Err("idempotency key was already used for another request".into()); }
            if record.catalog.sequence <= self.catalog.sequence {
                self.check_head(head.as_ref(), &current_hash)?;
                if record.catalog.sequence == self.catalog.sequence && hash(&record.catalog)? != current_hash {
                    return Err("request receipt conflicts with this catalog generation".into());
                }
                return Ok(record.catalog);
            }
            let intent = pending.as_ref().ok_or("prepared receipt has no reserved generation; reconciliation required")?;
            if record.previous_sequence != self.catalog.sequence || record.previous_hash != current_hash
                || record.catalog.signing_bytes()? != intent.catalog.signing_bytes()?
                || head.as_ref().is_some_and(|h| h.catalog.sequence > record.catalog.sequence)
                || head.as_ref().is_some_and(|h| h.catalog.sequence == record.catalog.sequence && hash(&h.catalog).ok() != hash(&record.catalog).ok()) {
                return Err("release history requires reconciliation above its recorded sequence".into());
            }
            if head.as_ref().is_some_and(|h| h.catalog.sequence < record.catalog.sequence) {
                self.check_head(head.as_ref(), &current_hash)?;
            }
            change.stage(&self, &hook)?;
            return self.install(&record, &head_path, &hook);
        }
        self.check_head(head.as_ref(), &current_hash)?;
        if expected != self.catalog.sequence { return Err(format!("stale expected sequence {expected}; current sequence is {}", self.catalog.sequence)); }
        // Check identity/continuity before exposing artifacts, and finish
        // staging before reserving a sequence. Refusals cannot publish bytes
        // or block unrelated renewals/withdrawals.
        let new_intent = pending.is_none();
        let next = if let Some(intent) = pending { intent.catalog } else { change.candidate(&self.catalog, now)? };
        encoded(&next, MAX_CATALOG_BYTES)?;
        change.stage(&self, &hook)?;
        if new_intent {
            let intent = Intent { idempotency_hash, request_hash: request_hash.clone(), previous_hash: current_hash.clone(),
                previous_sequence: self.catalog.sequence, catalog: next.clone() };
            hook("before-intent")?;
            immutable_json(&intent_path, &intent)?;
        }
        hook("before-signing")?;
        let signed = signer.sign(&ValidatedCatalog { catalog: next.clone() })?;
        verify_catalog(&signed, &self.anchor)?;
        if signed.signing_bytes()? != next.signing_bytes()? { return Err("signer changed the validated transaction".into()); }
        let record = Record { request_hash, previous_hash: current_hash, previous_sequence: self.catalog.sequence, catalog: signed };
        hook("before-receipt")?;
        immutable_json(&receipt_path, &record)?;
        self.install(&record, &head_path, &hook)
    }

    fn check_head(&self, head: Option<&Record>, current_hash: &str) -> Result<(), String> {
        if let Some(head) = head {
            if head.catalog.sequence != self.catalog.sequence || hash(&head.catalog)? != current_hash {
                return Err("catalog differs from durable release history; reconcile before publishing".into());
            }
        }
        Ok(())
    }

    fn install(&mut self, record: &Record, head_path: &Path, hook: &impl Fn(&str) -> Result<(), String>) -> Result<Catalog, String> {
        hook("before-head")?;
        atomic_json(head_path, record)?;
        hook("before-catalog")?;
        atomic_json(&self.catalog_path, &record.catalog)?;
        self.catalog = record.catalog.clone();
        hook("after-catalog")?;
        clear_pending(&self.state.join("pending.json"))?;
        Ok(self.catalog.clone())
    }
}

fn clear_pending(path: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| e.to_string())?;
    sync_parent(path)
}
fn valid_date_at(value: &str, now: &str) -> Result<(), String> {
    date(value)?;
    if value > now { return Err("prepared catalog date is in the future; check the operator clock".into()); }
    Ok(())
}

fn hash(catalog: &Catalog) -> Result<String, String> {
    Ok(blake3::hash(&serde_json::to_vec(catalog).map_err(|e| e.to_string())?).to_hex().to_string())
}

/// Strict Gregorian YYYY-MM-DD; device freshness remains wire-compatible.
fn date(value: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-'
        || bytes.iter().enumerate().any(|(i, b)| i != 4 && i != 7 && !b.is_ascii_digit()) {
        return Err("date must be YYYY-MM-DD".into());
    }
    let year: u32 = value[..4].parse().map_err(|_| "invalid year")?;
    let month: usize = value[5..7].parse().map_err(|_| "invalid month")?;
    let day: u32 = value[8..].parse().map_err(|_| "invalid day")?;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [31, if leap {29} else {28}, 31,30,31,30,31,31,30,31,30,31];
    if year == 0 || !(1..=12).contains(&month) || day == 0 || day > days[month - 1] { return Err("invalid calendar date".into()); }
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let mut bytes = Vec::new();
    File::open(path).map_err(|e| e.to_string())?.take(MAX_RECORD_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_RECORD_BYTES { return Err("release record exceeds size limit".into()); }
    serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn encoded(value: &impl Serialize, limit: usize) -> Result<Vec<u8>, String> {
    struct Bounded { bytes: Vec<u8>, limit: usize }
    impl Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
                return Err(std::io::Error::other("release record exceeds size limit"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
    let mut writer = Bounded { bytes: Vec::new(), limit };
    serde_json::to_writer(&mut writer, value).map_err(|e| e.to_string())?;
    Ok(writer.bytes)
}

fn create_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let bytes = encoded(value, MAX_RECORD_BYTES)?;
    let mut file = OpenOptions::new().create_new(true).write(true).open(path).map_err(|e| e.to_string())?;
    file.write_all(&bytes).and_then(|_| file.sync_all()).map_err(|e| e.to_string())?;
    sync_parent(path)
}

fn immutable_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let mut nonce = [0u8; 16];
    rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, &mut nonce).map_err(|e| e.to_string())?;
    let temporary = path.with_file_name(format!(".receipt-{}.tmp", hex::encode(nonce)));
    let result = (|| {
        create_json(&temporary, value)?;
        // Link is create-only and exposes fully synced bytes. A crash while
        // writing a temporary cannot leave a truncated idempotency receipt.
        fs::hard_link(&temporary, path).map_err(|e| e.to_string())?;
        sync_parent(path)
    })();
    let _ = fs::remove_file(temporary);
    result
}

fn atomic_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let mut nonce = [0u8; 16];
    rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, &mut nonce).map_err(|e| e.to_string())?;
    let temp = path.with_file_name(format!(".release-{}.tmp", hex::encode(nonce)));
    struct Temporary(PathBuf);
    impl Drop for Temporary { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }
    let temp = Temporary(temp);
    create_json(&temp.0, value)?;
    fs::rename(&temp.0, path).map_err(|e| e.to_string())?;
    sync_parent(path)
}
pub(crate) fn sync_parent(path: &Path) -> Result<(), String> {
    File::open(path.parent().ok_or("missing parent")?).and_then(|f| f.sync_all()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HubKey, signing::LocalCatalogSigner};
    struct Fixture { root: PathBuf, public: PathBuf, anchor: HubKey, working: HubKey }
    impl Fixture {
        fn new() -> Self {
            let mut nonce = [0u8; 16];
            rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, &mut nonce).unwrap();
            let root = std::env::temp_dir().join(format!("hub-release-{}", hex::encode(nonce)));
            fs::create_dir(&root).unwrap();
            let public = root.join("public");
            fs::create_dir(&public).unwrap();
            let f = Self { root, public, anchor: HubKey::generate(), working: HubKey::generate() };
            let mut catalog = Catalog::new(7, "2026-09-01", vec![]);
            f.working.sign_catalog(&mut catalog, &f.anchor.certify(&f.working.public_hex()).unwrap()).unwrap();
            create_json(&f.public.join("catalog.json"), &catalog).unwrap();
            f
        }
        fn open(&self) -> ReleaseStore {
            ReleaseStore::open(&self.public.join("catalog.json"), &self.root.join("private-state"), &self.anchor.public_hex()).unwrap()
        }
    }
    impl Drop for Fixture { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); } }

    #[test]
    fn interruption_at_each_renewal_boundary_keeps_a_valid_generation_and_retry() {
        for point in ["before-intent", "before-signing", "before-receipt", "before-head", "before-catalog", "after-catalog"] {
            let f = Fixture::new();
            let certificate = f.anchor.certify(&f.working.public_hex()).unwrap();
            let signer = LocalCatalogSigner { key: &f.working, anchor_certificate: &certificate };
            let result = f.open().renew_with_hook(7, "operation", "2026-09-25", &signer,
                |stage| if stage == point { Err("injected interruption".into()) } else { Ok(()) });
            assert!(result.is_err());
            let visible: Catalog = read_json(&f.public.join("catalog.json")).unwrap();
            verify_catalog(&visible, &f.anchor.public_hex()).unwrap();
            assert!([7, 8].contains(&visible.sequence));
            let retried = f.open().renew(7, "operation", "2026-09-25", &signer).unwrap();
            assert_eq!(retried.sequence, 8);
            assert_eq!(f.open().catalog().sequence, 8);
        }
    }

    #[test]
    fn concurrent_renewals_cannot_reuse_a_generation() {
        let f = Fixture::new();
        let barrier = std::sync::Barrier::new(4);
        let successes = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4).map(|i| {
                let f = &f;
                let barrier = &barrier;
                scope.spawn(move || {
                    let certificate = f.anchor.certify(&f.working.public_hex()).unwrap();
                    let signer = LocalCatalogSigner { key: &f.working, anchor_certificate: &certificate };
                    barrier.wait();
                    f.open().renew(7, &format!("operation-{i}"), "2026-09-25", &signer).is_ok()
                })
            }).collect();
            handles.into_iter().map(|h| h.join().unwrap() as usize).sum::<usize>()
        });
        assert_eq!(successes, 1);
        assert_eq!(f.open().catalog().sequence, 8);
    }

    #[test]
    fn prepared_generation_blocks_a_different_request_and_future_clock_retry() {
        for different_request in [true, false] {
            let f = Fixture::new();
            let certificate = f.anchor.certify(&f.working.public_hex()).unwrap();
            let signer = LocalCatalogSigner { key: &f.working, anchor_certificate: &certificate };
            assert!(f.open().renew_with_hook(7, "prepared", "2026-09-25", &signer,
                |stage| if stage == "before-head" { Err("interrupted".into()) } else { Ok(()) }).is_err());
            let (key, now) = if different_request { ("different", "2026-09-26") } else { ("prepared", "2026-09-24") };
            let rotated = HubKey::generate();
            let certificate = f.anchor.certify(&rotated.public_hex()).unwrap();
            assert!(f.open().renew(7, key, now, &LocalCatalogSigner { key: &rotated, anchor_certificate: &certificate }).is_err(),
                "a pending generation must reserve its identity and date");
            assert_eq!(f.open().catalog().sequence, 7);
            assert_eq!(f.open().renew(7, "prepared", "2026-09-25", &signer).unwrap().sequence, 8);
        }
    }

    fn approve(f: &crate::test_bundle::Fixture) -> crate::approval::ApprovedRelease {
        let gate = f.report(None);
        let worker = f.protocol_worker();
        let evidence = crate::runtime::validate(&f.bundle, &gate, &worker).unwrap();
        crate::approval::ApprovedRelease::new(evidence, &gate, "publisher-one", &f.publisher.public_hex(), "", "",
            crate::approval::OperatorReview { reviewer: "test-operator".into(), review_id: "review-one".into() }).unwrap()
    }

    #[test]
    fn crash_at_each_publish_boundary_keeps_a_complete_catalog() {
        for point in ["before-intent", "before-artifact", "after-artifact-rename", "before-pack", "after-pack-link", "before-validation-record", "after-validation-record-link", "before-signing", "before-receipt", "before-head", "before-catalog", "after-catalog"] {
            let f = Fixture::new();
            let bundle = crate::test_bundle::Fixture::new();
            let approved = approve(&bundle);
            let certificate = f.anchor.certify(&f.working.public_hex()).unwrap();
            let signer = LocalCatalogSigner { key: &f.working, anchor_certificate: &certificate };
            assert!(f.open().transact(7, "publish", "2026-09-25", &signer, Change::Publish(&approved),
                |stage| if stage == point { Err("interrupted".into()) } else { Ok(()) }).is_err());
            let visible = f.open().catalog().clone();
            verify_catalog(&visible, &f.anchor.public_hex()).unwrap();
            assert!([7, 8].contains(&visible.sequence));
            for entry in &visible.entries {
                let artifact = f.public.join(&entry.artifact);
                assert_eq!(crate::runtime::identity(&artifact).unwrap().0, entry.manifest.integrity.bundle_blake3);
                assert!(f.public.join(format!("{}.pack.json", entry.artifact)).is_file());
                assert!(f.public.join(format!("{}.validation.json", entry.artifact)).is_file());
            }
            assert_eq!(f.open().publish(7, "publish", "2026-09-25", &signer, &approved).unwrap().sequence, 8);
        }
    }

    #[test]
    fn concurrent_publishes_require_rebase_and_keep_both_releases() {
        let f = Fixture::new();
        let first = crate::test_bundle::Fixture::new();
        let mut second = crate::test_bundle::Fixture::new();
        second.manifest.id = "second-app".into();
        second.publisher = HubKey::from_bytes(&first.publisher.to_bytes());
        second.sign();
        let approvals = [approve(&first), approve(&second)];
        let barrier = std::sync::Barrier::new(2);
        let results = std::thread::scope(|scope| {
            let handles: Vec<_> = approvals.iter().enumerate().map(|(i, approval)| {
                let f = &f;
                let barrier = &barrier;
                scope.spawn(move || {
                    let certificate = f.anchor.certify(&f.working.public_hex()).unwrap();
                    let signer = LocalCatalogSigner { key: &f.working, anchor_certificate: &certificate };
                    barrier.wait();
                    f.open().publish(7, &format!("publish-{i}"), "2026-09-25", &signer, approval).is_ok()
                })
            }).collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>()
        });
        assert_eq!(results.iter().filter(|s| **s).count(), 1);
        let i = results.iter().position(|s| !s).unwrap();
        let certificate = f.anchor.certify(&f.working.public_hex()).unwrap();
        let signer = LocalCatalogSigner { key: &f.working, anchor_certificate: &certificate };
        let catalog = f.open().publish(8, &format!("publish-{i}"), "2026-09-25", &signer, &approvals[i]).unwrap();
        assert_eq!(catalog.sequence, 9);
        assert_eq!(catalog.entries.len(), 2);
        let withdrawn = f.open().withdraw(9, "withdraw", "2026-09-25", &signer, "example-app", "1.0.0", "test retirement").unwrap();
        assert_eq!(withdrawn.sequence, 10);
        assert_eq!(withdrawn.entries.len(), 2);
        assert!(!withdrawn.entries.iter().find(|e| e.app_id() == "example-app").unwrap().status.is_offered());
        for entry in &withdrawn.entries { assert!(f.public.join(&entry.artifact).is_dir()); }
    }

    #[test]
    fn preexisting_artifacts_cannot_be_overwritten_or_signed() {
        let f = Fixture::new();
        let bundle = crate::test_bundle::Fixture::new();
        let approved = approve(&bundle);
        let certificate = f.anchor.certify(&f.working.public_hex()).unwrap();
        let signer = LocalCatalogSigner { key: &f.working, anchor_certificate: &certificate };
        // A preexisting destination is immutable, even when it is only a
        // partially written directory from a writer outside the transaction.
        let artifact = f.public.join(&approved.entry.artifact);
        fs::create_dir_all(&artifact).unwrap();
        fs::write(artifact.join("owned.txt"), "must stay untouched").unwrap();
        assert!(f.open().publish(7, "bad-artifact", "2026-09-25", &signer, &approved).is_err());
        assert_eq!(fs::read_to_string(artifact.join("owned.txt")).unwrap(), "must stay untouched");
        assert_eq!(f.open().catalog().sequence, 7);
        assert_eq!(f.open().renew(7, "unrelated-renewal", "2026-09-25", &signer).unwrap().sequence, 8);
    }

    #[test]
    fn retry_cannot_skip_a_failed_existing_artifact_sync() {
        let f = Fixture::new();
        let bundle = crate::test_bundle::Fixture::new();
        let approved = approve(&bundle);
        let certificate = f.anchor.certify(&f.working.public_hex()).unwrap();
        let signer = LocalCatalogSigner { key: &f.working, anchor_certificate: &certificate };
        for failure in ["after-validation-record-link", "before-existing-validation-record-sync"] {
            assert!(f.open().transact(7, "retry-sync", "2026-09-25", &signer, Change::Publish(&approved),
                |stage| if stage == failure { Err("sync interrupted".into()) } else { Ok(()) }).is_err());
            assert_eq!(f.open().catalog().sequence, 7);
        }
        assert_eq!(f.open().publish(7, "retry-sync", "2026-09-25", &signer, &approved).unwrap().sequence, 8);
    }

    #[test]
    fn state_inside_the_catalog_directory_is_refused() {
        let f = Fixture::new();
        assert!(ReleaseStore::open(&f.public.join("catalog.json"), &f.public.join("state"), &f.anchor.public_hex()).is_err());
    }

    #[test]
    fn duplicate_versions_and_conflicting_owners_never_expose_artifacts() {
        for change_owner in [false, true] {
            let f = Fixture::new();
            let mut bundle = crate::test_bundle::Fixture::new();
            let first = approve(&bundle);
            let certificate = f.anchor.certify(&f.working.public_hex()).unwrap();
            let signer = LocalCatalogSigner { key: &f.working, anchor_certificate: &certificate };
            f.open().publish(7, "first", "2026-09-25", &signer, &first).unwrap();
            if change_owner { bundle.manifest.version = "2.0.0".into(); bundle.publisher = HubKey::generate(); }
            else { fs::write(bundle.bundle.join("additional.txt"), "same version, different bytes").unwrap(); }
            bundle.sign();
            let refused = approve(&bundle);
            assert!(f.open().publish(8, "refused", "2026-09-25", &signer, &refused).is_err());
            assert!(!f.public.join(&refused.entry.artifact).exists(), "rejected admission must precede public artifact exposure");
            assert_eq!(f.open().catalog().sequence, 8);
        }
    }

    #[test]
    fn oversized_serialized_records_are_refused_before_creating_a_file() {
        let f = Fixture::new();
        let path = f.root.join("oversized.json");
        assert!(create_json(&path, &"x".repeat(MAX_RECORD_BYTES)).is_err());
        assert!(!path.exists(), "a refused serialization must not leave a partial record");
    }

    #[test]
    fn signer_cannot_replace_the_validated_catalog() {
        let f = Fixture::new();
        struct WrongSigner<'a>(&'a Fixture);
        impl CatalogSigner for WrongSigner<'_> {
            fn sign(&self, transaction: &ValidatedCatalog) -> Result<Catalog, String> {
                let mut result = transaction.catalog().clone();
                result.sequence += 99;
                self.0.working.sign_catalog(&mut result, &self.0.anchor.certify(&self.0.working.public_hex())?)?;
                Ok(result)
            }
        }
        assert!(f.open().renew(7, "bad-signer", "2026-09-25", &WrongSigner(&f)).unwrap_err().contains("signer changed"));
        assert_eq!(f.open().catalog().sequence, 7);
    }
}
