//! Owned launch bytes. Updating an installation cannot change a running app.
use std::{fs, io::Read, path::{Path, PathBuf}};

pub(crate) const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_FILES: usize = 2048;
const MAX_DEPTH: usize = 32;

pub(crate) struct LaunchSnapshot {
    root: PathBuf,
}

impl LaunchSnapshot {
    pub(crate) fn copy(source: &Path, app_root: &Path) -> Result<Self, String> {
        let parent = app_root.join(".running");
        fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
        if fs::symlink_metadata(&parent).map_err(|e| e.to_string())?.file_type().is_symlink() {
            return Err("launch snapshot root may not be a symlink".into());
        }
        let mut nonce = [0u8; 16];
        rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, &mut nonce).map_err(|e| e.to_string())?;
        let root = parent.join(hex::encode(nonce));
        fs::create_dir(&root).map_err(|e| e.to_string())?;
        let snapshot = Self { root };
        let mut budget = CopyBudget { remaining: crate::gate::MAX_BUNDLE_BYTES + MAX_MANIFEST_BYTES, files: 0 };
        copy_bounded(source, &snapshot.root, 0, &mut budget)?;
        Ok(snapshot)
    }
    pub(crate) fn bundle(&self) -> &Path { &self.root }
}

impl Drop for LaunchSnapshot {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); }
}

struct CopyBudget { remaining: u64, files: usize }

fn copy_bounded(source: &Path, target: &Path, depth: usize, budget: &mut CopyBudget) -> Result<(), String> {
    if depth > MAX_DEPTH { return Err("installed bundle exceeds the directory depth limit".into()); }
    if fs::symlink_metadata(source).map_err(|e| e.to_string())?.file_type().is_symlink() {
        return Err("installed bundle may not hold a symlink".into());
    }
    fs::create_dir_all(target).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        budget.files += 1;
        if budget.files > MAX_FILES { return Err("installed bundle exceeds the file count limit".into()); }
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        let destination = target.join(entry.file_name());
        if kind.is_dir() {
            copy_bounded(&entry.path(), &destination, depth + 1, budget)?;
        } else if kind.is_file() {
            let max = if depth == 0 && entry.file_name() == octosense_app_policy::MANIFEST_FILE {
                budget.remaining.min(MAX_MANIFEST_BYTES)
            } else { budget.remaining };
            let mut bytes = Vec::new();
            fs::File::open(entry.path()).map_err(|e| e.to_string())?.take(max + 1)
                .read_to_end(&mut bytes).map_err(|e| e.to_string())?;
            if bytes.len() as u64 > max { return Err("installed bundle exceeds the launch size limit".into()); }
            budget.remaining -= bytes.len() as u64;
            fs::write(destination, bytes).map_err(|e| e.to_string())?;
        } else { return Err("installed bundle may only hold regular files and directories".into()); }
    }
    Ok(())
}

pub(crate) fn read_manifest(bundle: &Path) -> Result<String, String> {
    let mut text = String::new();
    fs::File::open(bundle.join(octosense_app_policy::MANIFEST_FILE)).map_err(|e| format!("cannot read installed manifest: {e}"))?
        .take(MAX_MANIFEST_BYTES + 1).read_to_string(&mut text).map_err(|e| e.to_string())?;
    if text.len() as u64 > MAX_MANIFEST_BYTES { return Err("installed manifest exceeds the size limit".into()); }
    Ok(text)
}
