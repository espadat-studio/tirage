# sonar

A one-bit chart of a coastline. A smooth random height field is cut at a water level. Land above the cut is a grid of square dots, sized by how far inland they sit. A thin band either side of the cut breaks up into sparse coloured specks.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range   | step | default | role                                           |
| -------- | ------- | ---- | ------- | ---------------------------------------------- |
| `level`  | 0..1    | 0.01 | 0.5     | where the field is cut                         |
| `scale`  | 1..10   | 0.1  | 2.2     | how much field fits across the frame           |
| `warp`   | 0..1    | 0.01 | 0.4     | how far a second field pushes the first around |
| `grid`   | 40..320 | 2    | 150     | dot columns across the frame                   |
| `depth`  | 0..1    | 0.01 | 0.7     | how fast dots grow inland                      |
| `fringe` | 0..1    | 0.01 | 0.45    | width of the coastal band                      |
| `spark`  | 0..1    | 0.01 | 0.6     | how many specks the band carries               |

Motion (`modes`, `amt`, `fps`, `frames`), grain and dither are off by default and not part of a Still. Taste bounds keep them at site defaults, so none of them is a Recipe Parameter. Default Palette: `#0a0f1c #3ddc97 #4361ee #ffd166 #ef476f #f1faee`.

Inks go by position, wrapping when the Palette is short: ink 0 is the water, ink 1 the land, inks 2 and 3 the two speck colours. Inks past the fourth are unused.

## Integer hash

All randomness is one stateless hash of two cell coordinates and a seed, all 32-bit with wrapping arithmetic: multiply each coordinate and the seed by its own large odd constant, add the three, xor in the value shifted right by 13, multiply by the seed constant again, xor in the value shifted right by 16. Read the result as unsigned and divide by 2^32 to get a value in [0, 1).

Coordinates are truncated to signed 32-bit first, so negative lattice cells hash fine. Seed offsets also wrap at 32 bits.

## Field

- Value noise: hash the four lattice corners around a point and blend them with a smoothstep weight on each axis.
- fbm: sum octaves of value noise, each at double the frequency and half the weight of the last, then divide by the total weight. Octave `i` adds `131 * i` to the seed.
- Warp: two 3-octave fbm fields at 0.6x the coordinates, offset by fixed constants `(11.3, 3.7)` with seed +7 and `(5.1, 19.9)` with seed +13. Each is centred on 0 by subtracting 0.5.
- Height: a 4-octave fbm with the Tool seed, sampled at the point plus both warp values times `warp * 2.4`.

## Grid

- `cols = max(12, grid)` and the cell width is `W / cols`.
- `rows = max(6, round(H / cell width))` and the cell height is `H / rows`, so cells are close to square.
- Each cell is sampled at its centre `(u, v)` in 0..1. The field point is `(u * scale * W/H, v * scale)`, with `scale` floored at 0.5.

## Cut

- The level is `0.15 + 0.7 * level`, clamped to 0.05..0.95.
- The band half-width is `0.012 + 0.07 * fringe`.
- `d` is the height minus the level.

## Painting

The whole frame is filled with ink 0 first. Then cells are visited row by row, left to right. Later dots paint over earlier ones.

- Inland, when `d` is above the band: let `t = min(1, (d - band) / (0.2 * (1.05 - 0.85 * depth)))`. The dot side is `cell width * (0.3 + 0.68 * t)`, in ink 1.
- Coast, when `d` is within the band: let `near = 1 - |d| / band`. The cell gets a speck when its hash at seed +53 is below `near² * spark * 1.35`. The speck side is `cell width * (0.34 + 0.42 * near)`. Its ink is 2 when the hash at seed +59 is below 0.5, else ink 3.
- Water, below the band: nothing.

Every dot is a square centred in its cell. Its corner is rounded to a whole pixel on each axis, and its side is rounded with a minimum of 1 px. Both cell edges use the cell width for the side, so dots stay square even when cells are not. Rounding goes half up, as JavaScript's `Math.round` does.

## Motion

The site has one motion mode, Tide. The land keeps its shape and the water level rises and falls, so the coast moves in and out.

- Site defaults: amount 0.6, 24 frames at 10 fps. These are sonar's Loop.
- Frame `f` sits at `T = f / frames` through the Loop.
- The number of tide cycles per Loop is `max(1, round(2 * amount))`, so 1 at the default.
- The offset is `sin(2π * cycles * T) * 0.1 * min(1.6, amount)`, so ±0.06 at the default.
- The offset is added to the cut level after the clamp to 0.05..0.95, not before.
- A whole number of cycles brings the level back to its start, so the last frame leads into the first. At `f = 0` the offset is exactly 0, so frame 0 is the Still.
- The band, the hashes and the Tool seed do not change with `f`.

## Fidelity notes

- Dots are whole-pixel axis-aligned rects with no anti-aliasing. The Still uses no trigonometry, only integer hashing and float arithmetic, so a port can match the export byte for byte.
- Loop frames past 0 call `sin`. Rust's and V8's `sin` may differ in the last bit, which can flip a rare cell that sits exactly on the cut.
- The site's "My colors" fills a 6-swatch Tool from a shorter set with derived tints. Reference exports therefore use a 6-ink Palette, so the site's swatches equal the Recipe's Palette.
