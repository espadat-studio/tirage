# sprig

A hand-drawn repeat of little plants. Blooms, leaves, petals, daisies, dots and stems are scattered evenly over a plain ground in one ink, each line drawn as a marker would lay it down: swelling and thinning, with a chewed edge. The pattern tiles, so whatever runs off one edge comes back on the other.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults, in page order:

| id       | range                  | step | default | role                                          |
| -------- | ---------------------- | ---- | ------- | --------------------------------------------- |
| `dirs`   | Garden, Blooms, Leaves |      | Garden  | which motifs are in the mix                   |
| `count`  | 2..120                 | 1    | 42      | how many motifs                               |
| `size`   | 0..1                   | 0.01 | 0.62    | motif size against the frame                  |
| `vary`   | 0..1                   | 0.01 | 0.5     | how much motif sizes differ                   |
| `solids` | 0..1                   | 0.01 | 0.06    | share of motifs filled in instead of outlined |
| `weight` | 0..1                   | 0.01 | 0.5     | line width                                    |
| `rough`  | 0..1                   | 0.01 | 0.45    | how much the line width swells and thins      |
| `wobble` | 0..1                   | 0.01 | 0.5     | how far each shape strays from its ideal      |
| `detail` | 0..1                   | 0.01 | 0.55    | odds of a centre, vein or ribs                |

On this page `size` is a Parameter and the export size is `sizePx`.

Motion: three modes, Drift (default), Boil and Swell, off by default. sprig is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off the drift offset is 0, the swell factor 1 and the boil salt 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

Default Palette, 2 inks: `#f9e4c8 #2d6a4f`. Ink 1 is the ground, ink 2 the line. Inks past the second are not drawn.

## Noise and random draws

- The hash and value noise are [sonar](sonar.md)'s.
- One sequential generator: xorshift on 32 bits (`<< 13`, logical `>> 17`, `<< 5`), started from `seed * 2654435761` wrapped to 32 bits as `Math.imul` does, 0 becoming 1. A draw is the state over 2^32.

## Layout

The tile is `fw = 1 / sqrt(H / W)` wide and `fh = sqrt(H / W)` tall, so `U = sqrt(W H)` scales it to the frame. The motif mix by `dirs`, drawn from by index:

- Garden: bloom, leaf, leaf, petal, daisy, dot, stem, bloom, leaf, bloom
- Blooms: bloom, bloom, petal, daisy, dot
- Leaves: leaf, leaf, leaf, leaf, stem, petal

For each of `max(1, count)` motifs, in this draw order:

1. Placement, best of 12: each candidate draws `x = draw * fw`, then `y = draw * fh`. Its score is the squared distance to the nearest motif placed so far, measured across the tile's wrap on both axes (a gap over half the tile counts the other way round). The first candidate with a strictly higher score than every earlier one wins; with no motif yet every candidate scores 1e9 and the first wins.
2. Kind: `mix[floor(draw * length)]`.
3. `u = draw`. The size factor is `k = 1 + (2.2 u² - 0.45) * vary`.
4. Angle `a = draw * 2π`.
5. Solid: a dot always is, with no draw. Any other kind draws, and is solid when the draw is under `solids`.
6. The motif's paths, below.
7. The motif's line salt: `floor(draw * 9973)`.

## Motifs

Each motif is a few paths in its own unit space. It first draws its shape salt `sd = floor(draw * 9973)`.

A ring `ring(N, rx, ry, pointy, salt)` is `N` points. Point `i` sits at angle `θ = 2π i / N`, with `c = cos θ`, `s = sin θ`:

- radius factor `r = 1 + (noise(1.9 c + 3.1, 1.9 s + 7.7, salt) - 0.5) * 0.5 * wobble`
- height `y = s`, or for a pointy ring `sign(s) * |s|^(1 + 0.9 pointy)`, with `sign(0) = 1`
- the point is `(c * rx * r, y * ry * r)`

Read off a circle, the wobble closes where it started.

- bloom: a closed ring `ring(40, 1, 0.78 + 0.3 draw, 0, sd)`. Then a draw: under `detail`, add a filled centre `ring(16, 0.16, 0.11, 0, sd + 7)`.
- leaf: `ry = 0.30 + 0.22 draw`, a closed pointy ring `ring(38, 1, ry, 1, sd)`. Then `d = draw`:
  - `d < detail / 2`: a vein, an open path of 11 points `(t, (noise(2.2 t + 0.3 sd, 1.7, sd + 3) - 0.5) * ry * 0.5 * wobble)`, `t` from -0.72 to 0.72 in tenths of the span.
  - else `d < detail`: `m = 3 + floor(4 draw)` ribs. Rib `j` is an open 2-point path from `(t - 0.06, -h)` to `(t + 0.06, h)`, with `t = -0.62 + 1.24 (j + 0.5) / m` and `h = ry * (1 - |t|^1.9) * 0.82`.
- petal: a closed pointy ring `ring(26, 0.55, 0.22, 1, sd)`. No draws.
- daisy: `m = 5 + floor(4 draw)` petals. Petal `j` draws its angle `2π j / m + 0.2 draw`, is the closed pointy ring `ring(22, 0.62, 0.2, 1, sd + 13 j)` shifted by 0.62 along x, then turned by that angle. Then a closed centre `ring(18, 0.2, 0.16, 0, sd + 91)`.
- dot: a filled `ring(24, 0.42, 0.40, 0, sd)`. No draws.
- stem: an open path of 15 points. The heading starts at `2π draw`, the pen at the origin. Each point is placed, then the heading turns by `(noise(0.31 i + 0.2 sd, 4.4, sd) - 0.5) * 0.9 * wobble` and the pen moves 0.13 along it.

## Painting

- Fill the frame with the ground ink.
- `base = 0.035 + 0.11 size`, line width `w = U * (0.004 + 0.020 weight)`.
- For each motif in order: scale `sc = base * max(0.18, k) * U`. It is drawn at 9 placings, `(x + gx fw, y + gy fh) * U` for `gx` then `gy` in -1, 0, 1. A placing more than `2 sc` outside the frame on either axis is skipped.
- Each path point `(px, py)` maps to the placing plus `(px cos a - py sin a, px sin a + py cos a) * sc`.
- A path is filled solid when it is a filled centre or dot, or when it is closed and the motif is solid. A solid fill is the polygon, nonzero.
- Any other path is inked as a marker line, below, with the motif's line salt.

## Marker line

For a path of `n ≥ 2` points, salt `q`:

- Each point's direction: on an open path the first and last use the segment next to them; every other point, and every point of a closed path, uses the vector from its previous to its next neighbour, wrapping on a closed path. The normal is that vector turned a quarter left, over its length (a zero length counts as 1).
- Half width `h = 0.5 w * (1 + (noise(2.6 cos θ + 0.11 q, 2.6 sin θ + 0.07 q, q) - 0.5) * 1.25 rough)` with `θ = 2π i / n`.
- On an open path, `t = i / (n - 1)` and `e = 7 min(t, 1 - t) + 0.3`. When `e < 1`, `h *= e`, so the ends taper.
- `h` is at least 0.2.
- The left edge is the point plus `h` along the normal, the right edge minus.
- Open path: one polygon, the left edge forward then the right edge back, filled nonzero.
- Closed path: two closed rings, the left edge forward and the right edge backward, filled even-odd. That leaves the band between them, an outline with nothing inside.

All fills are anti-aliased.

## Fidelity notes

- Everything is polygons with anti-aliased edges, so differences sit on the edges at a level or two.
- `sqrt`, `cos`, `sin` and `pow` may differ from V8's in the last bit. That moves a point by far less than a pixel.
- Reference exports use the 2-ink default Palette, as the site's My colors pads a shorter Palette.
