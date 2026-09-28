//! What an installed app declares about itself.
//!
//! The manifest is the ONLY thing an app may say about its own limits, and
//! saying it is not the same as getting it: every field is a request that
//! [`crate::policy`] resolves against the host's ceilings. Unknown fields are
//! refused rather than ignored, so a manifest written for a newer host does
//! not silently run with less containment than it asked for.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The manifest schema this build understands. A bundle declaring anything
/// else is refused: an older host must not guess at a newer grammar.
pub const SCHEMA: u32 = 1;

/// Every capability an app may request. The list is closed on purpose — a
/// capability that is not here cannot be granted, so adding one is a change
/// to this file and to the service that enforces it, together.
pub const KNOWN_CAPABILITIES: &[&str] = &[
    // Read and write inside the app's own storage jail.
    "storage",
    // Make requests, but only to the hosts in `network.hosts`.
    "net",
    // Raise a prompt the person answers (a permission ask, a confirmation).
    "prompt",
    // Read the shared ledger. Writing is always the app's own rows.
    "ledger.read",
    // Location, camera and clipboard reach the person's world; each is a
    // separate consent, never implied by another.
    "location",
    "camera",
    "clipboard",
    // Show pictures from any public https host, not just `network.hosts`:
    // a feed reader's thumbnails come from wherever its stories link.
    "images",
    // Open any public https page in the system WebView, which gets no way
    // back into the app: a reader for the stories it lists.
    "web",
    // Record sound with a camera video.
    "microphone",
    // Offer what it captures to the system photo library, where other apps
    // can see it; without this, captures stay in the app's own storage.
    "library",
    // Read and send mail through the host's mail service, from accounts the
    // person signs in to on the host's own sheet. The app never holds the
    // password or the connection.
    "mail",
    // See and arrange the assistant's LLM providers through the host's llm
    // service. Keys are typed, shown as a QR and scanned only on the host's
    // own sheets; the app sees masked status, never a key.
    "llm",
    // Read the host's news service: feeds and topic feeds it collects on a
    // schedule into the app's store, and the items' text. The app never
    // fetches arbitrary sites itself through it.
    "news",
    // Publish cards to the glance screen through the host's glance service:
    // L0 cards the shell checks, caps, rate-limits and expires, keyed to the
    // app itself. The app sees only its own cards and a card opens only it.
    "glance",
    // Make bounded one-shot model calls through the host's model service
    // (`model.complete`): the app names a model class ("fast" or "strong")
    // and a JSON Schema; the host picks the model from the person's own
    // providers, validates the reply against the schema and keeps a per-app
    // daily budget. No tools, memory or history; the app never sees the
    // provider, model id or key. The app's inputs go to the AI provider the
    // person configured. Not `llm`, which manages providers for os.* apps.
    "model",
    // Host services reached by exact name (see [`crate::services`]). Each is
    // a separate consent: a host adapter checks the exact name, the person's
    // per-instance grant and its own ceilings on every request. A prefix is
    // never a grant: `octos.` or `matrix.` alone is an unknown capability,
    // and history access does not imply starting a turn.
    "matrix.account_info",
    "matrix.device",
    "matrix.dm_find",
    "matrix.dm_open",
    "matrix.event",
    "matrix.favorite",
    "matrix.ignored_users",
    "matrix.invite",
    "matrix.invite_respond",
    "matrix.invites",
    "matrix.join",
    "matrix.low_priority",
    "matrix.mark_unread",
    "matrix.older_messages",
    "matrix.permalink",
    "matrix.pin",
    "matrix.pinned_events",
    "matrix.power_levels",
    "matrix.profile",
    "matrix.react",
    "matrix.read_messages",
    "matrix.read_receipt",
    "matrix.read_receipts",
    "matrix.reply",
    "matrix.room_info",
    "matrix.room_members",
    "matrix.room_preview",
    "matrix.room_threads",
    "matrix.rooms_info",
    "matrix.rooms_list",
    "matrix.rooms_messages",
    "matrix.rooms_search",
    "matrix.rooms_send",
    "matrix.search_room",
    "matrix.search_rooms",
    "matrix.send_message",
    "matrix.space_info",
    "matrix.space_rooms",
    "matrix.spaces",
    "matrix.successor",
    "matrix.thread_replies",
    "matrix.thread_reply",
    "matrix.typing",
    "matrix.unread",
    "matrix.user_profile",
    "octos.session.open",
    "octos.session.history",
    "octos.turn.start",
    "octos.turn.interrupt",
];

/// The permission profiles an app's agent session may ask for. Full access is
/// absent by construction: it is an operator setting for machines they own,
/// and no manifest may name it (ADR 0002, "the rule at the boundary").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileMode {
    /// No writes at all; every write asks.
    ReadOnly,
    /// Read and write inside the workspace; anything else asks.
    WorkspaceWrite,
    /// Read and write inside the workspace; anything else is refused
    /// outright rather than asked. The right default for an unattended app.
    WorkspaceWriteNeverAsk,
}

impl ProfileMode {
    /// The string the kernel's permission profile uses.
    pub fn as_kernel_mode(self) -> &'static str {
        match self {
            ProfileMode::ReadOnly => "read-only",
            ProfileMode::WorkspaceWrite => "workspace-write",
            ProfileMode::WorkspaceWriteNeverAsk => "workspace-write-never",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppManifest {
    /// Must equal [`SCHEMA`].
    pub schema: u32,
    /// Stable identity. Also the name of the app's storage jail, so it is
    /// constrained to the characters a path component may hold.
    pub id: String,
    /// Opaque to the host, but pinned: a different version is a different
    /// bundle and must be admitted again.
    pub version: String,
    /// What a person calls it.
    pub name: String,
    pub integrity: Integrity,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub network: Network,
    #[serde(default)]
    pub storage: Storage,
    #[serde(default)]
    pub compute: Compute,
    /// Absent means the app gets no agent at all, which is the default.
    #[serde(default)]
    pub agent: Option<AgentSpec>,
}

/// What the bundle must hash to. The digest covers the bundle bytes as they
/// were signed; a signature, when we have one, signs this manifest.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Integrity {
    /// Lowercase hex blake3 of the bundle.
    pub bundle_blake3: String,
    /// Detached signature over the canonical manifest bytes, if the host
    /// requires signing. Verified by a [`crate::verify::SignatureVerifier`].
    #[serde(default)]
    pub signature: Option<Signature>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Signature {
    /// Which key signed it, as the host knows the key.
    pub key_id: String,
    /// Lowercase hex signature bytes.
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    /// The hosts this app may reach, exactly. No wildcards, no schemes, no
    /// paths: a host and nothing else, and the request is HTTPS by the time
    /// the service makes it.
    #[serde(default)]
    pub hosts: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Storage {
    /// Whole-jail ceiling the app asks for. Clamped to the host's maximum.
    #[serde(default)]
    pub max_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Compute {
    /// Script instructions the app may run per session, cumulative — not the
    /// per-evaluation cap, which only stops one runaway expression.
    #[serde(default)]
    pub instruction_budget: Option<u64>,
    /// Ceiling for the isolate's heap.
    #[serde(default)]
    pub memory_bytes: Option<u64>,
}

/// The agent session an app asks for. Everything here is bounded by the same
/// manifest: its workspace is the app's jail, its network is the app's hosts.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentSpec {
    pub profile: ProfileMode,
    /// The tools the session may call. Closed list, resolved against the
    /// host's own allowlist; shell and arbitrary file tools are never in it.
    #[serde(default)]
    pub tools: Vec<String>,
    /// Model turns per request before the session stops and reports.
    #[serde(default)]
    pub max_iterations: Option<u32>,
    /// Tokens the session may spend per request.
    #[serde(default)]
    pub token_budget: Option<u64>,
    /// What the agent needs from a model, never which model (ADR 0002 §3).
    /// The host picks one from the person's providers that meets it.
    ///
    /// Every field added after the first release of schema 1 is skipped when
    /// it holds its default, so a manifest written before it existed
    /// serialises, and therefore signs, exactly as it did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelSpec>,
    /// The agent may run while the app is closed, woken by `triggers`. A
    /// request: the person grants or refuses it per app, and it requires at
    /// least one trigger.
    #[serde(default, skip_serializing_if = "is_false")]
    pub background: bool,
    /// What wakes the agent besides the person (ADR 0002 §2).
    #[serde(default, skip_serializing_if = "Triggers::is_empty")]
    pub triggers: Triggers,
    /// The bundle-relative path of the agent's instructions, conventionally
    /// [`crate::agent::AGENT_FILE`]. Named here so the file is declared, not
    /// merely present; the digest pins its bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// The skills the bundle ships under `skills/<name>/`. Installed into
    /// this app's peer workspace only, and data-only for a contained app.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// What a task needs from a model. Closed: a need that is not here cannot be
/// matched by any host, so adding one is a change here and in the host's
/// model selection, together.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelNeed {
    /// Structured tool calls. An agent with any tools needs this.
    ToolCalling,
    /// Image input, for example to critique a rendered card.
    Vision,
    /// A long context window (the host decides what "long" means today).
    LongContext,
    /// A reasoning (thinking) model.
    Reasoning,
    /// Reliable JSON output against a schema.
    StructuredOutput,
    /// Reads and writes more than one language well.
    Multilingual,
}

/// Every [`ModelNeed`], as the manifest spells it.
pub const KNOWN_MODEL_NEEDS: &[&str] =
    &["tool_calling", "vision", "long_context", "reasoning", "structured_output", "multilingual"];

/// How capable (and costly) a model the task deserves. The host maps a tier
/// to the person's providers; policy may lower it (budget, battery).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelTier {
    Fast,
    #[default]
    Standard,
    Strong,
}

/// The model an app's agent needs. Never a provider or a model name: the
/// app cannot know which ones the person configured, and never sees keys.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSpec {
    #[serde(default)]
    pub needs: Vec<ModelNeed>,
    #[serde(default)]
    pub tier: ModelTier,
    /// Only a model that runs on the device (or the person's own machine)
    /// may see this app's data. App-wide: a task cannot relax it.
    #[serde(default)]
    pub local_only: bool,
    /// Different requirements for named tasks, for example a fast model for
    /// `triage` and a strong one for `synthesis`. Task names are the app's
    /// own (`[a-z_]{1,32}`); `AGENT.md` says which task a step is.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub per_task: BTreeMap<String, TaskModel>,
}

/// One task's model requirements. `local_only` is deliberately absent: it
/// holds for the whole app.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskModel {
    #[serde(default)]
    pub needs: Vec<ModelNeed>,
    #[serde(default)]
    pub tier: ModelTier,
}

/// What wakes an app's agent without the person asking.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Triggers {
    /// Five-field cron expressions (minute hour day-of-month month
    /// day-of-week), in the device's local time: `"0 7 * * *"`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schedule: Vec<String>,
    /// Events from the app's own host service, in the app's namespace:
    /// `"news.items.new"`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<String>,
}

impl Triggers {
    pub fn is_empty(&self) -> bool {
        self.schedule.is_empty() && self.events.is_empty()
    }
}

/// The namespace an app's tools and events live in: the last segment of its
/// id (`os.news` and `dev.example.news` are both `news`). Peers are per app,
/// so two apps with the same short id never share a tool registry.
pub fn short_id(app_id: &str) -> &str {
    app_id.rsplit('.').next().unwrap_or(app_id)
}

impl AppManifest {
    /// Parse a manifest, refusing unknown fields and a foreign schema.
    pub fn parse(json: &str) -> Result<Self, String> {
        let manifest: AppManifest = serde_json::from_str(json).map_err(|e| format!("manifest is not valid: {e}"))?;
        if manifest.schema != SCHEMA {
            return Err(format!("manifest schema {} is not {}", manifest.schema, SCHEMA));
        }
        Ok(manifest)
    }

    /// The bytes a signature covers: the manifest without its own signature,
    /// serialised canonically, so the same manifest always signs the same way.
    pub fn signing_bytes(&self) -> Result<Vec<u8>, String> {
        let mut bare = self.clone();
        bare.integrity.signature = None;
        let value = serde_json::to_value(&bare).map_err(|e| e.to_string())?;
        Ok(canonical(&value).into_bytes())
    }
}

/// Canonical JSON: object keys sorted, no insignificant whitespace. Enough
/// for a stable signing input; it is not a general JCS implementation.
fn canonical(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let body: Vec<String> = keys
                .iter()
                .map(|k| format!("{}:{}", serde_json::Value::String((*k).clone()), canonical(&map[*k])))
                .collect();
            format!("{{{}}}", body.join(","))
        }
        serde_json::Value::Array(items) => {
            let body: Vec<String> = items.iter().map(canonical).collect();
            format!("[{}]", body.join(","))
        }
        other => other.to_string(),
    }
}
