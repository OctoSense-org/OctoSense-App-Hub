# App developer guide map

For a downloadable Hub app, begin with [Build your first Hub app](FIRST-APP.md).
The Hub owns publication requirements. The AppCard repository owns the UI
authoring tools and runtime-development guides.

| Task | Guide |
| --- | --- |
| Package, validate, sign and submit a Card bundle | [Publishing](PUBLISHING.md) |
| Set up an app-owned icon and bundled artwork | [Icons](ICONS.md) |
| Start a repository with metadata and agent instructions | [App starter](../templates/app/README.md) |
| Prepare shared Makepad/Octoscript dependencies | [Native workspace](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/docs/NATIVE-WORKSPACE.md) |
| Turn UI designs into native cards, bind service state and build flows | [Image-to-AppCard workflow](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/lab/image-to-appcard-flow/README.md) |
| Understand Card data, state, events, copy, themes and views | [L0 language](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/a2app-l0/framework/l0.md) |
| Inspect low-level Splash widgets and syntax | [Splash reference](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/a2app/framework/splash-manual.md) |
| Design source traceability, service actions and persistent journeys | [App Card design requirements](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/docs/app-card-design-requirements.md) |
| Test real native input, capture frames and clean up test instances | [Native instrument](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/lab/core/NATIVE-INSTRUMENT.md) |
| Add a type to the agent-generated app system | [Adding an App Card](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/docs/ADDING-AN-APP-CARD.md) |
| Explore application projects and ownership | [AppCard apps](https://github.com/OctoSense-org/Octoscript-AppCard/blob/main/apps/README.md) |

## Choose the appropriate delivery path

**Hub Card bundle:** `page.card`, its data and kit, local artwork, manifest and
listing. It runs within the host's existing capabilities. New native Rust/JNI
code, Python services and browser controllers are not installed by this format.

**Built-in native app:** source integrated into a shell release. Use the native
workspace and owning app's build instructions. Shared icon conventions still
apply, but an icon declaration alone does not make the app Hub-installable.

**Agent-generated app type:** specifications and lint rules teaching the agent
to compose a new kind of app. The adding-an-App-Card guide covers that system;
those specifications are not themselves a store bundle.

Some workflow examples include native services or website integration. Check
that every behavior of a proposed Hub app can run in the contained Card host;
do not assume copying a service project's source directory makes it installable.
The design-requirements document includes target product behavior, not a promise
that every capability is already implemented.

## Keep instructions in sync

Use the starter's `AGENTS.md` as a short entry point to these shared documents.
Merge it with an existing repository's instructions instead of overwriting them.
Keep app-specific behavior, data sources and tests in that app's repository.
Record the Hub and runtime revisions used for a release. When working offline,
an explicit versioned copy of the guides is preferable to an untracked copy
that silently becomes stale.

For new native tests, follow the direct-instrument guide. Older workflow stages
that use Studio remain separate; passing a compilation or packaging stage does
not imply visual acceptance, working input, or platform coverage.
