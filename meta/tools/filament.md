# filament

Bundles of hair-fine strands traced through a curling flow field on a dark ground. Each bundle starts from one spot, so its strands run together like a rope and fray apart. A few strands carry real weight, most stay fine. Small symbols in accent inks ride the strands, more of them near the far end, and a share of strands can carry a chain of beads in the strand inks.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults, in page order:

| id        | range                | step  | default | role                                       |
| --------- | -------------------- | ----- | ------- | ------------------------------------------ |
| `zoom`    | 0.3..6               | 0.05  | 1.9     | flow field frequency                       |
| `turn`    | 0.2..5               | 0.05  | 1.9     | how many turns the field's angle spans     |
| `oct`     | 1..6                 | 1     | 3       | field octaves                              |
| `curl`    | 0..1                 | 0.01  | 0.45    | weight of the second, curling field        |
| `tangle`  | 0..1                 | 0.01  | 0       | how far strands leave the field and cross  |
| `bundles` | 1..90                | 1     | 26      | how many bundles                           |
| `per`     | 1..40                | 1     | 12      | strands per bundle                         |
| `clump`   | 0..1                 | 0.01  | 0.6     | how hard bundles gather on the high ground |
| `tight`   | 0.005..0.2           | 0.005 | 0.04    | bundle spread, share of the short edge     |
| `len`     | 20..600              | 10    | 230     | most steps a strand walks                  |
| `step`    | 1..10                | 0.2   | 3.4     | step length                                |
| `wgt`     | 0.2..8               | 0.1   | 1.6     | base stroke weight                         |
| `wvar`    | 0..1                 | 0.01  | 0.6     | random spread of the weight                |
| `hier`    | 0..1                 | 0.01  | 0.55    | how few strands carry the heavy weight     |
| `hair`    | 0..1                 | 0.01  | 0.22    | share of strands that are hairlines        |
| `mstyles` | Symbols, Beads, Both |       | Symbols | which marks ride the strands               |
| `mark`    | 0..1                 | 0.01  | 0.3     | symbol density                             |
| `bead`    | 0..1                 | 0.01  | 0.28    | share of strands that carry beads          |
| `bgap`    | 2..14                | 1     | 4       | points between beads                       |
| `msize`   | 2..26                | 0.5   | 9       | mark size                                  |

`kinds` is a row of chips that switch symbol kinds on and off, all five on by default: arrow, ring, cross, hook, tick. It is a multi-select, not a slider or a one-of picker, so it is not a Parameter and stays at all five. On this page the export size is `size`, 600..6000, default 2400.

Motion (`modes`, `active`, `amt`, `fps`, `frames`) is off by default. filament is a Still Tool: only frame 0 is rendered. With motion off there is no sway, no travelling dash and no reveal alpha, so every strand is a solid stroke at full alpha. `active` still costs one draw per strand (below), but its value never reaches a Still, so it is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The site deals nothing from the seed beyond the trace itself. `derive` deals every Parameter (ADR 0003).

## Palette

filament is a palette-family Tool. Its Palette is the swatch row, read by position, any number of inks from 2. The site row holds 1 to 8. Default Palette, 3 inks: `#FF5DA2 #FF9ACB #FFE6F1`.

The site's family also sets a ground and three accents. They are Tool constants at the site defaults, not part of the Palette: ground `#120A1E`, accents `#F9F871 #4CE0D2 #B09CFF`. The family picker is not a Parameter.

## Space

The trace runs in a view box `VB = 1000` wide and `H = 1000 · h / w` tall for the ratio `w:h` (9:16 gives `H = 1777.7…`). A frame `W` px wide draws it scaled by `k = W / 1000`.

## Noise

The hash is [aura](aura.md)'s 4-argument integer hash `hash(x, y, z, seed)`. Value noise `vn(x, y, seed)` smoothsteps between the four corner hashes at `z = 0`, mixing along x first. `fbm(x, y, seed, oct)` sums `oct` octaves of `vn`, octave `i` at frequency `2^i`, weight `0.5^(i+1)`, seed `seed + 1319 i`, divided by the weight total. `seed` is the Tool seed `s`; seeds wrap as 32-bit integers.

The field angle at `(x, y)`:

- `u = x / VB · zoom`, `v = y / H · zoom · H / VB`
- `a = fbm(u, v, s, oct)`, `b = fbm(u + 31.7, v - 17.3, s + 404, oct)`
- `angle = a · 2π · turn + (b - 0.5) · 2π · curl`

## Draws

One xorshift stream, the arithmetic-shift one [warp](warp.md#draws) describes (aura's), started from `s · 2654435761 + 13` as a double, modulo 2^32, 0 becoming 1. `r` is the next draw. `margin = 0.12 · max(VB, H)`, `spread = tight · min(VB, H)`.

Bundles: until `bundles` are placed or `40 · bundles` tries are spent:

1. `bx = (-0.1 + 1.2 r) VB`, `by = (-0.1 + 1.2 r) H`.
2. Only when `clump > 0`: `fv = fbm(2.2 bx / VB, 2.2 by / H, s + 733, 2)`. One draw `r`; when `r > fv^(1 + 5 clump)` the try is spent and nothing else is drawn.
3. The bundle is placed; its number `b` counts from 1. One draw, unused.
4. Then `per` strands, each:
   1. start `x = bx + (r - 0.5) · 2 spread`, `y = by + (r - 0.5) · 2 spread`
   2. `ox = (r - 0.5) · 900 tangle`, `oy` likewise, `bias = 2π r`, `lean = tangle (0.3 + 0.9 r)`. All four are drawn at any `tangle`.
   3. Walk up to `len` steps from the start, no draws. Each step reads `angle` at `(x + ox, y + oy)`. When `tangle > 0` the heading bends: `atan2(sin angle + lean sin bias, cos angle + lean cos bias) + (hash(i, sid, 3, s + 55) - 0.5) · 0.55 tangle`, with `i` the step and `sid = 997 b + p` for strand `p` of the bundle. Move `step` along the heading. A point outside the frame by more than `margin` ends the walk unkept; otherwise it is kept.
   4. Fewer than 6 points, start included: the strand is dropped, nothing more is drawn for it.
   5. `hairline = r < hair`. Ink slot `ci`: the last ink for a hairline (no draw), else `floor(r^1.7 · n)` for `n` inks.
   6. `heavy = r^(1 + 4 hier)`. Weight `w = max(0.15, (0.45 if hairline else 1) · wgt · (1 - 0.5 wvar + wvar r) · (1 + 2.4 hier heavy))`.
   7. Three draws for motion (active, phase, direction), unused in a Still.
   8. Beads, only for Beads or Both: one draw `r`; when `r < bead`, for every point index `i` from 2 by `max(2, bgap)` while `i < N - 1` (`N` points): `off = (r - 0.5) · 0.5 msize`, bead ink `floor(n r)`, size `msize (0.3 + 0.55 r)`, one draw unused. The bead sits at point `i` moved `off` along `(-dy, dx) / L`, where `(dx, dy)` runs from point `i` to `i + 1` and `L` is its length (1 when 0).
   9. Symbols: `density = mark` for Symbols or Both, else 0. For every index `i` from 4 by 6 while `i < N`: one draw `r`; when `r ≤ density (0.25 + i / N)`, a symbol at point `i`, heading toward point `min(N - 1, i + 1)`: kind `floor(5 r)` of the kinds in the order above, accent `floor(3 r)`, size `msize (0.6 + 0.8 r)`, one draw unused.
   10. End symbol, only for Symbols or Both: one draw `r`; when `r < 1.4 density`, a symbol at the last point, heading from the point before it: kind, accent and size `msize (0.9 + 0.9 r)` as above, one draw unused.

## Painting

1. Fill the frame with the ground.
2. Every kept strand in trace order: a polyline through its points times `k`, stroked in its ink, width `max(0.25, w k)`, round caps and joins.
3. Every mark in trace order, in its ink (accent for symbols, Palette ink for beads), line width `max(0.5, 0.9 wgt k)`. With centre `(x, y)`, size `S` and heading `a`, all times `k`, and offsets `(dx, dy)` turned by `a`:
   - bead: a filled disc of radius `max(0.4, S / 2)`
   - arrow: a filled triangle `(0.5 S, 0)`, `(-0.2 S, 0.38 S)`, `(-0.2 S, -0.38 S)`
   - ring: a stroked circle of radius `0.34 S`
   - cross: two stroked segments, `(-0.4 S, -0.4 S)` to `(0.4 S, 0.4 S)` and `(-0.4 S, 0.4 S)` to `(0.4 S, -0.4 S)`
   - hook: a stroked arc of radius `0.36 S` from angle `a - 0.4` to `a + 2.4`, clockwise on screen
   - tick: a stroked segment `(0, -0.45 S)` to `(0, 0.45 S)`
     Mark strokes keep the round caps and joins of step 2.
4. The chassis grain, then dither.

## Fidelity notes

- A Still never sets a line dash or a stroke alpha below 1. Both belong to the Travel and Reveal motion modes, so the port needs no dash and no alpha ink.
- The hook is the only partial arc; every other curve is a full circle.
- The PNG export repaints at the export size; the trace is in view box units, so it does not change with the export size.
