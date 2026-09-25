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
        if fs::symlink_metadata(&catalog_path).map_err(|e| e.to_string())?.file_type().is_symlink() {
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
        let catalog: Catalog = read_json(&catalog_path)?;
        verify_catalog(&catalog, anchor)?;
        if catalog.schema != crate::CATALOG_SCHEMA { return Err("unsupported catalog schema".into()); }
        fs::create_dir_all(state).map_err(|e| e.to_string())?;
        if fs::symlink_metadata(state).map_err(|e| e.to_string())?.file_type().is_symlink() { return Err("release state may not be a symlink".into()); }
        let state = state.canonicalize().map_err(|e| e.to_string())?;
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

    fn renew_with_hook(mut self, expected: u64, idempotency_key: &str, now: &str, signer: &impl CatalogSigner,
        hook: impl Fn(&str) -> Result<(), String>) -> Result<Catalog, String> {
        date(now)?;
        date(&self.catalog.published)?;
        if self.catalog.published.as_str() > now { return Err("catalog date is in the future; check the operator clock".into()); }
        if idempotency_key.is_empty() || idempotency_key.len() > 256 { return Err("idempotency key must be 1 to 256 bytes".into()); }
        let request_hash = blake3::hash(format!("renew\n{}\n{expected}", self.anchor).as_bytes()).to_hex().to_string();
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
            let mut intended = self.catalog.clone();
            intended.sequence = self.catalog.sequence.checked_add(1).ok_or("catalog sequence exhausted")?;
            intended.published = intent.catalog.published.clone();
            if intended.signing_bytes()? != intent.catalog.signing_bytes()? { return Err("prepared renewal does not preserve the current releases".into()); }
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
            return self.install(&record, &head_path, &hook);
        }
        self.check_head(head.as_ref(), &current_hash)?;
        if expected != self.catalog.sequence { return Err(format!("stale expected sequence {expected}; current sequence is {}", self.catalog.sequence)); }
        let next = if let Some(intent) = pending { intent.catalog } else {
            let mut next = self.catalog.clone();
            next.sequence = next.sequence.checked_add(1).ok_or("catalog sequence exhausted")?;
            next.published = now.into();
            next.signature = None;
            next.key = None;
            encoded(&next, MAX_CATALOG_BYTES)?;
            let intent = Intent { idempotency_hash, request_hash: request_hash.clone(), previous_hash: current_hash.clone(),
                previous_sequence: self.catalog.sequence, catalog: next.clone() };
            hook("before-intent")?;
            immutable_json(&intent_path, &intent)?;
            next
        };
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
fn sync_parent(path: &Path) -> Result<(), String> {
    File::open(path.parent().ok_or("missing parent")?).and_then(|f| f.sync_all()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HubKey, signing::LocalCatalogSigner};
    struct Fixture { root: PathBuf, anchor: HubKey, working: HubKey }
    impl Fixture {
        fn new() -> Self {
            let mut nonce = [0u8; 16];
            rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, &mut nonce).unwrap();
            let root = std::env::temp_dir().join(format!("hub-release-{}", hex::encode(nonce)));
            fs::create_dir(&root).unwrap();
            let f = Self { root, anchor: HubKey::generate(), working: HubKey::generate() };
            let mut catalog = Catalog::new(7, "2026-09-01", vec![]);
            f.working.sign_catalog(&mut catalog, &f.anchor.certify(&f.working.public_hex()).unwrap()).unwrap();
            create_json(&f.root.join("catalog.json"), &catalog).unwrap();
            f
        }
        fn open(&self) -> ReleaseStore {
            ReleaseStore::open(&self.root.join("catalog.json"), &self.root.join("private-state"), &self.anchor.public_hex()).unwrap()
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
            let visible: Catalog = read_json(&f.root.join("catalog.json")).unwrap();
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
