# zig

Two-colour zigzag stripes. The frame is filled with one ink, then every other band between a stack of zigzag, staircase or wavy boundary lines is filled with a second ink. Each band is a polygon whose corners are rounded off.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id      | range    | step | default | role                                   |
| ------- | -------- | ---- | ------- | -------------------------------------- |
| `width` | 0.5..2   | 0.05 | 1       | stripe width                           |
| `depth` | 0.2..1.4 | 0.02 | 0.9     | how far a tooth reaches into its band  |
| `tooth` | 0.5..2   | 0.05 | 1       | tooth length along the stripe          |
| `round` | 0..1     | 0.01 | 0.8     | how much of each corner is rounded off |

The `styles` picker offers Auto, Teeth, Chevron, Stairs, Ricrac, Waves and Scales, default Auto. Auto picks one of the six others from the Tool seed: index `floor(hash(1, 2, 3, seed) * 6)` in the order Teeth, Chevron, Stairs, Ricrac, Waves, Scales. A Recipe names the style it draws, so the Choice is the six styles without Auto. The site's Auto is the same as naming the style it picks.

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. zig is a Still Tool: only frame 0 is rendered. With motion off the crawl shift is 0 and the colour pair is dealt at step 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

zig is a palette-family Tool. Its Palette is the swatch row only, read by position, any number of inks from 2. It has no ground, rule or ink fields of its own and no family picker. Default Palette, 6 inks: `#1b1b1b #f94144 #f8961e #f9c74f #43aa8b #577590`.

A Seed deals `styles`, `width`, `depth`, `tooth` and `round`.

## Hash

An integer hash of `(x, y, z, seed)`, all wrapping 32-bit, the seed read as signed:

1. `n = x 374761393 ^ y 668265263 ^ z 1440662683 ^ seed 1013904223`.
2. `n = (n ^ (n >> 15)) 2246822519`, then `n = (n ^ (n >> 13)) 3266489917`, then `n ^= n >> 16`, with logical shifts.
3. The draw is `n / 2^32`.

The page also seeds a xorshift generator for the stripes, but never draws from it. Nothing else is random.

## Inks

With `n` inks in the Palette, `a = floor(hash(0, 7, 1, seed) * n)` and `b = floor(hash(0, 7, 2, seed) * n)`. When `b == a` and `n > 1`, `b = (a + 1) mod n`. Ink `a` is the ground, ink `b` the stripes. The other inks are unused in a Still.

## Frame

Geometry lives in a box 1000 wide and `V = 1000 H / W` tall, 1777.78 at 9:16. Every point is scaled by `k = W / 1000` when drawn.

## Bands

Each style builds a list of boundary lines, each a list of points. Band `j` is the polygon made of line `j` followed by line `j + 1` reversed. Bands with even `j`, counted from 0, are filled; odd bands show the ground.

### Teeth and Chevron

Teeth run in columns down the frame; Chevron is the same pattern with x and y swapped, so it runs in rows.

- Along the stripes: `span` is 1000 for Teeth, `V` for Chevron. Across: `cross` is the other one.
- Stripe width `cw = 0.125 span width`. Tooth reach `d = cw depth / 2`. Half-tooth `h = 0.055 span tooth`, period `p = 2h`.
- Lines `j = -1 .. ceil(span / cw) + 2`. Line `j` sits at `j cw` and is offset along the stripe by `h` when `j` is odd (negative odd too).
- For `i = -8 .. ceil(cross / p) + 8`: the segment starts at `y0 = i p + offset`, ends at `y1 = y0 + p`, at `x = j cw + d` when `i` is even and `j cw - d` when odd (parity of negative `i` taken as positive). It adds points `(x, y0)` and `(x, y1)`.
- Chevron swaps each point's coordinates after the band polygon is built.

### Stairs

Diagonal staircase bands.

- Step `s = 70 tooth width`. Band spacing `bw = max(1, round(2 width)) 2 s`, with `round` half up.
- Lines `j = -2 .. ceil((1000 + V + 4 bw) / bw) + 1`. Line `j` starts at `x = -V - 3 bw + j bw`, `y = -4 s`.
- While `y < V + 4 s`: add `(x, y)`, step `x += s`, add `(x, y)`, step `y += s`.

### Ricrac, Waves and Scales

Horizontal rows.

- Row height `rh = 0.115 V width`. Period `p = 140 tooth`.
- Amplitude: Scales `p / 2 * depth`; Waves `0.42 rh depth`; Ricrac `0.42 rh depth 1.2`.
- Lines `j = -2 .. ceil(V / rh) + 2`, each based at `y = j rh`.
- Ricrac: half-period `q = p / 2`. For `i = floor(-4q / q) .. ceil((1000 + 4q) / q)`, add `(i q, base - amp)` when `i` is even, `(i q, base + amp)` when odd.
- Waves and Scales: x runs from `-3p` while `x <= 1000 + 3p`, stepping by `p / 14` with repeated addition. Scales offsets odd rows by `p / 2`. With `u = 2π (x + offset) / p`, Waves adds `(x, base + amp sin u)` and Scales adds `(x, base - amp |sin(u / 2)|)`.

## Corner radius

- Ricrac: `max(0, 70 tooth 0.9 round)`.
- Waves and Scales: 6. Their sampled curves are smooth already.
- Stairs: `70 tooth width 0.9 round`.
- Teeth and Chevron: `min(125 width depth / 2, 0.055 span tooth) 0.9 round`, with `span` as above.

## Rounded polygon

For each band, with its `n` points taken cyclically:

1. Start a path at the midpoint of points 0 and 1.
2. For each `i` from 0 to `n - 1`, with `p0, p1, p2` the points at `i, i + 1, i + 2`:
   - `l1 = |p1 - p0|` and `l2 = |p2 - p1|`, each 1 when 0.
   - The corner angle at `p1` is the angle between `p0 - p1` and `p2 - p1`, its cosine clamped to -1..1. `t = tan(angle / 2)`, or 1 when that is 0.
   - Trim `tl = min(r / t, l1 / 2, l2 / 2)`, and the safe radius `rr = max(0, tl t)`.
   - Round the corner at `p1` toward `p2` with radius `rr`, as canvas `arcTo` does: a straight run to the tangent point, then the arc.
3. Close the path and fill it, non-zero, anti-aliased.

The clamp keeps a sharp corner from eating more than half of either edge.

## Painting

1. Fill the frame with the ground ink.
2. Fill each even band, in line order, with the stripe ink.

## Fidelity notes

- All geometry is anti-aliased paths in two flat inks, so the diff is edge pixels only.
- Waves and Scales call `sin`; Rust's and V8's may differ in the last bit, far below a pixel.
- Collinear and near-collinear points on the sampled curves turn `arcTo` into a straight line to the corner. The Surface's `arc_to` treats a corner as straight when the sine of the angle between its unit edges is at most 1/4096. Without that tolerance the Reference exports fail.
- The site's "My colors" fills a 6-swatch Tool from a shorter set with derived tints. Reference exports therefore use a 6-ink Palette, so the site's swatches equal the Recipe's Palette.
