# atlas

A text-mode map. Warped noise splits the page into terraces. Each terrace fills its cells with one Palette ink and scatters characters from its own stretch of a monospace set over them, so every terrace reads as a different material.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults, in page order:

| id        | range                                     | step | default | role                                        |
| --------- | ----------------------------------------- | ---- | ------- | ------------------------------------------- |
| `scale`   | 1..9                                      | 0.1  | 2.4     | noise frequency across the page             |
| `warp`    | 0..1.6                                    | 0.01 | 0.5     | how far the noise is pushed around          |
| `bands`   | 2..14                                     | 1    | 6       | how many terraces                           |
| `mix`     | 0..1                                      | 0.01 | 0.55    | contrast of the noise before it is banded   |
| `sets`    | DOS, Stipple, Blocks, Code, Digits, Runes |      | DOS     | the characters drawn                        |
| `cols`    | 24..260                                   | 1    | 110     | character columns across the frame          |
| `density` | 0..1                                      | 0.01 | 0.62    | share of cells that get a character         |
| `weight`  | 0..1.5                                    | 0.01 | 0.5     | how much of the set each terrace draws from |

`weight` is labelled Variety on the page. It is a position on the set, not a font weight. The site also has a Custom set with a free text field. tirage drops it, as for [kiosk](kiosk.md) (#24). The six sets are kiosk's, character for character.

Motion: two modes, Shuffle (default) and Drift, off by default. atlas is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off the grid seed is the Tool seed and the drift is 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

Default Palette, 8 inks: `#0d2b45 #203c56 #544e68 #8d697a #d08159 #ffaa5e #ffd4a3 #ffecd6`. Any length works: terraces wrap over it by position.

## Noise

The hash, value noise and `fbm` are [sonar](sonar.md)'s: the integer hash with 32-bit wrapping, smoothstep value noise on the four lattice corners, and `fbm(x, y, s, n)` summing `n` octaves at weight 0.5, 0.25, … and frequency 1, 2, …, octave `i` at seed `s + 131 i`, divided by the weight total.

`seed` is the Tool seed. There are no sequential random draws.

## Grid

- `cols = max(8, cols)`, `rows = max(6, round(cols * (H / W) * 0.6))`. A monospace cell is about 0.6 as wide as it is tall, so the cells stay near square on the page. On 9:16 that is 117 rows at the default 110 columns.
- `n = max(2, bands)`. `len` is the set's character count.
- `span = max(1, round(1 + (weight / 1.5) * (len - 1)))`, the same for every cell.

For each cell `(x, y)`, rows then columns:

1. Sample point in page space: `u = x / cols * scale`, `v = (y / rows) * scale * (H / W)`.
2. Warp: `wx = u + warp * (fbm(1.6 u + 19, 1.6 v, seed + 41, 3) - 0.5) * 2.2`, and `wy = v + warp * (fbm(1.6 u, 1.6 v + 7, seed + 83, 3) - 0.5) * 2.2`.
3. Level: `t = fbm(wx, wy, seed, 4)`, then `t = (t - 0.5) * (0.7 + 1.6 mix) + 0.5`, clamped to `0..0.999`.
4. Terrace `b = floor(t * n)`. Its ground is Palette ink `b mod length`.
5. Its type ink: of the Palette, `hi` is the first ink with the highest lightness and `lo` the first with the lowest, lightness being `0.299 r + 0.587 g + 0.114 b` on 0..255 channels. The ink is `hi` when the ground is further from `hi` than from `lo` in lightness, else `lo`.
6. The terrace's stretch of the set starts at `lo = floor(b / n * len)`.
7. The cell gets a character when `hash(3x + 5, 5y + 11, seed + 57 b) < density * (0.35 + 0.9 (b + 1) / n)`. Higher terraces are denser.
8. That character is `(lo + floor(hash(7x + 1, 13y + 3, seed + 97 b) * span)) mod len` of the set.

Rounding goes half up, as `Math.round` does.

## Painting

- Cell width `cw = W / cols`, height `ch = H / rows`.
- Grounds first. Each row is cut into runs of equal ground ink. A run from column `x` to `x2` fills the rect at `(floor(x cw), floor(y ch))`, `ceil((x2 - x) cw) + 1` wide and `ceil(ch) + 1` tall. The extra pixel overlaps the next run and row, which then paint over it. All edges are whole pixels, so nothing is anti-aliased.
- Then the characters, rows then columns, each centred at `((x + 0.5) cw, (y + 0.55) ch)` in its type ink, at size `0.98 ch`. The size is not rounded.

## Glyphs

As [kiosk](kiosk.md#glyphs), with one change: the canvas font names no weight, so it is 400. The glyph is DejaVu Sans Mono Book 2.37, the face Chromium resolves for atlas's monospace stack when the bundled Book and Bold faces are the only fonts installed. The typo metrics, hinting and position rounding are kiosk's.

## Fidelity notes

- At 540x960 and 110 columns a character is about 8 px tall, so hinting decides most of its pixels. A wrong face or weight shows at once.
- The noise is plain float arithmetic in the site's order. A terrace edge can still move by a cell where `t * n` lands within a rounding error of a whole number.
- Reference exports use the 8-ink default Palette, as the site's My colors pads a shorter Palette.
