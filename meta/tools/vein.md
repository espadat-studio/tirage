# vein

Poured paint, cut flat. A smooth stripy field is sliced at evenly spaced levels. Each level is the outline of everything at or above it, filled as one shape over the levels below, so the sheet reads as stacked layers of marbled paint. The lowest levels are a dark ground; stars sit on that ground only.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range                 | step | default | role                                              |
| -------- | --------------------- | ---- | ------- | ------------------------------------------------- |
| `flows`  | Marble, Swirl, Ripple |      | Marble  | which stripe pattern the field makes              |
| `scale`  | 0..1                  | 0.01 | 0.5     | how many stripes fit across the frame             |
| `curve`  | 0..1                  | 0.01 | 0.6     | how far a slow warp bends the stripes             |
| `levels` | 3..14                 | 1    | 9       | how many cuts                                     |
| `tiger`  | 0..1                  | 0.01 | 0.5     | how hard cut lines crowd inside a mask            |
| `edges`  | 0..1                  | 0.01 | 0.4     | share of levels that get an outline               |
| `stars`  | 0..1                  | 0.01 | 0.5     | how many stars                                    |
| `tints`  | 0..1                  | 0.01 | 0.5     | how far each level's ink is lightened/shaded      |
| `runs`   | 0..1                  | 0.01 | 0.5     | how long one ink repeats over neighbouring levels |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. vein is a Still: only frame 0 is ported. Dither is the shared [chassis](chassis.md) post-pass, off by default.

Grain is the shared chassis grain, but vein opens with it on: on, Overlay, amount 0.4, size 1, specks 0.4, vignette 0.25.

Default Palette, 8 inks: `#0e1a2b #2e63b8 #7fa8e0 #e9dcc3 #f2892b #e0362f #2fa39a #8b5a2b`.

Inks go by position: ink 0 is the dark ground, and the rest are dealt to levels and stars. All `n` inks are used, so the Palette's length changes the picture.

The site picks the leading parameters from its seed when the seed changes (a "deal"). That deal is not ported: our `derive` replaces it (ADR 0003). Reference exports set the seed first and every Parameter after.

## Randomness

Two sources, both from the Tool seed `s`, read as a signed 32-bit integer. All seed arithmetic wraps at 32 bits.

- The integer hash `h(x, y, seed)` and value noise `vn` are the chassis ones ([sonar](sonar.md#integer-hash)).
- A xorshift32 stream for the sheet. Its start is `s * 2246822519` (wrapping) xor `0x27d4eb2f`, with 0 replaced by 1. Each step: `a ^= a << 13`, `a ^= a >> 17`, `a ^= a << 5` (logical shifts, 32 bits), and the draw is `a / 2^32`.

Field seeds: `s1 = 7s + 1` for the warp, `s2 = 7s + 2` for the detail noise, `s3 = 7s + 3` for the stripe mask.

## The sheet

Three stream draws, in this order:

1. The stripe angle `A = r * 2π`.
2. The swirl centre x, `(0.3 + 0.4r) * W / U`.
3. The swirl centre y, `(0.3 + 0.4r) * H / U`.

`U = sqrt(W * H)`. The field is sampled in units of `U`, so a point `(x, y)` in pixels is `(x / U, y / U)`.

The stripe frequency is `f = 0.6 + 1.6 * scale`.

## Field

At a point `(u, v)`:

- Warp: `wu = u + curve * 0.55 * (vn(1.3u + 3, 1.3v + 1, s1) - 0.5) * 2`, and `wv` the same with offsets `(8, 5)`. Both use `s1`.
- Detail: `n = 0.6 vn(2.4f wu + 7, 2.4f wv + 2) + 0.3 vn(5f wu + 3, 5f wv + 9) + 0.1 vn(11f wu + 4, 11f wv + 6)`, all at `s2`.
- Stripes, by flow. Each is `0.5 + 0.5 sin(π * phase)`:
  - Marble: `p = wu cos A + wv sin A`, phase `1.6 f p + 2.4 (n - 0.5)`.
  - Swirl: take the unwarped point relative to the centre, `(dx, dy)`, at distance `d`. Rotate it about the centre by `4 curve * exp(-d² / 0.16)`. `p` is the rotated point projected on `A`, phase `1.6 f p + 2.0 (n - 0.5)`.
  - Ripple: `p = 0.7 wv + 0.15 wu`, phase `2.6 f p + 1.4 (n - 0.5)`.
- Tiger, only when `tiger > 0`: a mask `m = vn(1.5u + 9, 1.5v + 4, s3)`, ramped to `k = clamp((m - 0.45) / 0.25, 0, 1)`. Add `0.09 * tiger * k * sin(10π * value)` to the value.

## Grid

- The cell is `c = U / 150` px.
- The grid is `ceil(W / c) + 3` by `ceil(H / c) + 3` values, stored as 32-bit floats.
- Its outer ring is -1, so every cut closes inside the grid. Interior node `(i, j)` holds the field at `((i - 1) c / U, (j - 1) c / U)`.
- Node `(i, j)` sits at pixel `((i - 1) c, (j - 1) c)`.
- Looking up the field at a pixel rounds `x / c` and `y / c` half up, adds 1, and clamps into the interior.

## Levels

`N = max(2, levels)`. Level `k` cuts at `(k + 0.5) / N`. The bottom `dark = max(1, round(0.3 N))` levels are ground.

Colour per level, `k = 0..N`:

- Ground levels: ink 0, shaded by 0.2 on even `k` and lightened by 0.12 on odd `k`. No outline.
- Other levels share inks in runs. The run state starts at ink `1 mod n`, ending at level 0. When `k` reaches the run end, a new ink is picked: `1 + floor(h(k, 4) * (n - 1))`. If that repeats the current ink and `n > 2`, it steps to `1 + (pick mod (n - 1))`. The run then ends at `k + 1 + floor(h(k, 5) * (1 + 3 runs))`.
- Its tint is `(2 h(k, 6) - 1) * 0.45 tints`.
- It gets an outline when `h(k, 7) < edges`. The outline ink is `1 + ((ink + 1 + floor(h(k, 8) * (n - 2))) mod (n - 1))`, lightened by 0.25.

`h(k, j)` here is the integer hash at the Tool seed itself.

Tinting moves an ink: a positive `t` lightens it toward white by `min(1, t)`. A negative `t` shades each channel by `1 - 0.9 min(1, -t)`. The result rounds half up and clamps to a byte.

## Cutting

Marching squares on the grid, for each level `t`:

- A cell's corners are top left, top right, bottom right, bottom left. Each corner counts as inside when its value is at least `t`.
- Where an edge crosses, its point sits at `(t - a) / (b - a)` along it, from corner `a` to `b`. A zero span uses 1e-9.
- Cells that are all inside or all outside add nothing. Others join their crossings in the usual pairs.
- The two saddles look at the cell's mean. Mean inside: the crossings pair up to cut off the two outside corners, so the middle joins the inside ones. Mean outside: they cut off the two inside corners instead.
- Crossings join into closed loops. A loop with fewer than 3 points is dropped.
- Each loop is smoothed: move to the midpoint of its first two points. Then, for each point in turn, draw a quadratic curve with that point as control, ending at the midpoint to the next point, until back at the start. Close it.
- Every coordinate is rounded half up to 0.1 px.

Since each loop is a closed cycle, its start point and direction do not change the shape.

## Painting

1. Fill the frame with ink 0.
2. For each level, low to high, with a non-empty cut: fill all its loops as one even-odd path, anti-aliased, in the level colour. If it has an outline, stroke the same path right after, anti-aliased, `max(0.6, 0.0035 U)` px wide with round joins.
3. Stars, `round(600 stars)` of them, `k = 0..`:
   - At `(h(k, 11) W, h(k, 12) H)`.
   - Skipped where the grid value at that pixel is at least `dark / N`, so stars only land on the ground.
   - The light ink is the brightest of inks `1..n` by `0.2126 r + 0.7152 g + 0.0722 b`, the first on a tie. A star takes it when `h(k, 13) < 0.7`, else ink `1 + floor(h(k, 14) * (n - 1))`, lightened by 0.4.
   - A filled anti-aliased circle of radius `U * (0.0012 + 0.002 h(k, 15))`.
4. Grain, then dither, as the chassis does.

## Fidelity notes

- Edges are anti-aliased, so a port cannot match byte for byte. Skia's analytic coverage and tiny-skia's supersampling disagree slightly on edge pixels.
- The field uses `sin`, `cos` and `exp`. A last-bit difference from V8 can move a rounded 0.1 px coordinate, which is well under an edge pixel.
- Path points are written to a string at 0.1 px and parsed back. Rounding the float to 0.1 gives the same value.
- The site's "My colors" fits a set to the Tool's swatch count, deriving tints for missing inks. vein has 8 swatches, so Reference exports use an 8-ink Palette and the site's inks equal the Recipe's.
- Taste bounds only cover the sliders. `flows` stays Marble in a derived Recipe: the triage never looked at the other two.
