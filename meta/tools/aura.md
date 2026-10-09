# aura

A soft glow of melting inks. Each ink owns a warped noise field. Every pixel mixes all inks, weighted by how strong each field is there, so the inks bleed into tints with no hard edge. The field is painted on a small buffer and scaled up smooth.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range                     | step | default | role                                     |
| -------- | ------------------------- | ---- | ------- | ---------------------------------------- |
| `styles` | Auto, Clouds, Mesh, Sweep |      | Clouds  | what shapes each ink's field             |
| `scale`  | 0.5..2                    | 0.05 | 2       | how much field fits across the frame     |
| `churn`  | 0..1                      | 0.01 | 1       | how hard a second field warps the first  |
| `punch`  | 0..1                      | 0.01 | 1       | how sharply the strongest ink takes over |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. aura is a Still Tool: one frame, motion stays off and is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default. Default Palette: `#ff8a5b #ffc15e #f4a7d6 #8e7cff`.

The site has 4 ink swatches, and every swatch takes part. Inks go by position, wrapping when the Palette is short. Inks past the fourth are unused, since the site's "My colors" fills its 4 swatches from the first 4 of a longer set.

## Random draws

One sequential generator, xorshift on 32 bits, all values unsigned unless said:

- Its start is the Tool seed times `2654435761` as a 64-bit float product, then wrapped to 32 bits. The product passes 2^53 for big seeds and loses low bits; the port must do the same float multiply. A start of 0 becomes 1.
- Each draw: `a ^= a << 13`, then `a ^= a >> 17` where this shift is arithmetic on `a` read as signed 32-bit, then `a ^= a << 5`. The draw is `a / 2^32`, in [0, 1).

Draws happen in this order:

1. Only when the style is Auto: one draw picks Clouds, Mesh or Sweep, as `floor(draw * 3)` into that list.
2. For each ink `i`, in Palette order, 5 draws: an offset `ox = draw * 37`, an offset `oy = draw * 43`, a centre `ax = (draw - 0.5) * 1.5`, a centre `ay = (draw - 0.5) * 1.5`, and an angle `θ = draw * 2π` giving a direction `(cos θ, sin θ)`. All 5 are drawn whatever the style.

So Auto shifts every ink's draws by one compared with the same style picked by hand.

## Noise

A stateless hash of a lattice cell `(x, y)`, a channel `k` and a noise seed `s`, all 32-bit with wrapping multiplies:

- `n = x·374761393 xor y·668265263 xor k·1440662683 xor s·1013904223`
- `n = (n xor n >> 15) · 2246822519`
- `n = (n xor n >> 13) · 3266489917`
- `n = n xor n >> 16`, read as unsigned and divided by 2^32.

Shifts here are logical. The noise seed is the Tool seed xor `0x51ed270b`.

Value noise at `(x, y)` on channel `k`: hash the four lattice corners `a` (floor), `b` (+x), `c` (+y), `d` (+x +y). With smoothstep weights `u`, `v` of the fractional parts, the value is `a + (b - a)·u + (c - a)·v + (a - b - c + d)·u·v`, summed left to right.

Two sums of octaves, each octave on its own channel:

- `fbm(x, y, k)`: `0.55·vn(x, y, k) + 0.28·vn(2.13x, 2.13y, k + 11) + 0.17·vn(4.31x, 4.31y, k + 23)`
- `fbm2(x, y, k)`: `0.62·vn(x, y, k) + 0.38·vn(2.13x, 2.13y, k + 11)`

## Buffer

The field is painted on a buffer of its own, then drawn over the whole frame:

- The buffer width is `max(90, min(700, round(W)))`. The height is `max(90, round(bw · H / W))`. 700 is the PNG export's budget. The live preview uses smaller budgets, which the port ignores.
- At a frame up to 700 px wide the buffer is the frame size and the draw is a plain copy.
- Wider, the buffer is scaled up to the frame with image smoothing on at "high" quality. Chrome samples that upscale with a bicubic filter.

## Field

- `exponent = 1.3 + 5.2·punch` and `warp = 0.4 + 2.4·churn`.
- `short = min(bw, bh)` and `zoom = 1.7·scale`.
- A buffer pixel `(x, y)` sits at `nx = (x - bw/2) / short · zoom`, `ny = (y - bh/2) / short · zoom`. Pixels are sampled at their top-left corner, not their centre.
- Warp: `q1 = fbm(nx + 11.3, ny + 7.9, 81)` and `q2 = fbm(nx + 3.7, ny + 19.1, 82)`. The warped point is `wx = nx + warp·(q1 - 0.5)`, `wy = ny + warp·(q2 - 0.5)`.
- Each ink `i` reads `nz = fbm2(1.15·wx + ox, 1.15·wy + oy, 60 + i)` and turns it into a strength `f` by style:
  - Clouds: `f = nz`
  - Mesh: a glow round the ink's centre, `f = 0.72·exp(-2.6·(dx² + dy²)) + 0.34·nz` with `dx = 0.8·wx - ax`, `dy = 0.8·wy - ay`
  - Sweep: a ramp along the ink's direction, `f = 0.66·clamp(0.5 + 0.5·(wx·cos θ + wy·sin θ), 0, 1) + 0.4·nz`
- The ink's weight is `max(f, 0.002) ^ exponent`.

## Mixing

Inks mix in squared space, so blends stay bright instead of muddy:

- Each ink channel `c` (0..255) is squared.
- A pixel's channel is the weighted mean of the squared inks, `Σ w·c² / Σ w`, summed in ink order, then square-rooted.
- It is stored as a byte, clamped and rounded half to even. Alpha is opaque.

## Fidelity notes

- At the 540x960 fixture size the buffer is the frame, so the fixtures check the field and the mix, not the upscale.
- A one-off 1080x1920 export, upscaled from a 700x1244 buffer, was within 1 level per channel of a Mitchell bicubic (B = C = 1/3), as were Catmull-Rom and bilinear. The field is too smooth for the filter to matter. Mitchell had the lowest mean error, and it is what tiny-skia's bicubic does.
- Mesh and Sweep call `exp`, `cos` and `sin`, and every style calls `pow`. Rust's and V8's results may differ in the last bit, which can flip a rare byte by one.
- The site's "My colors" truncates a longer set to its 4 swatches. Reference exports therefore use a 4-ink Palette, so the site's swatches equal the Recipe's Palette.
