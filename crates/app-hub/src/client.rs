//! The device's side: what a store app does before it shows or installs
//! anything (ADR 0003 §6).
//!
//! Everything here is refusal-first. A catalog that does not verify is not a
//! catalog. An entry that is withdrawn is not installable, and if it is
//! already installed it is not runnable. A bundle whose bytes do not match
//! the entry never reaches a jail.
use crate::index::{Catalog, Entry};
use crate::signing::{verify_catalog, PublisherKeys};
use octosense_app_policy::{digest_dir, AppPolicy, HostLimits, SignatureVerifier};
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

/// How stale a cached catalog may be before installs stop. Running apps are
/// unaffected: the point is that a device kept offline cannot become a place
/// revocation never reaches.
pub const CATALOG_FRESHNESS_DAYS: u64 = 14;

/// Where installed bundles live: `<app data root>/.bundles`, beside the
/// apps' storage. An app's storage is `<root>/<id>/` and it can write there,
/// so its reviewed bytes are kept elsewhere. A name starting with `.` is
/// never an app id, so this is no app's storage.
pub const INSTALLS_DIR: &str = ".bundles";

/// An app's installation: `bundle/`, and the staging an update uses.
pub fn install_root(app_data_root: &Path, app_id: &str) -> PathBuf {
    app_data_root.join(INSTALLS_DIR).join(app_id)
}

/// The installed bundle the store verifies and launches from.
pub fn installed_bundle_dir(app_data_root: &Path, app_id: &str) -> PathBuf {
    install_root(app_data_root, app_id).join("bundle")
}

/// Move every install made before [`INSTALLS_DIR`] out of its app's
/// storage. Idempotent and cheap when there is nothing to move.
pub fn adopt_legacy_installs(app_data_root: &Path) -> Result<(), String> {
    let entries = match std::fs::read_dir(app_data_root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("cannot read {}: {e}", app_data_root.display())),
    };
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let Some(id) = name.to_str() else { continue };
        if id.starts_with('.') || !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        adopt_legacy_install(app_data_root, id)?;
    }
    Ok(())
}

/// Move one app's install from its storage (`<root>/<id>/bundle`, the
/// layout before [`INSTALLS_DIR`]) to [`installed_bundle_dir`]. An update
/// interrupted in the old layout is settled first, as the store's recovery
/// would; an install already in the new place wins over a stale old copy.
/// Nothing found is followed through a symlink.
pub fn adopt_legacy_install(app_data_root: &Path, app_id: &str) -> Result<(), String> {
    let legacy = app_data_root.join(app_id);
    let (bundle, previous, next) = (legacy.join("bundle"), legacy.join(".bundle-previous"), legacy.join(".bundle-next"));
    let is_dir = |path: &Path| std::fs::symlink_metadata(path).is_ok_and(|m| m.is_dir());
    let remove = |path: &Path| match std::fs::symlink_metadata(path) {
        Ok(m) if m.is_dir() => std::fs::remove_dir_all(path),
        Ok(_) => std::fs::remove_file(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    };
    let fail = |what: &str, e: std::io::Error| format!("cannot move {app_id}'s install out of its storage ({what}): {e}");
    remove(&next).map_err(|e| fail("an interrupted download", e))?;
    if is_dir(&previous) && !is_dir(&bundle) {
        remove(&bundle).map_err(|e| fail("a stray entry", e))?;
        std::fs::rename(&previous, &bundle).map_err(|e| fail("the last complete bundle", e))?;
    } else {
        remove(&previous).map_err(|e| fail("a previous bundle", e))?;
    }
    if !is_dir(&bundle) {
        return remove(&bundle).map_err(|e| fail("a stray entry", e));
    }
    let target = installed_bundle_dir(app_data_root, app_id);
    if std::fs::symlink_metadata(&target).is_ok() {
        return remove(&bundle).map_err(|e| fail("a stale copy", e));
    }
    std::fs::create_dir_all(install_root(app_data_root, app_id)).map_err(|e| fail("its new place", e))?;
    std::fs::rename(&bundle, &target).map_err(|e| fail("the bundle", e))
}

/// Days from a catalog's `published` date (ISO 8601, date part) to `today`.
/// None when either date does not parse: an unreadable date is stale.
pub fn days_between(published: &str, today: &str) -> Option<u64> {
    fn ordinal(date: &str) -> Option<i64> {
        let mut parts = date.get(..10)?.split('-');
        let y: i64 = parts.next()?.parse().ok()?;
        let m: i64 = parts.next()?.parse().ok()?;
        let d: i64 = parts.next()?.parse().ok()?;
        if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
            return None;
        }
        // Days since a fixed epoch, via the civil-from-days inverse.
        let (y, m) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let doy = (153 * m + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        Some(era * 146_097 + doe)
    }
    let a = ordinal(published)?;
    let b = ordinal(today)?;
    Some((b - a).max(0) as u64)
}

#[derive(Clone)]
pub struct Store {
    /// The anchor this build trusts. Shipped in the binary.
    anchor_public_hex: String,
    /// Where installed apps live: one directory per app id.
    app_data_root: PathBuf,
    limits: HostLimits,
    catalog: Option<Catalog>,
    host_api_versions: BTreeMap<String, u32>,
    github_catalog_required: bool,
}

/// An installed release, verified and copied out of its jail, ready to
/// run (`launch.rs`). Dropping it removes the copy.
pub struct PreparedLaunch {
    pub policy: AppPolicy,
    pub manifest: octosense_app_policy::AppManifest,
    snapshot: crate::launch::LaunchSnapshot,
}

impl PreparedLaunch {
    /// The verified bytes to run: never the installed directory itself.
    pub fn bundle(&self) -> &Path {
        self.snapshot.bundle()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Availability {
    Installable,
    Installed { version: String },
    /// Installed, but the catalog no longer offers this version.
    Withdrawn { reason: String },
}

#[derive(Clone, Debug)]
pub struct Listing {
    pub app_id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    /// From the manifest: what the app will be allowed to do.
    pub permissions: Vec<String>,
    /// From the manifest: how it treats data and the network.
    pub privacy: Vec<String>,
    /// What the publisher wrote, as reviewed. None for an entry admitted
    /// before listings existed.
    pub about: Option<octosense_app_policy::Listing>,
    /// The hub's artifact path for this version, relative to the catalog:
    /// where its icon and screenshots are fetched from before install.
    pub artifact: String,
    pub availability: Availability,
    /// The installed release and an offered update, as separate facts.
    pub lifecycle: AppAvailability,
}

/// Opening belongs to the installed release; an offered update is a separate
/// fact and never changes what the installed release may do.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppAvailability {
    pub installed_version: Option<String>,
    pub can_open: bool,
    pub update_version: Option<String>,
    pub unavailable_reason: Option<String>,
}

impl Store {
    pub fn new(anchor_public_hex: &str, app_data_root: &Path, limits: HostLimits) -> Self {
        Store {
            anchor_public_hex: anchor_public_hex.to_string(),
            app_data_root: app_data_root.to_path_buf(),
            limits,
            catalog: None,
            host_api_versions: BTreeMap::new(),
            github_catalog_required: false,
        }
    }

    /// Bind this device's implemented API versions. Empty by default: an older
    /// embedding cannot accidentally claim new services merely by linking the contract.
    pub fn with_host_api_versions(mut self, versions: BTreeMap<String, u32>) -> Self {
        self.host_api_versions = versions;
        self
    }

    /// Select the GitHub-attested v2 channel. This is a host decision, never a
    /// field supplied by a downloaded catalog. Legacy documents are refused
    /// even before this instance has seen its first valid v2 catalog.
    pub fn with_github_catalog(mut self) -> Self {
        self.github_catalog_required = true;
        self.catalog = None;
        self
    }

    /// Reuse this host's authenticated state for a staging install directory.
    /// This does not accept a catalog or change its trust mode or permissions.
    pub fn for_install_root(&self, app_data_root: &Path) -> Self {
        let mut staged = self.clone();
        staged.app_data_root = app_data_root.to_path_buf();
        staged
    }

    /// Accept a catalog if it verifies and is not older than the one held.
    /// The sequence check is what stops a replayed catalog un-withdrawing an
    /// app that was pulled.
    pub fn accept_catalog(&mut self, json: &str) -> Result<(), String> {
        let catalog = if self.github_catalog_required {
            crate::github_catalog::verify_document(json.as_bytes())?.into_catalog()
        } else {
            let catalog: Catalog = serde_json::from_str(json).map_err(|e| format!("catalog is not valid: {e}"))?;
            verify_catalog(&catalog, &self.anchor_public_hex)?;
            catalog
        };
        self.accept_authenticated_catalog(catalog)
    }

    fn accept_authenticated_catalog(&mut self, catalog: Catalog) -> Result<(), String> {
        if catalog.schema != crate::index::CATALOG_SCHEMA {
            return Err(format!("catalog schema {} is not {}", catalog.schema, crate::index::CATALOG_SCHEMA));
        }
        if let Some(held) = &self.catalog {
            if catalog.sequence < held.sequence {
                return Err(format!(
                    "refusing catalog {} older than the one held ({})",
                    catalog.sequence, held.sequence
                ));
            }
            if catalog.sequence == held.sequence && catalog.signing_bytes()? != held.signing_bytes()? {
                return Err("refusing different catalog contents at the same sequence".into());
            }
        }
        self.catalog = Some(catalog);
        Ok(())
    }

    /// Verify first, then atomically persist the complete document (proof and
    /// sequence included), and only then replace this instance's accepted state.
    /// Hosts must keep v2 and legacy channel cache paths separate.
    pub fn accept_catalog_and_cache(&mut self, json: &str, path: &Path) -> Result<(), String> {
        use fs2::FileExt;
        let lock_path = path.with_extension("lock");
        let mut options = std::fs::OpenOptions::new();
        options.create(true).read(true).write(true).truncate(false);
        #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
        if std::fs::symlink_metadata(&lock_path).is_ok_and(|m| !m.is_file()) {
            return Err("catalog cache lock is not a regular file".into());
        }
        let lock = options.open(&lock_path).map_err(|_| "cannot open catalog cache lock")?;
        lock.try_lock_exclusive().map_err(|_| "catalog cache is busy")?;
        let mut next = self.clone();
        // Re-read under the lock: another instance may have accepted a newer
        // catalog since this worker last refreshed its in-memory Store.
        match std::fs::symlink_metadata(path) {
            Ok(_) => next.accept_catalog(&crate::admission::read_text(path, crate::github_catalog::MAX_DOCUMENT_BYTES as u64)?)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {},
            Err(_) => return Err("cannot read current catalog cache".into()),
        }
        next.accept_catalog(json)?;
        crate::github_catalog::write_atomic(path, json.as_bytes())?;
        *self = next;
        Ok(())
    }

    pub fn catalog(&self) -> Option<&Catalog> {
        self.catalog.as_ref()
    }

    /// How old the held catalog is on `today` (ISO date). None without one.
    pub fn catalog_age_days(&self, today: &str) -> Option<u64> {
        let catalog = self.catalog.as_ref()?;
        Some(days_between(&catalog.published, today).unwrap_or(u64::MAX))
    }

    /// Whether installs are allowed on `today`: the catalog must exist and be
    /// within the freshness window. Running what is installed is never gated
    /// by this; only taking on something new is.
    pub fn installs_allowed(&self, today: &str) -> Result<(), String> {
        match self.catalog_age_days(today) {
            None => Err("no catalog has been accepted".into()),
            Some(age) if age > CATALOG_FRESHNESS_DAYS => Err(format!(
                "the catalog is {age} days old; installs pause until the hub is reachable again"
            )),
            Some(_) => Ok(()),
        }
    }

    /// Everything to show, newest entry per app, with what is installed.
    pub fn listings(&self) -> Vec<Listing> {
        let Some(catalog) = &self.catalog else { return Vec::new() };
        let mut out: Vec<Listing> = Vec::new();
        // Newest entry per app: the catalog appends, so walk it backwards.
        for entry in catalog.entries.iter().rev() {
            if out.iter().any(|l| l.app_id == entry.app_id()) {
                continue;
            }
            let lifecycle = self.app_availability(entry.app_id());
            let availability = match (&lifecycle.installed_version, lifecycle.can_open, &entry.status) {
                (Some(version), true, _) => Availability::Installed { version: version.clone() },
                (Some(_), false, _) => Availability::Withdrawn { reason: lifecycle.unavailable_reason.clone().unwrap_or_default() },
                (None, _, crate::index::Status::Withdrawn(reason)) => Availability::Withdrawn { reason: reason.clone() },
                (None, _, _) => Availability::Installable,
            };
            out.push(Listing {
                app_id: entry.app_id().to_string(),
                name: entry.manifest.name.clone(),
                version: entry.version().to_string(),
                publisher: entry.publisher.clone(),
                permissions: entry.permissions_summary(),
                privacy: octosense_app_policy::privacy_summary_with_tools(&entry.manifest, &entry.tools),
                about: entry.listing.clone(),
                artifact: entry.artifact.clone(),
                availability,
                lifecycle,
            });
        }
        out
    }

    /// Case-insensitive search over the name, the id and the publisher.
    pub fn search(&self, query: &str) -> Vec<Listing> {
        let needle = query.trim().to_ascii_lowercase();
        self.listings()
            .into_iter()
            .filter(|l| {
                needle.is_empty()
                    || l.name.to_ascii_lowercase().contains(&needle)
                    || l.app_id.to_ascii_lowercase().contains(&needle)
                    || l.publisher.to_ascii_lowercase().contains(&needle)
                    || l.about.as_ref().is_some_and(|a| {
                        a.category.contains(&needle)
                            || a.subtitle.to_ascii_lowercase().contains(&needle)
                            || a.keywords.iter().any(|k| k.to_ascii_lowercase().contains(&needle))
                    })
            })
            .collect()
    }

    /// The publisher keys this catalog carries. The catalog is signed, so
    /// these are as trustworthy as the catalog itself, which is what lets a
    /// device check a publisher's signature offline.
    pub fn publisher_keys(&self) -> PublisherKeys {
        let mut keys = PublisherKeys::new();
        if let Some(catalog) = &self.catalog {
            for entry in &catalog.entries {
                if !entry.publisher_key.is_empty() {
                    keys = keys.with(&entry.publisher, &entry.publisher_key);
                }
            }
        }
        keys
    }

    /// The newest entry for an app: what the store offers to install or
    /// update to. A launch uses [`Store::release`], the installed version's.
    pub fn entry(&self, app_id: &str) -> Option<&Entry> {
        self.catalog.as_ref()?.entries.iter().rev().find(|e| e.app_id() == app_id)
    }

    /// The catalog's entry for exactly this version of an app.
    pub fn release(&self, app_id: &str, version: &str) -> Result<&Entry, String> {
        let catalog = self.catalog.as_ref().ok_or("no catalog has been accepted")?;
        let mut matches = catalog.entries.iter().filter(|e| e.app_id() == app_id && e.version() == version);
        let entry = matches
            .next()
            .ok_or_else(|| format!("{app_id} {version} is not in the verified catalog; refresh the App Hub"))?;
        if matches.next().is_some() {
            return Err(format!("{app_id} {version} has ambiguous catalog entries; refresh the App Hub"));
        }
        Ok(entry)
    }

    /// Whether the installed release opens, and whether an update is on
    /// offer. Hashes the installed bundle: interactive clients call it on a
    /// worker.
    pub fn app_availability(&self, app_id: &str) -> AppAvailability {
        let installed_version = self.installed_version(app_id);
        let launch = installed_version.as_ref().map(|_| self.may_run(app_id));
        let update_version = self
            .entry(app_id)
            .filter(|e| e.status.is_offered() && installed_version.as_deref().is_some_and(|v| v != e.version()))
            .map(|e| e.version().to_string());
        AppAvailability {
            installed_version,
            can_open: launch.as_ref().is_some_and(|result| result.is_ok()),
            update_version,
            unavailable_reason: launch.and_then(Result::err),
        }
    }

    /// Outside the app's storage: [`installed_bundle_dir`].
    pub fn install_dir(&self, app_id: &str) -> PathBuf {
        installed_bundle_dir(&self.app_data_root, app_id)
    }

    /// The version installed for this app, read from its own copy of the
    /// manifest rather than from anything the catalog says.
    pub fn installed_version(&self, app_id: &str) -> Option<String> {
        let json = crate::launch::read_manifest(&self.install_dir(app_id)).ok()?;
        octosense_app_policy::AppManifest::parse(&json).ok().map(|m| m.version)
    }

    /// Admit a bundle that has been unpacked at `staged`, then move it into
    /// place. Returns what the app will be allowed to do.
    ///
    /// The order matters: the catalog entry must offer this version, the
    /// staged bytes must hash to what the entry says, the manifest must be
    /// the entry's manifest, and only then does the policy resolve.
    pub fn install_staged(
        &self,
        app_id: &str,
        staged: &Path,
        verifier: &dyn SignatureVerifier,
        today: &str,
    ) -> Result<AppPolicy, String> {
        self.installs_allowed(today)?;
        // System apps ship with the build; a download may never take one's id,
        // and with it that app's jail.
        if app_id.starts_with("os.") {
            return Err(format!("{app_id} names a system app, which no store may install"));
        }
        // Nor a native app's id or the host's own names: the jail, and on
        // the device its tools and consent, are keyed by the id.
        octosense_app_policy::check_reserved_id(app_id)?;
        let entry = self.entry(app_id).ok_or_else(|| format!("{app_id} is not in the catalog"))?;
        if let crate::index::Status::Withdrawn(reason) = &entry.status {
            return Err(format!("{app_id} has been withdrawn: {reason}"));
        }
        let digest = digest_dir(staged)?;
        if !digest.eq_ignore_ascii_case(&entry.manifest.integrity.bundle_blake3) {
            return Err(format!(
                "the downloaded bundle hashes to {digest}, the catalog says {}",
                entry.manifest.integrity.bundle_blake3
            ));
        }
        let staged_manifest = std::fs::read_to_string(staged.join(octosense_app_policy::MANIFEST_FILE))
            .map_err(|e| format!("the bundle has no manifest: {e}"))?;
        let staged_manifest = octosense_app_policy::AppManifest::parse(&staged_manifest)?;
        // All of it, not only the id and version: the permissions are in it.
        if !same_manifest(&staged_manifest, &entry.manifest)? {
            return Err("the bundle's manifest is not the one the catalog admitted".into());
        }
        if let Some(signature) = &entry.manifest.integrity.signature {
            verifier.verify(&signature.key_id, &signature.value, &entry.manifest.signing_bytes()?)?;
        }
        // And against the key the catalog records, whatever the caller passed.
        let staged_json = serde_json::to_string(&staged_manifest).map_err(|e| e.to_string())?;
        let policy = octosense_app_policy::admit_and_resolve_dir(&staged_json, &digest, &self.limits, &self.publisher_keys())?;

        let target = self.install_dir(app_id);
        entry.manifest.check_host_apis(&self.host_api_versions)?;
        if target.exists() {
            std::fs::remove_dir_all(&target).map_err(|e| format!("cannot replace the installed app: {e}"))?;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        copy_tree(staged, &target)?;
        Ok(policy)
    }

    /// Remove an app and everything it stored. The jail goes with it: an
    /// uninstall that leaves data behind is not an uninstall. So does its
    /// installation, kept apart from the jail.
    pub fn remove(&self, app_id: &str) -> Result<(), String> {
        for dir in [self.app_data_root.join(app_id), install_root(&self.app_data_root, app_id)] {
            if dir.exists() {
                std::fs::remove_dir_all(&dir).map_err(|e| format!("cannot remove {app_id}: {e}"))?;
            }
        }
        Ok(())
    }

    /// May this installed app run right now? The installed version must be
    /// a release the catalog offers (a withdrawal of it stops it; a newer
    /// version on offer does not), and its manifest and bytes must still be
    /// the ones admitted. The policy is that release's own.
    pub fn may_run(&self, app_id: &str) -> Result<AppPolicy, String> {
        let entry = self.installed_release(app_id)?;
        let bundle = self.install_dir(app_id);
        crate::launch::check_bounds(&bundle)?;
        self.verify_release_bundle(entry, &bundle)
    }

    /// Verify the installed release and copy it out of the app's jail, for a
    /// launch to run. Hashes and copies: interactive clients call it on a
    /// worker, then [`Store::validate_prepared_launch`] before starting.
    pub fn prepare_launch(&self, app_id: &str) -> Result<PreparedLaunch, String> {
        let entry = self.installed_release(app_id)?;
        let snapshot = crate::launch::LaunchSnapshot::copy(&self.install_dir(app_id), &self.app_data_root)?;
        let policy = self.verify_release_bundle(entry, snapshot.bundle())?;
        Ok(PreparedLaunch { policy, manifest: entry.manifest.clone(), snapshot })
    }

    /// Recheck a prepared launch against the catalog held now: the release
    /// may have been withdrawn, or replaced, while it was being prepared.
    pub fn validate_prepared_launch(&self, prepared: &PreparedLaunch) -> Result<(), String> {
        let entry = self.release(&prepared.manifest.id, &prepared.manifest.version)?;
        if let crate::index::Status::Withdrawn(reason) = &entry.status {
            return Err(format!("{} {} was withdrawn: {reason}", entry.app_id(), entry.version()));
        }
        if !same_manifest(&entry.manifest, &prepared.manifest)? {
            return Err("the release changed while it was opening; try again".into());
        }
        octosense_app_policy::verify::verify_manifest(&entry.manifest, &self.publisher_keys())?;
        Ok(())
    }

    fn installed_release(&self, app_id: &str) -> Result<&Entry, String> {
        if self.catalog.is_none() {
            return Err("no catalog has been accepted".into());
        }
        let installed = self
            .installed_version(app_id)
            .ok_or_else(|| format!("{app_id} is not installed, or its manifest is unreadable"))?;
        self.release(app_id, &installed)
    }

    fn verify_release_bundle(&self, entry: &Entry, bundle: &Path) -> Result<AppPolicy, String> {
        entry.manifest.check_host_apis(&self.host_api_versions)?;
        if let crate::index::Status::Withdrawn(reason) = &entry.status {
            return Err(format!("{} {} was withdrawn: {reason}", entry.app_id(), entry.version()));
        }
        let text = crate::launch::read_manifest(bundle)?;
        if !same_manifest(&octosense_app_policy::AppManifest::parse(&text)?, &entry.manifest)? {
            return Err("the installed manifest is not the reviewed one; reinstall this app".into());
        }
        let digest = digest_dir(bundle)?;
        octosense_app_policy::admit_and_resolve_dir(&text, &digest, &self.limits, &self.publisher_keys())
            .map_err(|e| format!("{e}; reinstall this app"))
    }
}

/// The same manifest, field for field.
fn same_manifest(a: &octosense_app_policy::AppManifest, b: &octosense_app_policy::AppManifest) -> Result<bool, String> {
    Ok(serde_json::to_value(a).map_err(|e| e.to_string())? == serde_json::to_value(b).map_err(|e| e.to_string())?)
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        let target = to.join(entry.file_name());
        if kind.is_symlink() {
            return Err("a bundle may not hold a symlink".into());
        }
        if kind.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod catalog_cache_tests {
    use super::*;
    use crate::HubKey;

    fn fixture() -> (PathBuf, HubKey, HubKey) {
        let anchor = HubKey::generate();
        let working = HubKey::generate();
        let root = std::env::temp_dir().join(format!("hub-catalog-cache-{}", &working.public_hex()[..20]));
        std::fs::create_dir(&root).unwrap();
        (root, anchor, working)
    }
    fn signed(sequence: u64, anchor: &HubKey, working: &HubKey) -> String {
        let mut catalog = Catalog::new(sequence, &crate::today(), vec![]);
        working.sign_catalog(&mut catalog, &anchor.certify(&working.public_hex()).unwrap()).unwrap();
        serde_json::to_string(&catalog).unwrap()
    }

    #[test]
    fn v2_required_rejects_valid_legacy_even_before_first_success() {
        let (root, anchor, working) = fixture();
        let mut store = Store::new(&anchor.public_hex(), &root, HostLimits::default()).with_github_catalog();
        let legacy = signed(3, &anchor, &working);
        assert!(store.accept_catalog(&legacy).is_err());
        assert!(store.accept_catalog(r#"{"schema":2,"kind":"github-attested-catalog","catalog":"e30=","attestation":{}}"#).is_err());
        assert!(store.accept_catalog(&legacy).is_err());
        assert!(store.catalog().is_none());
        let cache = root.join("catalog-v2.json");
        std::fs::write(&cache, &legacy).unwrap();
        assert!(store.accept_catalog_and_cache(&legacy, &cache).is_err());
        assert_eq!(std::fs::read_to_string(cache).unwrap(), legacy);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn authenticated_catalogs_cannot_roll_back_or_equivocate_at_a_sequence() {
        let (root, anchor, _) = fixture();
        let mut store = Store::new(&anchor.public_hex(), &root, HostLimits::default()).with_github_catalog();
        // This isolates the post-verification state gate. Real Sigstore proof
        // acceptance is exercised separately against a public upstream fixture.
        let current = Catalog::new(5, "2026-10-08", vec![]);
        store.accept_authenticated_catalog(current.clone()).unwrap();
        store.accept_authenticated_catalog(current).unwrap();
        assert!(store.accept_authenticated_catalog(Catalog::new(4, "2026-10-08", vec![])).is_err());
        assert!(store.accept_authenticated_catalog(Catalog::new(5, "2026-10-09", vec![])).is_err());
        assert_eq!(store.catalog().unwrap().sequence, 5);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn staged_install_retains_authenticated_state_and_v2_requirement() {
        let (root, anchor, working) = fixture();
        let mut original = Store::new(&anchor.public_hex(), &root, HostLimits::default())
            .with_host_api_versions(BTreeMap::from([("model.image".into(), 1)]))
            .with_github_catalog();
        // Isolate cloning the post-verification state, without inventing a
        // successful production App Hub Sigstore proof for this unit test.
        original.accept_authenticated_catalog(Catalog::new(5, &crate::today(), vec![])).unwrap();
        let staging = root.join("staging");
        let mut staged = original.for_install_root(&staging);
        assert_eq!(staged.app_data_root, staging);
        assert_eq!(original.app_data_root, root);
        assert_eq!(staged.catalog().unwrap().sequence, 5);
        assert_eq!(staged.host_api_versions, original.host_api_versions);
        assert_eq!(staged.anchor_public_hex, original.anchor_public_hex);
        assert!(staged.github_catalog_required);
        assert!(staged.accept_catalog(&signed(6, &anchor, &working)).is_err());
        assert!(staged.accept_authenticated_catalog(Catalog::new(4, &crate::today(), vec![])).is_err());
        assert_eq!(original.catalog().unwrap().sequence, 5);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn atomic_cache_retains_proof_and_rechecks_persisted_high_watermark() {
        let (root, anchor, working) = fixture();
        let path = root.join("catalog.json");
        let mut first = Store::new(&anchor.public_hex(), &root, HostLimits::default());
        first.accept_catalog_and_cache(&signed(4, &anchor, &working), &path).unwrap();
        let mut stale = Store::new(&anchor.public_hex(), &root, HostLimits::default());
        assert!(stale.accept_catalog_and_cache(&signed(3, &anchor, &working), &path).is_err());
        assert!(stale.catalog().is_none());
        let before = std::fs::read(&path).unwrap();
        assert!(first.accept_catalog_and_cache("{broken", &path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(first.catalog().unwrap().sequence, 4);
        let impossible = root.join("missing").join("catalog.json");
        assert!(first.accept_catalog_and_cache(&signed(5, &anchor, &working), &impossible).is_err());
        assert_eq!(first.catalog().unwrap().sequence, 4);
        assert!(std::fs::read_dir(&root).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cache_contention_fails_bounded_without_overwriting() {
        use fs2::FileExt;
        let (root, anchor, working) = fixture();
        let path = root.join("catalog.json");
        let lock = std::fs::OpenOptions::new().create_new(true).write(true).open(path.with_extension("lock")).unwrap();
        lock.lock_exclusive().unwrap();
        let mut store = Store::new(&anchor.public_hex(), &root, HostLimits::default());
        assert_eq!(store.accept_catalog_and_cache(&signed(1, &anchor, &working), &path).unwrap_err(), "catalog cache is busy");
        assert!(!path.exists());
        assert!(store.catalog().is_none());
        drop(lock);
        std::fs::remove_dir_all(root).unwrap();
    }
}
