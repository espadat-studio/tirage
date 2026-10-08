# Reference exports

Test fixtures only. tirage reproduces the look of playgrnd.tools generators and is not affiliated with it. Each PNG here was exported from one Tool's live page, `https://www.playgrnd.tools/<tool>/`, for a Recipe of our own.

## Provenance

`manifest.json` records, for each Tool:

- `page_sha256`: SHA-256 of the Tool page HTML the exports were made from
- `chromium`: the Chromium build that rendered them
- `font_sha256`: SHA-256 of `fonts/DejaVuSansMono-Bold.ttf`, the only font Chromium could see
- `fixtures`: each file's Recipe, and its `frame` when the Tool's motion was on

`<tool>/<name>.png` is the export of the fixture with that name.

## Rights

The site's only rights line is "Exports are yours.", on its [homepage](https://www.playgrnd.tools/). These files are outputs of our own Recipes, which is what that line covers ([ADR 0005](../../meta/adr/0005-public-repo-with-reference-exports.md)).

They are not covered by any licence tirage grants, now or later. They will be removed on request from the site's author.

## Regenerate

`mise run refs [--update] [tool...]`. It is hand-run and never runs in CI. It fails when a live page hash differs from the manifest, unless `--update` is passed.

A fixture without `frame` is exported with motion off. A fixture with `frame` is exported with motion on and the timeline pinned to that frame, so a Loop Tool's Still is `frame: 0`.

A Tool may set `"threshold": {"max": <share>, "reason": "<why>"}` to tighten the 1.6% fidelity ceiling. It can never loosen it, and the reason is required.
