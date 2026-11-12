//! Local-only signed catalog for testing App Hub install/open. Never publish it.
//!
//! cargo run -p octosense-app-hub-app --example fixture -- target/app-hub/fixture
//! Read the generated environment.json and set OCTOSENSE_HUB,
//! OCTOSENSE_HUB_ANCHOR and OCTOSENSE_APP_DATA before launching OctoSense.
//! The actual card host consumes page.card + page.data.json + kit/native/light,
//! not main.splash. Each fixture uses those files to draw native widgets.
//!
//! `hub/catalog-next.json` is the same hub a release later (Trail Notes 1.1.0,
//! signed in the same run): copy it over `hub/catalog.json` after installing
//! Trail Notes to see its update offered.
//!
//! `--with-app-for-newer-octosense` adds Star Chart, which needs a host API no
//! OctoSense has (`fixture.future@1`): App Hub shows why it can't be had.
//!
//! `--published 2026-11-06T00:01:00Z` signs both catalogs as published then
//! rather than now: a test of the date a device shows for a UTC time.

use octosense_app_hub::{Catalog, Entry, HubKey, PublisherKeys, Source, Status};
use octosense_app_policy::{AppManifest, HostLimits, Listing};
use serde_json::{json, Value};
use std::path::Path;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let usage = "Usage: fixture OUTPUT_DIRECTORY (must be empty) [--with-app-for-newer-octosense] [--published YYYY-MM-DDTHH:MM:SSZ]";
    let result = args.next().ok_or_else(|| usage.to_string()).and_then(|path| {
        let (mut newer, mut published) = (false, octosense_app_hub::utc_now());
        while let Some(arg) = args.next() {
            if arg == "--with-app-for-newer-octosense" {
                newer = true;
            } else if arg == "--published" {
                published = args.next().and_then(|time| time.into_string().ok()).ok_or_else(|| usage.to_string())?;
            } else {
                return Err(usage.to_string());
            }
        }
        generate_at(Path::new(&path), newer, &published)
    });
    match result {
        Ok(environment) => println!("{}", serde_json::to_string_pretty(&environment).unwrap()),
        Err(error) => {
            eprintln!("fixture: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
fn generate(output: &Path, newer: bool) -> Result<Value, String> {
    generate_at(output, newer, &octosense_app_hub::utc_now())
}

fn generate_at(output: &Path, newer: bool, published: &str) -> Result<Value, String> {
    if output.exists()
        && std::fs::read_dir(output)
            .map_err(|e| e.to_string())?
            .next()
            .is_some()
    {
        return Err(
            "The output directory must be empty; use a new directory for each fixture catalog"
                .into(),
        );
    }
    std::fs::create_dir_all(output).map_err(|e| e.to_string())?;
    let output = std::fs::canonicalize(output).map_err(|e| e.to_string())?;
    let hub = output.join("hub");
    let apps = output.join("apps");
    std::fs::create_dir_all(hub.join("artifacts")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&apps).map_err(|e| e.to_string())?;
    // Keys are fresh for this run and remain in memory. Only public keys and
    // signatures are written; no production publisher or anchor is involved.
    let anchor = HubKey::generate();
    let working = HubKey::generate();
    let publisher = HubKey::generate();
    let mut entries = Vec::new();
    let mut releases = vec![
        (
            "trail-notes",
            "1.0.0",
            "Trail Notes",
            "Small notes for your next adventure",
            "3.2 km",
            "Ridge trail · an easy afternoon",
            "travel",
            &["storage"][..],
            0xff246753_u64,
            "The first release.",
        ),
        (
            "focus-timer",
            "1.0.0",
            "Focus Timer",
            "A quiet place to focus",
            "25:00",
            "Your next focus session",
            "productivity",
            &[][..],
            0xff345f95_u64,
            "The first release.",
        ),
        (
            "field-guide",
            "1.0.0",
            "Field Guide",
            "Photograph what you find and look it up",
            "142",
            "Birds seen this year",
            "photo-video",
            &["storage", "camera", "microphone", "location", "clipboard", "files", "prompt"][..],
            0xff7a4b9a_u64,
            "The first release.",
        ),
        (
            "trail-notes",
            "1.1.0",
            "Trail Notes",
            "Small notes for your next adventure",
            "5.8 km",
            "Lake loop · a longer afternoon",
            "travel",
            &["storage"][..],
            0xff246753_u64,
            "A longer walk on its screen.",
        ),
    ];
    if newer {
        // Before the later release, which stays last.
        releases.insert(releases.len() - 1, (
            "star-chart",
            "1.0.0",
            "Star Chart",
            "Name the stars above you",
            "1,204",
            "Stars named tonight",
            "travel",
            &["storage"][..],
            0xff2b3f73_u64,
            "The first release.",
        ));
    }
    for (id, version, name, subtitle, value, detail, category, capabilities, accent, notes) in releases {
        let artifact = format!("artifacts/{id}-{version}.bundle");
        let bundle = hub.join(&artifact);
        std::fs::create_dir_all(bundle.join("kit/native/light")).map_err(|e| e.to_string())?;
        let texts = [
            "LOCAL APP",
            name,
            subtitle,
            value,
            detail,
            "Installed from your local App Hub.\nThis is a static validation fixture.",
            if capabilities.contains(&"storage") {
                "Permission: its own local storage"
            } else {
                "No device permissions requested"
            },
        ];
        let roles = [
            "tag",
            "title",
            "subtitle",
            "value",
            "detail",
            "footer",
            "permission",
        ];
        let mut card = String::from("# Local App Hub validation fixture\ntheme light\ncomponent FixtureSurface(instance: text) { view Kit(component: \"surface\", instance: instance) { slot } }\ncomponent FixturePanel(instance: text) { view Kit(component: \"panel\", instance: instance) }\n");
        for role in roles {
            card.push_str(&format!("component Fixture{role}(text: text) {{ view Kit(component: \"{role}\", instance: \"{role}\", text: text) }}\n"));
        }
        card.push_str(
            "view root FixtureSurface(instance: \"page\") {\n FixturePanel(instance: \"panel\")\n",
        );
        for (role, text) in roles.into_iter().zip(texts) {
            card.push_str(&format!(
                " Fixture{role}(text: {})\n",
                serde_json::to_string(text).unwrap()
            ));
        }
        card.push_str("}\n");
        std::fs::write(bundle.join("page.card"), card).map_err(|e| e.to_string())?;
        let mut components = json!({
            "surface": {"style": {"t":"stack", "variant":"surface", "bg":0xfff4f7f5_u64}, "props":{}, "slot":true},
            "panel": {"style": {"t":"stack", "variant":"surface", "bg":accent, "radius":24}, "props":{}, "slot":false}
        });
        for (role, size, weight, color) in [
            ("tag", 12, 600, 0xff516c65_u64),
            ("title", 30, 700, 0xff182e35),
            ("subtitle", 16, 400, 0xff536762),
            ("value", 46, 700, 0xffffffff),
            ("detail", 15, 400, 0xffffffff),
            ("footer", 14, 400, 0xff536762),
            ("permission", 13, 500, 0xff536762),
        ] {
            components[role] = json!({"style":{"t":"text","size":size,"weight":weight,"color":color,"font_src":"makepad_widgets:resources/Inter.ttf","line_height":size as f64 * 1.5,"alignx":0},"props":{"text":"text"},"slot":false});
        }
        write_json(
            &bundle.join("kit/native/light/kit.json"),
            &json!({"schema_version":1,"theme":"light","tokens":{},"components":components}),
        )?;
        let mut placements = json!({});
        for (instance, component, x, y, w, h) in [
            ("page", "surface", 0, 0, 360, 640),
            ("panel", "panel", 24, 200, 312, 210),
            ("tag", "tag", 24, 34, 312, 24),
            ("title", "title", 24, 75, 312, 52),
            ("subtitle", "subtitle", 24, 128, 312, 54),
            ("value", "value", 44, 236, 272, 76),
            ("detail", "detail", 44, 324, 272, 48),
            ("footer", "footer", 24, 453, 312, 68),
            ("permission", "permission", 24, 550, 312, 44),
        ] {
            placements[instance] =
                json!({"component":component,"layout":{"x":x,"y":y,"w":w,"h":h}});
        }
        write_json(
            &bundle.join("page.data.json"),
            &json!({"$kit":{"theme":"light","placements":placements}}),
        )?;
        let rgb = accent & 0xffffff;
        std::fs::write(bundle.join("icon.svg"), format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="96" height="96" viewBox="0 0 96 96"><rect width="96" height="96" rx="24" fill="#{rgb:06x}"/><path d="M24 64 42 32 53 51 62 39 76 64Z" fill="white"/></svg>"##)).map_err(|e| e.to_string())?;
        std::fs::write(bundle.join("preview.png"), preview_png(accent)?)
            .map_err(|e| e.to_string())?;
        let description = if id == "star-chart" {
            format!("{name} is a small example for trying App Hub on this computer. It is made for a newer OctoSense than this one, so App Hub shows it but can’t offer it. Its screen is a still picture rather than a working app, and the picture here shows that screen.")
        } else {
            format!("{name} is a small example for trying App Hub on this computer: get it, then open it from here. Its screen is a still picture rather than a working app, and the picture here shows that screen.")
        };
        let listing_json = json!({
            "schema":1, "subtitle":subtitle,
            "description":description,
            "category":category, "keywords":["fixture","local","validation"], "icon":"icon.svg", "screenshots":["preview.png"],
            "platforms":["android","macos"],
            "publisher":{"name":"Sample Publisher","support":"https://example.invalid/local-fixtures","privacy_policy_url":"https://example.invalid/local-fixtures/privacy"},
            "release_notes":notes, "age_rating":"all", "license":"Apache-2.0"
        });
        let listing = Listing::parse(&listing_json.to_string())?;
        write_json(&bundle.join("listing.json"), &listing_json)?;
        let mut manifest = json!({
            "schema":1,"id":id,"version":version,"name":name,
            "integrity":{"bundle_blake3":octosense_app_policy::digest_dir(&bundle)?},
            "capabilities":capabilities,
        });
        if id == "star-chart" {
            // An API no OctoSense has: a newer one might.
            manifest["host_api"] = json!({"required":{"fixture.future":1}});
            manifest["requires"] = json!(["host-api-v1"]);
        }
        let mut manifest = AppManifest::parse(&manifest.to_string())?;
        octosense_app_hub::sign_manifest(&publisher, &mut manifest, "local-fixture-publisher")?;
        write_json(
            &bundle.join("manifest.json"),
            &serde_json::to_value(&manifest).map_err(|e| e.to_string())?,
        )?;
        let keys = PublisherKeys::new().with("local-fixture-publisher", &publisher.public_hex());
        let report = octosense_app_hub::check_bundle(&bundle, &HostLimits::default(), &keys, None)?;
        if !report.passed() {
            return Err(report.render());
        }
        write_json(
            &hub.join(format!("{artifact}.pack.json")),
            &serde_json::to_value(octosense_app_hub::pack_dir(&bundle)?)
                .map_err(|e| e.to_string())?,
        )?;
        entries.push(Entry {
            manifest,
            listing: Some(listing),
            tools: Vec::new(),
            artifact,
            publisher: "local-fixture-publisher".into(),
            publisher_key: publisher.public_hex(),
            source: Source {
                repository: "https://example.invalid/local-fixtures".into(),
                commit: "local-development-fixture".into(),
            },
            status: Status::Offered,
            admitted: octosense_app_hub::today(),
        });
    }
    let next = entries.pop().ok_or("The fixture has no later release")?;
    let later: Vec<Entry> = std::iter::once(next.clone())
        .chain(entries.iter().filter(|e| e.manifest.id != next.manifest.id).cloned())
        .collect();
    for (sequence, entries, file) in [(1, entries, "catalog.json"), (2, later, "catalog-next.json")] {
        let mut catalog = Catalog::new(sequence, published, entries);
        working.sign_catalog(&mut catalog, &anchor.certify(&working.public_hex())?)?;
        write_json(
            &hub.join(file),
            &serde_json::to_value(&catalog).map_err(|e| e.to_string())?,
        )?;
    }
    let environment = json!({"OCTOSENSE_HUB":hub,"OCTOSENSE_HUB_ANCHOR":anchor.public_hex(),"OCTOSENSE_APP_DATA":apps});
    write_json(&output.join("environment.json"), &environment)?;
    Ok(environment)
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    std::fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{}: {e}", path.display()))
}

// Diagrammatic fixture image for testing the details image rail. It is explicitly
// described as a layout diagram in the listing, not a device capture.
fn preview_png(accent: u64) -> Result<Vec<u8>, String> {
    use makepad_widgets::makepad_zune_png::{
        makepad_zune_core::{bit_depth::BitDepth, colorspace::ColorSpace, options::EncoderOptions},
        PngEncoder,
    };
    let mut pixels = vec![0u8; 360 * 640 * 4];
    for (x, y, width, height, color) in [
        (0, 0, 360, 640, 0xfff4f7f5_u64),
        (24, 34, 112, 12, 0xff516c65),
        (24, 75, 232, 32, 0xff182e35),
        (24, 132, 276, 12, 0xffa5b6b0),
        (24, 200, 312, 210, accent),
        (44, 246, 170, 40, 0xffffffff),
        (44, 324, 232, 12, 0xffdfece5),
        (24, 462, 268, 10, 0xffa5b6b0),
        (24, 486, 236, 10, 0xffa5b6b0),
        (24, 555, 250, 10, 0xffa5b6b0),
    ] {
        for row in y..y + height {
            for col in x..x + width {
                let offset = (row * 360 + col) * 4;
                pixels[offset..offset + 4].copy_from_slice(&[
                    (color >> 16) as u8,
                    (color >> 8) as u8,
                    color as u8,
                    255,
                ]);
            }
        }
    }
    let options = EncoderOptions::new(360, 640, ColorSpace::RGBA, BitDepth::Eight);
    let mut png = Vec::new();
    PngEncoder::new(&pixels, options)
        .encode(&mut png)
        .map_err(|e| format!("Could not encode fixture image: {e:?}"))?;
    Ok(png)
}

#[cfg(test)]
mod tests {
    use super::*;
    use octosense_app_hub_app::catalog::{Backend, EntryStatus};
    use octosense_appstore::source::Origin;
    use std::path::PathBuf;

    #[test]
    fn generated_catalog_verifies_and_every_app_installs_and_opens() {
        let scratch = tempfile::tempdir().unwrap();
        let environment = generate(&scratch.path().join("fixture"), false).unwrap();
        let root = PathBuf::from(environment["OCTOSENSE_APP_DATA"].as_str().unwrap());
        let hub = PathBuf::from(environment["OCTOSENSE_HUB"].as_str().unwrap());
        let mut backend = Backend::new(
            root,
            Origin::Directory(hub.clone()),
            environment["OCTOSENSE_HUB_ANCHOR"].as_str().unwrap().into(),
        );
        let snapshot = backend.refresh();
        assert!(snapshot.verified);
        // Published at a UTC time, which a device shows on its own calendar.
        assert!(snapshot.published.as_deref().is_some_and(|p| p.len() == 20 && p.ends_with('Z')), "{:?}", snapshot.published);
        assert_eq!(snapshot.entries.len(), 3);
        for entry in snapshot.entries {
            assert_eq!(entry.status, EntryStatus::Available);
            let installed = backend.install(&entry.consent.unwrap()).unwrap();
            assert!(backend.may_open(&entry.id).is_ok());
            assert!(installed.library.iter().any(|item| item.id == entry.id));
            let bundle = hub.join(format!("artifacts/{}-1.0.0.bundle", entry.id));
            assert!(bundle.join("page.card").is_file());
            assert!(bundle.join("page.data.json").is_file());
            assert!(bundle.join("kit/native/light/kit.json").is_file());
            assert!(hub
                .join(format!("artifacts/{}-1.0.0.bundle.pack.json", entry.id))
                .is_file());
        }
    }

    #[test]
    fn the_next_catalog_offers_an_update_to_what_is_installed() {
        let scratch = tempfile::tempdir().unwrap();
        let environment = generate(&scratch.path().join("fixture"), false).unwrap();
        let root = PathBuf::from(environment["OCTOSENSE_APP_DATA"].as_str().unwrap());
        let hub = PathBuf::from(environment["OCTOSENSE_HUB"].as_str().unwrap());
        let mut backend = Backend::new(
            root,
            Origin::Directory(hub.clone()),
            environment["OCTOSENSE_HUB_ANCHOR"].as_str().unwrap().into(),
        );
        let trail = backend.refresh().entries.into_iter().find(|e| e.id == "trail-notes").unwrap();
        backend.install(&trail.consent.unwrap()).unwrap();
        std::fs::copy(hub.join("catalog-next.json"), hub.join("catalog.json")).unwrap();
        let snapshot = backend.refresh();
        assert!(snapshot.verified);
        let offered = snapshot.entries.iter().find(|e| e.id == "trail-notes").unwrap();
        assert_eq!(offered.status, EntryStatus::UpdateAvailable);
        assert_eq!(offered.version, "1.1.0");
        assert!(snapshot.entries.iter().any(|e| e.id == "focus-timer"));
    }

    #[test]
    fn generator_refuses_to_replace_an_existing_catalog() {
        let scratch = tempfile::tempdir().unwrap();
        let output = scratch.path().join("fixture");
        let first = generate(&output, false).unwrap();
        let original = std::fs::read(output.join("hub/catalog.json")).unwrap();
        assert!(generate(&output, false).unwrap_err().contains("empty"));
        assert_eq!(
            std::fs::read(output.join("hub/catalog.json")).unwrap(),
            original
        );
        assert!(first["OCTOSENSE_HUB_ANCHOR"].as_str().unwrap().len() == 64);
    }

    #[test]
    fn both_catalogs_can_be_published_at_a_chosen_time() {
        let scratch = tempfile::tempdir().unwrap();
        let output = scratch.path().join("fixture");
        generate_at(&output, false, "2026-11-06T00:01:00Z").unwrap();
        for file in ["hub/catalog.json", "hub/catalog-next.json"] {
            let catalog: Value = serde_json::from_slice(&std::fs::read(output.join(file)).unwrap()).unwrap();
            assert_eq!(catalog["published"], "2026-11-06T00:01:00Z", "{file}");
        }
    }

    #[test]
    fn an_app_for_a_newer_octosense_is_listed_and_not_offered() {
        let scratch = tempfile::tempdir().unwrap();
        let environment = generate(&scratch.path().join("fixture"), true).unwrap();
        let root = PathBuf::from(environment["OCTOSENSE_APP_DATA"].as_str().unwrap());
        let hub = PathBuf::from(environment["OCTOSENSE_HUB"].as_str().unwrap());
        let mut backend = Backend::new(
            root,
            Origin::Directory(hub),
            environment["OCTOSENSE_HUB_ANCHOR"].as_str().unwrap().into(),
        );
        let snapshot = backend.refresh();
        assert!(snapshot.verified);
        assert_eq!(snapshot.entries.len(), 4);
        let star = snapshot.entries.iter().find(|e| e.id == "star-chart").unwrap();
        assert!(
            matches!(&star.status, EntryStatus::Unavailable(reason) if reason.contains("fixture.future@1")),
            "{:?}",
            star.status
        );
        assert!(star.consent.is_none());
    }
}
