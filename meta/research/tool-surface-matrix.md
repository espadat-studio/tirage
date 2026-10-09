# Surface needs and quirks of the 46 unported Tools

Resolves #91 (part of map #90). Researched 2026-10-09.

## Verdict

- 19 of the 46 Tools draw with nothing beyond today's `Surface` and `chassis`: benday, bloom, coral, culture, mist, mosh, motley, oddgrid (built-in motifs), pith, relief, sear, sprig, stipple, stitch, terrain, warp, weave, whorl, zig.
- The other 27 need 12 small enablers. Four of them are one-line options on existing calls: stroke caps and joins (10 Tools), alpha ink (9), nearest or bilinear upscale (5), blend modes (2). The rest: clip with a save/restore stack (6), transforms (4), partial arcs and ellipses (3), linear gradient (2), offscreen mask read-back (3), dash (1), blur (1), a 400-weight font (1).
- No Tool needs a radial gradient beyond frond's fade, `shadowBlur`, `createPattern`, `bezierCurveTo` or `strokeText`.
- 11 batches: 4 batches of "nothing new", 6 batches grouped by enabler, and atlas alone behind a font decision. mist and coral join batch 1, riso batch 8, splice batch 10.
- Four Tools break the Palette model: prism paints from a locked ramp and ignores its own `S.pal`; static has two loose colour fields, not a palette; oddgrid can draw an uploaded SVG motif; atlas draws text at weight 400 while the bundled face is DejaVu Sans Mono Bold.

## Method

- Source: the 46 Tool pages from `https://www.playgrnd.tools/<slug>/`, downloaded 2026-10-09, plus the 6 ported pages for cross-checks against the census. Each page holds 3 inline scripts; the first is the Tool script (28 to 51 KB).
- The renderer is the Tool script with the chassis block (from the `Grain (chassis)` comment to the end, 9,158 bytes on every page, 9,159 on oddgrid) stripped. Features are counted by regex over that renderer, one row per Canvas 2D API name, as the census did.
- Art KB is the renderer up to `const art=$("#art")`. For the 10 palette-family Tools the painter sits after that line (`filament: markPath`, `modular: paint`, `specimen: drawItem`, `terrain: paint`, `tokens: shapePath`), so counts always cover the whole renderer, not only the art prefix.
- Ambiguous hits were read by hand: every `.arc(` argument list, every `globalCompositeOperation` and `globalAlpha` assignment, every `drawImage` with the smoothing flags around it, every `getImageData`, every `clip`, every `.fill(` with arguments. `pith: DI.fill(1e9)`, `crowd: .fill(0)` and `oddgrid: .fill(255)` are typed-array fills, not path fills, and are dropped.
- Shell quirks come from the page: `#sizePx` vs `#size` in the HTML, `S.pal` vs `S.palette` and push/splice guards in the script, `GRAIN_DEFAULT`, `function deal*`, `function applySeed`, and the `motion` object in `S`.
- Code was read to learn; nothing is copied. Citations are `slug: identifier`, pointing at a function, constant or element id in that page.

## What counts as missing

Today's `Surface` (`src/surface.rs`): whole-frame fill, `fill_rect`, `fill_box`, nonzero and even-odd path fill, stroke with a width (round join, butt cap, no dash), `arc_to`, `quad_to`, full circles, one radial fade, `set_pixel`, `draw_smooth` (bicubic upscale of a small RGBA buffer to the frame), `edit_rgba`. `src/text.rs` fills one glyph from DejaVu Sans Mono Bold. So:

| Site call | Covered today | Missing |
|---|---|---|
| `arc(..., 0, TAU)` | yes, `push_circle` / `fill_circle` | partial arcs |
| `ellipse` | no | rotated oval, partial elliptical arc |
| `setLineDash` | no | dash |
| `lineCap`/`lineJoin` | butt cap, round join only | round cap, miter join |
| `clip`, `save`/`restore` | no | mask + state stack |
| `translate`/`rotate`/`scale` | no | transform on fill and stroke |
| `setTransform(1,0,0,1,0,0)` | identity is the default | nothing (all 25 sites are this reset) |
| `createLinearGradient` | no | linear shader |
| `createRadialGradient` | frond's fade only | nothing (no other Tool uses one) |
| `globalAlpha` | no | alpha ink; 11 of the 20 sites only reset to 1 |
| `globalCompositeOperation` | source-over | multiply (parcel), lighter (strand); 14 Tools only reset |
| `filter` blur | no | blur on a scratch buffer (strand) |
| `drawImage` | bicubic full-frame upscale | nearest (4), bilinear (1), SVG image (oddgrid custom) |
| `imageSmoothingEnabled` without `drawImage` | n/a | nothing: atlas, bloom, mosh, pane, relief, sampler, totem set it but never draw an image |
| `fillText` | one glyph, weight 700 | weight 400 (atlas) |
| `getImageData` on the frame | `edit_rgba` | read-back of an offscreen mask Surface (chaff, strand) |
| `putImageData`/`createImageData` | `edit_rgba`, `set_pixel`, `draw_smooth` | nothing |
| `clearRect` | `fill` | nothing (always followed by an opaque draw) |
| `Path2D`, `evenodd`, `quadraticCurveTo`, `arcTo` | yes | nothing |

## The matrix

Kind: pixel = writes a pixel field and draws no path; vector = paths and rects only; mixed = both. Seed: deal = `newSeed` rewrites Parameters; applySeed = the seed deals a few layout Parameters. My colors fit: how the site's one colour set lands on this Tool. Motion is for the record only.

| Tool | Category | Art KB | Kind | Missing on today's Surface | Seed | Size id | Palette model | My colors fit | Grain / budget | Motion default |
|---|---|---:|---|---|---|---|---|---|---|---|
| atlas | Textures | 10.5 | vector | `fillText` at weight 400 (bundled face is 700) | neither | `#size` | var `S.pal` 8 | by position |  | shuffle, 8 fps, 24 f |
| benday | Backgrounds | 8.0 | pixel | none | deal (`dealB`, also `S.pal`) | `#sizePx` | var `S.pal` 8 | by position |  | drift, 12 fps, 36 f |
| bloom | Backgrounds | 7.5 | vector | none | neither | `#size` | var `S.pal` 6 | by position |  | ripple, 8 fps, 24 f |
| carve | Backgrounds | 15.6 | mixed | `clip` (rect) x2, `save`/`restore` | neither | `#size` | var `S.pal` 6 | by position |  | drift, 12 fps, 36 f |
| chaff | Backgrounds | 10.7 | mixed | offscreen mask Surface + `getImageData` read-back (mask at W/2) | neither | `#sizePx` | var `S.pal` 2 | by position |  | drift, 12 fps, 36 f |
| cipher | Patterns | 9.9 | vector | `lineJoin` miter | deal (`dealG`) | `#sizePx` | var `S.pal` 7 | by position |  | drift, 12 fps, 36 f |
| coral | Backgrounds | 13.0 | pixel | none | deal (`dealC`) | `#sizePx` | var `S.pal` 5 | by position | grain on (multiply .45) | sway, 12 fps, 36 f |
| crowd | Backgrounds | 10.9 | mixed | `globalAlpha` 0.9, round cap/join | deal (`dealC`) | `#sizePx` | var `S.pal` 3 | by position | grain on (overlay .35) | sway, 12 fps, 36 f |
| culture | Backgrounds | 8.1 | pixel | none | neither | `#sizePx` | var `S.pal` 3 | by position |  | drift, 12 fps, 36 f |
| dahlia | Posters | 7.2 | vector | round cap/join | deal (`dealD`) | `#sizePx` | var `S.pal` 5 | by position | grain on (multiply .22) | pulse, 12 fps, 36 f |
| delta | Backgrounds | 8.2 | mixed | `drawImage` bilinear (field at 120..420 px), `globalAlpha` 0.85..1, `getImageData` x2 (field, frame strips) | neither | `#size` | var `S.pal` 6 | by position |  | shuffle, 12 fps, 24 f |
| fete | Posters | 12.1 | mixed | `drawImage` nearest (`res` buffer), round cap/join | neither | `#size` | family `S.palette` 4 (3..7) + line, dot | slots only |  | draw, 20 fps, 100 f |
| filament | Backgrounds | 9.4 | vector | `arc` partial x1, `setLineDash` x3, `globalAlpha` per mark, round cap/join | neither | `#size` | family `S.palette` 3 + accents 3 + bg | slots only |  | travel, 16 fps, 30 f |
| fold | Patterns | 9.1 | pixel | `drawImage` nearest (grid buffer) | deal (`dealF`) | `#sizePx` | var `S.pal` 6 | by position |  | drift, 12 fps, 36 f |
| hiss | Backgrounds | 11.1 | mixed | round cap | deal (`dealH`, also `S.pal`) | `#sizePx` | var `S.pal` 5 | by position |  | boil, 12 fps, 36 f |
| mist | Backgrounds | 5.6 | pixel | none (bicubic upscale = `draw_smooth`) | neither | `#size` | fixed 4: Ink 1..4 | by position | budget 700 on PNG | bleed, 12 fps, 72 f |
| modular | Posters | 6.9 | vector | `createLinearGradient`, `globalAlpha` 0.85 and rules | neither | `#size` | family `S.palette` 4 (3..8) + bg, rule | slots only |  | flow, 6 fps, 24 f |
| mosh | Textures | 11.7 | vector | none | neither | `#size` | var `S.pal` 8 | by position |  | shuffle, 12 fps, 24 f |
| motley | Patterns | 8.2 | vector | none (butt cap = default) | deal (`dealM`) | `#sizePx` | var `S.pal` 9 | by position |  | flicker, 12 fps, 36 f |
| oddgrid | Patterns | 14.6 | vector | none for built-in motifs; SVG `drawImage` for custom motif only | neither | `#size` | family `S.palette` 5 (3..10) + bg, ink | slots only |  | flow, 8 fps, 32 f |
| optic | Posters | 10.3 | vector | `clip` (path), `save`/`restore`, `translate`/`rotate` | applySeed (count, levels, sizeF, weight) | `#size` | fixed 2: Base, Ink | by position |  | slide, 12 fps, 72 f |
| pane | Backgrounds | 9.7 | vector | `createLinearGradient` | neither | `#size` | var `S.pal` 8 | by position |  | shuffle, 8 fps, 24 f |
| parcel | Posters | 8.2 | vector | `globalCompositeOperation` multiply | neither | `#size` | fixed 3: Base, Ink, Lines | by position |  | creep, 12 fps, 72 f |
| pith | Textures | 17.1 | pixel | none (scratch upscale is live preview only) | neither | `#sizePx` | var `S.pal` 5 | by position | live-only budget | boil, 12 fps, 36 f |
| prism | Backgrounds | 9.4 | vector | `globalAlpha` dots | applySeed (streams, reach, fringe) | `#size` | fixed, uneditable (`RAMP`, `BG`, `DOTC`) | no My colors |  | drift, 8 fps, 12 f |
| quilt | Patterns | 13.4 | pixel | `drawImage` nearest | neither | `#size` | fixed 4: Base, Weave A, Weave B, Pop | by position |  | scroll, 10 fps, 60 f |
| relief | Patterns | 10.8 | vector | none | neither | `#size` | var `S.pal` 6 | by position |  | shuffle, 6 fps, 24 f |
| rise | Backgrounds | 7.8 | vector | `clip` (circle), `save`/`restore` | applySeed (bands, count, depth, weight) | `#size` | fixed 3: Field, Bars, Accent | by position |  | ripple, 12 fps, 72 f |
| riso | Posters | 12.2 | vector | `clip` (band rect), `save`/`restore`, `rgba()` fills x2, round cap/join | applySeed (bands, rough, scrib) | `#size` | var `S.pal` 6 | by position |  | tear, 8 fps, 16 f |
| sampler | Patterns | 10.6 | vector | `save`/`restore`, `translate`/`scale`/`rotate` | neither | `#size` | var `S.pal` 7 | by position |  | shuffle, 6 fps, 24 f |
| sear | Backgrounds | 9.6 | pixel | none | neither | `#size` | var `S.pal` 6 | by position |  | drift, 12 fps, 36 f |
| specimen | Posters | 14.4 | vector | `ellipse` (rotated, full), `clip` (rect), `save`/`restore`, `globalAlpha` rules, round cap/join | neither | `#size` | family `S.palette` 7 (3..12) + page, ink | slots only |  | flow, 12 fps, 24 f |
| splice | Backgrounds | 10.9 | mixed | `ellipse` partial rotated (stroked), `globalAlpha` 0.85, round cap | neither | `#sizePx` | var `S.pal` 7 | by position |  | drift, 12 fps, 36 f |
| sprig | Patterns | 10.3 | vector | none (even-odd covered) | neither | `#sizePx` | var `S.pal` 2 | by position |  | drift, 12 fps, 36 f |
| static | Posters | 9.1 | pixel | `drawImage` nearest | neither | `#size` | loose `S.ink`, `S.bg` | ink then bg |  | flicker, 10 fps, 60 f |
| stipple | Textures | 8.0 | vector | none | neither | `#size` | family `S.palette` 3 (2..8) + bg, acc | slots only |  | drift, 14 fps, 30 f |
| stitch | Backgrounds | 7.2 | vector | none | deal (`dealS`) | `#sizePx` | var `S.pal` 6 | by position |  | drift, 12 fps, 36 f |
| strand | Backgrounds | 13.8 | mixed | `globalCompositeOperation` lighter, `filter` blur, offscreen mask read-back, round cap | neither | `#sizePx` | var `S.pal` 3 | by position |  | drift, 12 fps, 36 f |
| terrain | Backgrounds | 5.7 | pixel | none | neither | `#size` | family `S.palette` 6 (3..12) | slots only |  | drift, 12 fps, 30 f |
| tokens | Badges | 8.7 | vector | `globalAlpha` grid | neither | `#size` | family `S.palette` 9 (3..10) + bg, gridCol | slots only |  | cycle, 14 fps, 36 f |
| totem | Posters | 12.6 | vector | `save`/`restore`, `translate` + `scale(-1,1)` | neither | `#size` | var `S.pal` 5 | by position |  | shuffle, 8 fps, 24 f |
| vee | Patterns | 8.5 | vector | `clip` (rect, rotated rect) x2, `save`/`restore`, `translate`/`rotate` | neither | `#size` | fixed 4: Base, Ink 1, Ink 2, Cross | by position |  | travel, 12 fps, 72 f |
| warp | Patterns | 6.8 | vector | none | neither | `#size` | fixed 5: Base, Ink 1..4 | by position |  | bend, 12 fps, 72 f |
| weave | Patterns | 7.2 | vector | none | neither | `#size` | family `S.palette` 7 (4..10) | slots only |  | slide, 24 fps, 120 f |
| whorl | Patterns | 7.7 | pixel | none | neither | `#sizePx` | var `S.pal` 2 | by position |  | flow, 12 fps, 36 f |
| zig | Patterns | 10.1 | vector | none (`arcTo` covered) | neither | `#size` | family `S.palette` 6 (3..10) | slots only |  | crawl, 24 fps, 120 f |

Sources per row: `<slug>: paint` for the drawing calls; `<slug>: S` for defaults, size field and motion; `<slug>: buildSwatches` for the palette model; `<slug>: GRAIN_DEFAULT`; the named `deal*` or `applySeed` function. Specific sites: `carve: panelChev`, `panelGrid`, `chaff: paint` (mask `mx`, `MS=0.5`), `cipher: paint` (`lineCap="butt"`, `lineJoin="miter"`), `delta: paintField` (`res` 120..420), `fete: paint` (`tiny` at `S.res`), `filament: markPath` (the hook arc `a-0.4, a+2.4`), `fold: paint` (`OFFF`), `mist: paint` (its `budget` argument: 700 for PNG, 300 for video), `modular: paint` (gradient cell), `oddgrid: drawMark`, `optic: clipShape`, `pane: paint`, `parcel: paint`, `pith: paint` (`live`, `budgetP`), `prism: cellColor`, `quilt: paint`, `rise: paint`, `riso: paint` (`rgba(10,10,10,0.5)`, `rgba(255,255,255,0.4)`), `sampler: paint`, `specimen: drawItem` and `paint`, `splice: paint` (`ellipse(..., a.rot, a.a0, a.a1)`), `static: paint`, `strand: paint` (`filter="blur(...)"`, `lighter`), `tokens: paint`, `totem: paint` (`scale(-1,1)`), `vee: paint`.

## Per-feature Tool lists

| Feature | Tools (46) | With the 6 ported | Census said |
|---|---|---|---|
| `arc`, any | carve, cipher, dahlia, fete, filament, hiss, modular, motley, oddgrid, optic, rise, stipple, tokens (13) | + frond, kiosk, vein = 16 | 16 |
| `arc`, partial | filament (1 of 3 sites) | 1 | not split |
| `ellipse` | specimen (rotated full), splice (rotated partial) | 2 | 2 |
| `setLineDash` | filament | + frond = 2 | 2 |
| round cap or join, miter join | cipher, crowd, dahlia, fete, filament, hiss, riso, specimen, splice, strand (10) | | not counted |
| `clip` | carve, optic, rise, riso, specimen, vee (6) | 6 | 6 |
| `save`/`restore` | carve, optic, rise, riso, sampler, specimen, totem, vee (8) | 8 | 8 |
| `translate`/`rotate`/`scale` | optic, sampler, totem, vee (4) | 4 | 4 / 3 / 2 |
| `createLinearGradient` | modular, pane | 2 | 2 |
| `createRadialGradient` | none | + frond = 1 | 1 |
| `globalAlpha` not 1 | crowd, delta, filament, modular, prism, specimen, splice, tokens (8) | | 20 incl. resets |
| `rgba()` ink | riso | + frond = 2 | not counted |
| `globalCompositeOperation` not source-over | parcel (multiply), strand (lighter) | 2 | 2 |
| `filter` blur | strand | 1 | 1 |
| `drawImage` nearest | fete, fold, quilt, static (4) | | 9 total |
| `drawImage` bilinear | delta | | |
| `drawImage` bicubic ("high") | mist; pith (live preview only) | + aura = 3 | 3 |
| `drawImage` of SVG markup | oddgrid (custom motif only) | 1 | 1 |
| `fillText` | atlas | + kiosk = 2 | 2 |
| `getImageData` in the renderer | chaff, delta, strand (3) | + frond = 4 | 4 |
| `putImageData` | benday, carve, chaff, coral, crowd, culture, delta, fete, fold, hiss, mist, pith, quilt, sear, splice, static, strand, terrain, whorl (19) | + aura, frond, husk = 22 | 22 |
| `clearRect` | delta, fete, quilt, static (4) | 4 | 4 |

## Quirk lists

- Deal Tools (9): benday, cipher, coral, crowd, dahlia, fold, hiss, motley, stitch. Each `deal*` rewrites 6 to 11 Parameters; benday and hiss also pick `S.pal` from `PALETTES` (`benday: dealB`, `hiss: dealH`). totem's `dealRegion` deals one region's content inside the painter and is not a seed deal (`totem: newSeed` only redraws).
- applySeed Tools (4): optic (count, levels, sizeF, weight), rise (bands, count, depth, weight), riso (bands, rough, scrib), prism (streams, reach, fringe).
- Export size on `#sizePx` (16): benday, chaff, cipher, coral, crowd, culture, dahlia, fold, hiss, motley, pith, splice, sprig, stitch, strand, whorl. The other 30 use `#size`. Range 600..6000 step 100 default 2400 everywhere except oddgrid (500..6000, default 2000).
- Palette models: variable `S.pal` (27 Tools, 2 to 9 default slots, minimum 2, no maximum); fixed named roles (7: mist, optic, parcel, quilt, rise, vee, warp); palette-family `S.palette` (10: fete, filament, modular, oddgrid, specimen, stipple, terrain, tokens, weave, zig) with the bounds in the matrix and separate ground, rule, ink, page, accent or grid fields outside `#swatches`; static with two loose fields; prism fixed and uneditable.
- My colors (`fit` in the chrome script, `slots()`): writes the swatch inputs inside `#swatches` by position, padding or truncating to the Tool's slot count. Palette-family ground, rule and ink fields sit outside `#swatches`, so they keep their values. static has no swatches, so the fallback takes the page's loose colour inputs in DOM order: `#inkIn`, then `#bgIn`. prism has no `#mycTog`.
- Grain on by default (3): coral (multiply, amount .45, size 1, specks .4, vignette .3), crowd (overlay, .35, 1, .3, .2), dahlia (multiply, .22, 1, .5, .15).
- Reduced render budget: mist only. `mist: paint` takes a budget: 130 while playing, 260 for a still preview, 700 for PNG, 300 for video, the same as aura. pith's `budgetP` scales only the on-page canvas (`live` is true when `c.canvas.id==="art"`); exports render at full size. fete's low-res buffer is the `res` Parameter (default 64) and is identical in the export.
- Motion defaults in the matrix. Extremes among the 46: weave and zig 24 fps / 120 frames, prism 8 / 12, fete 20 / 100.

## Proposed batches

Cheapest first. A batch ships when its enablers exist; enablers are cumulative.

| # | Tools | Enabler added | Why here |
|---|---|---|---|
| 1 | mist, coral, whorl, sear, culture | none | pixel fields; mist reuses aura's buffer and budget; coral is the first deal Tool |
| 2 | benday, terrain, stitch, pith, mosh | none | pixel fields and rect grids; benday deals its palette; terrain is the first palette-family Tool; pith is the largest art code (17 KB) |
| 3 | bloom, weave, warp, zig, relief | none | vector rects and paths; zig uses `arc_to` |
| 4 | sprig, stipple, motley, oddgrid | none | vector with full circles and even-odd; oddgrid without custom motifs |
| 5 | fete, fold, quilt, static | nearest upscale (a `FilterQuality` option on `draw_smooth`); round cap/join for fete | small buffers drawn pixel-sharp; static brings the loose two-field palette |
| 6 | dahlia, hiss, crowd, cipher, tokens | round cap, miter join; alpha ink | strokes with caps and one alpha layer each |
| 7 | pane, modular, prism, parcel | linear gradient; blend multiply | one shader and one blend mode; prism brings the fixed palette |
| 8 | riso, rise, carve, specimen | clip + save/restore stack; rotated oval for specimen | all four clip to a rect or circle |
| 9 | optic, vee, sampler, totem | transforms (rotate, mirror) | clip plus rotated fills |
| 10 | splice, filament, chaff, delta, strand | partial arc and elliptical arc, dash, offscreen mask read-back, bilinear upscale, blend lighter, blur | the long tail: each Tool brings one feature nobody else needs |
| 11 | atlas | a 400-weight monospace face | gated by the font decision below |

Earliest join: mist and coral in batch 1, riso in batch 8, splice in batch 10. splice can move to batch 6 if the elliptical-arc enabler (about 30 lines, or `kurbo::Arc`) is pulled forward with it.

## The four odd Tools

### prism

- Colours are constants: field `BG` `#08080A`; dots `DOTC` `#FFFFFF` at alpha `S.dots * 0.8`; `RAMP` has 5 rings: `#FFFFFF`; `#FFE800 #FF9E00 #FFC400`; `#FF2D55 #FF00A8 #E600D8 #FF3B00`; `#2E7CFF #00C8FF #6A2BD9 #0038FF`; `#3A1ED8 #5A0FB0 #2B0FA8` (`prism: RAMP`).
- `S.pal` holds 6 hexes (`#0B0B0F #00F5D4 #F15BB5 #FEE440 #00BBF9 #9B5DE5`) but only feeds the UI accent and strip (`prism: accent`). `buildSwatches` returns early because the page has no `#swatches`, and the page has no `#mycTog`. No colour is editable.
- How a seed reaches the colours: `buildSpec` seeds a stream with the Tool seed and draws `SW`; `cellColor` picks the ring from the cell's distance to the nearest stream spine and the ring's hex by hashes keyed on `SW`, then darkens by up to 6% with another hash. So the seed decides which ramp entry a cell gets, never which hexes exist. `applySeed` deals streams from {2,3,3,3,4}, reach from {.42,.5,.55,.6,.7}, fringe from {.75,.9,1,1,1.15} using the seed xor `0x9e3779b9`.
- Consequence: a Palette Pin cannot replay on the site. The port should either refuse a Palette for prism or document that it ignores it.

### static

- Two loose fields: `S.ink` default `#FF3CAC`, `S.bg` default `#1B1B2F` (`static: S`). The HTML inputs carry `#5C1226` and `#8286EC` and are not synced to `S` at start-up; the swatch fills are (`static: $("#inkFill")`). The art always reads `S`.
- On a normal visit the chrome clicks `#shufColors`, which draws a pair from `PAIRS` and swaps it 40% of the time (`static: $("#shufColors")`). With `?seed=` pinned nothing touches the colours, so the Still uses the `S` defaults.
- My colors: first colour to `#inkIn`, second to `#bgIn`, so a 2-ink Palette maps as [ink, bg] on the site. `#swapColors` swaps them.
- Pixel field at `S.res` columns (default 84), drawn with nearest upscale (`static: paint`, `tiny`).

### oddgrid

- Built-in motifs (`oddgrid: MOTIFS`): none (default, with `motifAmt` 0), dot (filled circle of diameter `cs * markSize`), ring (stroked circle, line width `cs * markSize * 0.28`), square (`fillRect` of side `cs * markSize`), diag "Wedge" (right triangle filling the cell), mixed (per-cell hash picks dot 40%, square 25%, ring 20%, wedge 15%), custom. `markSize` default .52, `inkMark` true, ink `S.ink` (`oddgrid: drawMark`, `shapeOf`).
- Upload: `#svgFile` (accepts `.svg`) or a drop zone reads the file as text (`oddgrid: readFile`, `dropZones`). `parseSVG` parses it as `image/svg+xml`, removes `script`, `foreignObject`, `image` and `a`, takes `viewBox` or width/height, and keeps the inner markup; the result `{x, y, w, h, inner, name}` becomes `S.custom`, and `setMotif("custom")` switches the motif (`oddgrid: loadSVGText`). `ensureMark` wraps it in a 1x1 `viewBox` SVG, recolours fills and strokes to `S.ink` when `inkMark` is on (`inkify`), rasterises it through a Blob URL and `new Image()` at 3 times the cell size, and `drawMark` draws that image with smoothing on. `S.custom` is not part of any seed or preset.
- Looks (`oddgrid: LOOKS`): Patchwork (default), Bloom, Quilt, Scatter, Drift are Parameter presets applied by `applyLook`, not by the seed. `newSeed` only invalidates the frame cache.
- Palette family "Riso": 5 slots `#FF5E5B #FFED66 #00CECB #F2EDE4 #9B5DE5`, bg `#101820`, ink `#F2EDE4`, 3 to 10 slots. Export size 500..6000, default 2000. Motion flow, 8 fps, 32 frames.
- Consequence: built-in motifs are plain paths and fit today's Surface. A custom motif needs an SVG rasteriser (resvg) and a file input the Recipe has no field for. Ship built-ins only.

### atlas

- Font stack `MONO`: `ui-monospace, SFMono-Regular, Menlo, Consolas, "DejaVu Sans Mono", monospace`. The canvas font is `fs + "px " + MONO` with no weight, so weight 400 (`atlas: paint`). kiosk sets `"700 "` (`kiosk: paint`). The bundled `fonts/DejaVuSansMono-Bold.ttf` is therefore the wrong weight for atlas; the Book face `DejaVuSansMono.ttf` (about 340 KB) would be needed, or atlas accepts bold.
- Size `fs = 0.98 * cell height`, `textAlign` center, `textBaseline` middle, drawn at `((x + 0.5) cw, (y + 0.55) ch)`. `measureText` appears only in the SVG export's baseline calibration (`atlas: baseOffA`). The `weight` slider sets how far along the glyph ramp a cell can go, not a font weight.
- Glyph sets (`atlas: SETS`): DOS `0369#%&@!;,'()`, Stipple `.,;:'"^~*`, Blocks `░▒▓█■▪·`, Code `/\|_-+=<>[]{}`, Digits `0123456789`, Runes `†‡§¶®©≠∞≈`, Custom (default `.:-=+*#%@`, empty falls back to DOS). The same six sets as kiosk.
- Coverage: `fc-query` on the bundled Bold face lists U+0020..007E, U+00A0..01C3 (covers `§ ¶ © ® ·`), U+2020..2023 (`† ‡`), U+2217..2220 (`∞`), U+2241..2269 (`≈ ≠`) and U+2500..262F (`░ ▒ ▓ █ ■ ▪`). Every character in the six sets is present. The Book face shares DejaVu's character set but must be checked when added.

## Contradictions with the earlier notes

- Shell anatomy lists aura, mist, fete and pith under "reduced internal render budget". Only aura and mist carry the budget into the PNG (700). pith's budget scales the on-page canvas only; fete's small buffer is the `res` Parameter and is identical in the export.
- The census counts `imageSmoothingEnabled` in 18 Tools. In 7 of the 46 (atlas, bloom, mosh, pane, relief, sampler, totem) it is set without any `drawImage`, so it changes nothing.
- The census counts `drawImage` in 9 Tools. pith's call runs only for the live preview, so 8 matter to a port, and mist's is the same bicubic upscale aura already has.
- Shell anatomy gives oddgrid 12.3 KB of art code; the `const art=$("#art")` boundary used here gives 14.6 KB. The other 45 agree within 0.1 KB.
- Every other count matches the census once the 6 ported Tools are added back (see the per-feature table).

## Open points

- atlas: bundle `DejaVuSansMono.ttf` (Book) next to the Bold face, or draw atlas bold. Binary size vs fidelity.
- prism: refuse a Palette Pin, or accept and ignore. Either way the Recipe schema needs a Tool that carries no Palette.
- static: Palette order for a 2-ink Palette. The site's My colors order is [ink, bg]; tirage's other Tools put the ground first.
- Palette-family Tools: whether the ground, rule, ink, page, accent and grid fields become extra inks after the slots, or stay at site defaults. My colors never touches them, so a Palette longer than the slot count has no site replay.
- oddgrid custom motifs: out of scope unless a Recipe gains a file field.
- Deal and applySeed Tools: `derive` replaces the deal as for vein (ADR 0003); Reference exports must set the seed before the Parameters on all 13.
- The 11 `globalAlpha=1` and 14 `source-over` resets suggest the painters were written defensively; a port can skip them.
