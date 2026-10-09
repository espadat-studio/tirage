# Taste triage

How a port session picks a Tool's Taste bounds before the Tool is ported. The oracle is the live site, not our render.

## Rig

`mise run taste <slug>` renders from `https://www.playgrnd.tools/<slug>/` into `target/taste/<slug>/`:

- A sweep of every art slider: 5 tiles from its min to its max, the other sliders left as the page sets them. It runs at tool seed 9611518, the one `tirage derive --seed 1` deals for every Tool, so the sweep sits where the port's first Reference export will.
- 24 draws: a random tool seed and a random value for every art slider across its full site range.
- `index.html`, a contact sheet of both. Click a tile to mark it bad. It prints the bounds the unmarked sweep tiles span, ready to save as a `--bounds` file.

`mise run taste <slug> --bounds <file>` renders 24 draws inside candidate bounds, `{"tool": "<slug>", "<id>": [min, max], …}`, for a second round. A slider left out keeps its full range. A run keeps the other round's tiles on the sheet.

Tiles are 270x480 PNGs. `data.js` lists each tile's file, tool seed and slider values; read it and the tiles instead of the page. Art sliders are the range inputs outside the motion, dither, grain and export panels. Pickers and toggles stay at site defaults.

It drives the live site and refuses to run in CI, like `refs`.

## The bar

A draw is bad when the frame is one of:

- a single flat colour
- fewer than two inks
- no readable structure, noise or mush

## Procedure

1. `mise run taste <slug>`. Judge every sweep tile and every draw against the bar.
2. Cut each slider's bounds to the stretch whose tiles pass. Every other slider keeps its full site range. Grain, dither and motion stay at site defaults and get no bounds.
3. `mise run taste <slug> --bounds <file>`. Count the bad draws of 24. More than 2: cut again and rerun.
4. Record the bounds as each `Param`'s `taste` ticks in `src/<slug>.rs`.

Bounds are data. A consumer may tighten them later ([ADR 0003](../adr/0003-seed-derivation-of-recipes.md)).
