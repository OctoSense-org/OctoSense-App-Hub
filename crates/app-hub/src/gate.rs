//! Stage one of admission: the checks a model never makes (ADR 0003 §5).
//!
//! Everything here is a fact about the bundle, decided by code, reported the
//! same way whether it runs on a developer's machine before submitting or in
//! the hub's job after. A finding is either a refusal or a warning; an agent
//! scan runs afterwards on what passes, and never overrides a refusal.
use crate::index::{Catalog, Entry};
use octosense_app_policy::{digest_dir, policy, AppManifest, AppPolicy, HostLimits, Listing, SignatureVerifier, LISTING_FILE};
use std::path::Path;

/// Everything a bundle may hold besides its manifest, by extension. A bundle
/// is cards, data and artwork; anything else is a refusal, so a publisher
/// cannot smuggle a payload the checks do not understand. The one exception
/// is the app's own functions, `fns/*.wasm` (below).
const ALLOWED_EXTENSIONS: &[&str] = &["card", "json", "l0", "octoscript", "splash", "svg", "png", "jpg", "jpeg", "webp", "ttf", "otf", "txt", "md"];

/// The most WebAssembly modules (`fns/*.wasm`) a bundle may carry.
pub const MAX_FUNCTION_MODULES: usize = 8;

/// Ceiling for a whole bundle. Cards are text and artwork; a bundle bigger
/// than this is either shipping something it should not, or should be split.
pub const MAX_BUNDLE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    /// The bundle is not admitted.
    Refusal,
    /// Admitted, but the publisher and a reviewer should see it.
    Warning,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Finding {
    pub severity: Severity,
    pub check: &'static str,
    pub detail: String,
    /// Where in the bundle, when one file or property is at fault.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl Finding {
    fn refuse(check: &'static str, detail: impl Into<String>) -> Self {
        Finding { severity: Severity::Refusal, check, detail: detail.into(), path: None }
    }
    fn warn(check: &'static str, detail: impl Into<String>) -> Self {
        Finding { severity: Severity::Warning, check, detail: detail.into(), path: None }
    }
    pub(crate) fn at(check: &'static str, path: impl Into<String>, detail: impl Into<String>) -> Self {
        Finding { severity: Severity::Refusal, check, detail: detail.into(), path: Some(path.into()) }
    }
    fn warn_at(check: &'static str, path: impl Into<String>, detail: impl Into<String>) -> Self {
        Finding { severity: Severity::Warning, check, detail: detail.into(), path: Some(path.into()) }
    }
}

#[derive(Debug)]
pub struct GateReport {
    pub app_id: String,
    pub version: String,
    pub digest: String,
    pub findings: Vec<Finding>,
    /// What the app would actually get, when the gate passed.
    pub policy: Option<AppPolicy>,
    /// What the bundle's data and artwork reference, by JSON pointer.
    pub resources: Vec<crate::admission::ResourceReference>,
    /// The manifest this report checked, so an entry is made from exactly it.
    pub(crate) admitted_manifest: Vec<u8>,
}

impl GateReport {
    /// The report as `hub check --json` prints it.
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "schema": 1, "stage": "structural", "passed": self.passed(),
            "app_id": self.app_id, "version": self.version, "digest": self.digest,
            "findings": self.findings, "resources": self.resources,
        })
    }

    pub fn passed(&self) -> bool {
        !self.findings.iter().any(|f| f.severity == Severity::Refusal)
    }

    /// One line per finding, for a pull-request comment or a terminal.
    pub fn render(&self) -> String {
        let mut out = format!("{} {} — {}\n", self.app_id, self.version, if self.passed() { "PASSED" } else { "REFUSED" });
        for finding in &self.findings {
            let mark = match finding.severity {
                Severity::Refusal => "refused",
                Severity::Warning => "warning",
            };
            let path = finding.path.as_ref().map(|p| format!(" ({p})")).unwrap_or_default();
            out.push_str(&format!("  [{mark}] {}{path}: {}\n", finding.check, finding.detail));
        }
        if let Some(policy) = &self.policy {
            // The quota is resolved either way; the app gets storage only
            // when it declares it.
            let storage = if policy.capabilities.iter().any(|c| c == "storage") {
                format!("{} bytes", policy.storage_bytes)
            } else {
                "none".to_string()
            };
            out.push_str(&format!(
                "  grants: capabilities {:?}, hosts {:?}, storage {storage}, agent {}\n",
                policy.capabilities,
                policy.hosts,
                policy.agent.as_ref().map(|a| a.profile.as_kernel_mode()).unwrap_or("none")
            ));
            if let Some(scope) = &policy.research {
                out.push_str(&format!(
                    "  research scope (octos Scope): {}\n",
                    serde_json::to_string(scope).unwrap_or_default()
                ));
            }
        }
        out
    }
}

/// Run the deterministic gate over a bundle directory.
///
/// `previous` is the catalog the hub already published, used for the identity
/// rules: a version may not be republished, and once a publisher key is on
/// record every later version, and every new app of that publisher, must be
/// signed by it ([`crate::publishers`]). The caller authenticates `previous`
/// before passing it as trusted history.
pub fn check_bundle(
    bundle: &Path,
    limits: &HostLimits,
    verifier: &dyn SignatureVerifier,
    previous: Option<&Catalog>,
) -> Result<GateReport, String> {
    check_bundle_for(bundle, limits, verifier, previous, false, None)
}

/// Check a shipped system app during development, using the same resource
/// ceilings as the system runner. This report cannot become a store entry.
/// Store admission always uses [`check_bundle`], including with system limits.
pub fn check_system_bundle(
    bundle: &Path,
    verifier: &dyn SignatureVerifier,
) -> Result<GateReport, String> {
    check_bundle_for(bundle, &HostLimits::system(), verifier, None, true, None)
}

/// Check against an opaque authenticated publication history, without losing the
/// authenticated publisher bindings.
pub fn check_authenticated_bundle(bundle:&Path,limits:&HostLimits,verifier:&dyn SignatureVerifier,
    base:&crate::github_catalog::AuthenticatedCatalog)->Result<GateReport,String>{
    check_bundle_with_registry(bundle,limits,verifier,base.catalog(),base.publishers())
}

pub(crate) fn check_bundle_with_registry(bundle:&Path, limits:&HostLimits, verifier:&dyn SignatureVerifier,
    previous:&Catalog, registry:&dyn crate::PublisherRegistry)->Result<GateReport,String> {
    check_bundle_for(bundle,limits,verifier,Some(previous),false,Some(registry))
}

fn check_bundle_for(
    bundle: &Path,
    limits: &HostLimits,
    verifier: &dyn SignatureVerifier,
    previous: Option<&Catalog>,
    system_development: bool,
    registry: Option<&dyn crate::PublisherRegistry>,
) -> Result<GateReport, String> {
    // Metadata first: nothing is read or hashed beyond the limits.
    let files = crate::admission::inventory(bundle)?;
    let manifest_path = bundle.join(octosense_app_policy::MANIFEST_FILE);
    let manifest_json = crate::admission::read_text(&manifest_path, crate::admission::MAX_MANIFEST_BYTES)?;
    let manifest = AppManifest::parse(&manifest_json)?;
    let digest = digest_dir(bundle)?;
    let (mut findings, resources) = crate::admission::validate(bundle, &files);

    // ---- integrity ------------------------------------------------------
    if !digest.eq_ignore_ascii_case(&manifest.integrity.bundle_blake3) {
        findings.push(Finding::refuse(
            "digest",
            format!("the bundle hashes to {digest}, the manifest claims {}", manifest.integrity.bundle_blake3),
        ));
    }
    if let Err(e) = octosense_app_policy::verify::verify_manifest(&manifest, verifier) {
        findings.push(Finding::refuse("publisher-signature", e));
    }
    if manifest.integrity.signature.is_none() && manifest.integrity.github.is_none() {
        if limits.require_signature {
            findings.push(Finding::refuse("publisher-signature", "this hub requires a signed manifest"));
        } else {
            findings.push(Finding::warn("publisher-signature", "unsigned: accountability rests on the hub alone"));
        }
    }

    // ---- identity ------------------------------------------------------
    // Ids under `os.` are the system apps' (appstore::system): every device
    // refuses to install one from a store, so the hub refuses to offer one.
    if manifest.id.starts_with("os.") && !system_development {
        findings.push(Finding::refuse(
            "identity",
            format!("{} is under os., which is reserved for system apps that ship with the device", manifest.id),
        ));
    }
    if system_development && !manifest.id.starts_with("os.") {
        findings.push(Finding::refuse(
            "identity",
            "--system-app checks only shipped os.* apps; use ordinary check/scan for store apps",
        ));
    }
    // Native apps' ids and the host's own names (app-policy's
    // RESERVED_NAMES) are refused by the agent review below (as "identity")
    // and by the policy.

    // ---- contents -------------------------------------------------------
    let mut total = 0u64;
    let mut modules = 0usize;
    let mut function_files = Vec::new();
    for file in list_files(bundle)? {
        let path = bundle.join(&file);
        let size = std::fs::metadata(&path).map_err(|e| e.to_string())?.len();
        total += size;
        let extension = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        if extension == "wasm" {
            modules += 1;
            function_files.push(file.clone());
        } else if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
            findings.push(Finding::refuse(
                "contents",
                format!("{} has extension {extension:?}, which a bundle may not hold", file.display()),
            ));
        }
    }
    if total > MAX_BUNDLE_BYTES {
        findings.push(Finding::refuse("size", format!("the bundle is {total} bytes, over the {MAX_BUNDLE_BYTES} ceiling")));
    }

    // ---- the app's own functions ----------------------------------------
    // A WebAssembly module is the one executable a bundle may carry: the
    // app's own functions in `fns/` (admission checks each name and header),
    // run by the host's `wasm` service in a sandbox, under the `wasm`
    // capability a store shows the person.
    let declares_wasm = manifest.capabilities.iter().any(|c| c == "wasm");
    if modules > 0 && !declares_wasm {
        findings.push(Finding::refuse(
            "functions",
            format!("the bundle carries {modules} WebAssembly module(s) but does not declare the wasm capability"),
        ));
    }
    if modules > MAX_FUNCTION_MODULES {
        findings.push(Finding::refuse(
            "functions",
            format!("the bundle carries {modules} WebAssembly modules, over the {MAX_FUNCTION_MODULES} it may"),
        ));
    }
    if declares_wasm && modules == 0 {
        findings.push(Finding::warn("functions", "the bundle declares the wasm capability but carries no fns/*.wasm"));
    }
    // A component (OctoSense ADR 0014) also reaches the clock, random numbers
    // and, with `storage`, the app's own files. Admission has validated it
    // and its imports; the manifest must require the feature, so a host whose
    // `wasm` service loads only modules refuses the app instead of failing at
    // its first call, and must grant storage for files. Reviewers see what
    // each component reaches.
    let requires_components = manifest.requires.iter().any(|f| f == crate::functions::COMPONENTS_FEATURE);
    let declares_storage = manifest.capabilities.iter().any(|c| c == "storage");
    let declares_net = manifest.capabilities.iter().any(|c| c == "net");
    let mut components = 0usize;
    for file in &function_files {
        let name = octosense_app_policy::portable_path(file).unwrap_or_else(|| file.to_string_lossy().replace('\\', "/"));
        let Ok(bytes) = crate::admission::read_bounded(&bundle.join(file), MAX_BUNDLE_BYTES) else { continue };
        if !crate::functions::is_component(&bytes) {
            continue;
        }
        components += 1;
        // Admission reported a component that does not validate or imports
        // what no host gives it.
        let Ok(info) = crate::functions::inspect_component(&bytes) else { continue };
        if !info.refused_imports().is_empty() {
            continue;
        }
        if !requires_components {
            findings.push(Finding::refuse(
                "functions",
                format!("{name} is a WebAssembly component; the manifest must require {}", crate::functions::COMPONENTS_FEATURE),
            ));
        }
        if info.uses_files() && !declares_storage {
            findings.push(Finding::refuse(
                "functions",
                format!("{name} imports wasi:filesystem, the app's own files, which needs the storage capability"),
            ));
        }
        if info.uses_http() && !declares_net {
            findings.push(Finding::refuse(
                "functions",
                format!("{name} imports wasi:http, the network, which the app must declare with the net capability"),
            ));
        }
        findings.push(Finding::warn_at("functions", name.clone(), format!("{name} is a component that reaches {}", info.reach())));
    }
    if requires_components && components == 0 {
        findings.push(Finding::warn(
            "functions",
            format!("the manifest requires {} but fns/ holds no component; hosts without it refuse the app", crate::functions::COMPONENTS_FEATURE),
        ));
    }

    // ---- assets are local ----------------------------------------------
    // ADR 0002's prototype found a card with no network grant fetching nine
    // images over HTTP, because the resource loader is not the network module
    // the grant gates. Until that is closed in the runtime, the gate is what
    // keeps a bundle from reaching outside itself.
    for reference in external_references(bundle, &manifest)? {
        findings.push(Finding::refuse(
            "assets",
            format!("{reference} points outside the bundle; ship the asset with the app"),
        ));
    }

    // ---- secrets are the host's ----------------------------------------
    // A contained app never collects a password, a PIN or a code: the host's
    // sheet does, for the service that needs it (appstore::services). The
    // runtime makes such a field inert; the gate refuses the bundle, so a
    // publisher learns it before a person meets a dead field.
    for field in secret_fields(bundle)? {
        findings.push(Finding::refuse(
            "secrets",
            format!("{field}: apps may not ask for passwords or codes; a host service collects them on its own sheet"),
        ));
    }

    // ---- storage is asked for ------------------------------------------
    // An app gets its storage only when it declares `storage`; without it,
    // every fs call errors and the camera saves nothing. Warned, not
    // refused: the publisher may handle the error, and a camera preview
    // needs no storage.
    for warning in storage_warnings(bundle, &manifest)? {
        findings.push(Finding::warn("storage", warning));
    }

    // ---- the listing ----------------------------------------------------
    // A store shows nothing it has not reviewed: the listing ships in the
    // bundle, under the same digest, and its assets must be there too.
    match std::fs::read_to_string(bundle.join(LISTING_FILE)) {
        Err(_) => findings.push(Finding::refuse("listing", format!("no {LISTING_FILE}: a store needs a description, a category, screenshots, a publisher and the platforms it runs on"))),
        Ok(text) => match Listing::parse(&text) {
            Err(e) => findings.push(Finding::refuse("listing", e)),
            Ok(listing) => {
                // A screenshot is the one listing claim a reviewer can check
                // against the rendered card, and the icon is what the
                // launcher shows once installed: both are required, as on
                // every store people know.
                if listing.screenshots.is_empty() {
                    findings.push(Finding::refuse("listing", "no screenshots: at least one PNG in the bundle, named in the listing, is required"));
                }
                if listing.icon.is_none() {
                    findings.push(Finding::refuse("listing", "no icon: a square PNG or SVG in the bundle, named in the listing, is required"));
                }
                for asset in listing.screenshots.iter().chain(listing.icon.iter()) {
                    if !bundle.join(asset).is_file() {
                        findings.push(Finding::refuse("listing", format!("{asset} is named by the listing but is not in the bundle")));
                    }
                }
            }
        },
    }

    // ---- the app's agent and tools (ADR 0002 §3, §4) --------------------
    // tools.json, AGENT.md and skills are checked here and pinned by the
    // digest above like every other file. A native module's tools.json gets
    // the same checks from the app-peers broker (ToolManifest::load).
    let agent_review = octosense_app_policy::agent::review(bundle, &manifest);
    for issue in &agent_review.issues {
        findings.push(if issue.refusal {
            Finding::refuse(issue.check, issue.detail.clone())
        } else {
            Finding::warn(issue.check, issue.detail.clone())
        });
    }
    if let Some(agent) = &agent_review.bundle {
        for tool in agent.tools.iter().filter(|t| t.supervision().needs_person()) {
            // Recorded, not refused: a destructive or outward tool is
            // allowed, and the host always asks before it runs, whoever
            // calls it.
            let who = match tool.confirm {
                octosense_app_policy::Confirm::Host => "the host's approval",
                octosense_app_policy::Confirm::App => "the app's own confirmation sheet (an approval request when the person is away)",
            };
            findings.push(Finding::warn(
                "tools",
                format!(
                    "{} is {}: every call waits for {who}",
                    tool.name,
                    if tool.risk == octosense_app_policy::Risk::Destructive { "destructive" } else { "outward" }
                ),
            ));
        }
    }

    // ---- what it would get ----------------------------------------------
    let policy = match policy::resolve(&manifest, limits) {
        Ok(policy) => Some(policy),
        Err(e) => {
            findings.push(Finding::refuse("policy", e));
            None
        }
    };

    // ---- identity against what is already published ----------------------
    if let Some(catalog) = previous {
        for entry in &catalog.entries {
            if entry.app_id() == manifest.id && entry.version() == manifest.version {
                findings.push(Finding::refuse(
                    "version",
                    format!("version {} of {} is already published; publish a new version", manifest.version, manifest.id),
                ));
            }
        }
        let continuity = match registry {
            Some(registry) => crate::publishers::verify_continuity(&manifest, registry),
            None => crate::publishers::CatalogPublishers::from_catalog(catalog)
                .and_then(|registry| crate::publishers::verify_continuity(&manifest, &registry)),
        };
        if let Err(e) = continuity {
            findings.push(Finding::refuse("continuity", e));
        }
    }

    let admitted_manifest = serde_json::to_vec(&manifest).map_err(|e| e.to_string())?;
    Ok(GateReport { app_id: manifest.id, version: manifest.version, digest, findings, policy, resources, admitted_manifest })
}

/// Every file in the bundle except the manifest, relative to its root.
fn list_files(root: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
        for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_symlink() {
                return Err(format!("{}: a bundle may not hold a symlink", path.display()));
            }
            if kind.is_dir() {
                walk(root, &path, out)?;
            } else {
                let relative = path.strip_prefix(root).map_err(|e| e.to_string())?.to_path_buf();
                if relative != Path::new(octosense_app_policy::MANIFEST_FILE) {
                    out.push(relative);
                }
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    out.sort();
    Ok(out)
}

/// Anything in the bundle's text that reaches outside it: an absolute URL, or
/// a path that climbs out. Cards name their artwork in text, so this is a
/// textual check by necessity; it is a gate, not the runtime's enforcement.
fn external_references(root: &Path, manifest: &AppManifest) -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    for file in list_files(root)? {
        // The two metadata files carry URLs on purpose (a support page, a
        // privacy policy); nothing loads them. Their own asset paths are
        // checked by the listing rules.
        if file == Path::new(LISTING_FILE) || file == Path::new(octosense_app_policy::MANIFEST_FILE) {
            continue;
        }
        let extension = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        let text = match std::fs::read_to_string(root.join(&file)) {
            Ok(text) => text,
            Err(_) => continue, // not valid text: the extension check already covers it
        };
        // A script, and the agent's own files (instructions, skills, the
        // tool manifest), may name the hosts the app declares: the agent's
        // network is the app's network. Nothing else may reach out.
        if extension == "splash" || octosense_app_policy::agent::is_agent_file(&file, manifest) {
            found.extend(script_references(&file, &text, manifest));
            continue;
        }
        // Plain documentation (including a bundled font's required license)
        // does not load its links. Agent guidance was checked above; executable
        // card data and SVG resources remain subject to their normal checks.
        // This never adds a network grant to the app's resolved policy.
        if !matches!(extension.as_str(), "card" | "json" | "l0" | "octoscript") {
            continue;
        }
        for needle in ["http://", "https://", "file://", "../"] {
            if let Some(at) = text.find(needle) {
                let snippet: String = text[at..].chars().take(60).collect();
                found.push(format!("{} contains {}", file.display(), snippet.replace('\n', " ")));
                break;
            }
        }
    }
    Ok(found)
}

/// A script app fetches what it declares (ADR 0004): an `https://` address
/// may name only a host in the manifest's `network.hosts`, or any public host
/// when the app is granted `images` (pictures) or `web` (a web view). Plain
/// `http://`, `file://` and paths out of the bundle are refused outright. The
/// runtime holds the app to the same list on every request; this refuses the
/// bundle before a person installs it.
fn script_references(file: &Path, text: &str, manifest: &AppManifest) -> Vec<String> {
    let mut found = Vec::new();
    for needle in ["http://", "file://", "../"] {
        if let Some(at) = text.find(needle) {
            let snippet: String = text[at..].chars().take(60).collect();
            found.push(format!("{} contains {}", file.display(), snippet.replace('\n', " ")));
        }
    }
    let any_public = manifest.capabilities.iter().any(|c| c == "images" || c == "web");
    let mut rest = text;
    while let Some(at) = rest.find("https://") {
        rest = &rest[at + "https://".len()..];
        let host: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-').collect::<String>().to_ascii_lowercase();
        // `https://` followed by an interpolation or nothing names no host.
        if host.is_empty() || any_public || manifest.network.hosts.iter().any(|h| h.eq_ignore_ascii_case(&host)) {
            continue;
        }
        found.push(format!("{} reaches {host}, which the manifest does not declare in network.hosts", file.display()));
    }
    found.sort();
    found.dedup();
    found
}

/// Password and one-time-code fields declared in the bundle's scripts.
fn secret_fields(root: &Path) -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    for file in list_files(root)? {
        let extension = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        if !matches!(extension.as_str(), "card" | "l0" | "octoscript" | "splash") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root.join(&file)) else { continue };
        let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        for needle in ["is_password:true", "TextInputContentType.Password", "TextInputContentType.NewPassword", "TextInputContentType.OneTimeCode"] {
            if compact.contains(needle) {
                found.push(format!("{} declares {needle}", file.display()));
            }
        }
    }
    Ok(found)
}

/// The splash runtime's `fs` methods, all of which need the app's storage.
const FS_METHODS: &[&str] = &["read", "read_bytes", "write", "append", "exists", "list", "mkdir", "remove"];

/// What an app that did not declare `storage` will find does not work: each
/// script that calls `fs`, and the camera's captures, which land in the
/// app's storage.
fn storage_warnings(root: &Path, manifest: &AppManifest) -> Result<Vec<String>, String> {
    let has = |capability: &str| manifest.capabilities.iter().any(|c| c == capability);
    if has("storage") {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    for file in list_files(root)? {
        if file.extension().and_then(|e| e.to_str()) != Some("splash") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root.join(&file)) else { continue };
        let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        let calls: Vec<String> = FS_METHODS.iter().filter(|m| compact.contains(&format!("fs.{m}("))).map(|m| format!("fs.{m}")).collect();
        if !calls.is_empty() {
            found.push(format!("{} calls {}, which fail without the storage capability", file.display(), calls.join(", ")));
        }
    }
    if has("camera") {
        found.push("camera without storage: the preview shows, but a capture saves nothing; captures land in the app's storage".to_string());
    }
    Ok(found)
}

/// Build the index entry for a bundle the gate passed.
#[allow(clippy::too_many_arguments)]
pub fn entry_for(
    bundle: &Path,
    report: &GateReport,
    publisher: &str,
    publisher_key: &str,
    repository: &str,
    commit: &str,
    admitted: &str,
) -> Result<Entry, String> {
    if !report.passed() {
        return Err("cannot make an entry from a refused gate report".into());
    }
    let manifest_json = std::fs::read_to_string(bundle.join(octosense_app_policy::MANIFEST_FILE)).map_err(|e| e.to_string())?;
    let manifest = AppManifest::parse(&manifest_json)?;
    if manifest.id.starts_with("os.") {
        return Err("system development reports cannot become store entries".into());
    }
    // The entry is what the gate checked, byte for byte, or nothing.
    if serde_json::to_vec(&manifest).map_err(|e| e.to_string())? != report.admitted_manifest
        || manifest.id != report.app_id
        || manifest.version != report.version
        || digest_dir(bundle)? != report.digest
    {
        return Err("the bundle or its manifest changed after the gate; check it again".into());
    }
    match &manifest.integrity.signature {
        Some(signature) => {
            if publisher.is_empty() || publisher != signature.key_id {
                return Err(format!("the publisher must be the manifest's signer, {:?}", signature.key_id));
            }
            crate::PublisherKeys::new()
                .with(publisher, publisher_key)
                .verify(publisher, &signature.value, &manifest.signing_bytes()?)?;
        }
        None if publisher.is_empty() => return Err("an entry needs a publisher".into()),
        None if !publisher_key.is_empty() => return Err("an unsigned release records no publisher key".into()),
        None => {}
    }
    if let Some(github) = &manifest.integrity.github {
        if publisher != format!("github:{}",github.repository_id) {
            return Err("GitHub publisher must be github: followed by its authenticated repository id".into());
        }
        crate::github_publisher::verify(github, &manifest.signing_bytes()?)?;
        if repository != github.repository_url() || commit != github.commit {
            return Err("entry source does not match authenticated publisher repository and commit".into());
        }
    }
    let listing = std::fs::read_to_string(bundle.join(LISTING_FILE)).ok().and_then(|t| Listing::parse(&t).ok());
    let mut tools = octosense_app_policy::agent::read_tools(bundle)?.map(|t| t.tools).unwrap_or_default();
    // Catalog tools describe permissions; dispatch reads the digest-pinned
    // bundle instead. Older stores strictly parse ToolSpec, so including this
    // new routing field here would make them reject the entire catalog.
    // Keep every permission field, and leave the signed bundle untouched.
    for tool in &mut tools {
        tool.host_method = None;
    }
    Ok(Entry {
        artifact: format!("artifacts/{}-{}.bundle", report.app_id, report.version),
        manifest,
        listing,
        tools,
        publisher: publisher.to_string(),
        publisher_key: publisher_key.to_string(),
        source: crate::index::Source { repository: repository.to_string(), commit: commit.to_string() },
        status: crate::index::Status::Offered,
        admitted: admitted.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_script_app_reaches_only_the_hosts_it_declares() {
        let dir = std::env::temp_dir().join(format!("gate-script-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("main.splash"),
            "fn load(){ net.http_request({url: \"https://api.example.com/v1\"}, fn(r){}) }\nlet more = \"https://tracker.example.net/p\"\nImage{src: \"{{assets}}/a.png\"}",
        )
        .unwrap();
        let manifest = |caps: &[&str], hosts: &[&str]| {
            AppManifest::parse(&serde_json::json!({
                "schema": 1, "id": "dev.example.app", "version": "1.0.0", "name": "App",
                "integrity": {"bundle_blake3": ""}, "capabilities": caps, "network": {"hosts": hosts}
            }).to_string()).unwrap()
        };
        let refused = external_references(&dir, &manifest(&["net"], &["api.example.com"])).unwrap();
        assert_eq!(refused.len(), 1, "{refused:?}");
        assert!(refused[0].contains("tracker.example.net"));
        assert!(external_references(&dir, &manifest(&["net"], &["api.example.com", "tracker.example.net"])).unwrap().is_empty());
        assert!(external_references(&dir, &manifest(&["net", "images"], &["api.example.com"])).unwrap().is_empty(), "images reaches any public host");
        std::fs::write(dir.join("main.splash"), "let x = \"http://api.example.com\"").unwrap();
        assert!(!external_references(&dir, &manifest(&["net", "web"], &["api.example.com"])).unwrap().is_empty(), "plain http is never allowed");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_store_bundle_may_not_take_a_system_app_id() {
        let dir = std::env::temp_dir().join(format!("gate-os-id-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("main.splash"), "Label{text: \"hi\"}").unwrap();
        let write = |id: &str| {
            std::fs::write(
                dir.join(octosense_app_policy::MANIFEST_FILE),
                serde_json::json!({"schema": 1, "id": id, "version": "1.0.0", "name": "App", "integrity": {"bundle_blake3": ""}}).to_string(),
            )
            .unwrap()
        };
        let limits = HostLimits::default().with_require_signature(false);
        let identity = |dir: &Path| {
            check_bundle(dir, &limits, &octosense_app_policy::RefuseAllSignatures, None)
                .unwrap()
                .findings
                .into_iter()
                .filter(|f| f.check == "identity")
                .count()
        };
        write("os.mail");
        assert_eq!(identity(&dir), 1, "os. ids belong to system apps");
        write("dev.example.mail");
        assert_eq!(identity(&dir), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_bundle_that_asks_for_a_password_is_refused() {
        let dir = std::env::temp_dir().join(format!("gate-secrets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("cards")).unwrap();
        std::fs::write(dir.join("main.card"), "View{ name := TextInput{empty_text: \"Name\"} }").unwrap();
        std::fs::write(dir.join("cards/login.card"), "View{ pw := TextInput{ is_password : true } }").unwrap();
        std::fs::write(dir.join("cards/otp.card"), "TextInput{content_type: TextInputContentType.OneTimeCode}").unwrap();
        std::fs::write(dir.join("notes.md"), "Set is_password: true in your own app, not here.").unwrap();
        let mut found = secret_fields(&dir).unwrap();
        found.sort();
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(found[0].contains("login.card") && found[0].contains("is_password"));
        assert!(found[1].contains("otp.card") && found[1].contains("OneTimeCode"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_bundle_that_needs_storage_without_asking_is_warned() {
        let dir = std::env::temp_dir().join(format!("gate-storage-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("main.splash"), "fn save(text){ fs.write(\"notes.txt\", text) }\nfn load(){ fs.read(\"notes.txt\") }").unwrap();
        let write = |caps: &[&str]| {
            std::fs::write(
                dir.join(octosense_app_policy::MANIFEST_FILE),
                serde_json::json!({"schema": 1, "id": "dev.example.jotter", "version": "1.0.0", "name": "Jotter", "integrity": {"bundle_blake3": ""}, "capabilities": caps}).to_string(),
            )
            .unwrap()
        };
        let limits = HostLimits::default().with_require_signature(false);
        let warnings = |dir: &Path| -> Vec<String> {
            let report = check_bundle(dir, &limits, &octosense_app_policy::RefuseAllSignatures, None).unwrap();
            let storage: Vec<Finding> = report.findings.into_iter().filter(|f| f.check == "storage").collect();
            assert!(storage.iter().all(|f| f.severity == Severity::Warning), "{storage:?}");
            storage.into_iter().map(|f| f.detail).collect()
        };
        write(&[]);
        let found = warnings(&dir);
        assert_eq!(found.len(), 1, "one warning per script: {found:?}");
        assert!(found[0].contains("main.splash") && found[0].contains("fs.write"), "{found:?}");
        let grants = |dir: &Path| check_bundle(dir, &limits, &octosense_app_policy::RefuseAllSignatures, None).unwrap().render();
        assert!(grants(&dir).contains("storage none"), "{}", grants(&dir));
        write(&["storage"]);
        assert!(warnings(&dir).is_empty());
        assert!(grants(&dir).contains("storage 16777216 bytes"), "{}", grants(&dir));
        write(&["camera", "storage"]);
        assert!(warnings(&dir).is_empty());
        std::fs::write(dir.join("main.splash"), "Label{text: \"no files here\"}").unwrap();
        write(&["camera"]);
        let found = warnings(&dir);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("camera") && found[0].contains("capture"), "{found:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
