# benday

A turbulent heat field cut into flat bands along an ink ramp, seen through a turned screen of ringed dots. Dots grow where the field is cold. Each dot is a dark ring around a paper-tinted core, so the sheet reads as a cheap thermal print.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range | step | default | role                                         |
| -------- | ----- | ---- | ------- | -------------------------------------------- |
| `turb`   | 0..1  | 0.01 | 0.7     | how far the field is folded by its warps     |
| `streak` | 0..1  | 0.01 | 0.5     | how far the field is stretched along `dir`   |
| `dir`    | -1..1 | 0.01 | 0.35    | streak direction, -90° to 90°                |
| `scale`  | 0..1  | 0.01 | 0.5     | field feature size                           |
| `black`  | 0..1  | 0.01 | 0.35    | how much of the field falls to ink 0         |
| `bands`  | 2..16 | 1    | 8       | how many bands the ramp is cut into          |
| `rims`   | 0..1  | 0.01 | 0.12    | width of the hotter rim where bands meet     |
| `dot`    | 0..1  | 0.01 | 0.45    | screen cell size                             |
| `ring`   | 0..1  | 0.01 | 0.6     | core radius as a share of the dot            |
| `angle`  | 0..1  | 0.01 | 0.25    | screen angle, 0° to 80°                      |
| `bite`   | 0..1  | 0.01 | 0.5     | how fast dots shrink as the band gets hotter |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. benday is a Still Tool: only frame 0 is rendered, and with motion off the drift, churn and heat offsets are all 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

Default Palette, 8 inks: `#0a0a0c #5b1e9e #2c4be8 #31c6f0 #8fe23a #f5f03a #ff8a1e #ff2a1e`. The Palette is a ramp read by position, cold to hot, and takes any number of inks from 2. The site deals its Palette from the Tool seed together with every slider. We do not port that deal: a Recipe names its Palette, and the refs export sets it through My colors.

A Seed deals all 11 sliders.

## Random draws

None. All randomness is the chassis value noise, at four seeds `7 s + 1` to `7 s + 4` for Tool seed `s`, wrapping at 32 bits.

## Field

With `W x H` the frame and `U = sqrt(W H)`:

- `sc = U (0.25 + 0.7 scale)`.
- The streak angle is `dir π / 2`, with cosine `cd` and sine `sd`. `stretch = 1 + 3.5 streak`.
- `black' = 0.6 black`. The band count is `max(2, floor(bands))`. The rim width is `rw = 0.5 rims`.

Each pixel `(x, y)`:

1. `u = (x - W/2) / sc`, `v = (y - H/2) / sc`.
2. Rotate into the streak frame and stretch: `ru = (u cd + v sd) / stretch`, `rv = -u sd + v cd`.
3. Two warps: `q1 = noise(1.5 ru + 3.1, 1.5 rv + 7.7, seed 1)`, `q2 = noise(1.5 ru + 9.2, 1.5 rv + 1.3, seed 2)`. Then `wu = ru + 3 turb (q1 - 0.5)` and `wv = rv + 3 turb (q2 - 0.5)`.
4. `m = 0.65 noise(2 wu, 2 wv, seed 3) + 0.35 noise(4.7 wu, 4.7 wv, seed 4)`.
5. `val = 1.9 (m - 0.5) + 0.5`.

## Bands

The ramp at `t` in 0..1: `p = clamp(t, 0, 1) (n - 1)` for `n` inks, `k = min(n - 2, floor(p))`, and the colour is ink `k` lerped toward ink `k + 1` by `p - k`. Channels stay unrounded until the end.

- When `val < black'`, the band level `b` is 0 and the colour is ink 0.
- Otherwise `t = min(0.9999, (val - black') / (1 - black'))`. With `B` bands, `kb = floor(t B)`, `fb = t B - kb` and `b = (kb + 0.5) / B`. The colour is the ramp at `kb / (B - 1)`.
- When `rw > 0` and `fb < rw` or `fb > 1 - rw`, the colour is the ramp at `min(1, (kb + 2) / (B - 1))` instead: a rim of a hotter ink.

## Screen

- The cell is `max(2, U (0.006 + 0.03 dot))` px.
- The screen angle is `80 angle` degrees, with cosine `ca` and sine `sa`.
- Dot radius range `rmin = 0.13`, `rmax = 0.7`, and `γ = 1 / (0.45 + 1.4 bite)`. The core share is `0.8 ring`.
- Paper is `(244, 241, 232)`.

Each pixel, after its band colour `C`:

1. Grid coordinates `gx = (x ca + y sa) / cell`, `gy = (-x sa + y ca) / cell`. `d` is the distance from `(gx, gy)` to its cell centre, in cell units.
2. The dot radius is `r = rmin + (rmax - rmin) (1 - b)^γ`.
3. `edge = (r - d) cell + 0.5`. When it is 0 or less, the pixel keeps `C`.
4. Otherwise the dot colour starts at ink 0, the ring. The core edge is `ce = (0.8 ring r - d) cell + 0.5`. When `ce > 0`, the dot colour moves toward the core by `min(1, ce)`. The core is `C` moved toward paper by 0.7, or by 0.22 when `b` is 0.
5. The pixel moves from `C` toward the dot colour by `min(1, edge)`.

Channels are clamped to 0..255 and stored rounding half to even. Alpha is 255.

## Fidelity notes

- The field uses `sin`, `cos` and `sqrt`; `(1 - b)^γ` uses `pow`. Last-bit differences from V8 can flip a band edge or an anti-aliased dot rim by one level, well under the luma cutoff.
