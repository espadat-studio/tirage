# Motion census of the 52 Tools

Research for [#125](https://github.com/espadat-studio/tirage/issues/125) (map [#124](https://github.com/espadat-studio/tirage/issues/124)). For each Tool, in registry order, it describes the default site motion and what a Loop port needs. Algorithms are described in our own words; no site code was copied.

## Sources

- All 52 Tool pages from `https://www.playgrnd.tools/<slug>/`, downloaded 2026-10-11. Each page's inline script: `S.motion`, `MODES`, the `#frames`/`#fps`/`#amt` slider defaults, and every use of the default mode in `paint`. Citations use `slug: identifier`.
- Shell anatomy §6 and §7 (`git show research/shell-anatomy:docs/research/shell-anatomy.md`).
- `meta/tools/*.md`. Only sonar, kiosk and frond have a Motion section so far.
- `src/<slug>.rs` (`FRAMES`, `FPS`, the Still's code path), `src/surface.rs`, `src/render.rs`.
- `refs/export.ts`, `refs/page.ts`, `refs/manifest.ts`, `tests/refs/manifest.json`.
- `CONTEXT.md` (Loop, Still), ADR 0004.

## The answer in brief

- **All 52 Tools move on the site.** None is Still-only. Every page has `S.motion` with motion off by default, a `#modes` chooser (hidden on relief and sonar) and `#frames`/`#fps` sliders whose defaults match `S.motion`.
- **30 Tools have a frame 0 that is not their Still.** Once motion is on, their motion term is non-zero at `t = 0`. Examples: a circle walk starts at `cos 0 = 1`, a sine carries a seeded phase, or filament's active strands turn dashed. This breaks the glossary line "Its first frame is the Still of the same Recipe". It also breaks the map's plan to reuse today's motion-off Still ref as frame 0. This is the main open decision for the map. See [Frame 0 versus the Still](#frame-0-versus-the-still).
- **`FPS` mismatches: 7.** The `FPS = 1` placeholders on husk, benday, terrain, stitch, pith and mosh are confirmed: the site uses 12 on all six. riso has `FPS = 12` against the site's 8. All other constants match. `FRAMES = 1` on every Tool without a Loop, as expected.
- **`refs/export.ts` exports any frame as is, on all 52.** No Tool keeps state across frames, reads the wall clock in `paint` or calls `Math.random` in the art path. `#scrub` pauses playback and sets `curFrame`, and `#expPng` paints `curFrame`. The site's mid frame is deterministic everywhere.
- **The `frames/2` mid frame tests nothing on 6 Tools.** On sonar it equals the Still. On optic, vee and filament (2 laps per loop) it equals frame 0, and likely also on zig and quilt (2 periods per loop, checked by arithmetic only).
- **Closure groups** (§6 techniques): circular walk 23, whole-period travel 11, per-frame reseed 12, periodic scalar 5, plus fete's reveal ramp, which closes by a cut. hiss combines a walk and a reseed.

## Shell facts every Tool shares

- `t = f / frames`. The `#frames` slider changes how densely the loop is sampled, never the number of cycles. On every Tool only amount sets the cycle count.
- `#motionTog` sets `curFrame = 0`, calls `reframe` (scrub max = `frames − 1`) and starts playback. `#scrub` input calls `setPlaying(false)` and sets `curFrame`. `#expPng` paints `curFrame` into a fresh canvas after a 30 ms timeout (`husk: #expPng`). `refs/export.ts` turns motion on, sets Parameters and then scrubs, so playback is paused before export and a `reframe` from a Parameter cannot reset the frame.
- Render budgets differ by path, but each path is fixed: aura and mist paint with budget 700 for PNG. terrain coarsens its field step only while playing, so the paused export uses the Still step.
- Grain is fixed in space and does not move with the frame (§6).

## Census

Ours = `FRAMES @ FPS` in `src/<slug>.rs`. "t>0 needs" lists what the port does not have today: a Surface feature, `sin`/`cos` on `t` (the last-bit risk sonar's note flags, since Rust's and V8's `sin` can differ in the last bit), or state across frames. "Mid ok" means `refs/export.ts` can export `frames/2` as is. Frame 0 = Still? is for motion on.

| # | Tool | Default mode: moves / holds | Site frames @ fps | Ours | Closure | t>0 needs | Mid ok | Frame 0 = Still? | Default amount; effect |
|---|---|---|---|---|---|---|---|---|---|
| 1 | sonar | Tide (only mode): cut level rises and falls / land, band, hashes | 24 @ 10 | 24 @ 10 (Loop) | periodic scalar, whole cycles | `sin` on t (shipped) | yes, but `frames/2` = Still (sin π ≈ 0); manifest uses 6/18/9 | yes | 0.6; cycles `max(1, round(2a))` = 1, amplitude |
| 2 | husk | Drift: each husk traces its own closed loop (x, y phases offset) / shapes, sizes, rotation, texture | 36 @ 12 | 1 @ 1 **MISMATCH** | circular walk (position orbit) | `cos`/`sin` on t | yes | **no** | 0.6; radius `0.06a` of field width |
| 3 | vein | Drift: domain-warp sample point circles, marbling oozes / flow kind, levels, inks | 36 @ 12 | 1 @ 12 | circular walk (noise) | `cos`/`sin` on t | yes | **no** | 0.6; radius `0.25a` |
| 4 | aura | Flow: warped field coordinate circles, glows churn / ink centres, sweeps, style | 72 @ 12 | 1 @ 12 | circular walk (noise) | `cos`/`sin` on t | yes (PNG budget fixed at 700) | **no** | 0.8; radius `0.5a` |
| 5 | kiosk | Shuffle: kept cells, glyphs, inks re-dealt / rings, stripes, tiles, panel, grid | 24 @ 6 | 24 @ 6 (Loop) | per-frame reseed (`seed + 7919f`) | none (shipped) | yes | yes | 0.6; unused |
| 6 | frond | Boil: trace, block, stray-mark seeds re-dealt / streams, masses, pot | 36 @ 12 | 36 @ 12 (Loop) | per-frame reseed (`911f`) | none (shipped) | yes | yes | 0.6; unused |
| 7 | benday | Drift: tone and warp noise circle, dots swell and shrink / grid, angle, palette | 36 @ 12 | 1 @ 1 **MISMATCH** | circular walk (noise; fine octave at 2× radius) | `cos`/`sin` on t | yes | **no** | 0.6; radius `0.5a` |
| 8 | terrain | Drift: field from 4D noise, z/w circle, land morphs / dither, bands, block | 30 @ 12 | 1 @ 1 **MISMATCH** | circular walk (4D noise) | 4D value noise; `cos`/`sin` on t | yes (paused step 5) | **no** | 0.6; radius of z/w circle |
| 9 | stitch | Drift: base, stripe, block noise circle, stitch colours shift / grid, stitch shapes | 36 @ 12 | 1 @ 1 **MISMATCH** | circular walk (noise) | `cos`/`sin` on t | yes | **no** | 0.6; radius `0.6a` |
| 10 | pith | Boil: speckle, band-width, mottle, grain hashes re-dealt / cells, veins, distance field | 36 @ 12 | 1 @ 1 **MISMATCH** | per-frame reseed (`seed + 911f`) | none | yes | yes | 0.6; unused |
| 11 | mosh | Shuffle: band contents re-dealt / band heights, kinds, order | 24 @ 12 | 1 @ 1 **MISMATCH** | per-frame reseed (`seed + 7919f`) | none | yes | yes | 0.6; unused |
| 12 | mist | Bleed: ground and spray noise circle, colours bleed / spray position, inks | 72 @ 12 | 1 @ 12 | circular walk (noise) | `cos`/`sin` on t | yes (PNG budget 700) | **no** | 0.8; radius `0.45·min(1.6, a)` |
| 13 | coral | Sway: wobble-noise point circles, branches bend / fan, branches, ribs, dots | 36 @ 12 | 1 @ 12 | circular walk (noise) | `cos`/`sin` on t | yes | **no** | 0.6; radius `0.7a`; frozen if `wobble` = 0 |
| 14 | whorl | Flow: phase slides every band through the stripe field / centres, warp, count, weight | 36 @ 12 | 1 @ 12 | whole-period travel (`cyc` bands) | none (phase linear in t) | yes | yes | 0.6; `cyc = 1 + 2·floor(1.5a)` = 1; inks close only if ink count divides `cyc` |
| 15 | sear | Drift: both warp layers and the heat field offset round a circle / ramp, flattening, smear | 36 @ 12 | 1 @ 12 | circular walk (noise) | `cos`/`sin` on t | yes | **no** | 0.6; radius `1.4a` |
| 16 | culture | Drift: each colony orbits its centre, own phase / count, sizes, fuse, inks | 36 @ 12 | 1 @ 12 | circular walk (position orbit) | `cos`/`sin` on t per cell | yes | **no** | 0.6; orbit `0.05a` of width |
| 17 | bloom | Ripple: ring level stepped outward through the ink ramp / cells, warp, calm centre | 24 @ 8 | 1 @ 8 | whole-period travel (ramp turns) | none | yes | yes | 0.6; turns `max(1, round(2a))` = 1 |
| 18 | weave | Slide: each band's stripes shift sideways by whole pattern periods / bands, heights, colours | 120 @ 24 | 1 @ 24 | whole-period travel | none | yes | yes | 0.8; per band `max(1, round(n·a))`, n ∈ {1, 2} |
| 19 | warp | Bend: warp amplitude scaled by a seeded-phase sine / pattern, cells, angle, inks | 72 @ 12 | 1 @ 12 | periodic scalar | `sin` on t | yes | **no** | 0.8; bend depth `0.55a` |
| 20 | zig | Crawl: stripe polygons shift along their run by whole tooth periods / style, widths, colours | 120 @ 24 | 1 @ 24 | whole-period travel | none new (waves, scales already use `sin`) | yes, but `frames/2` likely = frame 0 (2 periods) | yes | 0.8; turns `max(1, round(2a))` = 2; stairs style may not close |
| 21 | relief | Shuffle (only mode): a share of unit cells re-dealt / lattice, repeat, face tones | 24 @ 6 | 1 @ 6 | per-frame reseed (`seed + 7919f`) | none | yes | yes | 0.6; share `min(1, a/2)` = 30% |
| 22 | atlas | Shuffle: whole grid re-dealt (bands, inks, glyphs) / cols, rows, glyph set, band count | 24 @ 8 | 1 @ 8 | per-frame reseed (`seed + 7919f`) | none | yes | yes | 0.6; unused |
| 23 | sprig | Drift: tiled motif field translates round an ellipse, wrapping / motifs, sizes, line | 36 @ 12 | 1 @ 12 | circular walk (field offset) | `cos`/`sin` on t | yes | **no** | 0.6; radius `0.25a` of a tile |
| 24 | stipple | Drift: field from 4D noise at a z/w circle, dots grow and vanish / lattice, grid, symmetry | 30 @ 14 | 1 @ 14 | circular walk (4D noise) | 4D value noise; `cos`/`sin` on t | yes | **no** | 0.3; radius |
| 25 | motley | Flicker: sparse patches re-deal dropped cells (`ns + 911f`) / patches, glyphs, layout | 36 @ 12 | 1 @ 12 | per-frame reseed | none | yes | yes | 0.6; unused |
| 26 | oddgrid | Flow: colour, coverage, detail from 4D noise at a z/w circle / grid, blocks, palette | 32 @ 8 | 1 @ 8 | circular walk (4D noise) | 4D value noise; `cos`/`sin` on t | yes | **no** | 0.8; radius (coverage 0.8×, detail 0.45×) |
| 27 | fete | Draw: doodle strokes sketch on over first 82% of loop, then hold / patch grid | 100 @ 20 | 1 @ 20 | reveal ramp + cut | none | yes | **no** (no doodle) | 0.8; unused |
| 28 | fold | Drift: patch sheet slides round a circle under fixed mirrors | 36 @ 12 | 1 @ 12 | circular walk (sample offset) | `cos`/`sin` on t | yes | **no** | 0.6; radius `0.12a` |
| 29 | quilt | Scroll: bands, tabs, zigzags, basket step outward from centre by whole periods / layout | 60 @ 10 | 1 @ 10 | whole-period travel (floor steps) | none | yes, but `frames/2` likely = frame 0 (2 periods) | yes | 0.8; periods `round(3a)` = 2 |
| 30 | static | Flicker: glitch pass and noise cells re-dealt, glitch strengthened / band engines | 60 @ 10 | 1 @ 10 | per-frame reseed | none | yes | **no** (glitch ×1.24) | 0.8; glitch `×(0.6 + 0.8a)` |
| 31 | splice | Drift: tone field glides round a circle behind fixed cuts | 36 @ 12 | 1 @ 12 | circular walk (field offset) | `cos`/`sin` on t | yes | **no** | 0.6; radius `0.09·U·a` |
| 32 | dahlia | Pulse: dashes run outward along rays, one dash period per loop / rays, beads, centre | 36 @ 12 | 1 @ 12 | whole-period travel (phase mod 1) | none; `(h + T) mod 1` float seam | yes | yes | 0.6; cycles `max(1, round(2a))` = 1 |
| 33 | hiss | Boil: grain and fleck hashes re-dealt (`911f`) and marble noise walks a circle / leaves | 36 @ 12 | 1 @ 12 | combo: circular walk + reseed | `cos`/`sin` on t | yes | **no** | 0.6; walk radius `0.15a` |
| 34 | crowd | Sway: each figure circles its home spot, own phase / arrows, ground | 36 @ 12 | 1 @ 12 | circular walk (position orbit) | `cos`/`sin` on t; phase and radius draws the Still skips | yes | **no** | 0.6; sway radius |
| 35 | cipher | Drift: wave-band and field noise inputs walk a circle / cell grid | 36 @ 12 | 1 @ 12 | circular walk (noise) | `cos`/`sin` on t | yes | **no** | 0.6; radius `0.4a` |
| 36 | riso | Tear: torn edges and streak columns wobble, each vertex on its own sine phase / bands, dot, scribbles | 16 @ 8 | 1 @ 12 **MISMATCH** | periodic scalar | `sin` on t per vertex | yes | **no** | 0.8; amplitude `min(1.6, a)`; always 1 cycle |
| 37 | rise | Ripple: band radii swing on a sine, phase lagged per band / bars, anchor | 72 @ 12 | 1 @ 12 | periodic scalar (integer cycles) | `sin` on t | yes | **no** (bands ≥ 1 offset) | 0.8; amplitude `0.3·bw·min(1.6, a)`; cycles `round(a)` = 1 |
| 38 | carve | Drift: stripe panels travel whole periods; grain panels shift and their streak sine turns / chevron, grid, flat, layout | 36 @ 12 | 1 @ 12 | whole-period travel + periodic scalar | `sin` on grain streaks | yes | yes | 0.6; turns `max(1, round(2a))` = 1; grain ramp clamps, jumps at seam |
| 39 | specimen | Flow: marble and tendril blocks resample 4D noise whose extra axes circle / flat blocks, grid, type | 24 @ 12 | 1 @ 12 | circular walk (4D noise) | 4D value-noise fBm; `cos`/`sin` on t | yes | **no** | 0.8; radius |
| 40 | pane | Shuffle: cell deal (diagonal, direction, ink pair, ramp) re-dealt / row heights, cell counts, widths | 24 @ 8 | 1 @ 8 | per-frame reseed (`seed + 7919f`) | none | yes | yes | 0.6; unused |
| 41 | modular | Flow: block and dot fields re-threshold as 4D noise walks z/w / layout, module types, rules | 24 @ 6 | 1 @ 6 | circular walk (4D noise) | 4D value noise; `cos`/`sin` into a threshold | yes | **no** | 0.6; z/w radius |
| 42 | prism | Drift: each dot cell's edge jitter swings on its own sine / streams, spines, widths, hues | 12 @ 8 | 1 @ 8 | periodic scalar | `sin` on t into ring thresholds | yes | **no** | 0.8; jitter `0.18·min(1.6, a)` |
| 43 | parcel | Creep: block-mask noise origin steps round a circle, N held steps / survey grids, chunk, cover, inks | 72 @ 12 | 1 @ 12 | circular walk (N discrete steps) | `cos`/`sin` into a threshold | yes | yes | 0.8; steps `max(2, round(6a))` = 5 |
| 44 | tokens | Cycle: active clusters roll fill and stroke inks through the Palette, staggered / positions, shapes, inactive clusters | 36 @ 14 | 1 @ 14 | whole-period travel (one Palette turn) | none | yes (export uses integer `curFrame`) | **no** (lagged tokens rolled) | no amount; `stagger` 0.55 lag, `active` 0.45 share |
| 45 | optic | Slide: ground and nested figure bars slide, alternate layers opposite / figure shapes, sizes, angles | 72 @ 12 | 1 @ 12 | whole-period travel | none | yes, but `frames/2` = frame 0 (2 laps) | yes | 0.8; laps `max(1, round(2a))` = 2 |
| 46 | vee | Travel: stripes in every band slide along their normal / band cuts, styles, angles, inks | 72 @ 12 | 1 @ 12 | whole-period travel | none | yes, but `frames/2` = frame 0 (2 laps) | yes | 0.8; laps = 2 |
| 47 | sampler | Shuffle: whole band deal re-dealt / cols, rows, ground | 24 @ 6 | 1 @ 6 | per-frame reseed (`seed + 7919f`) | none | yes | yes | 0.6; unused |
| 48 | totem | Shuffle: single rng stream restarts per frame, mat, carve, motifs, inks re-deal / border, unit, panel box | 24 @ 8 | 1 @ 8 | per-frame reseed (`seed + 7919f`) | none | yes | yes | 0.6; unused |
| 49 | filament | Travel: active strands become a 34% dash running along the strand / paths, widths, marks, inactive strands | 30 @ 16 | 1 @ 16 | whole-period travel (dash wraps length) | dashed stroke with offset in Surface; polyline length | yes, but `frames/2` = frame 0 (2 laps) | **no** (active strands dashed) | 0.8; laps = 2; `active` 0.5 share |
| 50 | chaff | Drift: whole blade pile slides round a small circle / layout, shapes, mottle | 36 @ 12 | 1 @ 12 | circular walk (mask offset) | `cos`/`sin` on a sub-pixel offset | yes | **no** | 0.6; radius `0.06a` |
| 51 | strand | Drift: plate and fill rods slide round a small circle / rod layout, slide angle, texture | 36 @ 12 | 1 @ 12 | circular walk (mask offset) | `cos`/`sin` on a sub-pixel offset | yes | **no** | 0.6; radius `0.035·U·a` |
| 52 | delta | Shuffle: glitch rng restarts per frame, pixelate inks, bands, slice cuts re-deal / field | 24 @ 12 | not ported (stuck, #122) | per-frame reseed (`seed + 7919f`) | none beyond its Still | yes | yes | 0.6; unused |

## Frame 0 versus the Still

These 30 Tools render something different at frame 0 with motion on than with motion off: husk, vein, aura, benday, terrain, stitch, mist, coral, sear, culture, warp, sprig, stipple, oddgrid, fete, fold, static, splice, hiss, crowd, cipher, riso, rise, specimen, modular, prism, tokens, filament, chaff, strand.

- Circular walks sit at `(r, 0)` on frame 0. The Still sits at the circle's centre, which no Loop frame visits (`husk: drift`, `sear: fieldGridS`, `fold: mx`, `splice: gx`, `cipher: drx`, `chaff: paint`, `strand: paint`). The 4D walks put z = r on frame 0 (`stipple: dots`, `oddgrid: computeFrame`, `specimen: scene`, `modular: cellOn`).
- Seeded or per-element phases: warp, riso, rise (band 0 alone holds), prism, crowd, culture.
- Mode-specific shapes: fete's Draw reveals nothing at t = 0. static's Flicker raises glitch strength 1.24× on every frame. tokens' lag rolls inks at t = 0. filament dashes its active strands.
- The other 22 Tools give frame 0 = Still, because their offset is zero at f = 0: sonar, kiosk, frond, pith, mosh, whorl, bloom, weave, zig, relief, atlas, motley, quilt, dahlia, carve, parcel, pane, optic, vee, sampler, totem, delta.
- Consequence: matching the site's motion-on frame 0 on those 30 contradicts `CONTEXT.md` (Loop: "Its first frame is the Still of the same Recipe") and ADR 0004 (the Still is frame 0 and the `<video poster>`). Options for the map: (a) amend the glossary and ADR 0004, so a Loop's frame 0 is the site's motion-on frame 0, with a motion-on `seed-N-f0` ref beside the Still ref, and recordreel's poster becomes a rendered frame 0; (b) start each Loop at a phase where it passes through the Still, which diverges from the site's frame indices and breaks frame-for-frame refs; (c) keep these 30 Still-only. The rig already supports (a): a fixture with `frame: 0` turns motion on in `refs/export.ts`.

## Per-Tool notes

- sonar: cycles `round(2a)`; the only Tool among 1–13 where amount sets the cycle count. Its manifest mids avoid the degenerate `frames/2`.
- husk: `husk: husksH` caches on a key that includes f, a memo, not state. y phase is 1.6× the x wobble, so the paths are Lissajous-like and close exactly at t = 1.
- terrain: once motion is on, `terrain: renderInto` switches to 4D `vnoise` (16-corner lerp across z and w, not scaled per octave). `src/terrain.rs` has a 4D `hash` but only 2D noise. Site video records 2 loops (§7, irrelevant to frame stepping).
- coral: Sway moves only the wobble term. If taste bounds can deal `wobble = 0`, that Seed's Loop is frozen. Not checked.
- whorl: geometry closes but inks do not. Band ink is the index mod ink count, and one loop moves each band index by `cyc`. At the default `cyc = 1`, the last frame cuts to frame 0 with each colour rotated one ink, unless only one ink is used (`whorl: paint`). Site behaviour, so a port reproduces it. A Rust closure test must expect it.
- zig: exact for teeth, chevron, ricrac, waves and scales at 2 turns. The stairs style shifts by `2s·turns` against a band pitch it may not divide; not verified (`zig: crawlShift`).
- stipple, oddgrid: our ports reuse `terrain::fbm` on the z = w = 0 slice. Loops need the full 4D lerp. oddgrid applies z/w in every mode.
- fete: Draw holds the whole doodle from frame 82 of 100, then cuts to the empty grid. None of the four §6 techniques fits; it closes by index wrap, like a reseed.
- carve: grain-ramp panels add the phase and clamp to 0..1 (`carve: panelGrain`), so they wash towards one ink and jump at the seam. Stripes close exactly.
- quilt: shift is measured from the centre column, so motion is mirror-symmetric. Diamond and cross step a parity key 8 times per loop, and cross also re-deals accents on that key (`quilt: renderGrid`).
- static: glitch rng reseeds with `seed + f·2654435761`. Block cells re-deal only on frames where a hash is under 0.25 (`static: cellOn`).
- dahlia: frame `frames` matches frame 0 only up to float rounding of `(h + 1) mod 1`.
- crowd: the Sway phase and radius are the two draws `meta/tools/crowd.md` step 8 says the Still skips.
- specimen: 4D fBm; marble iso contours keep the Still's 1-decimal rounding. Video records 3 loops.
- parcel: each of the 5 steps is held 14.4 frames, so steps change on `floor(tN)`. Frame 36 is step 2.
- tokens, filament: `active` (and tokens' `stagger`) already take one draw per cluster or strand in the Still path. A Loop fixes them at site defaults (0.45 / 0.55, 0.5). tokens' playback shows fractional positions on screen, but export uses integer frames.
- modular: Flow needs no x-periodic noise (only Shift does).
- delta: `paintField` reuses one buffer but rewrites it on every call; not state.

## Summary

### Movers versus Still-only

- Movers: 52 of 52. Loops shipped: sonar, kiosk, frond. To port: 48, plus delta after its Still unsticks.
- Still-only: none on the site's evidence. A Tool turns Still-only only by map decision (frame-0 policy) or by getting stuck.

### Closure groups

| Group | Count | Tools |
|---|---|---|
| Circular walk, noise space (cos/sin of 2πt; exact) | 14 | vein, aura, benday, terrain, stitch, mist, coral, sear, stipple, oddgrid, cipher, specimen, modular, hiss (with reseed) |
| Circular walk, position or sample offset | 8 | husk, culture, crowd, sprig, fold, splice, chaff, strand |
| Circular walk, stepped | 1 | parcel |
| Periodic scalar (sine with integer cycles) | 5 | sonar, warp, riso, rise, prism |
| Whole-period travel (integer periods, laps or ink turns) | 11 | whorl, bloom, weave, zig, quilt, dahlia, carve (plus a streak sine), tokens, optic, vee, filament |
| Per-frame reseed (closes by cut) | 12 | kiosk, frond, pith, mosh, relief, atlas, motley, static, pane, sampler, totem, delta |
| Reveal ramp + cut | 1 | fete |

The reseed constant is `7919f` on kiosk, mosh, relief, atlas, pane, sampler, totem and delta. It is `911f` on frond, pith, motley and hiss. static uses its own multiplier.

### Enablers shared by 3 or more Tools

| Enabler | Tools | Count |
|---|---|---|
| Frame-0 policy: glossary + ADR 0004 amendment and a motion-on frame-0 ref | the 30 above | 30 |
| Rust closure test that renders t = `frames` (`Frame::new` rejects `t >= frames`; needs a test-only path, or a check through `t = 0` vs a wrapped index) | every continuous mode: 52 − 12 reseed − fete | 39 |
| `sin`/`cos` on t: last-bit risk at thresholds and cuts (Rust std vs V8's fdlibm) | circular walks, periodic scalars, carve | 29 |
| Per-Tool mid frame instead of `frames/2` (the manifest already accepts any `frame`) | sonar, optic, vee, filament; likely zig, quilt | 4–6 |
| 4D value noise (16-corner lerp over z, w) | terrain, stipple, oddgrid, specimen, modular | 5 |
| `FPS` constant fix | husk, benday, terrain, stitch, pith, mosh, riso | 7 |

Single-Tool needs: a dashed stroke with offset in Surface (filament). The rig needs no change: `refs/export.ts` handles mid frames and motion-on frame 0 as is.

## Open questions

- The frame-0 policy for the 30 Tools, above. It decides whether the first batch carries a glossary change on top of the ADR 0004 amendment.
- Mid frames on zig and quilt are inferred from period arithmetic, not from an export. zig's stairs closure and coral's `wobble = 0` reachability are also unverified.
- Whether the trig last-bit risk is worth an fdlibm-faithful `sin`/`cos` (a new dependency or a port), or whether the 1.6% ceiling absorbs it. sonar shipped on Rust std and stayed in bounds.
