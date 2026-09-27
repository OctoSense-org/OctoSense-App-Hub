//! Resolving a manifest into what the app actually gets.
//!
//! Every rule here fails closed: an unknown capability, a host that is not a
//! bare host name, a tool the host does not offer, a quota above the host's
//! ceiling. A manifest asks; the host decides; the app is never consulted
//! again. This is the only place that produces [`AppPolicy`], and the two
//! containers are derived from it, never set by hand.
use crate::manifest::{short_id, AgentSpec, AppManifest, ModelSpec, ProfileMode, Triggers, KNOWN_CAPABILITIES};
use std::collections::BTreeSet;

/// The host's own ceilings. An app may ask for less and get it; asking for
/// more is clamped, not refused, because a bundle built for a roomier device
/// should still run here — just smaller.
#[derive(Clone, Debug)]
pub struct HostLimits {
    pub max_storage_bytes: u64,
    pub max_instruction_budget: u64,
    pub max_memory_bytes: u64,
    pub max_iterations: u32,
    pub max_token_budget: u64,
    /// Tools this host offers to contained apps at all. Shell, process and
    /// arbitrary-path file tools are absent from this list by design.
    pub offered_tools: Vec<String>,
    /// Whether a bundle must carry a signature to be admitted.
    pub require_signature: bool,
}

impl Default for HostLimits {
    /// Phone-sized defaults: the isolate jail's own ceiling for storage, a
    /// budget that cannot spin the UI thread for a second, and a tool list
    /// holding only what a card app legitimately needs.
    fn default() -> Self {
        HostLimits {
            max_storage_bytes: 16 * 1024 * 1024,
            max_instruction_budget: 20_000_000,
            max_memory_bytes: 64 * 1024 * 1024,
            max_iterations: 8,
            max_token_budget: 200_000,
            offered_tools: ["ledger.read", "ledger.write", "net.fetch", "storage.read", "storage.write", "card.render"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            require_signature: true,
        }
    }
}

impl HostLimits {
    /// Ceilings for a system app: a bundle that ships inside the build, like
    /// News or Photos, contained like any installed app but living for as
    /// long as the person keeps it open. An installed card's budget is sized
    /// for a card; an app that is used for an hour needs room for an hour.
    /// A system app is part of the signed build, so it is admitted by its
    /// digest alone.
    pub fn system() -> Self {
        HostLimits {
            max_storage_bytes: 64 * 1024 * 1024,
            max_instruction_budget: 4_000_000_000,
            max_memory_bytes: 128 * 1024 * 1024,
            require_signature: false,
            ..HostLimits::default()
        }
    }
}

/// What the app gets. Produced only by [`resolve`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppPolicy {
    pub app_id: String,
    pub version: String,
    pub display_name: String,
    /// Granted capabilities, sorted and deduplicated.
    pub capabilities: BTreeSet<String>,
    /// Exactly the hosts the app may reach. Empty means no network, whatever
    /// the `net` capability says.
    pub hosts: BTreeSet<String>,
    pub storage_bytes: u64,
    pub instruction_budget: u64,
    pub memory_bytes: u64,
    /// May the app cause a prompt the person has to answer.
    pub may_prompt: bool,
    /// None when the manifest asked for no agent.
    pub agent: Option<AgentPolicy>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentPolicy {
    pub profile: ProfileMode,
    /// The generic host tools granted (`agent.tools`). The app's own tools
    /// come from its tool manifest ([`crate::agent::ToolManifest`]).
    pub tools: BTreeSet<String>,
    pub max_iterations: u32,
    pub token_budget: u64,
    /// The model requirements, needs deduplicated and sorted.
    pub model: ModelSpec,
    /// The app ASKED to run in the background. Not a grant: the host asks
    /// the person, and a run while the app is closed needs both.
    pub background_requested: bool,
    pub triggers: Triggers,
    /// Bundle-relative path of the agent's instructions, when declared.
    pub instructions: Option<String>,
    /// The skills the bundle declares, sorted.
    pub skills: BTreeSet<String>,
}

impl AppPolicy {
    /// Whether a granted capability covers this action. The single question
    /// every host service asks before doing work on an app's behalf.
    pub fn allows(&self, capability: &str) -> bool {
        self.capabilities.contains(capability)
    }

    /// Whether the app may reach this host. Requires the capability AND the
    /// entry: a granted `net` with an empty list reaches nothing.
    pub fn allows_host(&self, host: &str) -> bool {
        self.allows("net") && self.hosts.contains(host)
    }
}

/// Resolve a parsed manifest against this host.
pub fn resolve(manifest: &AppManifest, limits: &HostLimits) -> Result<AppPolicy, String> {
    check_id(&manifest.id)?;
    if manifest.version.trim().is_empty() {
        return Err("manifest version is empty".into());
    }
    if limits.require_signature && manifest.integrity.signature.is_none() {
        return Err(format!("app {} is unsigned and this host requires a signature", manifest.id));
    }

    let mut capabilities = BTreeSet::new();
    for capability in &manifest.capabilities {
        if !KNOWN_CAPABILITIES.contains(&capability.as_str()) {
            return Err(format!("app {} requests unknown capability {:?}", manifest.id, capability));
        }
        capabilities.insert(capability.clone());
    }

    let mut hosts = BTreeSet::new();
    for host in &manifest.network.hosts {
        check_host(host)?;
        hosts.insert(host.to_ascii_lowercase());
    }
    // A host list without the capability is a manifest mistake, not a silent
    // grant: refuse it so the author notices before the app ships.
    if !hosts.is_empty() && !capabilities.contains("net") {
        return Err(format!("app {} lists hosts but does not request the net capability", manifest.id));
    }

    let agent = match &manifest.agent {
        None => None,
        Some(spec) => Some(resolve_agent(&manifest.id, spec, limits)?),
    };

    Ok(AppPolicy {
        app_id: manifest.id.clone(),
        version: manifest.version.clone(),
        display_name: manifest.name.clone(),
        may_prompt: capabilities.contains("prompt"),
        capabilities,
        hosts,
        storage_bytes: clamp(manifest.storage.max_bytes, limits.max_storage_bytes),
        instruction_budget: clamp(manifest.compute.instruction_budget, limits.max_instruction_budget),
        memory_bytes: clamp(manifest.compute.memory_bytes, limits.max_memory_bytes),
        agent,
    })
}

fn resolve_agent(app_id: &str, spec: &AgentSpec, limits: &HostLimits) -> Result<AgentPolicy, String> {
    let mut tools = BTreeSet::new();
    for tool in &spec.tools {
        if !limits.offered_tools.iter().any(|offered| offered == tool) {
            return Err(format!("app {app_id} requests tool {tool:?}, which this host does not offer contained apps"));
        }
        tools.insert(tool.clone());
    }
    let mut model = spec.model.clone().unwrap_or_default();
    model.needs.sort();
    model.needs.dedup();
    if model.per_task.len() > MAX_MODEL_TASKS {
        return Err(format!("app {app_id} names more than {MAX_MODEL_TASKS} model tasks"));
    }
    for (task, task_model) in model.per_task.iter_mut() {
        if task.is_empty() || task.len() > 32 || !task.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
            return Err(format!("app {app_id} model task {task:?} must be 1 to 32 of [a-z_]"));
        }
        task_model.needs.sort();
        task_model.needs.dedup();
    }
    check_triggers(app_id, &spec.triggers)?;
    // A background agent with nothing to wake it would either never run or
    // be woken by something the manifest does not show the person.
    if spec.background && spec.triggers.is_empty() {
        return Err(format!("app {app_id} asks for a background agent but declares no triggers"));
    }
    if let Some(path) = &spec.instructions {
        check_bundle_path(path)
            .map_err(|e| format!("app {app_id} agent.instructions: {e}"))?;
        if !path.ends_with(".md") {
            return Err(format!("app {app_id} agent.instructions {path:?} must be a Markdown (.md) file"));
        }
    }
    let mut skills = BTreeSet::new();
    for skill in &spec.skills {
        check_skill_name(skill).map_err(|e| format!("app {app_id}: {e}"))?;
        if !skills.insert(skill.clone()) {
            return Err(format!("app {app_id} names skill {skill:?} twice"));
        }
    }
    if skills.len() > MAX_SKILLS {
        return Err(format!("app {app_id} declares more than {MAX_SKILLS} skills"));
    }
    Ok(AgentPolicy {
        profile: spec.profile,
        tools,
        max_iterations: clamp_u32(spec.max_iterations, limits.max_iterations),
        token_budget: clamp(spec.token_budget, limits.max_token_budget),
        model,
        background_requested: spec.background,
        triggers: spec.triggers.clone(),
        instructions: spec.instructions.clone(),
        skills,
    })
}

/// Ceilings on what an agent declaration may list.
pub const MAX_MODEL_TASKS: usize = 8;
pub const MAX_SKILLS: usize = 16;
pub const MAX_TRIGGERS: usize = 16;

/// A skill's name is its directory under `skills/` and its octos identity.
pub(crate) fn check_skill_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 64 {
        return Err(format!("skill name {name:?} must be 1 to 64 characters"));
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        || name.starts_with('-')
    {
        return Err(format!("skill name {name:?} may hold only [a-z0-9_-] and may not start with '-'"));
    }
    Ok(())
}

/// A plain bundle-relative path: no root, no climbing, no backslash.
pub(crate) fn check_bundle_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains("://")
        || path.split('/').any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("{path:?} must be a plain bundle-relative path"));
    }
    Ok(())
}

/// Schedules are five-field cron; events are the app's own, under its
/// namespace. Anything else is refused rather than guessed at.
fn check_triggers(app_id: &str, triggers: &Triggers) -> Result<(), String> {
    if triggers.schedule.len() > MAX_TRIGGERS || triggers.events.len() > MAX_TRIGGERS {
        return Err(format!("app {app_id} declares more than {MAX_TRIGGERS} schedules or events"));
    }
    for entry in &triggers.schedule {
        let fields: Vec<&str> = entry.split_whitespace().collect();
        let well_formed = fields.len() == 5
            && fields.iter().all(|f| f.len() <= 32 && f.chars().all(|c| c.is_ascii_digit() || matches!(c, '*' | ',' | '/' | '-')));
        if !well_formed {
            return Err(format!(
                "app {app_id} schedule {entry:?} must be five cron fields (minute hour day month weekday) of digits and * , / -"
            ));
        }
    }
    let namespace = short_id(app_id);
    for event in &triggers.events {
        let mut parts = event.split('.');
        let head = parts.next().unwrap_or("");
        let rest: Vec<&str> = parts.collect();
        let well_formed = event.len() <= 64
            && head == namespace
            && !rest.is_empty()
            && rest.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
        if !well_formed {
            return Err(format!(
                "app {app_id} event {event:?} must be {namespace}.<name>: the app's own events, lowercase, at most 64 characters"
            ));
        }
    }
    Ok(())
}

/// An id is a path component of the app's jail, so it may not be empty, may
/// not navigate, and may not surprise a filesystem.
fn check_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err(format!("app id {id:?} must be 1 to 64 characters"));
    }
    if !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.') {
        return Err(format!("app id {id:?} may hold only lowercase letters, digits, '-' and '.'"));
    }
    if id.starts_with('.') || id.contains("..") {
        return Err(format!("app id {id:?} may not navigate the filesystem"));
    }
    Ok(())
}

/// A bare host: no scheme, no path, no port, no wildcard. The service adds
/// HTTPS; the app never names a scheme, so it cannot ask for plain HTTP.
fn check_host(host: &str) -> Result<(), String> {
    if host.is_empty() || host.len() > 253 {
        return Err(format!("host {host:?} must be 1 to 253 characters"));
    }
    if host.contains("://") || host.contains('/') {
        return Err(format!("host {host:?} must be a bare host name, with no scheme or path"));
    }
    if host.contains('*') {
        return Err(format!("host {host:?} may not use a wildcard"));
    }
    if host.contains(':') {
        return Err(format!("host {host:?} may not name a port"));
    }
    if !host.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.') {
        return Err(format!("host {host:?} holds a character a host name may not"));
    }
    if host.starts_with('.') || host.ends_with('.') || host.contains("..") {
        return Err(format!("host {host:?} is not a well-formed host name"));
    }
    Ok(())
}

fn clamp(asked: Option<u64>, ceiling: u64) -> u64 {
    asked.unwrap_or(ceiling).min(ceiling)
}

fn clamp_u32(asked: Option<u32>, ceiling: u32) -> u32 {
    asked.unwrap_or(ceiling).min(ceiling)
}
