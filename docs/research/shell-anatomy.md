# Anatomy of the shared playgrnd shell

Research for issue #3 (map #1). It describes how the 52 Tool pages on playgrnd.tools are built, in our own words. No code was copied.

## Sources

- All 52 Tool pages as served by `https://www.playgrnd.tools/<slug>/`, downloaded on 2026-10-08. Every claim below comes from reading their inline scripts and HTML.
- `~/.claude/skills/playgrnd/scripts/pg.ts`, our existing driver, for the control ids it relies on.
- Citations use the form `slug: identifier`. They point at a function, constant or element id inside that page.

## The answer in brief

- Each page loads three inline scripts. There is no shared bundle. The "shell" is a template that has been copied into every page.
  1. **Tool script** (27 to 50 KB). The Tool's own art code, followed by a copied block for canvas, playback, export and controls, followed by the grain and dither post-passes.
  2. **Embed bootstrap** (1.6 KB). Runs only when the page is iframed or has `?embed`. 49 pages share an identical copy. aura and mist have a one-character variant. warp has its own copy with its own ratio list.
  3. **Chrome script** (46 KB). Byte-identical on all 52 pages. It handles slider styling, My colors, the colour picker, `__freshPalette`, the variation history, `?seed=` pinning on normal visits, and the first-run tip.
- **Randomness is seeded.** Every drawing path uses a seeded xorshift generator plus seeded integer-hash noise. `Math.random` is only used for UI actions: a new seed, new colours, or adding a swatch.
- **A seed alone is not a Recipe.** The rendered output is a pure function of seed, Parameters, Palette, ratio, pixel size, frame index, and the grain and dither settings. The palette is separate state. On a normal visit it is randomised, so the same `?seed=` can show different colours unless the palette is pinned as well.
- **Loops close because the motion is periodic in frame index.** Each frame is drawn from `t = frame / frames`. Every motion mode is built so that `t = 1` looks the same as `t = 0`.
- **The art code is small.** It is 5 to 17 KB per Tool (median 9.7 KB), which is 7% to 20% of the page's JS (median 12%). The other 80% or more is shell.

## 1. Page structure

| Layer | Where | Same on all Tools? | Contents |
|---|---|---|---|
| Art | Tool script, everything before `fitCanvas` | no | `PALETTES`, `RATIOS`, `MODES`, the state object `S`, hash and noise helpers, the painter, `paint(ctx, W, H, frame)` |
| Shell, per page | Tool script, from `fitCanvas` to the grain block | copied template, with small per-Tool edits | `fitCanvas`, `draw`, `requestDraw`, `tick`, `setPlaying`, `toast`, `save`, `stamp`, PNG/SVG/video handlers, `pickVideoType`, `accent`, `segmented`, `buildRatios`, `buildSwatches`, `setPalette`, `bindRange`, `newSeed`, plus event wiring |
| Chassis | Tool script, the "Grain (chassis)" and "Dither (chassis)" blocks | identical, 9,091 bytes (oddgrid has 1 extra byte) | grain and dither post-passes. These wrap `paint` |
| Embed | script 2 | 3 variants | iframe and landing-page background mode |
| Chrome | script 3 | identical | UI behaviour shared across the whole site |

29 top-level function names appear in at least 40 Tool scripts. These are the copied shell. The rest are per-Tool. Many Tools carry their own copy of the hash, noise, rng and hex helpers under a one-letter suffix, for example `benday: hshB, vnB, rngB` and `frond: hshF, vnF, rngF`. The art is not always a single `draw<Tool>` function. Only aura and mist have one (`drawAura`, `drawMist`). Most Tools paint inside `paint` itself or through a small set of helpers.

## 2. Random generator and seed

- **Generator.** `rng(s)` is a 32-bit xorshift (shifts 13, 17, 5). It returns a float in [0, 1). A zero state is replaced by 1. Tools with suffixed copies (`rngB`, `rngS`, ...) use the same idea.
- **Noise.** `hashi(x, y, z, s)` mixes integer lattice coordinates with a seed through Math.imul multiply-xorshift rounds. Value noise (`vn`) blends the four lattice corners with smoothstep weights. `fbm` sums 2 or 3 octaves at roughly doubling frequency. The noise seed (`NS`) comes from the Tool seed through a fixed XOR, so the noise field and the rng stream are separate but both depend only on the seed.
- **From seed to rng.** Most Tools seed the art rng with the Tool seed times the 32-bit golden-ratio constant, wrapped to 32 bits (`buildArt`, `buildSpec`). Some add a small offset (`tokens`, `modular`, `specimen`). Tools that also deal layout numbers use a second stream, seeded by XORing the seed with a different constant (`optic, rise, riso, prism, static: applySeed`).
- **What the seed controls depends on the Tool.**
  - Most Tools: the seed drives only the painter's internal draws, such as positions, offsets and an "auto" style choice (`aura: buildArt` picks one of `STYLE_POOL` when the style is `auto`). Parameters stay where the user put them.
  - **Deal Tools** (benday, cipher, coral, crowd, dahlia, fold, hiss, motley, stitch, vein): `newSeed` calls `deal<X>(seed)`. That function overwrites most Parameters with seeded values and pushes them back onto the sliders. benday and hiss also pick the palette from `PALETTES` with the seed.
  - **applySeed Tools** (optic, rise, riso, prism): the seed deals a few layout Parameters, for example riso's band count, roughness and scribble.
- **Seed values.** New seeds are random integers from 1 to 99999 (`newSeed`). Typed seeds are parsed as integers and made non-negative. A non-numeric seed falls back to 1. Clicking the canvas or pressing Space also deals a new seed.
- **`?seed=`.** On a normal visit, the chrome script reads `?seed=N` or `#seed=N`. It writes N into `#seed` and fires `change`, which calls `newSeed(N)`. If no seed is pinned, it clicks `#shuf` (random seed) and `#shufColors` (random palette). It skips the palette click when My colors is on. In embed mode, script 2 does the same pinning or shuffling, but never touches colours.
- **Reproducibility.** Every `Math.random` call sits in a UI handler: new seed, new colours, add swatch, or `newPalette`. None is in a paint path. `performance.now` is only used to pace playback and video. Output is therefore fully reproducible from Tool + seed + Parameters + Palette + ratio + pixel size + frame + grain/dither settings. Two things to watch:
  - The palette is not derived from the seed, except in benday and hiss. To reproduce an image, pin the palette as well.
  - In deal Tools, setting the seed resets the Parameters. Set the seed first, then the Parameters.
- **Resolution changes the pixels, not the composition.** Painters scale by `min(W,H)` or `sqrt(W*H)`. The grain and dither grids are sized in picture units, so a 6000 px export carries the same grain as the preview (`__grain`, `__dither`). aura, mist, fete and pith render at a reduced internal *budget* resolution and then upscale. The budget is 130 px while playing, 260 for a still preview, 700 for PNG and 300 for video (`aura: paint, drawAura`). So the preview and the PNG differ in fine detail.

## 3. Parameter model

- **State.** Everything lives in one object, `S`, in the Tool script. It holds the seed, the ratio, each Parameter, the palette, the export size, and a `motion` sub-object. Defaults are the literal values in `S`. The slider's HTML `value` attribute matches them (checked on aura).
- **Sliders.** `bindRange(id, format, set, after)` attaches an `input` listener to `#<id>`. The listener parses a float, writes it into `S`, updates the label `#<id>Val`, and redraws through `reframe`/`requestDraw`. Ranges and steps live only in the HTML attributes.
- **Enums.** `segmented(host, items, get, set)` turns a container such as `#styles` or `#modes` into an accessible dropdown. When a Tool has only one motion mode, the chrome hides the mode row (relief, sonar).
- **Toggles.** Toggles are buttons that use `aria-pressed`: `#motionTog`, `#grainTog`, `#ditherTog`, `#mycTog`, plus Tool-specific ones. Clicking flips a boolean in `S` and shows or hides that group's body.
- **Derived values.** A Tool can compute one control from another and disable the passive one. The chrome then hides any disabled range row and brings it back when re-enabled (`syncDisabled`). Example: in specimen, Rows follows Columns while "square cells" is on. Deal and applySeed Tools write their dealt values back to the sliders (`syncControls`, `put`).
- **Name clash with `#size`.** In 19 Tools, export size is `#sizePx` (state field `size_px`), not `#size`: benday, chaff, cipher, coral, crowd, culture, dahlia, fold, frond, hiss, husk, motley, pith, splice, sprig, stitch, strand, vein, whorl. In 12 of them, `#size` is an art Parameter with a fractional range, about 0 to 1 (hiss goes from 0.4 to 1.4). The other 7 have no `#size` at all. pg.ts `--size` sets `#size`, so on these Tools it changes the art and never touches the export resolution.

## 4. Palette system

- **Slots.** A palette is an ordered list of hex colours, usually `S.pal`, with one swatch button per slot in `#swatches`. Each swatch holds a hidden `<input type=color>`. Its `input` handler writes to `S.pal[i]` and redraws. Slot counts vary by Tool:
  - **Fixed, with named roles** (no add or remove): aura and mist (4 inks), quilt (Base, Weave A, Weave B, Pop), vee (4), warp (5), rise (Field, Bars, Accent), parcel (Base, Ink, Lines), optic (Base, Ink). prism has a fixed 6-colour palette and no swatch UI, so its colours cannot be edited.
  - **Variable** (add with `+`, remove with the corner `x`, at least 2 kept): most `S.pal` Tools. The default sizes come from their `PALETTES` presets, from 2 inks (chaff, sprig, whorl) to 8 or 9 (atlas, benday, kiosk, motley, pane).
  - **Palette-family Tools** (fete, filament, modular, oddgrid, specimen, stipple, terrain, tokens, weave, zig) use `S.palette`. They have their own `shuffleColors` generator. Most also have named palette chips (`refreshPalettes`, or `refreshChips` in oddgrid). Some also have separate fields for ground, rule or ink (`#bgIn`, `#ruleIn`, `#inkIn`, `#pageIn`, `#gridIn`, `#accIn`). The upper limit is 6 to 12 slots.
  - static has no swatch row, only two loose colour fields (`#bgIn`, `#inkIn`).
- **New colors** (`#shufColors`). Most Tools pick a random preset and pass it through the chrome's `__freshPalette`. That function gives every slot a new hue from one random base hue plus a hue-offset scheme. There are 4 schemes: analogous steps, complementary pairs, triads, and a split scheme. A random direction flip and ±10° jitter are added. Each slot keeps roughly its original saturation and lightness, with small random scaling. Near-grey slots stay near-grey, so the ground and paper roles survive. mist, optic, rise and riso use their own `newPalette`, a hand-tuned HSL recipe, falling back to `__freshPalette` some of the time. Palette-family Tools use `shuffleColors`, which picks a scheme (stepped, triad or complementary) from a throwaway rng, plus a dark or light ground.
- **Rotate** (`#rotColors`, or the chrome's `#rotInks` for the 9 palette-family Tools that lacked one) moves every colour one slot along by writing through the swatch inputs.
- **My colors** (`#mycTog`, chrome script, on 51 Tools, not prism). One site-wide set is stored in `localStorage["playgrnd.mycolors"]` as `{v:1, cols:[hex...], on:bool}`. Input is parsed as hex words split on whitespace, commas or semicolons. 3-digit hex is expanded, everything is lowercased, and duplicates are dropped.
  - **Fitting** (`fit(cols, n)`). The Tool needs n slots, which are its swatch inputs or, if it has none, its loose colour fields. If the set has n or more colours, the first n are used in order. If it has fewer, each missing slot k is derived from set colour `k mod len`. It is lightened or darkened in HSL, alternating direction on each pass through the set and stepping further each pass. The direction flips for colours that are already light (L above 0.62). Lightness is clamped to 0.07 to 0.94, saturation is reduced slightly when lightening, and the code retries a few times to avoid duplicates. Derived slots are marked `data-derived`.
  - **Applying.** The set is applied by writing each swatch input's value and firing `input`, so the Tool's own handler does the work. With My colors on, New colors becomes Shuffle: it re-orders the set and re-fits it. Swatches can be dragged to re-order them. Pressing New, Space or changing the seed re-applies the set if a seed-dealt palette (benday, hiss) replaced it. A `storage` event syncs the set across tabs.
- **Accent.** `accent()` picks the palette's most luminous colour with relative luminance between 0.1 and 0.85 and uses it as the UI accent (`--accent`, `#strip`). It is cosmetic only.

## 5. Ratio and size

- **Ratios.** `RATIOS` is a list of `{id, w, h}`. 49 Tools offer 9:16, 3:4, 4:5, 1:1, 5:4, 4:3, 3:2 and 16:9. riso offers the same set in a different order, starting with 1:1. warp offers 9:16, 3:4, 1:1, 4:3, 3:2, 16:9, 2:1 and 3:1. `buildRatios` renders the `#ratios` buttons. Clicking one sets `S.ratio` and redraws.
- **Preview.** `fitCanvas` fits the largest box of the chosen ratio into the stage padding. It sets the canvas backing store to that size times the device pixel ratio, capped at 2, and the CSS size to the display size. It is re-run on resize through a ResizeObserver.
- **Export size.** The PNG and SVG width is `S.size` (or `S.size_px`), range 600 to 6000 px in steps of 100, default 2400 (oddgrid: 500 to 6000, default 2000). Height is the width times h/w, rounded. The embed bootstrap picks the ratio closest to the viewport in log space, so the cover-crop stays small.

## 6. Animation loop

- **Motion state.** `S.motion` = `{on, mode, amount, fps, frames}`, plus `active`/`stagger` in filament and tokens. Motion is off by default on every Tool. `#motionTog` turns it on, resets to frame 0, enables `#expVid` and `#scrub`, and starts playback unless the browser prefers reduced motion. Defaults are 12 fps / 36 frames, 12 / 72, or 8 / 24 depending on the Tool family. The extremes are weave and zig (24 fps / 120 frames) and prism (8 / 12).
- **Playback.** `tick` maps elapsed wall time onto a frame index modulo `frames`, at `fps`. It only redraws when the index changes. So the time base is the frame index, not wall time, and playback and export both see the same discrete frames. `#scrub` jumps to a frame and pauses. `p` toggles play.
- **Why loops close.** Each frame is drawn from `t = f/frames`, so t runs over [0, 1) and frame `frames` would equal frame 0. Every mode is built so the art at t = 1 matches t = 0. Four techniques are used, often several in one Tool:
  1. **Circular walk in noise space.** Drift and flow modes move the noise sample point around a circle, using the cosine and sine of 2πt. The field ends where it began (`aura: drawAura`, `benday: paint`).
  2. **Periodic scalars.** Pulse and breathe modes scale an amplitude or zoom by the cosine of 2πt (`aura` zoom).
  3. **Whole-period travel.** Scroll, slide, crawl, ripple and cycle modes move a pattern by a whole number of its own periods, or roll palette indices by a whole number of turns, over the loop. carve says in a comment that it uses floor instead of round so that the last frame is not a duplicate of the first (`carve: paint`, `bloom` ripple). Amount usually sets how many periods or turns happen per loop.
  4. **Per-frame reseed.** Shuffle and flicker modes re-seed the art rng with the seed plus a multiple of f (`delta: paint`, `frameSeed`). Frames are discrete, and the loop wraps because the index wraps. parcel steps a fixed `stepsPerLoop` count. These loops are seamless only in the sense that each frame is a separate image.
- **Grain** is fixed in space and seeded by constants, not by frame, so it holds still while the art moves.

## 7. Export paths

All exports call the same `paint`. Files are named `<slug>-<seed>-<WxH ratio>` (`stamp`), for example `aura-11-16x9.png`.

| Export | Button | Tools | How |
|---|---|---|---|
| PNG | `#expPng` | 52 | `paint` into an offscreen canvas at `S.size`, current frame, then `toBlob`. Grain and dither apply |
| Video | `#expVid` | 52, needs motion on | `MediaRecorder` on `canvas.captureStream(60)`. Prefers MP4 (H.264 baseline), then WebM VP9, VP8, generic. 16 Mbps on 47 Tools, 12 to 14 Mbps on the rest. Width is `min(cap, S.size)`, rounded to even, with a cap of 1120, 1280, 1600 or 1920 per Tool |
| SVG still | `#expSvg` | 29 | Two methods. (a) **Recording surface**: bloom, kiosk, mosh, pane, relief, sampler, sonar and totem pass a fake 2D context (`SvgSurface*`) to `paint`. It turns `fillRect` and paths into SVG elements, so the SVG always matches the canvas. (b) **Dedicated emitter**: other Tools rebuild the scene as SVG markup from the same layout data (`buildSVG`, `svgScene`, `svgDoc`, `markSVG`). Grain and dither are not included |
| Animated SVG | `#expSvgLoop` (13), `#expSvgAnim` (6) | fete, optic, parcel, prism, quilt, rise, riso, static, vee, vein, warp, weave, zig, and filament, modular, oddgrid, specimen, stipple, tokens | SMIL only (`<animate>`, plus `<animateTransform>` in optic and rise). Smooth modes become one scene with SMIL attributes. Discrete modes become a flip-book of frame groups, with visibility switched on a discrete clock (`rise: buildLoopSVG`). Duration = frames / fps |
| TXT | `#expTxt` | atlas only | Builds the glyph grid for the current frame and writes one line per row, with trailing spaces trimmed (`atlas: buildTXT`). atlas's SVG uses real `<text>` with `textLength`, so characters stay editable |

- **How video frames are paced.** In most Tools, video frames are painted one by one. Each frame is followed by a timeout of about 1000/fps − 4 ms and a `requestAnimationFrame`, while the stream captures at 60 Hz. Frame timing in the file therefore depends on the browser and is not exact. modular, oddgrid, specimen and tokens instead pace frames by `performance.now`. The number of loops recorded is 1 on most Tools, 2 on filament, terrain and tokens, and 3 on modular, oddgrid, specimen and stipple.
- **Post-passes.** The chassis replaces `paint` with a wrapper. The wrapper finds the argument that is a 2D context with `getImageData`, then runs `__grain` and then `__dither` on it. Grain is clumped value-noise dust in 5 blend modes, with specks and a vignette. Dither is ordered Bayer 4 or 8, or hashed noise, applied per block, with N levels per channel, through a lookup table. The SVG recording surfaces have no `getImageData`, so SVG output skips both passes. Grain starts on in coral, crowd, dahlia and vein (`GRAIN_DEFAULT`).

## 8. Art code size vs shell

Method: art code is the Tool script up to `fitCanvas`. That covers palettes, constants, state, noise and the painter. "Page JS" is the Tool script plus the embed and chrome scripts, about 75 to 98 KB. For the 10 palette-family Tools, the SVG emitter and palette chips sit after `fitCanvas` and are counted as shell, so their art share is slightly under-stated.

- Art code: 5.2 to 17.1 KB, median 9.7 KB.
- Per-page shell block in the Tool script: 12.9 to 28.5 KB, median 14.2 KB. It is larger where Tools add SVG and animated-SVG emitters (oddgrid 28.5, modular 22.6, filament 22.1, specimen 21.7, tokens 20.4).
- Fixed shell: chassis 9.1 KB, chrome 46.1 KB, embed 1.6 KB.

<details><summary>Per-Tool sizes (52 rows)</summary>

| Tool | Art code (KB) | Tool script (KB) | Art share of page JS |
|---|---:|---:|---:|
| atlas | 10.5 | 34.1 | 13% |
| aura | 5.2 | 27.1 | 7% |
| benday | 8.0 | 31.5 | 10% |
| bloom | 7.5 | 30.3 | 10% |
| carve | 15.6 | 38.5 | 18% |
| chaff | 10.7 | 33.7 | 13% |
| cipher | 9.9 | 33.1 | 12% |
| coral | 13.0 | 36.0 | 16% |
| crowd | 10.9 | 33.9 | 14% |
| culture | 8.1 | 30.9 | 10% |
| dahlia | 7.2 | 30.2 | 9% |
| delta | 8.2 | 31.1 | 10% |
| fete | 12.1 | 35.4 | 15% |
| filament | 9.4 | 39.9 | 11% |
| fold | 9.1 | 32.5 | 12% |
| frond | 16.1 | 39.2 | 19% |
| hiss | 11.1 | 34.8 | 14% |
| husk | 8.0 | 30.8 | 10% |
| kiosk | 12.8 | 36.6 | 15% |
| mist | 5.6 | 28.0 | 7% |
| modular | 7.0 | 37.9 | 8% |
| mosh | 11.7 | 34.9 | 14% |
| motley | 8.2 | 31.3 | 11% |
| oddgrid | 12.3 | 49.7 | 13% |
| optic | 10.3 | 34.3 | 13% |
| pane | 9.7 | 32.6 | 12% |
| parcel | 8.2 | 30.5 | 11% |
| pith | 17.1 | 40.1 | 20% |
| prism | 9.4 | 32.0 | 12% |
| quilt | 13.4 | 35.6 | 16% |
| relief | 10.8 | 33.7 | 13% |
| rise | 7.8 | 32.1 | 10% |
| riso | 12.2 | 36.8 | 15% |
| sampler | 10.6 | 33.7 | 13% |
| sear | 9.6 | 32.3 | 12% |
| sonar | 7.1 | 30.0 | 9% |
| specimen | 14.4 | 44.5 | 16% |
| splice | 10.9 | 33.9 | 14% |
| sprig | 10.3 | 33.2 | 13% |
| static | 9.1 | 31.0 | 12% |
| stipple | 8.0 | 34.7 | 10% |
| stitch | 7.2 | 30.8 | 9% |
| strand | 13.8 | 36.9 | 17% |
| terrain | 5.7 | 29.0 | 8% |
| tokens | 8.7 | 37.5 | 10% |
| totem | 12.6 | 36.0 | 15% |
| vee | 8.5 | 30.9 | 11% |
| vein | 13.4 | 37.0 | 16% |
| warp | 6.8 | 29.1 | 9% |
| weave | 7.2 | 30.1 | 9% |
| whorl | 7.7 | 30.5 | 10% |
| zig | 10.1 | 33.7 | 13% |

</details>

## 9. Tools that deviate from the common shell

| Deviation | Tools |
|---|---|
| Own export functions, SVG motif upload (`parseSVG`, `readFile`, `dropZones`), different code style, largest page | oddgrid |
| Palette-family model (`S.palette`, `shuffleColors`, named chips, separate ground fields, `#rotInks`) | fete, filament, modular, oddgrid, specimen, stipple, terrain, tokens, weave, zig |
| Export size on `#sizePx`, not `#size` | the 19 Tools listed in section 3 |
| Seed deals Parameters (and the palette in benday and hiss) | benday, cipher, coral, crowd, dahlia, fold, hiss, motley, stitch, vein |
| Seed deals some layout Parameters | optic, rise, riso, prism |
| Own `newPalette` | mist, optic, rise, riso |
| No swatch row or no My colors | prism (fixed palette, no colour UI), static (two loose fields) |
| Different ratio set or order, own embed copy | warp (adds 2:1 and 3:1, drops 4:5 and 5:4), riso (reordered) |
| Reduced internal render budget | aura, mist, fete, pith |
| Grain on by default | coral, crowd, dahlia, vein |
| Multi-loop or wall-clock-paced video | filament, modular, oddgrid, specimen, stipple, terrain, tokens |
| TXT export | atlas |
| Single motion mode (chooser hidden) | relief, sonar |

## Implications for visuals

- A Recipe must store the palette explicitly. Seed + Parameters is not enough, except in benday and hiss.
- For deal Tools, apply the seed first and the Parameters afterwards. Otherwise the deal overwrites the Parameters.
- Reference exports should drive the export size through `#sizePx` on the 19 Tools above. pg.ts `--size` needs a per-Tool switch.
- To render frame-exact Loops, step `paint(ctx, W, H, f)` for f from 0 to frames−1. Do not record a MediaRecorder stream: its frame timing is not exact.
- Our reproduction only has to cover the art layer. Grain and dither are two small post-passes that every Tool shares, so they can be built once.
