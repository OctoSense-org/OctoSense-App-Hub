# Developing this Hub app

> These instructions need no particular agent, model or vendor: every step is a shell command or a file edit. `AGENTS.md` is the source of truth; `CLAUDE.md` and `GEMINI.md` import it for agents that look for those names.

Before changing this app, read the shared
[first-app guide](https://github.com/OctoSense-org/OctoSense-App-Hub/blob/main/docs/FIRST-APP.md),
[publishing contract](https://github.com/OctoSense-org/OctoSense-App-Hub/blob/main/docs/PUBLISHING.md),
[submission guide](https://github.com/OctoSense-org/OctoSense-App-Hub/blob/main/docs/SUBMITTING.md),
[icon guidelines](https://github.com/OctoSense-org/OctoSense-App-Hub/blob/main/docs/ICONS.md)
and [development guide map](https://github.com/OctoSense-org/OctoSense-App-Hub/blob/main/docs/DEVELOPMENT.md).
If they are unavailable online, use the same files in a local Hub checkout and
record its revision. Do not invent missing requirements.

- This repository owns the app; `bundle/` is the release artifact directory.
  Keep authoring instructions, source tooling, keys and test evidence outside
  it.
- Use the image-to-card flow in OctoScript-App-Design-Flow
  (`flows/image-to-card/FLOW.md`) to generate the card UI, and the L0
  reference in OctoSense (`apps/appcard/a2app-l0/framework/l0.md`) for
  bindings. Do not hand-write L0 unless the app owner asks for that approach.
- Maintain the app's own icon at the path that `icon` in `listing.json`
  declares. Do not keep separate launcher or store copies of the artwork.
- Ask only for the capabilities that implemented features need. The starter
  requests no capabilities, hosts, storage or agent.
- Replace the placeholder identity, publisher URLs, artwork and platform
  claims. Use real native captures for the listing screenshots.
- Verify native behavior and appearance separately from bundle admission.
  Report exactly which platforms and flows you tested.
- Run `hub stamp` after every bundle edit, then `hub check`. Keep the review
  packet that `hub scan` writes outside the bundle.
- Run and capture the unsigned bundle: a host without a signature verifier,
  `card-host` included, refuses a signed one. Sign only the final bytes; any
  later edit needs a new stamp and a new signature. Keep publisher keys
  private and outside the repository.
- Keep `bundle/` byte-exact in Git: keep the starter's `.gitattributes` with the line
  `bundle/** -text`. A checkout that
  converts line endings (`core.autocrlf=true`, common on Windows) changes the
  digest, and `hub check` refuses the bundle.

Add app-specific requirements, build commands and tests here as the app grows.
For an existing repository, merge this guidance with its existing instructions.
