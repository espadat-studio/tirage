# chaff

A scatter of curved blades thrown across a sheet in one ink. The blades are drawn as a soft mask at half size, read back, and thresholded against two sizes of value noise, so their edges come out mottled like ink on rough paper and stray flecks land in the open ground.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults, in page order:

| id       | range               | step | default  | role                                   |
| -------- | ------------------- | ---- | -------- | -------------------------------------- |
| `count`  | 4..400              | 1    | 40       | how many blades                        |
| `size`   | 0..1                | 0.01 | 0.5      | base blade length                      |
| `vary`   | 0..1                | 0.01 | 0.55     | spread of blade lengths                |
| `apart`  | 0..1                | 0.01 | 0        | margin each blade knocks out around it |
| `shapes` | Crescent, Leaf, Bar |      | Crescent | the blade silhouette                   |
| `curve`  | 0..1                | 0.01 | 0.7      | how far a blade bends along its length |
| `slim`   | 0..1                | 0.01 | 0.45     | blade width, share of its length       |
| `taper`  | 0..1                | 0.01 | 0.3      | how sharp the ends are                 |
| `mottle` | 0..1                | 0.01 | 0.62     | strength of the edge noise             |
| `coarse` | 0..1                | 0.01 | 0.4      | size of the edge noise                 |
| `grain`  | 0..1                | 0.01 | 0.3      | per-pixel grey grain                   |

The Tool's own `grain` is a grain pass, not art: it stays at its site default and is not dealt, like the [chassis](chassis.md) passes. It is still a Parameter.

The site has no seed deal: changing the seed leaves every slider where it is.

Motion: Drift, Spin and Swell, off by default. chaff is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off the drift offset and the spin are 0 and the swell scale is 1. Grain and dither are the shared chassis post-passes, off by default.

## Palette

Two roles, read by position: ground, ink. Default `#0E3B43 #F2C14E`. The site row can hold more swatches, but only the first two are painted, so a Palette has at most 2 inks; a one-ink Palette wraps.

## Random numbers

The blade stream is the logical-shift xorshift32 the chassis uses (`x ^= x << 13; x ^= x >> 17; x ^= x << 5`, divided by 2^32), started from `seed · 2246822519` wrapped to 32 bits, 0 becoming 1. `seed` is the Tool seed.

The noise is the chassis hash `h(x, y, s)` and its smoothstep value noise `vn(x, y, s)` on 32-bit wrapping integers.

## Blades

For a `W x H` frame, `aspect = H / W`. Blades live in area units: the frame is `fw = 1 / sqrt(aspect)` wide and `fh = sqrt(aspect)` tall, so one unit is `sqrt(area)` pixels.

`base = 0.06 + 0.38 size`, `n = clamp(round(count), 1, 1200)`. Per blade, six draws in order:

1. `u`, then `k = 1 + (3.2 u² - 0.5) vary`, length `L = base · max(0.12, k)`
2. `x = (1.5 r - 0.25) fw`
3. `y = (1.5 r - 0.25) fh`
4. heading `a = 2π r`
5. `turn = -1` when `r < 0.5`, else `1`
6. `sd = floor(9973 r)`

The blades are then sorted longest first, keeping draw order on a tie, so the big ones sit underneath.

## Outline of one blade

Given the area unit in pixels `U`, a length scale and a slim value:

- length `Lp = L · scale · U`, width `wide = Lp (0.1 + 0.42 slim)`
- sweep `th = 2.3 curve · turn`, start heading `a0 = a - th / 2`
- walk 27 points from the origin: point `i` is where the walk stands before stepping `Lp / 26` along heading `a0 + th · i / 26`
- shift the walk so its mean point lands on `(x U, y U)`
- at each point, `t = i / 26`, heading `h = a0 + th t`, normal `(-sin h, cos h)`
- half width `w = prof(t) · wide / 2 · (1 + 0.34 (vn(2.9 t + 0.11 sd, 0.07 sd, sd) - 0.5))`
- the left edge is the point plus `w` along the normal, the right edge minus

The path runs down the left edge, back up the right edge, and closes.

Profiles, with `tp = taper`:

- crescent: `sin(π t) ^ (0.45 + 1.4 tp)`
- leaf: `min(1, sqrt(t / e)) · (1 - t) ^ (0.55 + 1.5 tp)` with `e = 0.05 + 0.1 tp`
- bar: `min(1, sqrt(t / e), sqrt((1 - t) / e))` with `e = 0.04 + 0.2 tp`

## Painting

1. The mask is a canvas of `MW = max(2, round(W / 2))` by `MH = max(2, round(H / 2))`, with `U = sqrt(MW · MH)`, filled black.
2. When `apart > 0.004`, each blade in turn: fill a fattened copy in black (scale and slim both times `1 + apart / 2`), then the blade itself in green. Otherwise all blades go into one path, filled green once, nonzero.
3. Read the mask back. For every frame pixel `(x, y)`, sample the green channel at `(x / 2, y / 2)` bilinear: the lower cell is the floor, the upper one the next cell, both clamped to the last row or column. `G` is that on 0..1.
4. `fine = sqrt(W H) (0.0022 + 0.007 coarse)`, `big = 7.5 fine`, `amp = 2.6 mottle`. `n = amp (0.66 (vn(x / fine, y / fine, seed + 41) - 0.5) + 0.34 (vn(x / big, y / big, seed + 13) - 0.5))`.
5. The pixel is the ink when `G + n > 0.5`, else the ground.
6. When `52 grain > 0.002`, add `(h(x, y, seed + 71) - 0.5) · 52 grain` to all three channels. Store clamped to 0..255, rounding half to even.
7. The chassis grain, then dither.

## Fidelity notes

- The frame is written per pixel, so only the mask's antialiasing has to match the site's. The mask is soft by design: its ramp is what the noise bites into.
- The site picks 3:4 as its opening ratio; Reference exports set 9:16.
- The PNG export repaints at the export size, so the mask and the noise follow the export, not the preview.
