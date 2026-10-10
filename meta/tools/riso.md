# riso

A risograph collage. The frame is cut into stacked horizontal bands, each printed in two inks: a ground and a pattern of torn paper shapes (blocks, streaks or scratchy static). A big torn dot may sit over one band, hand-drawn pen scribbles cross the sheet, and light and dark specks finish it.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id      | range  | step | default | role                                   |
| ------- | ------ | ---- | ------- | -------------------------------------- |
| `bands` | 2..5   | 1    | 3       | how many bands                         |
| `rough` | 0.1..1 | 0.01 | 0.5     | how torn the edges are                 |
| `scrib` | 0..1   | 0.01 | 0.6     | how many scribbles, `round(3 scrib)`   |
| `dotp`  | 0..1   | 0.01 | 0.5     | chance of the big dot                  |
| `grain` | 0..1   | 0.01 | 0.5     | the Tool's own specks, not the chassis |

`grain` is the Tool's own speck layer. It is a Parameter, but a Seed always deals the site default 0.5, as for every Tool's own grain.

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. riso is a Still Tool: only frame 0 is rendered. With motion off every tear, flicker and scribble-reveal term is 0 and every scribble is drawn whole. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The site deals `bands`, `rough` and `scrib` from the seed on every seed change. That deal is not ported: `derive` deals every Parameter (ADR 0003), and a Reference export sets every slider after the seed.

## Palette

Variable, any number of inks, read by index. Default, 6 inks: `#3a86ff #ff006e #fb5607 #1b1b2f #ffbe0b #f7f3ea`. Every ink index below is taken modulo the Palette length `nc`.

Two inks are Tool constants, not part of the Palette: the scribble ink `#1a1a1a`, and the specks, `rgb(10, 10, 10)` at alpha 0.5 and white at alpha 0.4.

A second ink "against" a base ink `b`, given a draw `u`: start at `floor(u nc)` and walk forward through the Palette, wrapping. Take the first index other than `b` whose relative luminance differs from `b`'s by more than 0.09. If none does, take `(b + 1) mod nc`. Relative luminance is the WCAG one: each channel `v / 255`, linear as `v / 12.92` when `v ≤ 0.03928`, else `((v + 0.055) / 1.055)^2.4`, weighted 0.2126, 0.7152, 0.0722.

## Space

The scene is built in a view box 1000 wide, `VH = 1000 H / W` tall. It is drawn at `k = W / 1000` px per unit on both axes.

## Random numbers

`s` is the Tool seed. One stream: the arithmetic-shift xorshift32 that [aura](aura.md) uses, started at `s 2654435761` as a double product modulo 2^32, 0 becoming 1.

Everything else comes from the four-input integer hash `hash(x, y, z, seed)` that aura also uses. Most shapes hash against `SW`, the last stream draw; the scribble walk hashes against `s` itself.

## Stream draws, in order

1. Band weights: for each of `n = bands` bands, `w_i = 0.6 + 1.2 r`.
2. Band kinds: start from `blocks, streaks, static, blocks, streaks` and shuffle it Fisher-Yates from the last slot down to slot 1, swapping slot `i` with `floor(r (i + 1))`. Four draws. Band `i` gets kind `i`.
3. Per band, top to bottom: ground `bg = floor(r nc)`; pattern ink `fg` = against `bg` with the next draw; then `v = r` and one unused draw. Band `i` spans `y0..y1` with height `w_i / Σw · VH`, stacked from 0.
4. Scribbles, `round(3 scrib)` of them (halves round up). Each takes four draws: start `x = 300 r`, `y = 0.5 VH r`, heading `a = 2π r`, step `20 + 20 r`.
5. The dot: one draw; the dot exists when it is below `dotp`. Then, only for a dot: band index `floor(r n)`, radius `R = min(1000, VH) (0.14 + 0.1 r)`, `cx = 1000 (0.3 + 0.4 r)`, `cy = clamp(y0 + (y1 - y0) r, y0 + 0.3 R, y1 - 0.3 R)` (the upper bound wins when they cross), its ink = against that band's `bg` with the next draw, then one unused draw.
6. `SW = floor(1e9 r)`.

## Torn polygon

A rect `x, y, w, h` with roughness `g` and keys `k1, k2` becomes 20 points. With `j(q, e) = (hash(k1, q, e + 7 k2, SW) - 0.5) g`:

- top, `q = 0..=5`: `(x + w q/5, y + j(q, 1))`
- right, `q = 1..=5`: `(x + w + j(q, 2), y + h q/5)`
- bottom, `q = 4..=0`: `(x + w q/5, y + h + j(q, 3))`
- left, `q = 4..=1`: `(x + j(q, 4), y + h q/5)`

## Band patterns

`g = 16 (0.4 + 1.6 rough)`, band height `bh`, band index `id`. All hashes take `SW`.

blocks: `cols = 5 + floor(3 v)`, `rows = max(1, round(bh / 160))`, cell `cw = 1000 / cols`, `ch = bh / rows`. Each cell `(cx, cy)`, rows outer, is on when `hash(cx, cy, 3 id + 1) < 0.58`. An on cell is a torn polygon at `(cx cw + jx, y0 + cy ch + jy)` sized `cw (0.7 + 0.55 h13)` by `ch (0.75 + 0.5 h14)`, with `jx = (h11 - 0.5) 0.4 cw`, `jy = (h12 - 0.5) 0.3 ch`, `h_m = hash(cx, cy, id + m)`. Roughness `g`, keys `cx + 31 cy` and `id`.

streaks: `cols = 7 + floor(3 v)`, `cw = 1000 / cols`. Column `i` is skipped when `hash(i, 0, 7 id + 2) < 0.16`. Otherwise for `q = 0..=6` at `yy = y0 + bh q / 6`: sway `b = (hash(i, q, 7 id + 3) - 0.5) 1.1 cw (0.4 + rough)` and width `d = cw (0.34 + 0.5 hash(i, q, 7 id + 4))`. The left edge is `(i cw + cw/2 + b - d/2, yy)`, the right `+ d/2`; the polygon is the left edge down, then the right edge back up.

static: `rows = max(3, round(bh / 28))`, `rh = bh / rows`. Row `ry` holds `2 + floor(6 hash(ry, 1, 9 id + 1))` dashes. Dash `d` starts at `x0 = 1000 hash(ry, 2d, 9 id + 2)`, runs `L = 1000 (0.03 + 0.16 hash(ry, 2d + 1, 9 id + 3))`, and is a torn polygon at `(x0, y0 + ry rh + 0.2 rh)` sized `L` by `0.55 rh`, roughness `g / 2`, keys `17 ry + d` and `id`.

## Dot

36 points at `a = 2π q / 36`, radius `R (1 + (hash(q, 1, 77, SW) - 0.5) 0.05 rough)`.

## Scribbles

Scribble `i` walks 70 points. At each step `q`: record the point, turn `a += (hash(i, q, 3, s) - 0.5) 1.4`, then reflect: `a = π - a` when `x < 0` or `x > 1000`, `a = -a` when `y < 0` or `y > VH`, then step by `(cos a, sin a)` times the step length. The walk is then smoothed: the ends stay, every inner point becomes `(p[q-1] + 2 p[q] + p[q+1]) / 4` of the raw walk.

## Specks

`round(700 grain)` specks. Speck `i`: `(1000 hash(i, 1, 91, SW), VH hash(i, 2, 92, SW))`, dark when `hash(i, 3, 93, SW) < 0.55`.

## Painting

All in pixels, `k` per unit. The canvas starts transparent.

1. Each band: clip to the rect `0, y0 k - 0.5, W, (y1 - y0) k + 1`. Under the clip, fill `0, y0 k - 1, W, (y1 - y0) k + 2` in `bg`, then fill each pattern polygon in `fg`, nonzero. Drop the clip.
2. The dot polygon in its ink, unclipped.
3. Each scribble as one polyline in `#1a1a1a`, width `max(1, 3.5 k)`, round cap and join.
4. Each speck a `max(1, 1.6 k)` square at `(x k, y k)`, dark `rgb(10, 10, 10)` at 0.5 or white at 0.4.
5. The chassis grain, then dither.

## Fidelity notes

- Band clips sit on half pixels and are anti-aliased, so a band seam is a blended row and both clip edges overlap the neighbour by a pixel.
- The site picks 1:1 as its opening ratio; Reference exports set 9:16.
