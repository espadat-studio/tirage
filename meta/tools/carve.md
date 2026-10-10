# carve

A report-cover poster. The frame is cut into rectangular panels, each a flat ink or one of four patterns: stripes, stacked chevrons, a grainy ramp, or a node grid.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range | step | default | role                                   |
| -------- | ----- | ---- | ------- | -------------------------------------- |
| `cuts`   | 1..16 | 1    | 7       | how many times a panel is split        |
| `uneven` | 0..1  | 0.01 | 0.55    | how far a split strays from the middle |
| `gap`    | 0..1  | 0.01 | 0       | gutter between panels                  |
| `mix`    | 0..1  | 0.01 | 0.7     | chance a panel is patterned            |
| `pitch`  | 0..1  | 0.01 | 0.4     | stripe width                           |
| `grain`  | 0..1  | 0.01 | 0.5     | noise in the ramp panels               |
| `nodes`  | 0..1  | 0.01 | 0.5     | grid density                           |

`grain` is the Tool's own grain, not the chassis pass. Like every slider named grain it stays at its site default: a Parameter, never dealt.

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. carve is a Still Tool: only frame 0 is rendered, where every phase and colour roll is 0. Motion stays at site defaults and is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default. `size` is the export width, not a Parameter.

The site has no seed deal: a seed change only moves the layout and the panel treatments.

carve is a variable-palette Tool, any number of inks from 2, read by index. Default Palette, 6 inks: `#1A1A1A #F5F2EC #FF5C00 #00A3FF #FF00A8 #A8FF00`. Ink 0 is also the ground.

## Random numbers

`s` is the Tool seed. Two kinds of xorshift32 stream (the chassis stream, logical right shift, a zero start becoming 1), and the chassis hash `h(x, y, seed)`.

- Layout stream `L`, started at `s * 2654435761 mod 2^32`.
- Panel stream `P_i` for sorted panel `i`, started at `(s + 7919 i) * 2246822519 mod 2^32`.

## Layout

All panels live in 0..1 of the frame. `aspect = H / W`. Start with one panel `(0, 0, 1, 1)`. Repeat `max(1, round(cuts))` times:

1. Order the panels by area, largest first, stable. Draw `L` twice: pick the one at rank `min(count - 1, floor(L L 3))`.
2. `wide` when `w ≥ h aspect`. Draw `L`: below 0.18 flips the choice. A vertical cut when `wide` (after the flip).
3. Draw `L`: split share `t = 0.5 + (L - 0.5) min(0.92, uneven) 0.86`.
4. Remove the panel and append its two halves, left/top first: a vertical cut gives widths `w t` and `w (1 - t)`, a horizontal one heights `h t` and `h (1 - t)`.

Then sort panels by `y`, then `x`.

## Treatments

`n` is the ink count. One grid is allowed per picture. Panel `i`, in sorted order, draws from `P_i`:

1. patterned when `P < mix`. Only then a second draw `roll`: grid when the grid is still free, `roll > 0.86` and area `w h < 0.3` (this uses up the grid); else stripe below 0.34, chevron below 0.6, otherwise ramp. Unpatterned is flat.
2. ink `a = floor(P n)`, ink `b = (a + 1 + floor(P (n - 1))) mod n`
3. `dir`: 1 when `P ≥ 0.5`
4. `brk = 0.3 + 0.45 P`
5. `seed = floor(99999 P)`

## Painting

The frame is filled with ink 0. Gap `g = gap min(W, H) 0.02`. Panel box: `x = px W + g/2`, `y = py H + g/2`, `w = max(1, pw W - g)`, `h = max(1, ph H - g)`. Panels paint in sorted order, with `A` = ink `a` and `B` = ink `b`. Every rect is an anti-aliased fill.

- flat: the box in `A`.
- stripe: the box in `A`. `span` is `h` when `dir`, else `w`. `base = max(3, (0.006 + 0.075 pitch) min(w, h) + 2)`, break `k = brk` of the other side (`w` when `dir`, else `h`). Two passes, pitch `p = base` then `1.9 base`; pass 0 covers the cross side from 0 to `k`, pass 1 from `k` to the end. Each pass walks `i = -1 .. ceil(span / 2p) + 1`, offset `o = 2 p i + (seed mod 7) / 7 p`. Skip while `o + p < 0`; stop once the stripe start passes the box. The stripe is `p` long, cut at the box edge, in `B`.
- chevron: the box in `A`, then, clipped to the box, `rows = max(2, round(h / max(14, 0.55 min(w, h))))` arrows of height `step = h / rows`, filled in `B`. With `a` the near side (`x` when `dir`, else `x + w`), `b` the far side and `t = 0.42`, row `j` at `y0 = y + j step` is the hexagon `(a, y0) (b, y0 + step/2) (a, y0 + step) (a, y0 + (1 - t) step) (b - (b - a)(1 - t), y0 + step/2) (a, y0 + t step)`.
- ramp: an opaque pixel block written straight onto the frame (no blending) at `X = round(x)`, `Y = round(y)`, `round(w)` by `round(h)` (each at least 1), cut at the frame edge. With angle `θ = (seed mod 360) π / 180`, `u = i / W' - 0.5`, `v = j / H' - 0.5`: `t = 0.5 + u cos θ + v sin θ + 0.06 sin(11 (u sin θ - v cos θ) + seed)`, clamped to 0..1. Colour mixes `A` to `B` by `t`. When `grain > 0.002`, every channel gets `(h(i, j, seed) - 0.5) grain 104 (1 - 1.1 |t - 0.5|)`. Channels store rounded half to even, clamped.
- grid: the box in `A`, then, clipped to the box: `step = max(14, min(w, h) / (2 + round(6 nodes)))`. Column lines at `x, x + step, …` while `≤ x + w + 0.5`, rows the same, accumulating the step. One path of every full-height and full-width line, stroked in `B` at `max(1, 0.004 min(w, h))`, butt caps. Then at each crossing `(c, r)` with `h(c, r, seed) > 0.82`, a filled `B` circle of radius `max(1.6, 0.016 min(w, h))`.

Then the chassis grain, then dither.

## Fidelity notes

- Rounds are half up (`Math.round`), except the ramp's stored channels.
- The 14 px floors are in output pixels, so the layout of patterns depends on the export size.
- The site opens at 1:1; Reference exports set 9:16.
