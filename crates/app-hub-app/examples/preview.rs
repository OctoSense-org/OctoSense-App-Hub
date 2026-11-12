//! Native design preview at a phone's size, or OCTOSENSE_PREVIEW_SIZE. Installation
//! uses OCTOSENSE_APP_DATA; launch actions are logged here. Use the OctoSense shell
//! to open apps. In a Mac style the window is what the shell gives App Hub (its
//! `unified_chrome`): the app draws to the top edge. OCTOSENSE_PREVIEW_ICONS names a
//! folder of `<app>.svg` that stands in for the art a shell installs.
pub use makepad_widgets;
use makepad_widgets::*;
app_main!(App);
fn preview_size() -> Vec2d {
    std::env::var("OCTOSENSE_PREVIEW_SIZE").ok().and_then(|value| {
        let (width, height) = value.split_once('x')?;
        let (width, height) = (width.parse::<f64>().ok()?, height.parse::<f64>().ok()?);
        (width >= 320.0 && height >= 480.0).then_some(dvec2(width, height))
    }).unwrap_or(dvec2(406.0, 820.0))
}
fn mac_style() -> bool {
    std::env::var("MAKEPAD_WIDGET_STYLE").is_ok_and(|style| style.starts_with("macos"))
}
fn install_icons(cx: &mut Cx) {
    let Ok(folder) = std::env::var("OCTOSENSE_PREVIEW_ICONS") else { return };
    let Ok(files) = std::fs::read_dir(folder) else { return };
    let style = desktop_style::DesktopStyle::Macos;
    let mut assets = app_icon::load_assets(style);
    for file in files.flatten() {
        let path = file.path();
        let (Some(name), Ok(svg)) = (path.file_stem().and_then(|s| s.to_str()), std::fs::read_to_string(&path)) else { continue };
        match assets.iter_mut().find(|asset| asset.name == name) {
            Some(asset) => asset.svg = svg,
            None => assets.push(app_icon::IconAsset { name: name.into(), svg }),
        }
    }
    app_icon::install(cx, style, &assets);
}
script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*
    startup() do #(App::script_component(vm)) {
        ui: Root {
            main_window := Window {
                window.title: "App Hub · Native preview"
                window.inner_size: #(preview_size())
                body +: {padding: 0 margin: 0 spacing: 0 hub := AppHubView{}}
            }
        }
    }
}
#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}
impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        makepad_widgets::script_mod(vm);
        if mac_style() {
            script_eval!(vm, { mod.theme.ui_unified = true });
        }
        octosense_app_hub_app::view::script_mod(vm);
        self::script_mod(vm)
    }
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::Startup = event {
            install_icons(cx);
        }
        if let Event::Actions(actions) = event {
            for action in actions {
                if let Some(action) = action.as_widget_action() {
                    if let octosense_app_hub_app::AppHubAction::Launch(id) = action.cast() {
                        log!("apphub-preview: launch {id}");
                    }
                }
            }
        }
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
