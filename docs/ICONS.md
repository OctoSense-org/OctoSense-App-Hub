# App icons and bundled artwork

This is the shared icon convention for apps published through App Hub. Start
with [your first app](FIRST-APP.md); use [publishing](PUBLISHING.md) for the full
listing and admission contract.

## Own one canonical icon

Keep the icon in the app's own repository and ship it in the bundle:

```text
bundle/
  manifest.json
  listing.json
  assets/
    icon.svg
  screenshots/
    01-main.png
```

Set `"icon": "assets/icon.svg"` in the **complete** `listing.json`. PNG is also
supported: use `assets/icon.png` and update the field. The path is relative to
the bundle root, not the repository root. These filenames are conventions;
another valid local path is accepted. Do not add unrecognized fields such as
`dark_icon` or `adaptive_icon` to schema 1.

The installed app's launcher and store artwork should resolve this same
declaration. Do not maintain independently edited copies for the launcher,
Recents, store rows or detail page. Export artwork from one source when a build
needs another format.

## Technical requirements and current enforcement

| Requirement | Where it is checked today |
| --- | --- |
| Declare an icon and include the referenced file | Hub admission gate |
| Use a bundle-relative PNG/SVG path, without an absolute path, `..` or URL | Listing validator |
| No symlinks; bundle contents within the 8 MiB limit | Hub admission gate |
| Icon at most 1 MiB; positive square dimensions | Hub admission and mobile installed-icon loader |
| Icon PNG no larger than 1024×1024; valid PNG bytes | Hub full decode and mobile loader |
| SVG parses into drawable geometry with a square logical canvas | Mobile SVG loader |
| Clear silhouette, suitable padding and readable contrast | Author's visual review |

`hub check` decodes bitmap artwork with dimension/output limits and checks SVG XML, dimensions and resource references. `hub test` additionally checks supported native SVG geometry. Neither proves visual quality. A gate pass does not prove that an icon will
render. The mobile bounds above describe its current installed-icon loader;
invalid artwork falls back to a generic icon. Test in the target shell.

Sources: [listing parser](../crates/app-policy/src/listing.rs),
[admission gate](../crates/app-hub/src/gate.rs), and the mobile repository's
`apps/app-hub/src/icons.rs` implementation. Keep this table aligned with those
implementations when requirements change.

## Design and export recommendations

- Prefer SVG for geometric marks. Use a square `viewBox`, such as `0 0 64 64`,
  explicit fill/stroke colors, and simple paths, rectangles and circles.
- Keep SVGs self-contained: outline any lettering, embed no scripts, and avoid
  external fonts, stylesheets or image references. The SVG namespace URI is
  markup, not an external artwork dependency. Avoid filters, masks and other
  advanced features unless verified in Makepad's native renderer.
- For raster artwork, export a square PNG, normally 512×512 or 1024×1024, within
  the 1 MiB limit. Preserve transparency when it is part of the design.
- Use one recognizable mark. Keep the app name in the launcher/store label.
  Avoid small text, fine details and photographic clutter.
- Leave balanced internal space; roughly 10–15% inset is a useful starting
  point, then compare optical size with adjacent launcher icons. This is a
  recommendation, not a measured admission rule.
- Include any branded background tile in the artwork. Do not assume every
  surface adds a background, mask, shadow or tint. Check both light and dark
  backgrounds; retain the artwork's own colors.

Each app uses its own identity. The OctoSense-logo shopping bag identifies the
App Hub store itself; other apps do not need that bag or the OctoSense logo.
Keep authoring files and provenance outside the release bundle unless needed at
runtime. A screenshot must show the actual app, not an icon concept or mockup.

## Review and release

1. Inspect the icon at 24, 32, 48 and 64 pixels on light and dark backgrounds.
2. Verify the native launcher and Hub list/detail use the intended artwork,
   without clipping, unwanted tint or a fallback icon.
3. Test an update from the previous version if changing an installed icon.
4. After the final artwork or screenshot change, run `hub stamp`, check again,
   then sign. Reusing a published version number is not supported.

## Built-in native apps

Native apps compiled into the shell can use this ownership convention too:
declare a canonical asset and embed it through their build resource mapping.
The mobile App Hub's two-field `listing.json` is an **icon-only native build
declaration**, not a complete publishable Card listing. Native binaries are
distributed with the shell, rather than installed as Hub Card bundles.
Existing host theme overrides may remain during migration; use a shared
resolver so all surfaces agree. Theme variants are not new schema-1 fields.
