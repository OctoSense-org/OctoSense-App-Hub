//! The privileged operator's review decision, bound to runtime-checked bytes.
//! A submission endpoint must supply the authenticated reviewer identity itself;
//! accepting these fields from a publisher would not establish review authority.
use crate::{Entry, GateReport, runtime::RuntimeEvidence};
use serde::Serialize;
use std::path::Path;

#[derive(Clone, Serialize)]
pub struct OperatorReview { pub reviewer: String, pub review_id: String }

/// No deserialization or caller-supplied passed flag can create this proof.
pub struct ApprovedRelease {
    pub(crate) entry: Entry,
    pub(crate) id: String,
    evidence: RuntimeEvidence,
    review: OperatorReview,
}
impl ApprovedRelease {
    pub fn new(evidence: RuntimeEvidence, gate: &GateReport, publisher: &str, publisher_key: &str,
        repository: &str, commit: &str, review: OperatorReview) -> Result<Self, String> {
        for value in [&review.reviewer, &review.review_id] {
            if value.trim().is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
                return Err("a recorded operator reviewer and review ID (1–128 characters) are required".into());
            }
        }
        evidence.verify_bundle(evidence.bundle())?;
        let mut entry = crate::entry_for(evidence.bundle(), gate, publisher, publisher_key, repository, commit, "")?;
        entry.artifact.clear();
        let id = blake3::hash(&serde_json::to_vec(&serde_json::json!({
            "entry":entry,"review":review,"validation":evidence
        })).map_err(|e| e.to_string())?).to_hex().to_string();
        entry.artifact = format!("artifacts/{id}.bundle");
        Ok(Self { entry, id, evidence, review })
    }
    pub fn id(&self) -> &str { &self.id }

    pub(crate) fn stage(&self, root: &Path, private_state: &Path, hook: &impl Fn(&str) -> Result<(), String>) -> Result<(), String> {
        self.evidence.verify_bundle(self.evidence.bundle())?;
        let pack = crate::pack_dir(self.evidence.bundle())?;
        let pack_bytes = serde_json::to_vec(&pack).map_err(|e| e.to_string())?;
        let artifact = root.join(&self.entry.artifact);
        let artifacts = root.join("artifacts");
        std::fs::create_dir_all(&artifacts).map_err(|e| e.to_string())?;
        if std::fs::symlink_metadata(&artifacts).map_err(|e| e.to_string())?.file_type().is_symlink() {
            return Err("artifact directory may not be a symlink".into());
        }
        crate::release::sync_parent(&artifacts)?;
        if let Ok(meta) = std::fs::symlink_metadata(&artifact) {
            if !meta.is_dir() { return Err("immutable artifact is not a real directory".into()); }
            self.evidence.verify_bundle(&artifact)?;
            hook("before-existing-artifact-sync")?;
            sync_bundle(&artifact)?;
            crate::release::sync_parent(&artifact)?;
        } else {
            let mut nonce = [0u8; 16];
            rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, &mut nonce).map_err(|e| e.to_string())?;
            let staging = private_state.join(format!("artifact-{}", hex::encode(nonce)));
            struct Staging(std::path::PathBuf);
            impl Drop for Staging { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }
            let staging = Staging(staging);
            crate::unpack(&pack, &staging.0)?;
            self.evidence.verify_bundle(&staging.0)?;
            sync_bundle(&staging.0)?;
            hook("before-artifact")?;
            // The catalog lock serializes all writers; an existing immutable
            // destination is verified above, never overwritten. State/staging
            // must be on the same filesystem as the public artifact root.
            std::fs::rename(&staging.0, &artifact).map_err(|e| format!("cannot atomically publish artifact (state and public root must share a filesystem): {e}"))?;
            hook("after-artifact-rename")?;
            crate::release::sync_parent(&artifact)?;
        }
        hook("before-pack")?;
        publish_file(&root.join(format!("{}.pack.json", self.entry.artifact)), &pack_bytes, private_state, hook, "pack")?;
        hook("before-validation-record")?;
        let mut proof = serde_json::to_value(&self.evidence).map_err(|e| e.to_string())?;
        proof["review"] = serde_json::to_value(&self.review).map_err(|e| e.to_string())?;
        proof["approval_id"] = self.id.clone().into();
        publish_file(&root.join(format!("{}.validation.json", self.entry.artifact)), &serde_json::to_vec(&proof).map_err(|e| e.to_string())?, private_state, hook, "validation-record")?;
        self.evidence.verify_bundle(&artifact)?;
        Ok(())
    }
}

fn publish_file(path: &Path, bytes: &[u8], private_state: &Path, hook: &impl Fn(&str) -> Result<(), String>, kind: &str) -> Result<(), String> {
    if path.exists() {
        let found = crate::admission::read_bounded(path, bytes.len() as u64)?;
        if found != bytes { return Err(format!("immutable artifact conflict at {}", path.display())); }
        hook(&format!("before-existing-{kind}-sync"))?;
        std::fs::File::open(path).and_then(|f| f.sync_all()).map_err(|e| e.to_string())?;
        return crate::release::sync_parent(path);
    }
    let mut nonce = [0u8; 16];
    rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, &mut nonce).map_err(|e| e.to_string())?;
    let temporary = private_state.join(format!("artifact-{}.tmp", hex::encode(nonce)));
    let result = (|| {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().create_new(true).write(true).open(&temporary).map_err(|e| e.to_string())?;
        file.write_all(bytes).and_then(|_| file.sync_all()).map_err(|e| e.to_string())?;
        std::fs::hard_link(&temporary, path).map_err(|e| format!("create-only artifact publication failed: {e}"))?;
        hook(&format!("after-{kind}-link"))?;
        crate::release::sync_parent(path)
    })();
    let _ = std::fs::remove_file(temporary);
    result
}

pub(crate) fn sync_bundle(root: &Path) -> Result<(), String> {
    for file in crate::admission::inventory(root)? {
        let path = root.join(file.path);
        std::fs::File::open(&path).and_then(|f| f.sync_all()).map_err(|e| e.to_string())?;
        let mut parent = path.parent();
        while let Some(directory) = parent {
            std::fs::File::open(directory).and_then(|f| f.sync_all()).map_err(|e| e.to_string())?;
            if directory == root { break; }
            parent = directory.parent();
        }
    }
    Ok(())
}
