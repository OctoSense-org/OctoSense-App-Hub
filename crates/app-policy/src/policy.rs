//! Resolving a manifest into what the app and its agent actually get.
//!
//! What the app may do (capabilities, hosts, storage, budgets, research
//! scope) is the app contract's [`octosense_app_contract::policy::resolve`]
//! (ADR 0005); every rule there fails closed. This module adds the app's
//! agent, which only hosts that run app agents resolve: a tool the host does
//! not offer, a kernel tool, a malformed trigger are refused here. The host
//! [`AppPolicy`] holds the contract's policy and the agent's, and the two
//! containers are derived from it, never set by hand.
use crate::manifest::{short_id, AgentSpec, AppManifest, ModelSpec, ProfileMode, Triggers};
use std::collections::BTreeSet;
use std::ops::Deref;

pub use octosense_app_contract::policy::HostLimits;

/// The octos kernel tools a contained app's agent may keep (a plain name in
/// `agent.tools`; the host's own tools are dotted, `net.fetch`). Only
/// `ask_user_question`: a question to the person, shown and answered on the
/// device's own surfaces, with no side effect of its own. Every other kernel
/// tool (shell, files, the web, memory, peers, spawning) stays out of reach
/// whatever a host offers: [`resolve`] refuses it, and so does the gate's
/// review of the agent ([`crate::agent::review`]).
pub const KERNEL_TOOLS: &[&str] = &["ask_user_question"];

/// Whether `tool` names an octos kernel tool rather than a host tool
/// (`net.fetch`) or another app's (`mail.send`): a plain name.
pub fn is_kernel_tool_name(tool: &str) -> bool {
    !tool.contains('.')
}

/// What the app and its agent get. Produced only by [`resolve`].
///
/// The app's part is the contract's [`octosense_app_contract::AppPolicy`],
/// reached through [`AppPolicy::app`] or, field by field, through `Deref`
/// (`policy.capabilities`, `policy.allows_host(..)`), so code written when
/// both were one struct reads the same.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppPolicy {
    /// What the app may do: capabilities, hosts, storage, budgets, research.
    pub app: octosense_app_contract::AppPolicy,
    /// None when the manifest asked for no agent.
    pub agent: Option<AgentPolicy>,
}

impl Deref for AppPolicy {
    type Target = octosense_app_contract::AppPolicy;

    fn deref(&self) -> &Self::Target {
        &self.app
    }
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

/// Resolve a parsed manifest against this host: the contract's rules for
/// the app, then the agent's.
pub fn resolve(manifest: &AppManifest, limits: &HostLimits) -> Result<AppPolicy, String> {
    let app = octosense_app_contract::policy::resolve(manifest, limits)?;
    let cross_reads = manifest.requires.iter().any(|f| f == "cross-app-read-v1");
    if cross_reads && (!manifest.capabilities.iter().any(|c| c == "app_tools") || manifest.host_api.as_ref().and_then(|api| api.required.get("app_tools.read")) != Some(&1)) {
        return Err("cross-app-read-v1 requires app_tools capability and app_tools.read@1".into());
    }
    let agent = match &manifest.agent {
        None => None,
        Some(spec) => Some(resolve_agent(&manifest.id, spec, limits, cross_reads)?),
    };
    Ok(AppPolicy { app, agent })
}

fn resolve_agent(app_id: &str, spec: &AgentSpec, limits: &HostLimits, cross_reads: bool) -> Result<AgentPolicy, String> {
    let mut tools = BTreeSet::new();
    for tool in &spec.tools {
        if is_kernel_tool_name(tool) && !KERNEL_TOOLS.contains(&tool.as_str()) {
            return Err(format!(
                "app {app_id} requests the octos kernel tool {tool:?}; a contained app's agent may keep only {}",
                KERNEL_TOOLS.join(", ")
            ));
        }
        // Deferred requests are not execution grants. At invocation, the host
        // resolves an admitted installed shared script read and both consents.
        let requested_read = cross_reads && tool.len() <= 256 && tool.contains('.')
            && tool.split('.').all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'));
        if !limits.offered_tools.iter().any(|offered| offered == tool) && !requested_read {
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

fn clamp(asked: Option<u64>, ceiling: u64) -> u64 {
    asked.unwrap_or(ceiling).min(ceiling)
}

fn clamp_u32(asked: Option<u32>, ceiling: u32) -> u32 {
    asked.unwrap_or(ceiling).min(ceiling)
}

#[cfg(test)] mod cross_app_read_tests {
    use super::*;
    // Exercise policy resolution only; actual bundle signing is tested by
    // the two independently signed installed-app probe.
    fn limits() -> HostLimits { HostLimits::default().with_require_signature(false) }
    fn manifest(tool: &str, enabled: bool) -> AppManifest {
        let mut value = serde_json::json!({"schema":1,"id":"readfixture","name":"Read fixture","version":"0.1.0","integrity":{"bundle_blake3":"0".repeat(64)},"capabilities":["app_tools"],"requires":["host-api-v1"],"host_api":{"required":{"app_tools.read":1}},"agent":{"profile":"read-only","tools":[tool]}});
        if enabled { value["requires"].as_array_mut().unwrap().push("cross-app-read-v1".into()); }
        AppManifest::parse(&value.to_string()).unwrap()
    }
    #[test] fn ordinary_unknown_tool_still_refused() {
        assert!(resolve(&manifest("other.read_events",false),&limits()).unwrap_err().contains("does not offer"));
    }
    #[test] fn explicit_read_request_is_deferred_not_an_execution_grant() {
        let policy=resolve(&manifest("other.read_events",true),&limits()).unwrap();
        assert!(policy.agent.unwrap().tools.contains("other.read_events"));
        assert!(!HostLimits::default().offered_tools.contains(&"other.read_events".into()));
    }
    #[test] fn kernel_and_malformed_names_remain_refused() {
        for tool in ["shell","read_file","peer_send_input","bad..read","other.READ"] {
            let error=resolve(&manifest(tool,true),&limits()).unwrap_err();
            assert!(error.contains(if tool.contains('.') { "does not offer" } else { "kernel tool" }),"{tool}: {error}");
        }
    }
    #[test] fn native_bridge_requirement_is_mandatory() {
        let mut m=manifest("other.read_events",true);m.host_api=None;
        assert!(resolve(&m,&limits()).unwrap_err().contains("cross-app-read-v1"));
    }
}
