# pith

Cut-tissue cells on a paper ground. Each cell is a union of rounded boxes with a lumpy, ragged outline, banded from its edge inward: an outer band, a thin rim, then a mottled core. Branching veins carve the ground back through each cell, and speckle dissolves the bands into one another.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range | step | default | role                                         |
| -------- | ----- | ---- | ------- | -------------------------------------------- |
| `count`  | 1..60 | 1    | 28      | how many cells are dealt                     |
| `size`   | 0..1  | 0.01 | 0.45    | cell size against an even share of the sheet |
| `zoom`   | 1..8  | 0.1  | 1       | magnifies the sheet about its middle         |
| `round`  | 0..1  | 0.01 | 0.55    | box corner radius                            |
| `wobble` | 0..1  | 0.01 | 0.5     | how lumpy and ragged the outlines are        |
| `band`   | 0..1  | 0.01 | 0.45    | width of the outer band and rim              |
| `dither` | 0..1  | 0.01 | 0.55    | speckle across band edges, not chassis       |
| `veins`  | 0..1  | 0.01 | 0.5     | how many veins each cell grows               |
| `thick`  | 0..1  | 0.01 | 0.45    | vein width                                   |
| `grain`  | 0..1  | 0.01 | 0.4     | the Tool's own grain, not chassis            |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. pith is a Still Tool: only frame 0 is rendered, and with motion off boil is 0, grow is 1 and flow is 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default. The `dither` and `grain` sliders above are separate from them.

The live page renders its preview smaller and scales it up while a control moves. Exports always render at full size, so that budget is not ported.

Default Palette, 5 inks: `#fbf8ef #f26ca7 #7b2cbf #2b1b4a #141414`. Inks go by position: ink 0 the ground, ink 1 the outer band, ink 2 the rim, ink 3 the core, ink 4 the grain flecks. A short Palette falls back: the rim to ink 1, the core to ink 2 then ink 1, the flecks to ink 1. Inks past the fifth are unused.

A Seed deals `count`, `size`, `zoom`, `round`, `wobble`, `band`, `veins` and `thick`. `dither` stays at 0.55 and `grain` at 0.4, the site defaults, like the chassis passes.

## Random draws

The layout stream is xorshift32, as in [husk](husk.md), starting at `(s + 3) 2654435761` for Tool seed `s`, wrapping at 32 bits, a zero state becoming 1.

Per cell, in this order: width, height, x, y, the box count, then per box width, height, x, y. Then the vein stream's seed, `trunc(draw (2^32 - 1))`, and the cell's noise seed, `trunc(9973 draw)`.

Each cell's veins come from their own xorshift32 stream at that seed, so the vein count never moves the layout.

All noise is the chassis hash and value noise.

## Cells

With `W x H` the frame:

- Margin `m = 0.085 min(W, H)`, inner frame `IW = W - 2m`, `IH = H - 2m`.
- `n = max(1, round(count))`, `base = sqrt(IW IH / n) (0.28 + 0.95 size)`.
- `z = max(1, zoom)`, about the centre `(W/2, H/2)`.

Each cell:

1. Size `w = base (0.75 + 1.40 r)`, `h = base (0.75 + 1.25 r)`. Corner `x0 = m + r max(1, IW - w)`, `y0 = m + r max(1, IH - h)`.
2. `3 + floor(2 r)` boxes. The first box is `w (0.62 + 0.32 r)` by `h (0.58 + 0.34 r)`, the rest `w (0.34 + 0.44 r)` by `h (0.34 + 0.46 r)`. Each sits at `x0 + r (w - bw)`, `y0 + r (h - bh)`. Its corner radius is `0.5 min(bw, bh) round`.
3. Draw the two seeds.
4. Scale every box centre away from the frame centre by `z`, and its half sizes and radius by `z`.
5. Skip the cell when its box bounds lie wholly more than `base z` outside the frame.
6. Grow `round(6 veins)` veins. The step is `0.095 z min(w, h)`. Each vein, from its own stream: pick a box `floor(r count)`, start at its centre plus `(r - 0.5) 1.1` times each half size, heading `2π r`. Walk `4 + floor(5 r)` steps at depth 0.

A walk of `k` steps at depth `d`, each step: add the segment one step along the heading, move to its end, turn by `0.9 (r - 0.5)`. Then when `d < 2` and `r < 0.42`, branch: a sub-walk from here at heading plus `±(0.6 + 0.6 r)`, the sign minus when `r < 0.5`, of `max(1, floor(0.6 k))` steps at depth `d + 1`. The sign draw comes before the angle draw.

## Fields

`cs = base z`. Two 32-bit float fields over the frame start at 1e9.

- Wobble: lump size `ns = cs (0.10 + 0.18 wobble)`, amplitude `A = 0.12 cs wobble`, pad `ceil(2.2 A + 0.04 cs)`, reach `1.5 A + 2`.
- Vein half width `vt = cs (0.0067 + 0.033 thick)`.

Per cell, every pixel of its box bounds grown by the pad, clipped to the frame:

- `d` is the least signed distance to its rounded boxes. A rounded box with centre `c`, half sizes `hw, hh` and radius `r`, clamped to both half sizes: `q = |p - c| - (hw - r, hh - r)`, then `d = |max(q, 0)| + min(max(qx, qy), 0) - r`.
- When `d` is past the reach, it goes in as is. Otherwise add `2 A (noise(x / ns, y / ns, seed) - 0.5) + 0.8 A (noise(x / 0.28 ns, y / 0.28 ns, seed + 31) - 0.5)`.
- The cell field keeps the least value.

Per vein segment, every pixel of its bounds grown by `vt + 1`: distance to the segment minus `vt`, and the vein field keeps the least value.

## Painting

- `E = cs (0.033 + 0.19 band)` the outer band, `R2 = cs (0.020 + 0.070 band)` the rim, `nb = 0.31 cs`.
- Grid `gsc = max(1, sqrt(W H) / 620)`. Speckle `dth = E (0.12 + 1.5 dither)`.

Each pixel starts as ground. When the cell field `d < 0`:

1. `t = -d + (hash(floor(x / gsc), floor(y / gsc), s) - 0.5) dth + 1.4 dth (noise(x / 6.5 gsc, y / 6.5 gsc, s + 17) - 0.5)`.
2. Band width `Ev = E (0.40 + 1.35 noise(x / nb, y / nb, s + 101))`.
3. A pixel with the vein field below 0, or `t < 0`, stays ground. `t < Ev` is band, `t < Ev + R2` is rim. Otherwise core, each channel plus `46 grain (noise(x / 2.6 gsc, y / 2.6 gsc, s + 53) - 0.5)`.

Then, when `grain > 0`, `h = hash(floor(x / gsc), floor(y / gsc), s + 7919)`. Above `1 - 0.016 grain` the pixel is averaged with the fleck ink. Below `0.010 grain` it is scaled by 0.55.

Channels are clamped to 0..255 and stored rounding half to even. Alpha is 255.

## Fidelity notes

- Both fields store 32-bit floats; the comparison is against the stored value.
- `sqrt`, `sin` and `cos` in the walk compound over a vein, so a last-bit difference can shift a deep branch by a fraction of a pixel.
