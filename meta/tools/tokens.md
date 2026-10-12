# tokens

A board game laid out mid-play. Round and square counters sit on a faint ruled grid, grouped in runs, blocks, plus signs and singles. Each counter is a disc or square of one ink ringed by another, and a few are hollow.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id          | range   | step | default | role                                                   |
| ----------- | ------- | ---- | ------- | ------------------------------------------------------ |
| `cols`      | 4..120  | 1    | 16      | board columns across the frame                         |
| `ruleEvery` | 1..10   | 1    | 4       | a vertical rule every this many columns                |
| `grid`      | 0..1    | 0.01 | 0.6     | opacity of the ruled grid                              |
| `count`     | 1..400  | 1    | 14      | how many clusters to place                             |
| `wRun`      | 0..100  | 1    | 44      | weight of a run                                        |
| `wBlock`    | 0..100  | 1    | 18      | weight of a block                                      |
| `wPlus`     | 0..100  | 1    | 10      | weight of a plus                                       |
| `wOne`      | 0..100  | 1    | 28      | weight of a single                                     |
| `clump`     | 0..1    | 0.01 | 0.7     | how hard clusters gather on the high ground of a field |
| `align`     | 0..1    | 0.01 | 0.55    | chance a cluster snaps to a guide row or column        |
| `runLen`    | 2..12   | 1    | 5       | longest run                                            |
| `upright`   | 0..1    | 0.01 | 0.18    | chance a run stands vertical                           |
| `sizeT`     | 0.03..1 | 0.01 | 0.5     | counter size as a share of the cell                    |
| `svar`      | 0..1    | 0.01 | 0.45    | how much the size varies by cluster                    |
| `hier`      | 0..1    | 0.01 | 0.65    | how strongly a few clusters come out large             |
| `square`    | 0..1    | 0.01 | 0.34    | chance a cluster is squares                            |
| `strokeW`   | 0..8    | 0.5  | 1.5     | outline width, in board units                          |
| `hollow`    | 0..1    | 0.01 | 0.18    | chance a counter is hollow                             |
| `kin`       | 0..1    | 0.01 | 0.2     | how far in hue an outline ink may sit from its fill    |
| `break`     | 0..1    | 0.01 | 0.3     | chance a counter takes its own inks                    |

Motion (`modes`, `active`, `stag`, `fps`, `frames`) is off by default and not part of a Still. With motion off no counter cycles or blinks. The Loop turns motion on at site defaults, and motion is not a Parameter. See [Motion](#motion). `active` takes one draw per cluster in the Still too, see below. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The page has no seed deal: a new seed only reshuffles the layout.

tokens is a palette-family Tool. Its Palette is the swatch row, read by position, any number of inks from 2. Default Palette, 9 inks: `#1B1B1B #E63946 #3A86FF #FFBE0B #2A9D8F #FF006E #8338EC #FB5607 #00B4D8`. The board `#F4F1EA` and the rule ink `#D9D4C7` are Tool constants, not Palette.

## Random numbers

`s` is the Tool seed.

- One xorshift32 stream with the arithmetic right shift: the 17-bit shift copies the sign bit, as aura's does, not the chassis stream. Its start is `s × 2654435761 + 17` worked in doubles, so the product rounds once it passes 2^53, then taken mod 2^32. A zero start becomes 1.
- `h(x, y, seed)` is the xor-multiply integer hash aura uses, with no channel term.
- `field(x, y)` is value noise on `h` at seed `s + 301` (wrapping): smoothstep fade, lerp along x then y.

## Board

The board is 1000 units wide, `VH = 1000 H / W` units tall. `C = max(3, cols)`, `R = max(3, round(C VH / 1000))`, `cell = 1000 / C`. A board unit is `k = W / 1000` pixels.

## Placing

`n` is the Palette length. For each ink `i`, `near[i]` lists the other ink indices sorted by hue distance from ink `i`, ties in index order. Hue is the HSV hue in degrees, 0 for a grey; the distance is the shorter way round the circle.

An outline for fill `f`: `reach = max(1, round(1 + kin (n - 2)))`, take `near[f][floor(r reach)]`. If it equals `f`, use `(f + 1) mod n`.

Kinds are run, block, plus and one, weighted `wRun wBlock wPlus wOne`. To pick one, take `r × total` (total 1 if every weight is 0) and subtract weights in that order; the first kind that takes it to 0 or below wins, else one.

Footprints, as cell offsets:

- one: the cell itself
- plus: the cell and its four neighbours, in the order up, left, centre, right, down
- block: `w = 2 + (r < 0.4)`, then `h = 2 + (r < 0.4)`, cells row by row
- run: length `2 + floor(r max(1, runLen - 1))`, then vertical when `r < upright`; cells along it

Guides: `g = max(2, round(2 + 4 (1 - align)))`. For each, a row `floor(r R)` then a column `floor(r C)`.

Then up to `140 count` tries, stopping once `count` clusters are placed. Each try, in this order of draws:

1. the kind, then its footprint draws
2. origin `ox = floor(r C)`, `oy = floor(r R)`
3. when `align > 0` and `r < align`: when `r < 0.5` the origin row becomes a guide row `floor(r g)`, else the column a guide column
4. when `clump > 0`: `fv = field(2.6 ox / C + 0.5, 2.6 oy / R + 0.5)`. Skip the try when `r > fv^(1 + 6 clump)`
5. skip the try unless every footprint cell is on the board and no cell, nor any of its eight neighbours, is taken
6. take the cells. Fill `f = floor(r n)`, outline as above
7. shape: square when `r < square`, else circle
8. `big = r^(1 + 3.2 hier)`, then `size = sizeT (1 - svar / 2 + svar r) (1 + 2.6 hier big)`
9. per cell in footprint order: when `r < break` it draws its own fill `floor(r n)` and an outline for it. Then the shape flips when `r < 0.12`, the counter is hollow when `r < hollow`, and one more draw (phase)
10. three more draws for the cluster: phase, direction and active, which is `r < 0.45`, the site's default moving share. Only active is used, by the Loop

## Painting

1. Fill the frame with the board ink.
2. When `grid > 0`, one path stroked once in the rule ink at alpha `grid`, line width `max(0.5, 1.1 k)`: a vertical line at `round(i cell k) + 0.5` for `i = step, 2 step, … < C` with `step = max(1, ruleEvery)`, and a horizontal line at `round(j cell k) + 0.5` for every `j = 1..R-1`, each across the whole frame. Overlaps do not stack the alpha.
3. Counters in placing order. Radius `rad = max(0.05, min(size cell / 2, max(0.05, cell / 2 - strokeW)))` board units, centre `((x + 0.5) cell, (y + 0.5) cell)`. When `strokeW > 0`, fill the shape at radius `rad + strokeW` in the outline ink, then fill it at `rad` in the fill ink, or the board ink when hollow. Everything scales by `k`. A square is the axis-aligned square of half side the radius.
4. The chassis grain, then dither.

## Motion

The site has two motion modes: Cycle and Blink. Cycle is the default and the only one ported: moving share 0.45, stagger 0.55, 36 frames at 14 fps. tokens has no amount slider.

- Frame `f` sits at `T = f / frames`.
- Only active clusters move. Each of their counters rolls both its inks through the Palette by the same step `floor((T + lag) n)`, fill and outline alike, mod `n`. Hollow counters keep the board ink inside; only their ring rolls.
- The lag staggers the roll: `lag = 0.55 (c / m + 0.06 i)` for cluster `c` of `m` placed clusters and counter `i` within it. So later clusters and later counters in a cluster run ahead.
- Fill and outline move by the same step, so they never meet: the outline is never the fill in the Still, and stays apart.
- Nothing else moves. Positions, sizes, shapes, hollows and the inactive clusters hold.
- The Loop closes by one whole Palette turn: at `T = 1` every step is `n` more, which is the same ink mod `n`. `(1 + lag) n` could round across a whole number where `lag n` does not, but on the 3 Seeds the closure test renders, frame `frames` is frame 0, byte for byte.
- At `f = 0` the step is `floor(lag n)`, which is above 0 for most active counters. So frame 0 is not the Still and gets a reference export of its own.
- The mid frame is 18: half a turn, every active ink about `n / 2` steps on from frame 0, as far from it as the Loop gets.
- Blink is not ported. It flips the hollow of active counters on a staggered sine. It is not reachable from a Recipe.

## Fidelity notes

- Counters and squares are antialiased fills at fractional positions.
- The site opens at 1:1; Reference exports set 9:16.
