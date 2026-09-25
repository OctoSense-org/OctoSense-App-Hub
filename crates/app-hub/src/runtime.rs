//! Process protocol for native validation. A publisher-supplied report is not
//! approval: the Hub executes its operator-selected validator itself.
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const CHECK_VERSION: u32 = 1;
pub const CARD_RUNTIME: &str = "octosense-card-native-v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeReport {
    pub schema: u32,
    pub check_version: u32,
    pub runtime: String,
    pub target: String,
    pub digest: String,
    pub manifest_digest: String,
    pub checks: Vec<String>,
}

/// Stable failure envelope shared by the CLI and worker. A failed process is
/// never evidence, even when it cannot produce this envelope (abort/timeout).
pub fn failure_json(detail: &str) -> serde_json::Value {
    serde_json::json!({"schema":1,"stage":"runtime","passed":false,"findings":[{
        "severity":"refusal","check":"runtime-validation-failed","detail":detail
    }]})
}

/// Created only by executing a trusted operator-selected worker. JSON supplied
/// by an app cannot construct this proof or replace the worker's result.
#[derive(Debug, Serialize)]
pub struct RuntimeEvidence {
    validator_digest: String,
    report: RuntimeReport,
    #[serde(skip)]
    snapshot: crate::launch::LaunchSnapshot,
    #[serde(skip)]
    _scratch: Scratch,
}

#[derive(Debug)]
struct Scratch(std::path::PathBuf);
impl Drop for Scratch { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

impl RuntimeEvidence {
    pub fn report(&self) -> &RuntimeReport { &self.report }
    pub fn validator_digest(&self) -> &str { &self.validator_digest }
    pub fn bundle(&self) -> &Path { self.snapshot.bundle() }
    pub fn verify_bundle(&self, bundle: &Path) -> Result<(), String> {
        let (digest, manifest_digest) = identity(bundle)?;
        if self.report.digest != digest || self.report.manifest_digest != manifest_digest {
            return Err("bundle or manifest changed after runtime validation".into());
        }
        Ok(())
    }
}

pub fn identity(bundle: &Path) -> Result<(String, String), String> {
    crate::admission::inventory(bundle)?;
    let manifest = octosense_app_policy::AppManifest::parse(&crate::admission::read_text(
        &bundle.join("manifest.json"), crate::admission::MAX_MANIFEST_BYTES)?)?;
    let digest = octosense_app_policy::bundle::digest_dir_limited(bundle, crate::gate::MAX_BUNDLE_BYTES, crate::admission::MAX_ENTRIES, crate::admission::MAX_DEPTH)?;
    let manifest_digest = blake3::hash(&serde_json::to_vec(&manifest).map_err(|e| e.to_string())?).to_hex().to_string();
    Ok((digest, manifest_digest))
}

pub fn validate(bundle: &Path, gate: &crate::GateReport, validator: &Path) -> Result<RuntimeEvidence, String> {
    if !gate.passed() { return Err("runtime validation requires a passed structural gate".into()); }
    let original = identity(bundle)?;
    if original.0 != gate.digest || original.1 != gate.manifest_digest() { return Err("bundle changed after structural validation".into()); }
    let executable = validator.canonicalize().map_err(|e| format!("validator executable is unavailable: {e}"))?;
    // Binary identity includes the compiled runtime, its dependencies and check
    // code; a hand-written version string alone cannot identify a validator.
    fn binary_digest(path: &Path) -> Result<String, String> {
        let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut hash = blake3::Hasher::new();
        hash.update_reader(&mut file).map_err(|e| e.to_string())?;
        Ok(hash.finalize().to_hex().to_string())
    }
    let validator_digest = binary_digest(&executable)?;
    let mut nonce = [0u8; 16];
    rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, &mut nonce).map_err(|e| e.to_string())?;
    let scratch = std::env::temp_dir().join(format!("octosense-validation-{}", hex::encode(nonce)));
    std::fs::create_dir(&scratch).map_err(|e| e.to_string())?;
    let scratch = Scratch(scratch);
    let snapshot = crate::launch::LaunchSnapshot::copy(bundle, &scratch.0)?;
    if identity(snapshot.bundle())? != original { return Err("bundle changed while preparing runtime validation".into()); }
    let bytes = crate::process::run(&executable, &[snapshot.bundle().as_os_str()], &scratch.0, vec![], Default::default())?;
    let report: RuntimeReport = serde_json::from_slice(&bytes).map_err(|e| format!("invalid runtime validation report: {e}"))?;
    if report.schema != 1 || report.check_version != CHECK_VERSION || report.runtime != CARD_RUNTIME
        || report.target != format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
        || report.digest != original.0 || report.manifest_digest != original.1
        || ["structural", "card-preparation", "native-widget-load", "startup-shutdown"].iter().any(|check| !report.checks.iter().any(|found| found == check)) {
        return Err("runtime validation report has the wrong bundle, runtime or checks".into());
    }
    if binary_digest(&executable)? != validator_digest { return Err("validator binary changed during validation".into()); }
    let evidence = RuntimeEvidence { validator_digest, report, snapshot, _scratch: scratch };
    evidence.verify_bundle(bundle)?;
    Ok(evidence)
}
