# husk

Scattered seed husks on a plain ground. Each husk is a lumpy oval with a dark rim. Inside the rim a light fill is bitten away, either into ragged islands or into a dot screen that thins toward the edge. Where husks overlap, their dark parts merge but their fills stay apart.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id      | range         | step | default | role                                        |
| ------- | ------------- | ---- | ------- | ------------------------------------------- |
| `count` | 1..70         | 1    | 30      | how many husks                              |
| `size`  | 0..1          | 0.01 | 0.62    | husk radius                                 |
| `vary`  | 0..1          | 0.01 | 0.5     | how much radii differ between husks         |
| `lump`  | 0..1          | 0.01 | 0.42    | how far the outline wobbles off the oval    |
| `bites` | Crumble, Dots |      | Crumble | how the fill is bitten                      |
| `eat`   | 0..1          | 0.01 | 0.55    | how much of the fill is bitten              |
| `tex`   | 0..1          | 0.01 | 0.45    | bite scale: crumble patch size, dot pitch   |
| `grain` | 0..1          | 0.01 | 0.3     | the Tool's own per-pixel noise, not chassis |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. husk is a Still Tool: only frame 0 is rendered, so its motion modes are never used. Grain and dither are the shared [chassis](chassis.md) post-passes, also off by default. The `grain` slider above is separate from the chassis grain. Default Palette: `#e0c3fc #1b1b1e #f9f871`.

A Seed deals `count`, `size`, `vary`, `lump`, `eat` and `tex`. `bites` stays at Crumble and `grain` at 0.3, the site defaults: Dots is a different look, and the `grain` slider is grain, which stays at site defaults like the chassis passes.

Inks go by position: ink 0 is the ground, ink 1 the silhouette, ink 2 the fill. A 2-ink Palette uses ink 1 for the fill too. Inks past the third are unused.

## Random draws

One xorshift32 stream drives the husk layout:

- The state starts at the Tool seed times `2654435761`, wrapping at 32 bits. A zero state becomes 1.
- Each draw: xor in the state shifted left 13, then shifted right 17 (logical), then shifted left 5, all on 32 bits. The draw is the new state divided by 2^32.

Per husk, in this order, 13 draws: centre x, centre y, radius, x stretch, y stretch, rotation, wobble phase, three harmonic phases `a1 a2 a3`, then three harmonic picks `h1 h2 h3`. The wobble phase only feeds motion.

All other randomness is the chassis integer hash and value noise.

## Depth field

The husks are drawn into a small field of depths, not at frame size, and the frame reads it back scaled.

- `aspect = H / W`. When the frame is taller than wide, the field is `round(560 / aspect)` wide, at least 8. Otherwise it is 560 wide. Its height is `round(width * aspect)`, at least 8.
- Depths are stored as 32-bit floats, all starting at 0.
- `base = (0.045 + 0.115 * size) * sqrt(field width * field height)`, so a wide frame is not a denser one.
- `lump' = 0.55 * lump`.
- The husk count is `max(1, round(count))`.

Each husk, from its draws `r`:

- Centre: `(-0.12 + 1.24 r) * field width`, and the same with the field height, so some husks hang off the edge.
- Radius `R = base * (1 + (r - 0.5) * 2 * vary * 0.7)`.
- Stretches `ex, ey = 0.78 + 0.5 r`.
- Rotation and the three phases are `r * π * 2`.
- Harmonics: `h1 = 2 + floor(2 r)`, `h2 = 4 + floor(3 r)`, `h3 = 7 + floor(3 r)`.

It only visits field cells within `R * (1 + lump') * max(ex, ey) + 2` of its centre on each axis, clipped to the field, and skips itself when that box is empty. Each cell `(i, j)`:

1. Offset from the centre, rotated into the husk's frame by its rotation, then divided by `ex` and `ey`.
2. `dist` is the length of that offset. Skip when it is past `R * (1 + lump')`.
3. `ang` is its angle by `atan2`. The wobble is `0.5 sin(h1 ang + a1) + 0.33 sin(h2 ang + a2) + 0.2 sin(h3 ang + a3)`.
4. The edge is `R * (1 + lump' * wobble)`. Skip when the edge is at most 0.001 or `dist` reaches it.
5. The depth `1 - dist / edge` replaces the stored one when it is larger, so the deepest husk wins. The comparison is against the stored 32-bit value, and the store rounds to 32 bits.

## Painting

Every frame pixel is written directly, row by row. No shape is rasterised.

- The depth at `(x, y)` is a bilinear read of the field at `(x * field width / W, y * field height / H)`. The lower cell is the truncated coordinate, clamped to the last cell, and the fraction is taken after the clamp. The upper cell is one more, also clamped.
- At depth 0.001 or less the pixel is ground.
- Otherwise it is fill when the bite keeps it, else silhouette.

Crumble keeps a pixel when `depth > 0.02 + eat * (0.06 + 1.5 n)`:

- `n` is fbm of the pixel coordinates times `1 / (2 + 18 tex)`, at the Tool seed + 13. This fbm has 3 octaves, starts at weight 0.5, and each octave is 2.07x the frequency and 0.55x the weight. Octave `i` adds `131 i` to the seed. The sum is divided by the total weight.
- `n` is then pushed to its extremes by smoothstep, `n² (3 - 2n)`, so bites come in patches.

Dots keeps a pixel when it is inside its screen dot:

- The dot pitch is `max(3, (0.004 + 0.03 tex) * min(W, H))` px. Dots sit at the centre of each pitch square from the top left.
- `rad` is the distance from the pixel to its dot centre, in pitch units.
- `v` is `(depth - 0.05) / 0.95`, clamped to 0..1. The pixel is kept when `rad < 0.52 sqrt(v) (1.35 - 0.85 eat)`.

Then the Tool's grain: when `40 * grain` is above 0.002, each channel gets `(hash(x, y, Tool seed + 71) - 0.5) * 40 * grain` added. The channels are stored as bytes, clamped and rounded half to even. Alpha is 255.

## Fidelity notes

- The field uses `sin`, `cos`, `atan2` and `sqrt`. Rust's and V8's results may differ in the last bit, which can flip a rare cell on a husk edge or a bite boundary.
- The 32-bit field store matters: comparing against 64-bit depths would let a later husk win a cell it ties.
