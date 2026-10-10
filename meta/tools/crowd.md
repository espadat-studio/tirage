# crowd

A crowd of heads and shoulders seen through a heat camera. Each figure is a soft blob pushed through a false-colour ramp: ground where nobody stands, a halo round every figure with a darker rim at its outer edge, and a solid core. Small arrows lie over the top in patches, all in the core ink.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id          | range | step | default | role                                   |
| ----------- | ----- | ---- | ------- | -------------------------------------- |
| `crowd`     | 1..48 | 1    | 30      | how many figures                       |
| `scale`     | 0..1  | 0.01 | 0.4     | figure size                            |
| `wobble`    | 0..1  | 0.01 | 0.35    | how far noise pushes a figure's edge   |
| `blur`      | 0..1  | 0.01 | 0.5     | how soft the mask is                   |
| `halo`      | 0..1  | 0.01 | 0.5     | halo band width                        |
| `edge`      | 0..1  | 0.01 | 0.3     | how dark the rim is                    |
| `arrows`    | 0..1  | 0.01 | 0.5     | how much of the frame the arrows cover |
| `arrowSize` | 0..1  | 0.01 | 0.5     | arrow spacing and size                 |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. crowd is a Still Tool: only frame 0 is rendered. Motion stays fixed at site defaults and is not a Parameter. Dither is the shared [chassis](chassis.md) post-pass, off by default.

Grain is the chassis pass too, but the page opens with it on: overlay blend, amount 0.35, size 1, specks 0.3, vignette 0.2. Those are crowd's grain defaults.

Changing the page seed also deals every slider (`dealC`). That deal is not ported: `derive` deals every Parameter ([ADR 0003](../adr/0003-seed-derivation-of-recipes.md)).

crowd is a palette-family Tool. Its Palette is the swatch row, read by role, any number of inks from 2. Default Palette, 3 inks: `#B5BAB6 #D6EE3A #F08AE6`.

## Inks by role

- ink 0 is the ground
- ink 1 is the core
- inks 2 and on are the halo bands, outermost first. With 2 inks the only halo band is the core ink.
- the dark tone is the first halo band at 45% plus `(30, 0, 70)` at 55%
- the rim is the first halo band mixed toward the dark tone by `edge`

## Random numbers

`s` is the Tool seed, read as a 32-bit integer; every product and sum below wraps mod 2^32.

- `hash(x, y, seed)` and `vn(x, y, seed)` are the chassis integer hash and value noise ([chassis](chassis.md)).
- The figure stream is the chassis xorshift32 (unsigned shifts, zero start is 1), started at `(s * 2246822519) xor 0x27d4eb2f`.
- `sd1 = 7s + 1`, `sd2 = 7s + 2`, `sd3 = 7s + 3`.

## Figures

`U = sqrt(W H)`. For each of `n = crowd` figures, draws in this order:

1. `big`; head radius `hr = U (0.018 + 0.06 scale) (0.5 + 1.3 big^1.6)`
2. body height `bh = hr (2.4 + 3.2r)`
3. body width `bw = hr (1.5 + 0.8r)`
4. `rows = 3 + floor(3r)`
5. `row = floor(rows r)`
6. `cx = W (-0.06 + 1.12r)`
7. `cy = H (row + 1.2r - 0.3) / rows`
8. sway phase, then sway radius: two draws a Still ignores
9. corner radius `rr = hr (0.6 + 0.5r)`

Figure `k` has noise seed `ns = 13s + 7k`.

## Mask

A coarse grid of cell size `gs = max(2, round(U / 420))`, `fw = ceil(W / gs) + 2` by `fh = ceil(H / gs) + 2` cells, zeroed. Grid cell `(i, j)` sits at `px = (i - 1) gs + gs / 2`, `py = (j - 1) gs + gs / 2`. Cells are 32-bit floats.

`haloW = U (0.012 + 0.06 halo)` and margin `marg = 2 haloW + 0.03 U`. Per figure, over the cells `i` from `trunc((cx - bw/2 - hr - marg) / gs)` (at least 0) to `trunc((cx + bw/2 + hr + marg) / gs)` (at most `fw - 1`), and `j` from `trunc((cy - 1.2hr - marg) / gs)` to `trunc((cy + hr + bh + marg) / gs)`, clamped likewise:

1. head distance `d = |(px, py) - (cx, cy)| - hr`
2. shoulders: a rounded box centred at `(cx, by)`, `by = cy + 0.9hr + bh/2`, half sides `bw/2 - rr` and `bh/2 - rr`, corner `rr`. Its signed distance replaces `d` when smaller.
3. `d += (vn(px wf + 3, py wf + 7, ns) - 0.5) wobble 1.6hr`, `wf = 1 / (1.4hr)`
4. `v = clamp(0.5 - d / (2 haloW), 0, 1)`; the cell keeps the larger of its value and `v`.

Then a box blur of radius `rad = (0.004 + 0.04 blur) U / gs`, skipped under 1, with integer radius `r = trunc(rad)` and width `2r + 1`. Three passes, each a horizontal pass into a scratch grid then a vertical pass back, edges clamped, as a running sum in double precision stored to 32-bit floats.

## Ramp

1024 entries over mask value `m = i / 1023`. Stops `(position, colour)`: `(0, ground)`, `(0.05, ground)`, `(0.2, rim)`, then halo band `k` of `nb` at `0.32 + k span / nb` with `span = max(0.06, 0.62 - 0.32)`, then `(0.74, core)` and `(1, core)`. Each entry walks to the stop pair holding `m` (never past the last pair), smoothsteps the fraction between them and mixes. Entries are 32-bit floats.

## Painting

1. Each pixel samples the grid bilinearly at `gx = (x + gs) / gs - 0.5`, `gy = (y + gs) / gs - 0.5`, the second column and row clamped to the grid. `m` is capped at 1, and the pixel takes ramp entry `trunc(1023 m)`, each channel rounded half to even.
2. Arrows, when `arrows > 0`. Spacing `sp = 0.019 U (0.7 + 0.9 arrowSize)`, size `sz = 0.55 sp`. Over `ceil(H / sp) + 1` rows `j` and `ceil(W / sp) + 1` columns `i`, centre `(x, y) = ((i + 0.5) sp, (j + 0.5) sp)`:
   - skip when `0.7 vn(3.2x/U + 1, 3.2y/U + 5, sd1) + 0.3 vn(9x/U + 4, 9y/U + 2, sd2) < 1 - 0.95 arrows`
   - skip when `hash(i, j, sd3) < 0.3`
   - jitter `(hash(i, j, sd3 + 1) - 0.5) 0.6 sp`, then the same with `sd3 + 2` for y
   - angle `a = 0.75π + (vn(2.2x/U + 9, 2.2y/U + 3, sd2) - 0.5) 1.6π + (hash(i, j, sd3 + 3) - 0.5) 0.9`
   - the shaft runs from `-sz/2` to the tip at `+sz/2` along `a` from the jittered centre; the head is a second subpath from one barb through the tip to the other, the barbs `0.42 sz` back and `0.34 sz` aside.

   Every arrow joins one path, stroked once in the core ink at alpha 0.9, width `max(0.8, 0.15 sz)`, round caps and joins. Overlapping arrows do not stack their alpha.
3. Grain, then dither.
