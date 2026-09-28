---
name: card-studio
description: Inspect a generated L0 card before publishing it to the glance screen — render at glance, phone and desktop sizes, measured checks, vision critique payload. Triggers: card render, check my card, critique card, glance card, before glance.publish, card-studio.
version: 0.1.0
author: OctoSense
always: false
---

# card-studio

ADR 0002 section 7: an app agent's card is rendered and critiqued before it
is published. This skill does the rendering and the measuring; the vision
critique is a request it prepares for you to send.

## Loop

1. Write the card (L0) and its data (the host-resolved sources, e.g. the
   `sys.digest` payload). Try two styles when the run's budget allows.
2. `card_render {card, data}` — renders each size in a hidden `card-host
   --remote` and returns the report. Read `summary.pass` and every finding
   with `severity: "error"`: `text_truncated` / `text_hidden` /
   `text_clipped` / `overflow` (text is cut: shorten it, drop a row, or let
   it wrap), `does_not_fit` (the glance tile is 350x160pt: show fewer items),
   `source_failed`, `lint`, `realize_truncated`, `lower_failed`, `log_error`.
   Warnings (`state_visible`, `missing_value`, `overlap`, `source_pending`)
   are worth a revision when the budget allows.
3. Revise and render again until there is no error, or the budget is spent.
4. `card_critique_payload {report, rubric, inline: true}` — send its
   `prompt`, then for each entry of `sizes` a text line naming the size with
   its `widgets` and `findings`, then its image, to a vision model. Keep the
   answer with the run (the critique record). A `revise` verdict with
   concrete `fix`es is another revision, within budget.
5. Publish only a card whose last render passed (admission follows:
   level check, lint, approval pin).

## Install

The skill runs `main` (the `card-studio` binary) with cwd set to the
session workspace and a filtered environment, so its defaults live next to
it in `card-studio.json`:

```json
{"card_host": "/abs/path/card-host", "kit": "/abs/path/octoscript-makepad/components/l0", "out_dir": "/abs/path/card-studio-runs"}
```

```sh
cargo build --release -p octosense-card-host -p octosense-card-studio
dest=<octos data dir>/profiles/<profile>/data/skills/card-studio
mkdir -p "$dest" && cp skills/card-studio/{SKILL.md,manifest.json,rubric.md} "$dest/"
cp target/release/card-studio "$dest/main"   # card-host may stay where it was built
```

Rendering needs a graphical session (the window is hidden, not headless):
run it on the desktop or a server with a display, or on the phone only while
charging; the measured checks alone run anywhere (`card-studio check`).
