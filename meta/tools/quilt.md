# quilt

Pixel quilt cloth. One pattern engine fills a chunky grid with four ink roles, measured from the centre so the whole sheet mirrors, then the grid is blown up with hard edges.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range                                                                                                       | step | default | role                                   |
| -------- | ----------------------------------------------------------------------------------------------------------- | ---- | ------- | -------------------------------------- |
| `styles` | Auto, Bands, Tabs, Plaid, Dither, Steps, Zigzag, Diamond, Cross, Basket, Rings, Star, Waves, Gingham, Burst |      | Auto    | the pattern; Auto lets the seed choose |
| `cells`  | 28..72                                                                                                      | 2    | 44      | grid columns across the frame          |
| `chunk`  | 0.7..1.8                                                                                                    | 0.05 | 1       | scales every motif size                |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. The Loop turns motion on at site defaults, and motion is not a Parameter. See [Motion](#motion). Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

quilt is fixed-role. Its Palette is four inks by role, the swatch row in order: Base, Weave A, Weave B, Pop. Default Palette: `#F3E9DC #2A6F97 #C8553D #F4D35E`. The cap is 4.

## Random numbers

`s` is the Tool seed. One xorshift32 stream: aura's, with the signed right shift and the float-product start ([aura](aura.md)).

The integer hash `h(x, y, c, seed)` is aura's four-input hash.

## The pattern

`ch = chunk`. `round` is half up. Draws from the stream, in this order:

1. Style: when `styles` is Auto, `[bands, tabs, plaid, dither, steps, zigzag, diamond, cross, basket, rings, star, waves, gingham, burst][floor(14r)]`. A fixed style takes no draw.
2. `rs = floor(1e9 r)`, the hash seed.
3. Bands: `bh = max(2, round(2.2 ch))`, `L = max(4, round((6 + 4r) ch))`.
4. Tabs: `tw = max(2, round((3 + 2r) ch))`, `th = max(1, round(1.6 ch))`, `gx = max(1, round(2 ch))`, `gy = max(1, round(1.4 ch))`.
5. Plaid: `per = max(6, round((9 + 4r) ch))`, `pw = max(1, round(1.8 ch))`.
6. Dither: `f1 = 0.22 + 0.25r`, `f2 = 0.09 + 0.08r`, `warp = 1.2 + 2.2r`, `ph0 = 2πr`.
7. Steps: `rw = max(2, round((3 + 2r) ch))`, `rh = max(1, round(2 ch))`, `off = floor(97r)`, `wide = max(5, round((7 + 5r) ch))`, `t1 = max(1, round(1.2 ch))`, `w2 = max(2, round((2.5 + 2r) ch))`, `w3 = max(2, round((2 + 2r) ch))`. Then one draw: under 0.5 the run is `0×wide 3×t1 2×w2 1×w3 3×t1`, else `0×wide 1×w3 2×w2 3×t1`.
8. Zigzag: `zper = max(6, round((10 + 6r) ch))`, `zslope = 0.5 + 0.5r`, `zrh = max(2, round(2.5 ch))`. One draw: under 0.5 the run is `1 0 2 0`, else `1 0 2 3 0`, each `zrh` long except the 3, which is `t1`.
9. Diamond: `dw = max(6, round((9 + 5r) ch))`, `dh = max(6, round((7 + 4r) ch))`.
10. Cross: `ct = max(5, round((7 + 3r) ch))`, `ca = max(1, round(1.1 ch))`.
11. Basket: `bb = max(4, round((5 + 3r) ch))`, `bs = max(1, round(1.4 ch))`.
12. Rings: `rw1 = max(2, round((2 + 2r) ch))`. One draw: under 0.5 the run is `0×(rw1+1) 1×rw1 0×rw1 2×rw1 3×t1`, else `0×(rw1+1) 2×rw1 3×t1 1×rw1`.
13. Star: `stt = max(7, round((10 + 4r) ch))`.
14. Waves: `wper = max(6, round((8 + 5r) ch))`, `wamp = max(2, round(2.6 ch))`, `wh = max(2, round(2.4 ch))`. One draw: under 0.5 the run is `1 0 2 0`, else `1 0 3 2 0`, each `wh` long except the 3, which is `t1`.
15. Gingham: `gg = max(2, round((3 + 2r) ch))`.
16. Burst: `bn = 3 + floor(3r)`, `bcore = max(2, round(3 ch))`.

Every style's draws happen whatever the style, so the stream position does not depend on it. A run is a lookup table: each entry repeated its width, read at `d mod length`, kept non-negative.

## Grid

`gw = cells`, `gh = max(8, round(gw H / W))`. Cell `(u, v)` has `ax = u - mx`, `ay = v - my` from the centre `mx = (gw - 1) / 2`, `my = (gh - 1) / 2`, and `ux = |ax|`, `vy = |ay|`. Each cell is one role, 0..3:

- bands: row `b = floor(vy / bh)`, shift `floor(h(0, b, 2) L)`, `uu = ux + shift`, block `k = floor(uu / L)`. Role `floor(3 h(k, b, 5))`. When `h(k, b, 9) < 0.22`, a centred pop of width `bh` (from `(L - bh) >> 1`) inside the block is role 3.
- tabs: pitch `py = th + gy`, `px = tw + gx`. Row `floor(vy / py)`; the gap rows are 0. Odd rows shift by `px >> 1`. Outside the tab width is 0. A tab is 2 when `h(floor(uu / px), row, 3) < 0.12`, else 1.
- zigzag: `uu = ux mod zper`, `tri = |2 uu - zper| / 2`, run at `floor(vy + zslope tri)`.
- diamond: tile `tX = round(ax / dw)`, `tY = round(ay / dh)`, centre `(mx + tX dw, my + tY dh)`. A tile that spills past the frame edge (by more than half a cell) is 0. `md = |ax - tX dw| / (dw/2) + |ay - tY dh| / (dh/2)`, parity `p = (tX + tY) & 1`. `md <= 0.4`: 3 when p else 1. `md <= 0.95`: 1 when p else 2. Else 0.
- cross: tile `ct` square, arm `L = max(2, floor(0.38 ct))`, a tile whose arms spill past the frame is 0. In the arms (`du < ca` and `dv <= L`, or the transpose) the role is 3 when `h(|tX|, |tY|, 4) < 0.15`, else 1 for odd `tX + tY` and 2 for even. Else 0.
- basket: blocks of `bb`. Odd blocks are horizontal stripes, 1 when `floor(vy) mod 2bs < bs`; even blocks vertical stripes, 2 when `floor(ux) mod 2bs < bs`. Else 0.
- rings: run at `floor(max(ux gh / gw, vy))`.
- star: tile `stt`, `R = 0.46 stt`, a tile that spills past the frame is 0. Inside `du + dv <= 0.95R` or `max(du, dv) <= 0.55R`: the centre (`du + dv <= 0.4R` and `max <= 0.4R`) is 3 when p else 2; the rest is 1 when p else 2. Else 0.
- waves: `x = 2 (ux mod wper) / wper - 1`, `bump = wamp sqrt(max(0, 1 - x^2))`, run at `floor(vy + bump)`.
- gingham: `hs = floor(vy / gg) & 1`, `vs = floor(ux / gg) & 1`. Both 2, one 1, none 0.
- burst: `q = max(ux gh / gw, vy)`. Under `bcore` it is 3. Else `k = floor(atan2(vy + 0.5, ux + 0.5) / (π/2) bn)` and the role is `[1, 0, 2, 0][k mod 4]`.
- plaid: `uu = ux mod per`, `vv = vy mod per`. Both under `pw` is 2, one is 1. Else 3 when `floor(uu)` or `floor(vv)` equals `pw + ((per - pw) >> 1)`, else 0.
- dither: `t = 0.5 + 0.5 sin((ux + vy) f1 + warp sin(f2 ux) + ph0)`. Role 1 when `t` is over `(B[(floor(vy) & 3) 4 + (floor(ux) & 3)] + 0.5) / 16`, else 0, with `B` the 4x4 Bayer matrix `0 8 2 10 / 12 4 14 6 / 3 11 1 9 / 15 7 13 5`.
- steps: run at `floor(vy + floor((ux + off) / rw) rh)`.

Rounding of tile indices is JavaScript's `Math.round`, half toward positive infinity, including for negative values.

## Motion

The site has two motion modes: Scroll and Cycle. Scroll is the default and the only one ported: amount 0.8, 60 frames at 10 fps.

- Frame `f` sits at `T = f / frames`.
- `perLoop = max(1, round(3 amount))`, 2 at the default: the number of periods a style travels per Loop.
- A sliding style adds `sh(p) = floor(T perLoop p)` cells to `ux` before its period is read, with `p` its own period. Since `ux` is measured from the centre column, the cloth streams outward from the centre on both sides and stays mirror-symmetric.
  - bands: `uu = ux + sh(L) + shift`. tabs: `uu = ux + sh(px) + offset`. zigzag `sh(zper)`, waves `sh(wper)`, plaid `sh(per)`: `uu = (ux + sh) mod period`. basket: `ux + sh(2 bb)` in both the block and the stripe test. gingham: `ux + sh(2 gg)` in `vs`.
  - rings: the run is read at `floor(q) - sh(rPT)`, so the frames move inward. steps: `vy + sh(PT)` inside the floor, so the diagonals step through the run.
- Tiled styles step a key `key = floor(T max(1, 4 perLoop))`, 8 times per Loop:
  - diamond and star: parity `p = (tX + tY + key) & 1`, so the inner and outer inks swap on each step.
  - cross: the pop hash reads channel `4 + key`, so the pops are dealt again on each step. The 1 and 2 parity holds.
  - burst: `k = floor(angle / (π/2) bn) + floor(4 T perLoop)`, so the rays rotate one ink per step.
- dither: the inner sine gains a phase, `sin(f2 ux + 2π T perLoop)`, so the dither field flows.
- Nothing else moves: the grid, the style, every size and every run hold.
- Most styles close by whole periods: at `T = 1` every `sh(p)` is `2p` and the key is 8.
  - zigzag, waves, plaid, rings, steps, gingham: the shift is a whole number of periods, read mod the period or with the same parity.
  - diamond, star: 8 is even, so the parity is back. burst: 8 is a multiple of 4.
  - basket: the blocks keep their parity, and the vertical stripes, period `2 bs`, close when `2 bs` divides `4 bb`. At chunk 1, `bs = 1`, so they always do.
  - dither: the phase is `4π`, so `sin` is back up to its last bit, which the closure test would catch.
- Three styles do not close, as on the site:
  - bands: each block's ink is hashed from its index, and the shift moves two whole blocks, so the frame after the last shows each block's colours two blocks further in. The Loop jumps at the seam.
  - tabs: the tab layout closes, but the rare role-2 tabs are hashed from the tab index, two tabs further in, so they jump.
  - cross: the pops are hashed on channel 12, not 4, so they are dealt again at the seam.
- At `f = 0` every shift and the key are 0 and the dither phase is 0, so frame 0 is the Still, byte for byte.
- The mid frame is 15, `frames / 4`: half a period along on the sliding styles. Frame 30 is a whole period, which on most styles is frame 0 again. At frame 15 the key is 2, so diamond and star show frame 0's parity there.
- Cycle is not ported. It rotates the three ink roles through fixed steps and is not reachable from a Recipe.

## Painting

Each cell takes its role's ink; the grid is drawn over the frame with image smoothing off.

## Fidelity notes

- Every cell is a hard-edged block, so only the nearest upscale's choice at cell edges can differ.
