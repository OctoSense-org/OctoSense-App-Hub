//! The app: one card bundle in a policy-configured splash isolate.
//!
//! The order is the one ADR 0002 fixes: admit, resolve, apply, then evaluate.
//! Nothing here may widen what the manifest asked for, and the two settings
//! the runtime cannot enforce yet are printed rather than assumed.
use makepad_widgets::*;
use crate::args::Args;
use octosense_app_policy::{admit_and_resolve_dir, AppPolicy, HostLimits, RefuseAllSignatures};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

app_main!(App, font_set: International);

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    startup() do #(App::script_component(vm)) {
        ui: Root {
            main_window := Window {
                window.title: "Card host"
                window.inner_size: vec2(412, 892)
                pass +: { clear_color: #fff }
                body +: {
                    padding: 0 margin: 0 spacing: 0 flow: Overlay
                    card := Splash { width: Fill height: Fill }
                    sheet := Splash { visible: false width: Fill height: Fill }
                }
            }
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    mounted: bool,
    #[rust]
    assets: Option<octosense_app_policy::AssetServer>,
    #[rust]
    app_id: String,
    #[rust]
    host_dir: PathBuf,
}

/// The command line, parsed by `main` before the app starts.
static ARGS: OnceLock<Args> = OnceLock::new();

/// Start the app with `args`. Returns when the app quits.
pub fn run(args: Args) {
    let _ = ARGS.set(args);
    app_main();
}

/// Read `--static` directories into memory for the life of the process.
fn load_statics(mounts: &[(String, PathBuf)]) -> octosense_app_policy::StaticAssets {
    let mut out: Vec<(&'static str, &'static [u8])> = Vec::new();
    for (prefix, dir) in mounts {
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(bytes) = std::fs::read(&path) else { continue };
            let name = format!("{prefix}/{}", entry.file_name().to_string_lossy());
            out.push((Box::leak(name.into_boxed_str()), Box::leak(bytes.into_boxed_slice())));
        }
    }
    Box::leak(out.into_boxed_slice())
}

/// Admit the bundle and resolve what it gets. Refusals are fatal: a card that
/// cannot be admitted must not be drawn, not even partially.
fn policy_for(args: &Args) -> Result<AppPolicy, String> {
    let manifest_path = args.bundle.join(octosense_app_policy::MANIFEST_FILE);
    let mut manifest_json = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("{}: {e}", manifest_path.display()))?;
    let digest = octosense_app_policy::digest_dir(&args.bundle)?;

    if args.stamp {
        let mut value: serde_json::Value = serde_json::from_str(&manifest_json).map_err(|e| e.to_string())?;
        value["integrity"]["bundle_blake3"] = serde_json::Value::String(digest.clone());
        manifest_json = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
        std::fs::write(&manifest_path, format!("{manifest_json}\n")).map_err(|e| e.to_string())?;
        log!("card-host: stamped {} with digest {}", manifest_path.display(), digest);
    }

    // A system app's source manifest leaves its digest empty: the build
    // stamps it into the packed copy. Developing one, stamp it in memory.
    if args.system && !args.stamp {
        let mut value: serde_json::Value = serde_json::from_str(&manifest_json).map_err(|e| e.to_string())?;
        if value["integrity"]["bundle_blake3"].as_str().unwrap_or("").is_empty() {
            value["integrity"]["bundle_blake3"] = serde_json::Value::String(digest.clone());
            manifest_json = serde_json::to_string(&value).map_err(|e| e.to_string())?;
        }
    }

    let limits = if args.system {
        HostLimits::system()
    } else {
        HostLimits { require_signature: !args.allow_unsigned, ..HostLimits::default() }
    };
    // The digest is computed from the directory, so the manifest's claim is
    // checked against what is actually there.
    admit_and_resolve_dir(&manifest_json, &digest, &limits, &RefuseAllSignatures)
}

/// Lower the card to isolate source: realize it, then lower it with the kit
/// that ships in the bundle. Nothing is read from outside the bundle.
///
/// An L0 card also logs one `card-host: realize {json}` line (see
/// [`realize_report`]) before it is lowered, so a harness reading the log —
/// card-studio does — gets the lint result, the realize report and each
/// source's `$state`, whether or not the card then lowers.
fn card_source(bundle: &Path, asset_origin: &str) -> Result<String, String> {
    // A script app runs its own program.
    if let Some(script) = octosense_app_policy::script_source(bundle, asset_origin) {
        return script;
    }
    let card = std::fs::read_to_string(bundle.join("page.card")).map_err(|e| format!("page.card: {e}"))?;
    let data_text = std::fs::read_to_string(bundle.join("page.data.json")).unwrap_or_else(|_| "{}".into());
    let mut data: serde_json::Value = serde_json::from_str(&data_text).map_err(|e| format!("page.data.json: {e}"))?;
    octosense_app_policy::rewrite_assets(&mut data, asset_origin);
    let mut report = realize_report(&card, &data);
    let lowered = lower(&card, &data, &bundle.join("kit"));
    match &lowered {
        Ok((_, lowering)) => report["lowering"] = serde_json::Value::String((*lowering).into()),
        Err(e) => report["lower_error"] = serde_json::Value::String(e.clone()),
    }
    log!("card-host: realize {report}");
    // The isolate's prelude opens a `View{`; the body continues it, exactly as
    // a splash app's body does.
    lowered.map(|(ui, _)| format!("width:Fill height:Fill flow:Overlay {ui}"))
}

/// Lower a realized card to Makepad UI, naming the lowering used.
///
/// A card built from a native kit pack (the image-to-card flow's measured
/// placements) goes through the measured-design lowering, as before. A plain
/// L0 card composed from the role kit (`Surface`, `Col`, `TextTitle`, …) has
/// no placements, so the measured lowering refuses it ("unsupported measured
/// design node"); it falls back to the kit lowering the Card runner uses,
/// with inspectable ids (`beauty_0_1_…`) so `/snap` can name every node.
fn lower(card: &str, data: &serde_json::Value, kit: &Path) -> Result<(String, &'static str), String> {
    let mut prepared = octoscript_makepad::l0::prepare(card, data, kit)?;
    match octoscript_makepad::design::to_makepad_ui(&prepared.tree) {
        Ok(ui) => Ok((ui, "design")),
        Err(e) if prepared.native_components => Err(e),
        Err(_) => {
            octoscript_makepad::l0::inspectable(&mut prepared.tree);
            Ok((octoscript_makepad::to_makepad_l0_ui(&prepared.tree), "l0-kit"))
        }
    }
}

/// The card's lint result, realize report and source lifecycle, as JSON.
///
/// This is what ADR 0002 §7 asks a render tool to return next to the frame:
/// `lint` is `check_ui_l0` (valid, level, diagnostics), `realize` is the
/// realization the card is drawn from (`nodes`, `truncated` — a bound was
/// reached and the tree is partial — and diagnostics), and `sources` gives
/// each declared source's `$state` as the card sees it: the host's
/// `$status` entry if it reported one, else `ready` with a value, `pending`
/// without.
fn realize_report(card: &str, data: &serde_json::Value) -> serde_json::Value {
    use octoscript_ui_l0 as l0;
    let diagnostics = |ds: &[l0::SyntaxDiagnostic]| -> Vec<serde_json::Value> {
        ds.iter()
            .map(|d| serde_json::json!({"line": d.line, "column": d.column, "message": d.message}))
            .collect()
    };
    let lint = l0::check_ui_l0(card);
    let realized = l0::realize(card, data, Default::default());
    let sources: Vec<serde_json::Value> = l0::source_plan(card)
        .requests
        .iter()
        .map(|request| {
            let reported = data.get("$status").and_then(|s| s.get(&request.name)).and_then(|v| v.as_str());
            let present = request
                .name
                .split('.')
                .try_fold(data, |value, key| value.get(key))
                .is_some_and(|value| !value.is_null());
            let state = reported.unwrap_or(if present { "ready" } else { "pending" });
            serde_json::json!({"name": request.name, "helper": request.helper, "state": state})
        })
        .collect();
    serde_json::json!({
        "lint": {
            "valid": lint.valid,
            "level": format!("{:?}", lint.level),
            "diagnostics": diagnostics(&lint.diagnostics),
            "diagnostics_truncated": lint.diagnostics_truncated,
        },
        "realize": {
            "nodes": realized.nodes,
            "truncated": realized.truncated,
            "diagnostics": diagnostics(&realized.diagnostics),
        },
        "sources": sources,
    })
}

/// Give every card isolate the kit vocabulary it needs to draw.
///
/// An isolate starts with the standard widgets only, so a lowered L0 card,
/// which names `DesignSurface`, `KitButton` and friends, does not evaluate
/// without this. Note the shape of the hook: it is PROCESS-WIDE, so every
/// isolate gets the same vocabulary. ADR 0002 phase 1 wants this per app,
/// from the manifest; until the runtime can do that, a host must choose one
/// vocabulary for all its cards and keep it to drawing.
fn register_card_vocabulary() {
    use makepad_widgets::widget_async::register_splash_isolate_mod;
    fn design(vm: &mut ScriptVm) {
        octoscript_widgets::design::script_mod(vm);
    }
    fn kit(vm: &mut ScriptVm) {
        octoscript_widgets::kit::script_mod(vm);
    }
    register_splash_isolate_mod(design);
    register_splash_isolate_mod(kit);
    register_splash_isolate_mod(makepad_widgets::splash::register_agent_module);
}

impl App {
    fn mount(&mut self, cx: &mut Cx) {
        let Some(args) = ARGS.get() else {
            error!("card-host: started without its arguments");
            return;
        };
        if let Some((w, h)) = args.size {
            // A sized window is a render target (card-studio): the card gets
            // the whole inner size, so the caption bar goes.
            let mut window = self.ui.widget(cx, ids!(main_window));
            script_apply_eval!(cx, window, { show_caption_bar: false });
            let size = dvec2(w, h);
            self.ui.window(cx, ids!(main_window)).configure_window(cx, size, dvec2(40.0, 40.0), false, "Card host".into());
            self.ui.window(cx, ids!(main_window)).resize(cx, size);
            log!("card-host: window sized {w}x{h}");
        }
        let policy = match policy_for(args) {
            Ok(policy) => policy,
            Err(e) => {
                error!("card-host: refused: {e}");
                return;
            }
        };
        log!(
            "card-host: {} {} admitted — capabilities {:?}, hosts {:?}, storage {} bytes, agent {}",
            policy.app_id,
            policy.version,
            policy.capabilities,
            policy.hosts,
            policy.storage_bytes,
            policy.agent.as_ref().map(|a| a.profile.as_kernel_mode()).unwrap_or("none"),
        );

        self.app_id = policy.app_id.clone();
        self.host_dir = args.app_data.join(".host");
        let mut settings = policy.isolate_settings(&args.app_data);
        if let Err(e) = std::fs::create_dir_all(&settings.jail_root) {
            error!("card-host: cannot make the app's jail at {}: {e}", settings.jail_root.display());
            return;
        }
        // The app's artwork is served from a loopback origin of our own,
        // serving only its bundle, and that origin — this port, no other — is
        // the one loopback entry the isolate may reach.
        let server = match octosense_app_policy::AssetServer::start_with_static(&args.bundle, load_statics(&args.statics)) {
            Ok(server) => server,
            Err(e) => {
                error!("card-host: cannot serve the app's artwork: {e}");
                return;
            }
        };
        settings.hosts.push(server.allowlist_entry());
        let origin = server.origin().to_string();
        self.assets = Some(server);
        let splash = self.ui.splash(cx, ids!(card));
        let applied = octosense_app_policy::splash_adapter::apply(&splash, cx, &settings);
        log!(
            "card-host: isolate jailed at {} with {} bytes, {} capability(ies), {} host(s), {} instructions, {} bytes of heap, prompts {} — all enforced",
            settings.jail_root.display(),
            applied.storage_quota,
            applied.capabilities,
            applied.hosts,
            applied.instruction_budget,
            applied.memory_bytes,
            settings.host_prompts,
        );

        // The session profile this app's agent would run under. Printed here
        // so the two containers can be compared at a glance; asking the kernel
        // for it is the next phase.
        if let Some(session) = policy.session_profile(&args.app_data) {
            match serde_json::to_string(&session) {
                Ok(json) => log!("card-host: agent session profile {json}"),
                Err(e) => error!("card-host: cannot render the session profile: {e}"),
            }
        }

        match card_source(&args.bundle, &origin) {
            Ok(source) => splash.set_text(cx, &source),
            Err(e) => error!("card-host: the card did not lower: {e}"),
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        makepad_widgets::script_mod(vm);
        octoscript_widgets::design::script_mod(vm);
        octoscript_widgets::kit::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if !self.mounted {
            self.mounted = true;
            register_card_vocabulary();
            self.mount(cx);
        }
        self.ui.handle_event(cx, event, &mut Scope::empty());
        // Host services (a sheet the service raises, answers from its
        // workers), exactly as the Card runner does.
        let (card, sheet) = (self.ui.splash(cx, ids!(card)), self.ui.splash(cx, ids!(sheet)));
        octosense_appstore::services::pump(cx, &self.app_id, &self.host_dir, &card, &sheet);
    }
}
