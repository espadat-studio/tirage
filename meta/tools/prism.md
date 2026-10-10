# prism

An aurora on a black dot grid. A few pixel streams flood in from the edges of the frame. Each cell of a coarse grid takes its colour from how close it sits to the nearest stream spine: white at the core, then hot, then cool rings, then the bare black field. A faint white dot sits dead-centre in every cell on top.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id        | range     | step | default | role                                 |
| --------- | --------- | ---- | ------- | ------------------------------------ |
| `cols`    | 24..72    | 2    | 44      | grid columns                         |
| `streams` | 1..5      | 1    | 3       | how many streams                     |
| `reach`   | 0.3..0.85 | 0.01 | 0.55    | stream length, share of the diagonal |
| `fringe`  | 0.5..1.6  | 0.01 | 1       | stream width                         |
| `dots`    | 0..1      | 0.01 | 0.55    | dot grid opacity                     |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. prism is a Still Tool: only frame 0 is rendered. With motion off the surge, drift and shimmer terms are all 0. Motion stays fixed at site defaults and is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The site deals `streams`, `reach` and `fringe` from the seed on every seed change. That deal is not ported: `derive` deals every Parameter (ADR 0003), and a Reference export sets every slider after the seed.

## Colours

prism takes no Palette. The page keeps a swatch row in its state but never paints from it, and it has no My colors panel. Every colour is a Tool constant:

- field `#08080A`
- dots `#FFFFFF` at alpha `0.8 dots`
- a ramp of 5 rings, each a short list of inks:
  - ring 0: `#FFFFFF`
  - ring 1: `#FFE800 #FF9E00 #FFC400`
  - ring 2: `#FF2D55 #FF00A8 #E600D8 #FF3B00`
  - ring 3: `#2E7CFF #00C8FF #6A2BD9 #0038FF`
  - ring 4: `#3A1ED8 #5A0FB0 #2B0FA8`
- ring thresholds `0.55 0.75 0.95 1.12 1.24`

## Random numbers

`s` is the Tool seed. One xorshift32 stream with the arithmetic right shift (aura's `Xorshift`), started at `s 2654435761 mod 2^32` computed in doubles, a zero start becoming 1.

`h(x, y, z, k)` is the 4-argument integer hash (aura's `hash`) with key `k`.

Draw order:

1. 7 draws shuffling the entry anchors.
2. Per stream, 7 draws: tangent pick, tangent weight, aim jitter, length, bend, width, phase. The phase only feeds motion.
3. One draw for the cell key `K = floor(r 10^9)`.

## Streams

`md = min(W, H)`, `dg = hypot(W, H)`.

Anchors start as the corners then the edge midpoints: `(0,0) (W,0) (0,H) (W,H) (W/2,0) (W/2,H) (0,H/2) (W,H/2)`. A Fisher-Yates shuffle runs `i` from 7 down to 1, swapping `i` with `floor(r (i + 1))`.

Stream `i` of `streams` starts at anchor `A = anchors[i mod 8]`:

1. Edge tangents, in this order: on the top edge (`y <= 0`) `0` when `x < W/2` else `π`; on the bottom edge (`y >= H`) the same; on the left edge (`x <= 0`) `π/2` when `y < H/2` else `-π/2`; on the right edge (`x >= W`) the same. A corner has two, a midpoint one. Pick `floor(r n)` of the `n`.
2. `inward = atan2(H/2 - Ay, W/2 - Ax)`, weight `tw = 0.35 + 0.45 r`.
3. `d = tw (cos tangent, sin tangent) + (1 - tw) (cos inward, sin inward)`, normalised (a zero length counts as 1). `aim = atan2(d) + (r - 0.5) 0.5`.
4. Length `L = dg reach (0.7 + 0.4 r)`.
5. `P0 = A - md 0.15 (cos aim, sin aim)`, `P2 = A + L (cos aim, sin aim)`.
6. Bend `b = (r - 0.5) L 1.1`, `P1 = (P0 + P2)/2 + b (cos(aim + π/2), sin(aim + π/2))`.
7. The spine is 29 points of the quadratic Bezier `P0 P1 P2` at `t = q/28`, `q = 0..=28`.
8. Width `w0 = md (0.16 + 0.08 r) fringe`, then one phase draw.

## Grid

`gw = max(8, cols)`, `gh = max(8, round(gw H / W))`, cell `cw = W/gw`, `ch = H/gh`. Cell `(gx, gy)` has centre `((gx + 0.5) cw, (gy + 0.5) ch)` and covers `round(gx cw)..round((gx + 1) cw)` by `round(gy ch)..round((gy + 1) ch)`.

## Cell colour

1. Distance: over every stream in order and every spine point `q` in order, `d = |centre - p_q| / (w0 (1.15 - 0.45 q / 29))`. Keep the smallest, only replacing on a strict `<`, with its `q` as `bq` and its stream index as `bi`.
2. Jitter `j = (h(gx, gy, 1, K) - 0.5) 0.32 + (h(gx >> 1, gy >> 1, 15, K) - 0.5) 0.42`. `dn = d + j`.
3. Ring: the first ring whose threshold `dn` is below. Past them all: when `dn < 1.7` and `h(gx, gy, 7, K) < 0.07` the ring is `2 + floor(2.4 h(gx, gy, 8, K))`, else the cell stays bare field.
4. Ink in ring list `L` of length `n`: `floor(n h(7 bi + ring, floor(bq / 5), 21, K))`, replaced by `floor(n h(gx, gy, 9 + ring, K))` when `h(gx, gy, 3 + ring, K) < 0.22`.
5. Dim: each channel `c` becomes `round(c - c t)` with `t = 0.06 h(gx, gy, 5, K)`, rounding halves up.

## Painting

1. Fill the frame with the field.
2. Every non-bare cell, row by row, as an opaque pixel-aligned rect.
3. When `dots > 0`: for every cell, a white square of side `rr = max(1, 0.07 min(cw, ch))` centred on the cell centre, at alpha `0.8 dots`, source-over. The squares sit on fractional coordinates, so their edges are antialiased.
4. The chassis grain, then dither.

## Fidelity notes

- The cells are integer rects, so only the dots carry antialiasing.
- The site picks 1:1 as its opening ratio; Reference exports set 9:16.
