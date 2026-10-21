# App icons and bundled artwork

English | [简体中文](ICONS.zh-CN.md)

Every app published through App Hub owns one icon, ships it in its bundle and
declares it in `listing.json`. Start with
[Build your first Hub app](FIRST-APP.md); the
full listing and admission contract is in [Publishing](PUBLISHING.md).

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

Set `"icon": "assets/icon.svg"` in `listing.json`. For a PNG,
use `assets/icon.png` and update the field. The path is relative to the bundle
root, not the repository root. These file names are conventions; any valid
bundle-relative path works. Listing schema 1 has no `dark_icon` or
`adaptive_icon` field, and the listing parser refuses fields it does not know.

The launcher and the store both read this one declaration. Do not keep
separately edited copies for the launcher, Recents, store rows or the detail
page. When a build needs another format, export it from the same source.

<a id="technical-requirements-and-current-enforcement"></a>

## Technical requirements and who checks them

`hub check` runs the gate, including the listing parser, before publication.
The installed-icon loader is the code every OctoSense shell links to draw an
installed app's icon; when it rejects the artwork, the shell draws its generic
icon instead.

| Requirement | Checked by |
| --- | --- |
| Declare an icon and include the file it names | `hub check` |
| Use a bundle-relative PNG or SVG path: no leading `/`, no `..`, no URL | The listing parser |
| Ship no symlinks; keep the whole bundle within 8 MiB | `hub check` |
| Keep the icon file within 1 MiB and square | `hub check`, and the installed-icon loader |
| Keep a PNG within 1024×1024 pixels, with valid PNG bytes | `hub check` decodes it; the loader checks its header and size |
| Use an SVG that parses, with square numeric `width` and `height` or a square `viewBox`; use no `<script>`, `<foreignObject>` or `on…` event attribute; load nothing from outside the file in CSS (no external `url()`, no `@import`); use no CSS escapes or comments, which can hide a URL | `hub check`; the loader checks that it parses and is square |
| Give the icon a clear silhouette, suitable padding and readable contrast | Your own visual review |

`hub check` checks the file rules in the table, **not how the icon looks**.
Test the icon in the target shell.

Sources: the [listing parser](../crates/app-policy/src/listing.rs), the
[gate](../crates/app-hub/src/gate.rs), the
[admission checks](../crates/app-hub/src/admission.rs) and the
[installed-icon loader](../crates/app-hub-app/src/icons.rs).

<!-- Maintainers: update the table above when these sources change. -->

## Design and export recommendations

New OctoSense app artwork follows the [approved illustrated icon family, version 1](https://github.com/OctoSense-org/OctoSense/blob/a48518feda00d518949f9eecdbf661201e02e6a4/docs/design/icon-family/README.md).
The family defines the visual language; this document owns bundle packaging,
size limits and admission behavior. Existing app branding remains its own
identity. A gate pass does not certify visual conformity to the family.

- Design one dominant object representing the app's purpose, with at most one
  supporting cue, broad silhouettes, rounded object edges and two or three
  overlapping planes. Use warm ivory `#fffdf7`, ink `#233d40` for contrast and
  restrained mint, apricot or coral accents on an app-specific colored ground.
  Keep any gradient subtle and depth local to the object.
- Give every app a distinct identity. The template is a format example, not
  artwork to publish unchanged. Do not copy another app's icon or stamp the
  OctoSense mark onto unrelated apps.

- Prefer SVG for geometric marks. Use a square `viewBox`, such as `0 0 64 64`,
  explicit fill and stroke colors, and simple paths, rectangles and circles.
- Keep SVGs self-contained: outline any lettering, embed no scripts and
  reference no external fonts, stylesheets or images. The SVG namespace URI is
  markup, not an external dependency. Avoid filters, masks and other advanced
  features unless you have verified them in Makepad's native renderer.
- For raster artwork, export a square PNG of 512×512 or 1024×1024 pixels,
  within 1 MiB. Keep transparency when it is part of the design.
- Use one recognizable mark. Leave the app name to the launcher and store
  labels; do not draw it in the icon. Avoid small text, fine detail and
  photographic clutter.
- Use a 64-point square master with every foreground detail inside the central
  66%: approximately 10.88–53.12 on both axes, or a 17% inset on each edge.
  Compare optical size with neighboring family icons. The gate does not measure
  safe-area bounds or optical balance.
- Export a square, full-bleed background with no baked platform corners,
  circular crop or outer drop shadow. OctoSense's shared shell renderer applies
  the selected platform frame to the declared artwork. Other hosts and store
  views may show the canonical square; do not create separately framed copies
  or rely on the host to invent a branded ground. Keep intentional PNG
  transparency and the artwork's own colors.
- Keep authoring files and provenance outside the release bundle unless the
  app needs them at runtime.

Give each app its own identity. The shopping bag with the OctoSense logo
identifies the App Hub store itself; do not use the bag or the logo in another
app's icon.

## Review and release

1. Inspect the master at 64, 40 and 24 pixels on light and dark backgrounds,
   beside the approved family. Check the central 66% foreground and the icon's
   distinct silhouette; retain the editable source and provenance outside the
   release bundle unless needed at runtime.
2. Preview Android, macOS and iOS frames, then check a real target shell's
   launcher and the Hub's list and detail views. Confirm there is no clipping,
   unwanted tint, double frame or generic fallback. A static platform preview
   is not device acceptance; report untested platforms as unverified.
3. When you change an installed icon, test an update from the previous
   version.
4. After the final artwork or screenshot change, run `hub stamp`, run
   `hub check` again, then prepare the GitHub-attested release as
   [Produce the final bytes](SUBMITTING.md#5-produce-the-final-bytes)
   describes. Give every release a new version: the gate refuses a version
   that is already published.

## Built-in native apps

Apply the same authoring format and ownership convention to native apps compiled
into a shell: declare one canonical asset and embed it through the build's
resource mapping.
OctoSense's built-in family is a local, explicit app-ID overlay on Makepad's
catalog. New store apps need only their own bundle icon and listing declaration;
they do not enter that overlay or modify Makepad resources. Unowned upstream
identities and installed third-party artwork continue through their normal paths.
App Hub's own native store does this with a two-field `listing.json`
(`schema` and `icon`, in `crates/app-hub-app/`). That file is an **icon-only
native build declaration**, not a complete publishable listing. Native apps
ship with the shell; they are not installed as Hub bundles. If a native app
has theme variants, resolve them in one shared function so the launcher and
the store agree.
