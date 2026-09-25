//! Pack every system app under `system-apps/` for `include_str!`.
//!
//! A bundle's source manifest leaves `integrity.bundle_blake3` empty: the
//! digest is of the bundle's own files, so it is computed here, from exactly
//! the bytes that get packed, and stamped into the packed manifest. A system
//! app can therefore never ship a digest that disagrees with its program.
use std::path::{Path, PathBuf};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../system-apps");
    println!("cargo:rerun-if-changed={}", root.display());
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let mut index = String::from("pub const PACKS: &[(&str, &str, &str)] = &[\n");
    let mut names: Vec<_> = std::fs::read_dir(&root)
        .map(|entries| entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect())
        .unwrap_or_default();
    names.sort();
    for dir in names {
        watch(&dir);
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let staged = out.join("system-apps").join(&name);
        let _ = std::fs::remove_dir_all(&staged);
        copy_tree(&dir, &staged);
        let digest = octosense_app_policy::digest_dir(&staged).expect("digest");
        let manifest_path = staged.join(octosense_app_policy::MANIFEST_FILE);
        let mut manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&manifest_path).expect("manifest")).expect("manifest json");
        manifest["integrity"]["bundle_blake3"] = serde_json::Value::String(digest);
        std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
        let id = manifest["id"].as_str().expect("id").to_string();
        let display = manifest["name"].as_str().expect("name").to_string();
        let pack = octosense_app_hub::pack::pack_dir(&staged).expect("pack");
        let pack_path = out.join(format!("{name}.pack.json"));
        std::fs::write(&pack_path, serde_json::to_string(&pack).unwrap()).unwrap();
        index.push_str(&format!("    ({id:?}, {display:?}, include_str!({:?})),\n", pack_path.display().to_string()));
    }
    index.push_str("];\n");
    std::fs::write(out.join("system_packs.rs"), index).unwrap();
}

fn watch(dir: &Path) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        println!("cargo:rerun-if-changed={}", path.display());
        if path.is_dir() {
            watch(&path);
        }
    }
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}
