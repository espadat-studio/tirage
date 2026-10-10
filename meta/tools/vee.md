# vee

Poster chevrons. The frame is cut into stacked horizontal bands. Each band is filled with straight stripes at an angle, laid out one of four ways: one diagonal field, a chevron of two mirrored halves, four mirrored quarters, or two crossing stripe sets with a third ink where they cross.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range      | step | default | role                                  |
| -------- | ---------- | ---- | ------- | ------------------------------------- |
| `styles` | picker     |      | Auto    | the band layout, or Auto to deal one  |
| `angle`  | 15..75     | 1    | 45      | stripe angle in degrees               |
| `width`  | 0.4..2.2   | 0.05 | 1       | stripe period, `0.055 min(W, H)` at 1 |
| `weight` | 0.15..0.85 | 0.01 | 0.5     | share of a period that is inked       |
| `bands`  | 1..4       | 1    | 2       | how many bands                        |

The picker labels are `Auto`, `Chevron`, `Quad`, `Diagonal` and `Cross`.

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. vee is a Still Tool: only frame 0 is rendered. With motion off the stripe travel and the ink blink are both 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The site has no seed deal of its own: a new seed only reruns the layout.

## Palette

Fixed-role, four inks in swatch order: Base, Ink 1, Ink 2, Cross. Default Palette: `#F4EFE3 #D62828 #003049 #1B1B1B`. The cap is 4. Inks are read modulo the Palette length.

`ink(i)` is slot `1 + (i mod 3)`, so the three stripe inks cycle Ink 1, Ink 2, Cross.

## Space

Everything is in canvas pixels, `W` by `H`. `md = min(W, H)`. The stripe period is `per = 0.055 md width`, and a stripe is `per weight` thick. `a` is `angle` in radians.

## Random numbers

`s` is the Tool seed. One stream: the arithmetic-shift xorshift32 that [aura](aura.md) uses, started at `s 2654435761` as a double product modulo 2^32, 0 becoming 1. No hashes.

## Stream draws, in order

1. Band weights: for each of `n = bands` bands, `w_i = 0.6 + 0.9 r`.
2. Per band, top to bottom:
   - the layout, only when `styles` is Auto: `floor(4 r)` into `chevron, quad, diag, cross`. Otherwise the picked layout, and no draw.
   - `flip`: `+1` when `r < 0.5`, else `-1`.
   - `ph = per r`, the stripe phase.
   - `base = floor(2 r)`.

Band cuts: `c_0 = 0`, `c_{i+1} = round(c_i + w_i / Σw · H)` with half up, then `c_n = H`. Band `b` spans `y0 = c_b` to `y1 = c_{b+1}`.

## Stripe set

A stripe set over a box `x0, y0, x1, y1` at angle `θ`: centre `(cx, cy)` is the box centre, reach `R = hypot(x1 - x0, y1 - y0) / 2 + 2 per`. For every whole `k` from `ceil((-R - ph) / per)` to `floor((R - ph) / per)`, one rect `(-R, k per + ph, 2R, per weight)` in a frame translated to `(cx, cy)` and then rotated by `θ`.

## Regions

Per band, with `A = a flip`, `cA = ink(base)`, `cB = ink(base + 1)`, `cC = ink(base + 2)` and `mx = W / 2`:

- diag: one region `0, y0, W, y1`, one set at `A` in `cA`.
- chevron: `0, y0, mx, y1` at `A`, then `mx, y0, W, y1` at `-A`, both in `cA`.
- quad: with `my = (y0 + y1) / 2`, the four quarters top left, top right, bottom left, bottom right at `A, -A, -A, A`, all in `cA`.
- cross: halves `0..mx` with sign `+1`, then `mx..W` with sign `-1`. Each half holds set 1 at `A sgn` in `cA` and set 2 at `-A sgn` in `cB`, both with phase `ph`, plus a cross in `cC`.

A region box `x0, y0, x1, y1` clips to the rect `x0, y0, x1 - x0 + 1, y1 - y0 + 1`: one pixel past its right and bottom edges. The stripe sets use the box itself.

## Painting

1. Fill the frame with Base.
2. Each region in order, under its anti-aliased rect clip:
   1. each set: each of its rects filled separately in the set's ink, through its translate and rotate.
   2. a cross region then clips again, inside the rect clip, to the union of set 1's rects through set 1's transform, and fills set 2's rects in `cC` through set 2's transform. Where the two stripe sets overlap, the cross ink shows.
3. The chassis grain, then dither.

## Fidelity notes

- Rect fills under a rotation are anti-aliased one by one, so a seam between two stripes that touch is a blended edge, not a hard one.
- The site picks 1:1 as its opening ratio; Reference exports set 9:16.
