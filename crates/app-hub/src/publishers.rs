//! Trusted publisher identity, kept apart from a submission's key claims.
//!
//! The catalog is the record: once a release signed by a key is on it, that
//! key is the publisher's, for every later version and for their other apps.
//! A submission names a key; it never redefines one. V1 uses the publisher
//! id as the signature's key id, and has no authorized rotation: history
//! that disagrees with itself needs an operator, not a choice of entry.
//!
//! Unsigned releases (`--allow-unsigned`) put an owner on record but no key.
//! The app's first signed update records the key from then on.
use crate::{Catalog, PublisherKeys};
use octosense_app_policy::{AppManifest, SignatureVerifier};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublisherBinding {
    pub publisher_id: String,
    pub key_id: String,
    pub public_key_hex: String,
}

/// Read-only identity lookup. Authenticated registry or rotation records can
/// implement it later without letting a submission change a key.
pub trait PublisherRegistry {
    fn binding(&self, key_id: &str) -> Option<&PublisherBinding>;
    fn app_owner(&self, app_id: &str) -> Option<&str>;
}

#[derive(Default)]
pub struct CatalogPublishers {
    bindings: BTreeMap<String, PublisherBinding>,
    owners: BTreeMap<String, String>,
}

impl CatalogPublishers {
    /// The bindings and owners `catalog` records. The caller authenticates
    /// the catalog first. Every entry counts, withdrawn ones too, so the
    /// order of entries never picks a key.
    pub fn from_catalog(catalog: &Catalog) -> Result<Self, String> {
        let mut registry = Self::default();
        for entry in &catalog.entries {
            if entry.publisher.is_empty() {
                return Err(format!("{} has no publisher on record; operator reconciliation required", entry.app_id()));
            }
            if let Some(owner) = registry.owners.get(entry.app_id()) {
                if owner != &entry.publisher {
                    return Err(format!("conflicting catalog owners for {}; operator reconciliation required", entry.app_id()));
                }
            }
            if let Some(signature) = &entry.manifest.integrity.signature {
                if signature.key_id != entry.publisher {
                    return Err(format!("{}'s catalog publisher does not match its manifest signature", entry.app_id()));
                }
                if entry.publisher_key.is_empty() {
                    return Err(format!("{} is signed but has no key on record; operator reconciliation required", entry.app_id()));
                }
                let binding = PublisherBinding {
                    publisher_id: entry.publisher.clone(),
                    key_id: signature.key_id.clone(),
                    public_key_hex: entry.publisher_key.to_ascii_lowercase(),
                };
                if registry.bindings.get(&binding.key_id).is_some_and(|previous| previous != &binding) {
                    return Err(format!("conflicting catalog bindings for {:?}; operator reconciliation required", binding.key_id));
                }
                PublisherKeys::new()
                    .with(&binding.key_id, &binding.public_key_hex)
                    .verify(&signature.key_id, &signature.value, &entry.manifest.signing_bytes()?)?;
                registry.bindings.insert(binding.key_id.clone(), binding);
            }
            registry.owners.insert(entry.app_id().to_string(), entry.publisher.clone());
        }
        Ok(registry)
    }

    /// `keys` with every recorded binding added, so a known publisher's
    /// release verifies against its recorded key. A supplied key that
    /// disagrees with one stays in `keys`, and verification refuses both.
    pub fn trusted_keys(&self, mut keys: PublisherKeys) -> PublisherKeys {
        for binding in self.bindings.values() {
            keys = keys.with(&binding.key_id, &binding.public_key_hex);
        }
        keys
    }
}

impl PublisherRegistry for CatalogPublishers {
    fn binding(&self, key_id: &str) -> Option<&PublisherBinding> {
        self.bindings.get(key_id)
    }
    fn app_owner(&self, app_id: &str) -> Option<&str> {
        self.owners.get(app_id).map(String::as_str)
    }
}

/// Check `manifest` against the publishers on record, using only the
/// recorded keys, never a caller's.
pub fn verify_continuity(manifest: &AppManifest, registry: &dyn PublisherRegistry) -> Result<(), String> {
    let owner = registry.app_owner(&manifest.id);
    let Some(signature) = &manifest.integrity.signature else {
        return match owner {
            Some(owner) => Err(format!("{} is already published by {owner:?}; an update must carry that key", manifest.id)),
            None => Ok(()),
        };
    };
    if let Some(owner) = owner {
        if signature.key_id != owner {
            return Err(format!(
                "{} was published by {owner:?}; this version is signed by {:?}. Re-keying is a reviewed change.",
                manifest.id, signature.key_id
            ));
        }
    }
    if let Some(binding) = registry.binding(&signature.key_id) {
        PublisherKeys::new()
            .with(&binding.key_id, &binding.public_key_hex)
            .verify(&signature.key_id, &signature.value, &manifest.signing_bytes()?)
            .map_err(|e| format!("not signed by the key on record for {:?}: {e}", binding.publisher_id))?;
    }
    Ok(())
}
