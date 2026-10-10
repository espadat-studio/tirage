# specimen

A type-foundry specimen sheet. A grid of cells on a pale page, some left bare, some merged into larger blocks. Each block is a small study in one of eight motifs: marbled contour bands, wandering tendrils, looped ellipses, flat colour, nested rings, a fan of blades, stacked arcs or scattered spikes. Thin grid rules run over the whole sheet.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id         | range   | step  | default | role                                       |
| ---------- | ------- | ----- | ------- | ------------------------------------------ |
| `cols`     | 2..16   | 1     | 5       | grid columns                               |
| `fill`     | 0.1..1  | 0.01  | 0.26    | chance a free cell starts a block          |
| `span`     | 0..1    | 0.01  | 0.45    | chance a block tries to span several cells |
| `gut`      | 0..0.3  | 0.005 | 0       | gutter inset around each block             |
| `rules`    | 0..1    | 0.01  | 0.14    | opacity of the grid rules, 0 for none      |
| `wMarble`  | 0..100  | 1     | 26      | weight of the marble motif                 |
| `wTendril` | 0..100  | 1     | 16      | weight of the tendril motif                |
| `wLoop`    | 0..100  | 1     | 12      | weight of the loop motif                   |
| `wFlat`    | 0..100  | 1     | 14      | weight of the flat motif                   |
| `wRing`    | 0..100  | 1     | 10      | weight of the ring motif                   |
| `wFan`     | 0..100  | 1     | 10      | weight of the fan motif                    |
| `wArc`     | 0..100  | 1     | 10      | weight of the arc motif                    |
| `wSpike`   | 0..100  | 1     | 8       | weight of the spike motif                  |
| `mScale`   | 0.6..6  | 0.1   | 2.2     | marble noise scale                         |
| `mBands`   | 2..7    | 1     | 4       | marble bands                               |
| `density`  | 0.1..1  | 0.01  | 0.55    | how many strokes or shapes a motif draws   |
| `lineW`    | 0.4..10 | 0.2   | 2       | stroke width, in sheet units               |
| `inkMix`   | 0..1    | 0.01  | 0.35    | chance a block's ground is the ink         |

The square-cells toggle opens on and stays on, so the row count follows the canvas and the `rows` slider is disabled. `rows` is not a Parameter. Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. specimen is a Still Tool: only frame 0 is rendered, where every motion term is 0. Grain and dither are the shared [chassis](chassis.md) post-passes, both off by default.

The seed does not deal the sliders on this page.

specimen is a palette-family Tool. Its Palette is the swatch row, any number of inks from 2, read by index modulo its length. Default Palette, 7 inks: `#E63946 #457B9D #F4A261 #2A9D8F #E9C46A #8338EC #1B1B1B`. The page `#F5F1E8` and the ink `#1B1B1B` are colour pickers outside the swatch row. They stay at their site defaults and are Tool constants, not Palette.

## Sheet

The sheet is 1000 units wide and `1000 H / W` tall; every length below is in sheet units and is drawn scaled by `k = W / 1000`. With `C = cols` columns there are `R = max(1, round(C H / W))` rows, round half up. A cell is `cw = 1000 / C` by `ch = sheet height / R`.

## Random numbers

`s` is the Tool seed.

- The layout stream is xorshift32 with an arithmetic `>> 17` (aura's stream), started at `(s 2654435761 + 7) mod 2^32`, computed in doubles, a zero start becoming 1.
- `h(x, y, z, w, seed)` is terrain's 5-integer hash.
- `fbm(x, y, seed, 3)` is terrain's 3-octave value-noise fBm (octave seeds `seed + 1319 o`). The page's noise is 4D, but at frame 0 its two motion axes sit at 0, where it reduces to the 2D one.

## Layout

Walk cells row by row, left to right, with a taken mask. Skip taken cells. For each free cell:

1. draw `r`. When `r > fill`, mark the cell taken (bare page) and move on.
2. size 1x1. Draw `r`; when `r < span`, draw one key per shape in the list `2x1 1x2 2x2 2x1 1x2 2x2 3x1 1x3 2x3 3x2 3x3 4x1 1x4` (width x height), sort the shapes by key (stable), and take the first that fits inside the grid on free cells.
3. mark the block's cells taken.
4. the motif: draw `r`, then walk marble, tendril, loop, flat, ring, fan, arc, spike, subtracting each weight from `r · total` (total = weight sum, or 1 when it is 0); the first to reach `<= 0` wins, flat when none does.
5. ten more draws in this order: `ci, ci2, ci3, ci4` as `floor(r n)` with `n` inks, `onInk = r < inkMix`, `phase`, `rot = r 2π`, `dir`, `v`, `v2`.

Blocks get ids `0, 1, …` in creation order. `ci3`, `ci4`, `phase` and `dir` only feed motion, but their draws keep the stream in step.

A block's box is its cells inset by `g = gut min(cw, ch) / 2` on every side: `(x cw + g, y ch + g, w cw - 2g, h ch - 2g)`.

## Motifs

`col(i)` is Palette ink `i mod n`. The ground is the ink when `onInk`, else `col(ci)`. The stroke ink, used by tendril, loop, ring, arc and spike, is `col(ci2)` when `onInk`, else the ink. Every motif first fills its box: marble with `col(ci)`, flat with `col(ci)`, the rest with the ground. `m = min(bw, bh)`, `M = max(bw, bh)`, `(cx, cy)` the box centre. Counts are rounded half up.

- flat: the box fill only.
- marble: a field on an `(nx + 1) x (ny + 1)` lattice, `nx = max(8, round(28 sqrt(bw / bh)))`, `ny = max(8, round(28 sqrt(bh / bw)))`. Lattice point `(i, j)` maps to `p = (i / nx)(1.32) - 0.16`, `q = (j / ny)(1.32) - 0.16`, so it overhangs the box by 16% each side. Its value is `fbm(p mScale bw / m + 40 v, q mScale bh / m + 40 v2, s + 97 id, 3)`, stored as a 32-bit float. With `lo`, `hi` the field's extremes, band `k = 1 .. mBands - 1` is filled even-odd in `col(ci + k)` at level `lo + (hi - lo) k / mBands`.
  - Contours: marching squares per lattice cell, corners top-left `a`, top-right `b`, bottom-right `c`, bottom-left `d`, bits 8, 4, 2, 1 for corners above the level. Crossings interpolate linearly along an edge (a zero difference counts as 1e-9). Cases 1/14 join left-bottom, 2/13 bottom-right, 3/12 left-right, 4/11 top-right, 6/9 top-bottom, 7/8 left-top, 5 gives left-top then bottom-right, 10 top-right then left-bottom.
  - Chaining: segments in creation order. Each unused segment starts a chain at its two points and grows from its tail: the first unused segment, in creation order, with an end on the tail (points matched to 4 decimals) adds its other end. A chain stops when no segment matches or it returns to its first point. Chains of more than 3 points become subpaths.
  - Each point maps back to `x = bx + p bw`, `y = by + q bh`, rounded to 1 decimal, then scaled by `k`, and each subpath is closed.
- tendril: `round(8 + 40 density)` strands of `round(30 + 70 density)` steps, step length `1.3 m / steps`. Strand `i` starts at `(bx + h(id, i, 0, 1) bw, by + h(id, i, 0, 2) bh)`. Each step turns to angle `(n - 0.5) 2.6 · 2π` with `n = fbm(7 x / m + 3 i, 7 y / m, s + 13 id + 5, 3)` and moves one step. Stroked at width `0.45 lineW`.
- loop: `round(2 + 6 density)` ellipses about the centre, each a 65-point polyline (angles `2π q / 64`, `q = 0..=64`) of radius `rr = m (0.18 + 0.42 h(id, i, 3, 0))`, squash `0.35 + 0.6 h(id, i, 4, 0)` on its minor axis, turned by `rot + 0.7 i`.
- ring: `round(3 + 12 density)` rotated ellipses about the centre, `rmax = 0.46 m`, ring `i` with radii `rmax q` and `rmax q (0.35 + 0.65 v)`, `q = (i + 1) / n`, rotation `(rot + 0.12 i) mod 2π`. Each is a full ellipse path, stroked, not filled, butt cap.
- fan: `round(3 + 10 density)` blades from pivot `(bx + bw (0.5 + 0.4 (v - 0.5)), by + bh (0.5 + 0.4 (v2 - 0.5)))`, length `0.62 M`, tip width `wd = 0.09 m`, spread `π (0.4 + 0.5 v)`. Blade `i` at angle `rot - spread / 2 + spread i / (n - 1)` (the middle when `n < 2`) is a quad: pivot ± 0.4 of the half-width normal, tip ± the half-width normal, filled in `col(ci2 + i)`.
- arc: `round(3 + 12 density)` half-ellipse polylines about `(cx, by + 1.05 bh)`, `rmax = 1.15 M`, arc `i` of radius `rr = rmax (0.25 + 0.75 (i + 1) / n)`, 41 points at angles `π + π q / 40`, `y` radius `0.9 rr`.
- spike: `round(3 + 14 density)` triangles filled in the stroke ink. Spike `i` has base `(bx + h(id, i, 7, 0) bw, by + h(id, i, 8, 0) bh)`, angle `rot + 2π h(id, i, 9, 0)`, length `m (0.3 + 0.6 h(id, i, 10, 0))` and base width `0.045 m`: base ± the half-width normal, and the tip.

Polylines (tendril, loop, arc) are stroked at `max(0.3, lineW k)` px (tendril `0.45 lineW`), round cap and join. Rings stroke at the same width.

## Painting

1. Fill the frame with the page.
2. Blocks in id order, each clipped to its box: the box fill, then its motif.
3. When `rules > 0`: vertical rules at `x = i cw`, `i = 1 .. C - 1`, and horizontal at `y = j ch`, `j = 1 .. R - 1`, positions rounded to 1 decimal, full length, stroked as one path in the ink at width `max(0.5, k)` px and opacity `rules`, butt cap, unclipped.
4. The chassis grain, then dither.

## Fidelity notes

- The clip is anti-aliased, as Chrome's canvas clip is.
- The site picks 1:1 as its opening ratio; Reference exports set 9:16.
