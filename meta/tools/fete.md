# fete

A festival poster. A coarse grid of noise patches in bright inks, blown up with hard pixel edges, under one big white freehand doodle and a scatter of black dots.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range                                            | step | default | role                                    |
| -------- | ------------------------------------------------ | ---- | ------- | --------------------------------------- |
| `motifs` | Auto, Rings, Spiral, Burst, Atom, Wave, Scribble |      | Auto    | which doodle; Auto lets the seed choose |
| `scale`  | 1.2..6                                           | 0.1  | 2.6     | how many noise patches fit across       |
| `res`    | 36..120                                          | 4    | 64      | grid columns across the frame           |
| `lw`     | 0.4..2.2                                         | 0.05 | 1       | doodle line width                       |
| `dots`   | 0..80                                            | 2    | 32      | how many dots                           |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. The Loop turns it on at site defaults, and motion is not a Parameter. See [Motion](#motion). Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

fete is a palette-family Tool. Its Palette is the swatch row, read by position, any number of inks from 2. The site row holds 2 to 7. Default Palette, 4 inks: `#7B2CBF #FF6D00 #00BBF9 #FFD60A`.

The line and dot colours are their own fields on the page, `#FFFFFF` and `#0A0A0A`. My colors only fills the swatch row, so they are Tool constants, not part of the Palette.

## Random numbers

`s` is the Tool seed. Two xorshift32 streams, both with the signed right shift aura uses ([aura](aura.md)): xor in the state shifted left 13, then the state read as signed 32-bit and shifted right 17, then shifted left 5. The draw is the state over 2^32 and a zero start is 1. Unlike aura, the start is not multiplied:

- the doodle stream starts at `s xor 0x51ed270b`
- the dot stream starts at `s xor 0x2545f491`

The patch noise uses aura's four-input integer hash `h(x, y, c, seed)` with `c = 0`. The site hash takes two more inputs for motion; at frame 0 both are 0 and their weights are 0, so they drop out.

## Patches

`gw = res` columns and `gh = max(8, round(gw H / W))` rows, rounding half up. For 9:16 at the default that is 64 x 114.

At cell `(u, v)`, with `k` inks:

1. `x = u / gw * scale`, `y = v / gw * scale`. Rows use `gw` too, so patches stay round.
2. `n` is three octaves of value noise. Octave `i` has weight `0.5, 0.25, 0.125`, frequency `1, 2, 4` and seed `s + 1319 i` (wrapping). The sum is divided by the weight total.
3. Value noise at `(x, y)`: corners `h(xi, yi)`, `h(xi + 1, yi)`, `h(xi, yi + 1)`, `h(xi + 1, yi + 1)` with `xi = floor x`, `yi = floor y`. Lerp along x by `fade(x - xi)`, top row then bottom row, then lerp the two along y by `fade(y - yi)`. `fade(t) = t^2 (3 - 2t)`.
4. The ink is `floor(((n - 0.5) 1.9 + 0.5) k)`, clamped to `0..k-1`.

The grid is drawn over the whole frame with image smoothing off: each cell is a hard-edged block.

## Doodle

A view box 1000 wide and `VH = 1000 H / W` tall. `r` is a draw from the doodle stream, in this order:

1. Motif: when `motifs` is Auto, `["rings", "spiral", "burst", "atom", "wave", "scribble"][floor(6r)]`. A fixed motif takes no draw.
2. Centre `cx = 1000 (0.32 + 0.36r)`, then `cy = VH (0.3 + 0.4r)`.
3. Wobble phase `wph = 7r`.

`ring(n)` samples a curve at `t = i / n` for `i = 0..=n` and, when closed, repeats the `t = 0` point at the end. Jitter moves point `i` by `(4 sin(1.7i + wph), 4 cos(1.3i + wph))`.

The motifs, each a list of polylines:

- rings: `2 + floor(2r)` circles. Each draws radius `1000 (0.16 + 0.18r)`, then `ox = cx + 300 (r - 0.5)`, then `oy = cy + 0.2 VH (r - 0.5)`. 80 steps, closed, jittered.
- spiral: `4 + floor(3r)` arms, then a curl: sign `-1` when `r < 0.5` else `1`, times `1.6 + 1.6r`. Arm `i` starts at angle `a0 = 2πi / arms`; at `t` its radius is `1000 (0.05 + 0.33t)` and angle `a0 + t curl`. 40 steps, open, jittered.
- burst: `9 + floor(8r)` rays. Ray `i` has angle `2πi / rays + 0.2 (r - 0.5)`, then length `1000 (0.18 + 0.34r)`. Each ray is one segment from the centre. No jitter.
- atom: 3 ellipses. Ellipse `i` has rotation `πi / 3 + 0.25 (r - 0.5)`, then `rx = 1000 (0.3 + 0.12r)`, then `ry = rx (0.3 + 0.15r)`. 80 steps, closed, jittered.
- wave: `n = 7 + floor(7r)` segments in one open polyline. Point `i` sits at `x = 1000 * 0.06 + i / n * 1000 * 0.88` and `y = cy ± VH (0.05 + 0.22r) m`, `+` for even `i` and `-` for odd, with `m = 0.2` at both ends and 1 inside. No jitter.
- scribble: `4 + floor(3r)` loops in one open polyline from `(150, cy)`. Each loop draws `nx = clamp(x + 1000 (0.1 + 0.22r), 0, 920)`, then `ny = clamp(cy + 0.5 VH (r - 0.5), 0.08 VH, 0.92 VH)`, then the control point `mx = (x + nx) / 2 + 300 (r - 0.5)`, then `my = (y + ny) / 2 + 0.4 VH (r - 0.5)`. It adds 18 points of the quadratic Bezier from `(x, y)` through `(mx, my)` to `(nx, ny)`, at `t = k / 18` for `k = 1..=18`. No jitter.

## Dots

From the dot stream, `dots` times: draw `r`. Under 0.4 the dot sits on a doodle vertex: the vertices after each polyline's first, in order, and the pick is `floor(r count)`. Otherwise the dot goes to `(1000 (0.04 + 0.92r), VH (0.04 + 0.92r))`, x drawn first.

## Painting

`k = W / 1000`.

1. The patch grid, nearest.
2. Each polyline as its own path, stroked in `#FFFFFF`, width `1000 * 0.009 * lw * k`, round caps and joins.
3. Each dot a filled circle in `#0A0A0A`, radius `1000 * 0.0085 * k`.

## Motion

The site has two motion modes: Draw and Morph. Draw is the default and the only one ported: amount 0.8, 100 frames at 20 fps.

- Draw sketches the doodle on. At frame `f`, `t = f / 100` and the reveal is `clamp(t / 0.82, 0, 1)`. The strokes get a length budget of reveal times the doodle's total length, in view box units.
- The budget is spent along the polylines in order. Each polyline is its own path from its first point. A segment longer than what is left ends part way, at the share of it the budget covers, and the walk stops there. A polyline the budget never reaches is not stroked.
- Dots come in with the line. A dot on a doodle vertex carries the length walked up to that vertex, over `max(1, total)`, counted across all polylines in order. It is drawn once that share is at most the reveal. A free dot has share 0, so it is there from frame 0.
- Nothing else moves. The patch grid, the doodle's shape and the dots' places hold. Draw never feeds `t` into the patch noise.
- The amount does not change Draw.
- The reveal reaches 1 at frame 82, and the whole doodle holds to frame 99. The Loop closes by a cut: the frame after the last is frame 0, the bare grid again. A cut has no closure test.
- At `f = 0` the budget is 0, so frame 0 is the patch grid and the free dots, with no line. It is not the Still and gets a reference export of its own.
- The mid frame is 50, the doodle about 61% drawn. It tests the part-way segment and the dots that wait for the line.
- Morph is not ported. It walks a circle through two extra noise inputs, so the patches shift and the doodle holds. It is not reachable from a Recipe.

## Fidelity notes

- Patch edges are hard but land off the pixel grid: 540 / 64 is 8.4375 px a cell. The nearest upscale decides which column owns each pixel.
- At a Still the site walks a length budget equal to the doodle's total length, so the last segment can stop a rounding error short. That is far below a pixel; the port strokes every segment whole.
