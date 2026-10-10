# coral

A sea fan printed on textile. Ribbed branches grow from a root off the edge of the frame and fork as they spread, so they stay evenly spaced. Torn paper lobes lie over and under them, with a dot screen printed on the paper. Every pixel is worked out at full size, with no buffer.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id         | range  | step | default | role                                            |
| ---------- | ------ | ---- | ------- | ----------------------------------------------- |
| `branches` | 2..12  | 1    | 6       | branches leaving the root                       |
| `spacing`  | 0..1   | 0.01 | 0.5     | the gap the branches keep                       |
| `spread`   | 0..1   | 0.01 | 0.7     | how wide the fan opens                          |
| `width`    | 0.1..1 | 0.01 | 0.6     | branch thickness, as a share of the gap         |
| `wobble`   | 0..1   | 0.01 | 0.4     | how far the branches bend                       |
| `cover`    | 0..1   | 0.01 | 0.5     | how much paper there is                         |
| `size`     | 0..1   | 0.01 | 0.5     | how big the paper lobes are                     |
| `beneath`  | 0..1   | 0.01 | 0.35    | the share of paper that sits under the branches |
| `ribs`     | 0..1   | 0.01 | 0.8     | how strongly the ribs print across the branches |
| `dots`     | 0..1   | 0.01 | 0.7     | dot screen size                                 |
| `shadow`   | 0..1   | 0.01 | 0.6     | how dark and long the shadows are               |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. coral is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off every motion offset is zero.

The [chassis](chassis.md) grain is on by default on this page, a printed textile: blend Multiply, amount 0.45, size 1, specks 0.4, vignette 0.3. Dither is off.

Changing the site's seed deals fresh slider values from it. tirage does not port that deal: `derive` deals the Parameters ([ADR 0003](../adr/0003-seed-derivation-of-recipes.md)), and the Tool seed only feeds the drawing.

## Palette

Variable length, default 5 inks: `#33081c #e8449e #f7b8dc #45be6e #c9f24e`. Inks are read by role, by position:

- ground: ink 1
- branch: ink 2, or ink 1 if the Palette had one ink
- rib: ink 3, else the branch lifted 0.55 of the way to white
- lobe: ink 4, else the branch ink
- dot: ink 5, else the lobe lifted 0.5 of the way to white
- dark: the ground times 0.55, channel by channel

Lifting is `c + (255 - c)·k` per channel, kept as a float. Inks past the fifth are not read. A Palette has at least 2 inks, so the one-ink case never arises.

## Noise and draws

The chassis hash and value noise (`hash(x, y, s)`, bilinear smoothstep `vn(x, y, s)`), on 5 seeds made from the Tool seed `t` read as signed 32-bit: `sd_i = 7·t + i` for `i` in 1..5, wrapped to 32 bits.

One draw stream, the chassis xorshift with logical shifts, started from `t·2246822519` (a 32-bit wrapping multiply) xor `0x27d4eb2f`, 0 becoming 1. It is only used to place the root:

1. `side = draw`.
2. Below 0.6: the root sits below the frame, `rx = W·(0.2 + 0.6·draw)`, `ry = H + U·(0.1 + 0.25·draw)`.
3. Below 0.8: left of the frame, `rx = -U·(0.1 + 0.25·draw)`, `ry = H·(0.3 + 0.5·draw)`.
4. Else right of it, `rx = W + U·(0.1 + 0.25·draw)`, `ry = H·(0.3 + 0.5·draw)`.

`U = sqrt(W·H)` throughout.

## Constants

- The fan points at the frame centre: `θ0 = atan2(H/2 - ry, W/2 - rx)`. Its angle is `range = (0.35 + 0.65·spread)·π`, starting at `θs = θ0 - range/2`.
- Gap `s = (0.032 + 0.075·spacing)·U`, root count `N0 = max(2, branches)`.
- Half width `hwMax = width·s·0.5·0.62`.
- Rib period `ribP = max(1.5, 0.11·s)`.
- Paper shadow throw `shOff = 0.012·U·shadow`, read back at `(round(shOff), round(0.7·shOff))` pixels, rounding half up. Branch shadow throw `gap = 0.1·s·shadow`.
- Lobe size `Lsz = U·(0.08 + 0.16·size)`, paper threshold `thr = 0.62 - 0.16·cover`, dot cell `cell = max(2, 0.0095·U)`.

## Cached fields

Both are stored as 32-bit floats and read back with plain bilinear interpolation.

- The lobe field, on a 2 px grid of `(W >> 1) + 2` by `(H >> 1) + 2` nodes. Node `(i, j)` sits at `(x, y) = (2i, 2j)`. Rotated onto the fan, `xr = (x cos θ0 + y sin θ0) / (1.9·Lsz)` and `yr = (-x sin θ0 + y cos θ0) / Lsz`. The value is `0.58·vn(xr, yr, sd2) + 0.3·vn(2.3·xr + 3.1, 2.3·yr + 1.7, sd3) + 0.12·vn(6.1·xr + 9, 6.1·yr + 4, sd3)`. A pixel reads it at `(x/2, y/2)`, the cell index truncated.
- Two slow fields on an 8 px grid of `ceil(W/8) + 1` by `ceil(H/8) + 1` nodes at `(8i, 8j)`: the under field `vn(x/(3·Lsz) + 11, y/(3·Lsz) + 5, sd4)` and the dot weight `vn(x/(0.16·U) + 23, y/(0.16·U) + 9, sd5)`.

## Per pixel

### Where it sits on the fan

- `dx = x - rx`, `dy = y - ry`, `r = sqrt(dx² + dy²)`, `θ = atan2(dy, dx)`.
- If `wobble > 0`, with `far = min(1, r/(0.6·U))`, two bends in turn, the second reading the bent `θ`:
  - `θ += 0.55·wobble·far·(vn(r/(0.38·U) + 7, 1.4·θ, sd1) - 0.5)`
  - `θ += 0.22·wobble·far·(vn(r/(0.16·U) + 3, 4.2·θ + 5, sd1) - 0.5)`
- `a = θ - θs` wrapped into [0, 2π) by a float remainder, twice.
- Defaults: `inB = false`, `arc = 1e9`, `hw = 1`, `side = 0`, `bid = 0`.
- Only when `a ≤ range` and `r > 1`:
  - `L = log2(r·range / (N0·s))`. Below 0: `Nf = N0`, `ease = 1`, not forking, `n = 0`. Else `n = floor(L)`, `Nf = N0·2^(n+1)`, `ease` the smoothstep of `L - n`, forking.
  - `u = a/range·Nf`, `sp = r·range/Nf`, `best = 1e9`.
  - For branch `k` from `floor(u) - 1` to `floor(u) + 2`, skipping `k < 0` and `k > Nf`:
    - `centre = k`, `sc = 1`. When forking and `k` is odd, the young branch slides out of its parent: `parent = k - 1` if bit 1 of `k` is set, else `k + 1`. Then `centre = parent + (k - parent)·ease` and `sc = 0.22 + 0.78·ease`.
    - `id = k/Nf`, `d = |u - centre|·sp`. Skip when `d ≥ best·hwMax·1.2`.
    - When forking, the branch's level: `q = k`, `lvl = n + 1`, halving `q` and dropping `lvl` while `lvl > 0` and `q` is even. When `lvl > 0` and `hq = hash(round(8192·id), 9, t) < 0.28` the branch stops short: `rend = (N0·s/range)·2^(lvl-1)·(1.5 + 6·hq)` and `e = (rend - r) / max(1, hwMax·sc)`. Skip when `e < 0`; when `e < 1`, `sc *= sqrt(max(0, 1 - (1 - e)²))`, a rounded end.
    - The tube breathes: `h = hwMax·sc·(1 + 0.2·sin(r/(0.85·s) + 137.5·id))`, `ratio = d / max(0.5, h)`.
    - When `ratio < best`: `best = ratio`, `arc = d`, `hw = h`, `side = 1` if `u ≥ centre` else -1, `bid = round(8192·id)`.
  - `inB = best < 1`.

### The paper

- `fv` is the lobe field here, `inL = fv > thr`, `under =` the under field here `< beneath`.
- The shadow is read at `(sx, sy) = (x - offI, y - offJ)`: `fs` is the lobe field there when both are at least 0, else 0.
- `lobeSh = shadow > 0`, not `inL`, and `fs > thr`. `lobeShUnder = lobeSh` and the under field at `(sx, sy) < beneath`.

### Composing, ground up

Mixing toward a colour by `k` is `c + (target - c)·k` per channel.

1. Start at the ground.
2. `lobeShUnder`: mix toward dark by `0.55·shadow`.
3. `inL` and `under`: the lobe ink, on paper.
4. `inB`: the branch ink, off paper.
   - If `ribs > 0`: `rv = 0.5 + 0.5·sin(r/ribP·2π + 6.28·hash(bid, 3, t))`. The phase factor is 6.28, not 2π. When `rv > 0.7` and `arc < 0.86·hw`, mix toward the rib by `ribs·min(1, 8·(rv - 0.7))`.
   - If `shadow > 0`, `side > 0` and `arc > 0.58·hw`: mix toward dark by `0.55·shadow·min(1, (arc - 0.58·hw)/(0.42·hw))`, a rim.
5. Else, if `shadow > 0`, `side > 0` and `arc < hw + gap`: mix toward dark by `0.5·shadow`, the branch's shadow.
6. `lobeSh` and not `lobeShUnder`: mix toward dark by `0.6·shadow`.
7. `inL` and not `under`: the lobe ink, on paper.
8. On paper and `dots > 0`: `ddx`, `ddy` are the fractional parts of `x/cell` and `y/cell` minus 0.5. `edge = clamp((thr + 0.13 - fv)/0.13, 0, 1)`. `rad = 0.5·dots·(0.3 + 0.4·edge + 0.3·w)` with `w` the dot weight here. Inside `ddx² + ddy² < rad²` the pixel is the dot ink.

Channels are stored as bytes, clamped and rounded half to even. Alpha is opaque. Then the chassis grain, then dither.

## Fidelity notes

- The cached fields are 32-bit floats; the port must round to f32 at the same points or the paper edge drifts.
- Every pixel calls `atan2`, `sin` and `log2`. V8's results may differ in the last bit, which can flip a rare pixel on a branch or paper edge.
