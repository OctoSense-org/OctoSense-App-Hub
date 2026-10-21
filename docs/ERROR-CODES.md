# Gate error codes

Every refusal and warning the gate prints carries a stable machine code and a
one-line fix, so a tool — an agent repairing its own bundle, a CI job — can
act on a failure without parsing English. The human message is unchanged; the
code follows it in the text report and in `--json`:

```
dev.example.app 1.0.0 — REFUSED
  [refused] digest: the bundle hashes to 4d6f…, the manifest claims 9c1e…
    fix [hub-digest-mismatch]: run `hub stamp <bundle>` after every edit (tools/octo check stamps first), then check again
```

```json
{"severity": "refusal", "check": "digest", "detail": "the bundle hashes to …",
 "code": "hub-digest-mismatch", "hint": "run `hub stamp <bundle>` after every edit …"}
```

A code, once printed, never changes what it means; new codes may be added.
Match on `code`, never on `detail`. The `check` field stays as the coarser
grouping it always was.

A few warnings are notes for a reviewer rather than a fault to repair — what
a component reaches, what it is built from, a tool that always waits for a
person. Those print as they always did, with no code.

## Structural

| Code | The gate means | Fix |
| --- | --- | --- |
| `hub-bundle-invalid` | The bundle cannot be read at all (`--json` only) | `manifest.json` must be present and valid JSON the manifest schema accepts (schema, id, version, name) |
| `hub-entry-invalid` | No entry the runtime can start | Add `main.splash` (a script app) or `page.card` with its kit (a card) |
| `hub-contents-invalid` | A file is not what its extension says | Repair or re-export it: UTF-8 text within the size limit, a decodable image, a sized scriptless SVG, an admissible wasm |
| `hub-resource-missing` | A card's image or font, or an SVG `href`, is not in the bundle | Ship the referenced file at that bundle-relative path, or drop the reference |
| `hub-font-invalid` | A `font_src` is not a font path, a built-in font, or one token | Set it to a bundled font path, a supported built-in font, or one token resolving to a string |
| `hub-file-type` | An extension a bundle may not hold | Remove the file, or convert it to `.card .json .l0 .octoscript .splash .svg .png .jpg .jpeg .webp .ttf .otf .txt .md` (or `fns/*.wasm`) |
| `hub-too-big` | Over the 8 MB ceiling | Compress the artwork or split the app; a bundle is cards, data and artwork only |

## Integrity and signing

| Code | The gate means | Fix |
| --- | --- | --- |
| `hub-digest-mismatch` | The manifest's digest is stale | `hub stamp <bundle>` after every edit (`tools/octo check` stamps first), then check again |
| `hub-signature-invalid` | The signature or GitHub attestation no longer matches the manifest | Restamp (`hub stamp`), then sign again (`hub sign-manifest`) or re-attach the attestation (`hub publisher-attach`) |
| `hub-signature-required` | This hub admits signed manifests only | Sign (`hub keygen` once, then `hub sign-manifest`) or attach GitHub provenance, or check with `--allow-unsigned` while developing |
| `hub-unsigned` | Warning: unsigned, accountability rests on the hub | Sign before submitting |

## Identity and history

| Code | The gate means | Fix |
| --- | --- | --- |
| `hub-id-system` | The id is under the reserved `os.` | Pick your own reverse-domain id, e.g. `dev.<you>.<app>` |
| `hub-id-not-system` | `--system-app` with an id that is not `os.*` | Drop `--system-app` for a store app; it checks a shipped `os.*` bundle during development, nothing else |
| `hub-id-reserved` | The id (or its last segment) is a native app's or a host's name | Rename it |
| `hub-version-published` | That version is already in the catalog | Bump `version`; a published version is immutable |
| `hub-publisher-continuity` | Not signed by the publisher on record | Sign (or attest) with the identity the catalog records for this publisher |

## Reaching outside the bundle

| Code | The gate means | Fix |
| --- | --- | --- |
| `hub-asset-external` | A data file or script names `http://`, `file://` or `../` | Ship the asset inside the bundle and reference it bundle-relatively (`{{assets}}` in a script) |
| `hub-secret-field` | A password or one-time-code field | Remove it; the host's sheet collects secrets for the service that needs one |

`hub-host-undeclared` is retired and stays reserved: HTTPS destinations are
declarations for reviewers, not refusals, so the gate no longer prints it.

## Grants

| Code | The gate means | Fix |
| --- | --- | --- |
| `hub-grants-invalid` | The manifest asks for something the contract does not offer | Check capabilities, hosts, agent and research against [PUBLISHING.md](PUBLISHING.md) |
| `hub-storage-undeclared` | Warning: `fs.*` (or a camera capture) without the `storage` capability | Declare `storage` in the manifest, or handle the fs errors and keep nothing |

## The listing

| Code | The gate means | Fix |
| --- | --- | --- |
| `hub-listing-missing` | No `listing.json` | Add one: description, category, screenshots, publisher, platforms ([PUBLISHING.md](PUBLISHING.md#the-listing)) |
| `hub-listing-invalid` | `listing.json` does not parse against the listing rules | Fix it against [PUBLISHING.md](PUBLISHING.md#the-listing) |
| `hub-listing-no-screenshots` | No screenshots named | Capture a PNG of the running app (`tools/octo shot`) and name it in `screenshots` |
| `hub-listing-no-icon` | No icon named | Add a square PNG or SVG and name it in `icon` |
| `hub-listing-asset-missing` | A named screenshot or icon is not in the bundle | Put the file at that path, or drop it from the listing |

## The app's agent and tools

| Code | The gate means | Fix |
| --- | --- | --- |
| `hub-tools-invalid` | `tools.json` breaks the tool-manifest rules | Fix it against [PUBLISHING.md](PUBLISHING.md#toolsjson-the-apps-tools): namespace, `risk`, `implemented_by`, schema |
| `hub-agent-files` | An agent file the manifest does not name, or agent files without an `agent` | Name every file in `agent.instructions` / `agent.skills`, and declare `agent` |
| `hub-skills-invalid` | A skill holds more than data | Only `.md`/`.json`/`.txt`, the manifest named as its directory, no executable fields |
| `hub-agent-review` | Any other agent-review finding | Fix what the line names against [PUBLISHING.md](PUBLISHING.md#the-apps-agent-and-tools) |

## The app's own functions (WebAssembly)

| Code | The gate means | Fix |
| --- | --- | --- |
| `hub-wasm-undeclared` | Warning: `fns/*.wasm` without the `wasm` capability | Add `wasm` to the capabilities, or remove the modules |
| `hub-too-many-functions` | Over the 8 modules a bundle may carry | Keep at most 8; split the app, or move shared logic into a shared component |
| `hub-wasm-missing` | Warning: `wasm` declared, no `fns/*.wasm` carried | Ship the modules the capability discloses, or drop `wasm` |
| `hub-component-requires` | A component, but the manifest does not require `wasm-components-v1` | Add it to `requires`, so hosts without components never install the app |
| `hub-component-host-api` | The component imports a host API the manifest does not require | Require it (`host_api.required`, with `requires host-api-v1`), or rebuild without that import |
| `hub-wasi-net-undeclared` | Warning: `wasi:http` or `wasi:sockets` without `net` | Disclose net usage in the capabilities |
| `hub-wasi-storage-undeclared` | Warning: `wasi:filesystem` without `storage` | Disclose storage usage in the capabilities (the jail and quota still apply) |
| `hub-component-crates` | Warning: the component does not list the crates it is built from | Rebuild with `tools/octo wasm build` (or App Flow's release workflow) |
| `hub-component-requires-unused` | Warning: requires `wasm-components-v1`, but `fns/` holds no component | Build the component into `fns/`, or drop the requirement |
| `hub-crate-advisory` | Warning (`--advisory-db`): a listed crate has a RustSec advisory | Upgrade to a fixed version and rebuild; re-run with `--advisory-db` to confirm |

## Shared components

| Code | The gate means | Fix |
| --- | --- | --- |
| `hub-components-unnamed` | Warning: requires `wasm-shared-components-v1`, but names no components | Name each in the manifest's `components`, or drop the requirement |
| `hub-component-bundled` | A store bundle ships `components/` | Remove it; a store app's components come from the catalog |
| `hub-component-not-manifest` | A bundled `components/*.wasm` the manifest does not name | Name the file's BLAKE3 digest in `components`, or remove the file |
| `hub-component-not-bundled` | A system app's component is not in the bundle | Ship it as `components/<blake3>.wasm` |
| `hub-component-unresolved` | The catalog has no matching offered component | Fix the id, version and pinned digest against the catalog |
| `hub-component-unverified` | Warning: no `--catalog`, so the component is not resolved | Re-run check with `--catalog` to confirm it is offered and matches its digest |

## Component releases (`hub component-check`)

| Code | The gate means | Fix |
| --- | --- | --- |
| `hub-component-release-invalid` | The release's fields break the component rules | Fix them against [PUBLISHING.md](PUBLISHING.md), or prepare the release again (`hub component-prepare`) |
| `hub-component-too-big` | Over the 8 MB component ceiling | Trim dependencies, or split the component |
| `hub-component-digest-mismatch` | The file's digest or size is not the release's | Prepare the release again from this exact `.wasm`; both are pinned in it |
| `hub-component-invalid` | Not an admissible WebAssembly component | Rebuild with only the imports a host allows (`hub component-info` lists them) |
| `hub-component-release-stale` | The release's imports or exports are not the file's | Prepare the release again from this file |
| `hub-component-provenance-invalid` | The GitHub attestation does not match the file | Re-attest the exact file from its GitHub workflow and attach the new attestation |
| `hub-component-provenance-required` | No GitHub attestation where one is required | Attest from the release's GitHub workflow; App Hub accepts only GitHub-attested component releases |
| `hub-component-id-taken` | The component's id is an app's id in the catalog | Pick a component id no app uses |
