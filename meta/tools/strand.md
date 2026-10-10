# strand

Chains of short rods printed in two plates that miss each other. Each chain walks across the sheet, turning a little at every step and now and then splitting. The rods along it are torn, lopsided outlines with a notch at every joint. A plate ink shows as a thick rim where the fill slid off it, and the fill breaks up into specks, drags or a dot screen towards its rim.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id         | range                 | step | default | role                                      |
| ---------- | --------------------- | ---- | ------- | ----------------------------------------- |
| `count`    | 1..40                 | 1    | 8       | chains                                    |
| `len`      | 3..90                 | 1    | 22      | rods per chain                            |
| `wander`   | 0..1                  | 0.01 | 0.6     | how far a chain turns per rod             |
| `branch`   | 0..1                  | 0.01 | 0.22    | chance a chain splits                     |
| `thick`    | 0..1                  | 0.01 | 1       | rod width                                 |
| `rod`      | 0..1                  | 0.01 | 0.5     | rod length, in rod widths                 |
| `notch`    | 0..1                  | 0.01 | 0.18    | gap between rods                          |
| `rough`    | 0..1                  | 0.01 | 0.55    | how torn and lopsided a rod is            |
| `offset`   | 0..1                  | 0.01 | 0.5     | how far the plate slid                    |
| `edge`     | 0..1                  | 0.01 | 0.35    | how much fatter the plate is              |
| `tex`      | 0..1                  | 0.01 | 0.4     | how much fill fails to take               |
| `texKinds` | Stipple, Drag, Screen |      | Stipple | the shape of the gaps in the fill         |
| `grain`    | 0..1                  | 0.01 | 0.42    | the Tool's own edge noise and pixel grain |

`grain` is the Tool's own grain, not the chassis pass. Like every slider named grain it stays at its site default: a Parameter, never dealt.

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. strand is a Still Tool: only frame 0 is rendered, where the pulse is 1, the drift is 0 and the plate slides at its fixed angle. Motion stays at site defaults and is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default. `sizePx` is the export width, not a Parameter.

The site has no seed deal: a seed change only moves the chains.

## Palette

Fixed roles, 3 inks: Ground, Plate, Fill. Default `#FFD166 #1B1B1B #EF476F`. The site row keeps at least 2 and reads only the first 3, so a Palette may have at most 3 inks. With 2 inks the fill is the plate ink, as on the site, not ink 0.

## Random numbers

`s` is the Tool seed. One xorshift32 stream `R` (the chassis stream, logical right shift, a zero start becoming 1), started at `s · 2654435761 mod 2^32`. The chassis hash `h(x, y, seed)` and its value noise `vn(x, y, seed)` give every other number.

## Chains

The frame is measured in area units: `aspect = H / W`, frame width `fw = 1 / √aspect`, height `fh = √aspect`, so one unit is `U = √(W H)` pixels.

Rod width `wid = 0.02 + 0.09 thick`, length `seg = wid (0.4 + 2.2 rod)`, stride `step = seg + wid + 0.95 notch wid`. The stride clears a whole width past the segment, so the rounded ends never close the notch.

Seed the walk with `count` chains, in order, each taking three draws: `x = (-0.12 + 1.24 R) fw`, `y = (-0.12 + 1.24 R) fh`, heading `a = 2π R`. Each has `left = len` rods to lay and depth 0. They go on a stack.

While the stack is not empty and fewer than 7000 rods are laid, pop the last chain pushed and walk it. Each step, while `left` (counted down before the checks below) was above 0 and the cap is not reached:

1. Lay a rod `(x, y, a, seg, sd)` with `sd = floor(9973 R)`.
2. Move `x += step cos a`, `y += step sin a`. Turn `a += (R - 0.5) · 0.9 wander`.
3. Only when depth is below 3 and `left > 4`, draw `R`. Below `0.22 branch`, push a new chain at the current point: heading `a ± (0.5 + 0.6 R)` (minus when a first `R < 0.5`), `left = max(3, floor(left (0.4 + 0.5 R)))`, depth + 1.
4. Outside `-0.32..1.32` of the frame on either axis, steer home: `da` is the angle from the chain to the frame centre less `a`, wrapped into `-π..π`, and `a += 0.2 da`.

## A rod

A rod at `(x, y)` with heading `a`, length `seg` and width `wid` in pixels, roughness `r` and growth `g`:

- Heading nudge: `a += 0.42 r (h(sd, 17, 3) - 0.5)`. With `(c, s) = (cos a, sin a)` the normal is `(-s, c)`.
- `full = seg + wid`, half width `half = max(0.05 wid, wid / 2 + g)`.
- End openings `e0 = 0.06 + 0.34 r h(sd, 3, 7)` and `e1 = 0.06 + 0.6 r h(sd, 9, 7)`.
- 19 stations `u = i / 18`, `i = 0..18`:
  - `cap = max(0, min(u / e0, (1 - u) / e1, 1))^0.45`
  - left edge swell `wl = 1 + 1.3 r (vn(6.1 u + 0.11 sd, 0.07 sd, sd) - 0.5)`
  - right edge swell `wr = 1 + 1.3 r (vn(5.3 u + 0.17 sd, 0.05 sd, sd + 911) - 0.5)`
  - axis drift `off = 0.55 r wid (vn(2.2 u + 0.09 sd, 7.3, sd + 41) - 0.5)`
  - axis point: `t = u full - wid / 2` along the heading from `(x, y)`, then `off` along the normal
  - left point at `half cap wl` along the normal, right point at `half cap wr` against it
- The outline runs down the left points, back up the right points, and closes.

## The mask

The rods are drawn first into a black mask at `MW = max(2, round(W / 2))` by `MH = max(2, round(H / 2))`, with `MU = √(MW MH)`. Half size is deliberate: every edge gets a ramp a few pixels wide, which the edge noise later bites into.

At frame 0: rod width `wide = wid U`, plate slide `off = 0.055 offset U` at angle `3π/4`, plate width `fat = wide (1 + 0.55 edge)`. A rod at area point `(x, y)` with a shift `(ax, ay)` in frame pixels lands at `(x MU + ax / 2, y MU + ay / 2)`. Rods are drawn with `seg MU` and `wide / 2`, at roughness `rough`.

1. Plate: every rod shifted by the slide, growth `(fat - wide) / 4`, as one non-zero path in `#f00`.
2. Fill: every rod unshifted, growth 0, as one path in `#0f0` over it. Where the fill lands, red is painted over.
3. When `tex > 0.004`: depth. Erosion `eat = 0.095 wide` (half the rod's half width at mask scale, times 0.38). Every rod unshifted with growth `-eat`, as one path in `#00f`, blurred with standard deviation `eat · 0.8` (two decimals), and added (`lighter`). Blue is empty before, so the blue channel becomes how deep inside a rod a pixel sits.

## Painting

Every frame pixel `(x, y)` reads the mask bilinear at `(x / 2, y / 2)`: the lower sample index is the floor, clamped to the last row or column; the upper is one more, clamped on rows but wrapping to column 0 on columns. `R`, `G` and `B` are the mask channels over 255.

- edge noise `n = (vn(x / cell, y / cell, s + 29) - 0.5)(0.8 grain + 0.06)`, with `cell = max(1.4, 0.0026 U)`
- the fill takes the pixel when `G > 0.5 + n`
- with `tex > 0.004`, a fill pixel can drop out. `miss = tex (0.12 + 0.88 (1 - B))`. With `θ = 1.75` and the frame turned by it, `u = x cos θ + y sin θ`, `v = y cos θ - x sin θ`:
  - Stipple: a gap when `h(floor(x / 1.7 cell), floor(y / 1.7 cell), s + 13) < miss`
  - Drag: a gap when `h(floor(u / 13 cell), floor(v / 1.5 cell), s + 17) < miss`
  - Screen: dot pitch `4.4 cell`. `fu` and `fv` are the fractional parts of `u` and `v` over the pitch, less 0.5. A gap when `fu² + fv² > 0.72² (1 - miss)` (no dot when `miss ≥ 1`)
- the colour is Fill where the fill takes it, else Plate when `R > 0.5 + n`, else Ground
- pixel grain: when `46 grain > 0.002`, every channel gets `46 grain (h(x, y, s + 71) - 0.5)`
- channels store rounded half to even, clamped, alpha opaque. The block replaces the frame.

Then the chassis grain, then dither.

## Fidelity notes

- The blur is the canvas `blur()` filter, which Chrome's software canvas approximates with three box blurs. Ours uses the same three-box approximation, on a layer the size of the mask, so rods off the mask edge do not blur in.
- The upper column sample wraps to column 0 on the last frame column, a site quirk kept as is.
- The site opens at 3:4; Reference exports set 9:16.
