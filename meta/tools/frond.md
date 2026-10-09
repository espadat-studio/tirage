# frond

A paste-up of a plant. Soft shadows and a few rounded paper cut-outs sit on a sheet. Torn blocks of ordered 1-bit noise bite in from the edges. Over them a plant is drawn as a broken line of tiny squares and diamonds, then a few thin registration circles and rules go over the top.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id        | range                   | step | default | role                                          |
| --------- | ----------------------- | ---- | ------- | --------------------------------------------- |
| `dirs`    | Potted, Bouquet, Fronds |      | Potted  | what is growing                               |
| `masses`  | 0..6                    | 1    | 3       | how many cut-out masses                       |
| `size`    | 0..1                    | 0.01 | 0.62    | size of the masses, the pot and the leaves    |
| `round`   | 0..1                    | 0.01 | 0.55    | corner radius of masses and pot               |
| `growth`  | 0..1                    | 0.01 | 0.55    | how many leaves and stems, and how long       |
| `detail`  | 0..1                    | 0.01 | 0.5     | head size and ring count, spines, stray marks |
| `coarse`  | 0..1                    | 0.01 | 0.42    | mark size                                     |
| `breakup` | 0..1                    | 0.01 | 0.35    | gaps, jitter, skipped marks, wobble           |
| `noise`   | 0..1                    | 0.01 | 0.45    | how many noise blocks                         |
| `patch`   | 0..1                    | 0.01 | 0.5     | noise block size and grain                    |
| `circles` | 0..1                    | 0.01 | 0.5     | how many registration circles                 |
| `rules`   | 0..1                    | 0.01 | 0.45    | how many rules                                |

Grain and dither are the shared [chassis](chassis.md) post-passes, both off by default. frond does not open with grain on.

Default Palette, 5 inks: `#ede4d3 #ff8c42 #0b3d91 #1a1a1a #a89f8c`.

Inks go by role: 0 paper, 1 mass, 2 ink, 3 grit (the noise blocks), 4 shade. A short Palette does not wrap. A missing ink falls back instead: ink falls back to mass, grit to ink, shade to mass. Inks past the fifth are unused.

The site has no deal: changing the seed only changes the picture. `derive` deals every Parameter (ADR 0003).

## Randomness

All from the Tool seed `s`. Arithmetic wraps at 32 bits.

- `h(x, y, seed)` and value noise `vn` are the chassis ones ([sonar](sonar.md#integer-hash)).
- xorshift32 streams are vein's: start `a`, 0 replaced by 1, each step `a ^= a << 13`, `a ^= a >> 17`, `a ^= a << 5`, logical shifts, draw `a / 2^32`.
- The base is `b = (s + 1) * 2654435761`. Four streams start from it, one per layer, so adding to one layer does not move another:
  - M, masses and shadows: `b ^ 0x9e3779b9`
  - P, the plant and stray marks: `b ^ 0x85ebca6b`
  - N, noise blocks: `b ^ 0xc2b2ae35`
  - G, the registration geometry: `b ^ 0x27d4eb2f`
- `r` below is one draw of the named stream. `seed()` is `floor(r * 9973)`.
- Rounding goes half up, as `Math.round` does.

`U = sqrt(W * H)`. `boil` is 0 on a Still (see Motion).

## Painting

Draw order, all anti-aliased unless stated:

1. Fill the frame with paper.
2. Shadows (M), 3 of them. Draws: `cx = W(0.2 + 0.6r)`, `cy = H(0.2 + 0.6r)`, `R = U(0.18 + 0.26r)`. A radial gradient centred on `(cx, cy)` from radius 0 to `R`, shade at alpha 0.22 fading to shade at alpha 0, fills the square `cx - R, cy - R, 2R, 2R`.
3. Masses (M), in mass. `big = U(0.22 + 0.42 size)`, `round(masses)` of them. Draws, in order: `w = big(0.7 + 0.9r)`, `h = big(0.45 + 0.85r)`, `x = W(0.5 + 0.95(r - 0.5)) - w/2`, `y = H(0.1 + 0.5r) - h/2`. Corner radius `rad = min(w, h) * 0.5 * round`. Then `lobes = 1 + floor(3r)`. Each lobe draws `ox = w(r - 0.5)/2`, `oy = h(r - 0.5)/2`, `lw = w(0.55 + 0.6r)`, `lh = h(0.55 + 0.6r)`, and fills a rounded rect at `(x + ox, y + oy)` sized `lw x lh` with radius `rad`.
4. The pot, Potted only, in mass. `potW = U(0.34 + 0.3 size)`, `potH = U(0.2 + 0.2 size)`, foot centre `(W/2, 0.94H)`. A trapezoid: lip half-width `potW/2` at `y = 0.94H - potH`, foot half-width `0.36 potW` at `y = 0.94H`. The foot's two corners are rounded by `potW * 0.16 * (0.3 + round)` with a quadratic curve whose control is the corner.
5. Noise blocks (N), `round(10 noise)` of them. See below.
6. The plant (P), in ink. See below.
7. Stray marks (P), `round(90 detail)`. Draws `x = rW`, `y = 0.85rH`, size `mark(0.6 + 1.2r)`. Mark `i` is a diamond when `(i + boil)` is odd.
8. Registration (G), in ink. Line width `max(0.6, 0.0013U)`, butt caps.
   - `round(4 circles)` circles. Draws: `R = U(0.24 + 0.3r)`, `cx = W(0.2 + 0.6r)`, `cy = H(0.2 + 0.6r)`. Stroke the full circle.
   - `round(9 rules)` rules. Draws: `y = H(0.05 + 0.9r)`, `x0 = 0.35rW`, `x1 = W(0.65 + 0.4r)`. Stroke `x0..x1` at `y`. Then when `r < 0.35`: side `q = U(0.006 + 0.008r)`, and fill the square of side `q` centred at `(x0 + (x1 - x0)(0.15 + 0.7r), y)`.
9. Grain, then dither, as the chassis does.

### Rounded rect

`rad` is first capped at half of each side. Start at `(x + rad, y)`. Go clockwise: each side is a line, and each corner is an `arcTo` towards the corner and then on along the next side, with radius `rad`. Close and fill nonzero.

`arcTo(p1, p2, r)` from the current point `p0`: when `p0 = p1`, `p1 = p2`, the three are on one line or `r = 0`, it is a line to `p1`. Otherwise the arc of radius `r` touches both lines `p0 p1` and `p1 p2`. Draw a line to where it touches the first, then the arc, the short way, to where it touches the second.

### Noise blocks

Each block draws, in order:

1. `pw = U(0.06 + 0.2 patch)(0.5 + 1.1r)`, `ph = U(0.05 + 0.17 patch)(0.5 + 1.1r)`.
2. `edge = r < 0.5`.
3. On an edge block, `px` is `-0.35pw` when `r < 0.5`, else `W - 0.65pw`, and then `py = r(H - ph)`. Otherwise `px = r(W - pw)` and then `py` is `-0.3ph` when `r < 0.5`, else `H - 0.7ph`.
4. `bs = seed() + boil`.

The block is `x = max(0, round(px))`, `y = max(0, round(py))`, `w = round(min(pw, W - max(0, px)))`, `h = round(min(ph, H - max(0, py)))`. It is skipped when `w < 2` or `h < 2`. The field scale is `c = U(0.01 + 0.03 patch)`.

Each pixel `(i, j)` of the block, with frame pixel `(X, Y) = (x + i, y + j)`:

- `f = 0.62 vn(X / c, Y / c, bs) + 0.38 vn(X / 0.34c, Y / 0.34c, bs + 91)`. The second scale is `c * 0.34`.
- `v = 2.3(f - 0.5) + 0.42 - 0.34 j / h`.
- The threshold is `(B + 0.5) / 16`, with `B` the 4x4 Bayer value at row `j mod 4`, column `i mod 4`: rows `0 8 2 10`, `12 4 14 6`, `3 11 1 9`, `15 7 13 5`.
- Grit when `v` is below the threshold, else paper. The pixel is replaced, not blended. Pixels past the frame are dropped.

## The plant

The mark size is `mark = U(0.0013 + 0.006 coarse)`. From `breakup`: `gap = mark(1.35 + 1.5 breakup)`, `jit = mark(0.5 + 3.2 breakup)`, `skip = 0.42 breakup`, `wob = 0.5 + 0.7 breakup`.

`nLeaf = max(1, round(2 + 9 growth))`. `headN = round(14 + 92 detail)`.

`trace(points, closed, ts)` draws a line of marks along a polyline, with `ts` the given seed plus `boil`.

### Fronds

`nLeaf` leaves from the ground. Each draws: `bx = W(0.12 + 0.76r)`, `by = H(0.86 + 0.16r)`, `ang = -π/2 + 1.5(r - 0.5)`, then the leaf length `U(0.34 + 0.5 growth)(0.6 + 0.7r)`, width `U(0.055 + 0.115 size)`, curl `2.4(r - 0.5)` and leaf seed `seed()`. Trace the outline closed with `seed()`. Then when `r < detail`, trace the spine open with `seed()`.

### Potted and Bouquet

`stems` is `2 + round(5 growth)` for a Bouquet, else 1. For stem `k`:

- Base: x is `W/2`, plus `U * 0.1(r - 0.5)` for a Bouquet only. y is `0.94H - 0.92 potH` when Potted, else `0.86H`.
- `spread` is `1.5(k / max(1, stems - 1) - 0.5)(0.4 + growth)` for a Bouquet, else 0.
- The stem: angle `-π/2 + spread`, length `U(0.24 + 0.28 growth)`, half-width `0.006U(0.6 + size)`, stem seed `seed()`. Trace the left side open with `seed()`, then the right side open with `seed()`.
- `per = max(1, round(nLeaf / stems))` leaves. Leaf `i` sits at `t = 0.12 + 0.7(i + 0.5) / per`, on left-side point `round(22t)`. `side` is 1 for odd `i`, else -1. `drop = 1.75 - t`. Draws: angle `-π/2 + side(0.7 + 0.7r) + spread`, length `U(0.19 + 0.32 growth)(0.55 + 0.7r) drop`, width `U(0.032 + 0.08 size) drop`, curl `side(0.6 + 1.5r)`, leaf seed `seed()`. Trace the outline closed with `seed()`. Then when `r < 0.7 detail`, trace the spine open with `seed()`.
- The head, at the stem tip. `hw = U(0.1 + 0.16 detail)`, `hh = 1.05hw`. `max(6, round(headN / sqrt(stems)))` rings. Each draws `a = 2πr`, `d = sqrt(r)`, centre `(tx + cos(a) d hw, ty + sin(a) d hh - 0.45hh)`. Then radius x `s2 = U(0.008 + 0.02r)(0.5 + detail)`, radius y `s2(0.5 + 0.8r)`, rotation `2πr`, ring seed `seed()`. Trace it closed with `seed()`.

### Shapes

All three walk a heading `a` in `n` steps of `len / n`. Each step stores the point and the normal `(-sin a, cos a)`, turns `a`, then moves by `(cos a, sin a) len / n`.

- Leaf, `n = 26`, from the base at angle `ang`, seed `ls`. Point `i` has `t = i / 26`. The turn after it is `curl / 26 + 0.5 wob (vn(3.3t + 0.11ls, 0.07ls, ls) - 0.5)`. The half-width at `t` is `wid * sin(πt)^0.62 * (1 + 0.5 wob (vn(4.1t + 0.2ls, 3.7, ls + 5) - 0.5))`. The outline is the 27 points pushed out along the normal, then the 27 pushed in, in reverse. The spine is the 27 points.
- Stem, `n = 22`, seed `ss`. The turn after point `i` is `0.3 wob (vn(0.29i + 0.13ss, 2.9, ss) - 0.5)`. Its left and right sides are the 23 points pushed out and in by the half-width. The tip is where the walk ends, one step past the last point.
- Ring, 14 points, seed `rs`. Point `i` is at `θ = 2πi / 14`. Its radius scale is `1 + 0.9 wob (vn(2.1 cos θ + 3.1, 2.1 sin θ + 7.7, rs) - 0.5)`. It sits at `(rx cos θ, ry sin θ)` times that scale, rotated by the rotation about the centre.

### Trace

The counter starts at `k = ts`. Carry `need = 0` from segment to segment. A closed polyline has a segment back to its first point.

For each segment from `p` to `q` of length `L`, skipped below 1e-6: walk `pos` from 0. While `need <= L - pos`:

1. `pos += need`, `k += 1`.
2. When `h(17k, ts, ts + 7) >= skip`, drop a mark at `p + pos * (q - p) / L`, offset by `jit(h(k, ts, 3) - 0.5)` in x and `jit(h(k, ts, 9) - 0.5)` in y. Its size is `mark(0.55 + h(k, ts, 13))`. It is a diamond when `k xor ts` is odd.
3. `need = gap(0.7 + 0.7 h(k, ts, 21))`.

After the segment, `need -= L - pos`.

### Marks

A mark of size `m` at `(x, y)`:

- Square: fill the rect at `(x - 0.78m, y - 0.78m)`, side `1.56m`.
- Diamond: fill the path through `(x, y - m)`, `(x + m, y)`, `(x, y + m)`, `(x - m, y)`.

## Motion

The site has three motion modes: Boil, Grow and Drift. Taste bounds leave motion at its site defaults, so only Boil is ported: amount 0.6, 36 frames at 12 fps.

- Boil re-deals the marks every frame: `boil = 911f` at frame `f`. It is added to every trace seed, to every noise block seed, and to each stray mark's diamond test.
- Nothing else moves. The streams, masses, pot, shapes and registration stay put.
- At `f = 0` boil is 0, so frame 0 is the Still. The amount does not change Boil.
- Grow and Drift are not ported. Grow scales lengths and widths by a cosine pulse, Drift slides everything sideways. Neither is reachable from a Recipe.

## Fidelity notes

- Marks are sub-pixel at 540x960: `mark` runs 0.9..5.2 px. Their edges are anti-aliased, so a port cannot match byte for byte.
- The shapes use `sin`, `cos`, `pow`, `sqrt` and `hypot`. A last-bit difference from V8 moves a mark by far less than a pixel. It could in theory flip a `need <= L - pos` test and shift a trace's counter, but that needs an exact tie.
- arcTo is drawn as one cubic per corner. Skia draws an exact conic. The two differ by under 0.03% of the radius.
- The site's "My colors" fits a set to the Tool's swatch count. frond has 5 swatches, so Reference exports use a 5-ink Palette. recordreel's 6-ink Palette has its sixth ink unused, and the site would drop it too.
- `derive` deals `dirs` evenly from all three, by its own keyed draw. Taste bounds cover only the sliders.
