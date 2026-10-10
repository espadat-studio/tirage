# sear

A smeared thermal image. A warped noise field is spread flat so each ink of the ramp covers about the same share of the frame, then posterised into blocks. Each row holds a colour along a run until the field changes enough, so edges drag into streaks, and some bands of rows tear sideways.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range | step | default | role                                     |
| -------- | ----- | ---- | ------- | ---------------------------------------- |
| `scale`  | 0..1  | 0.01 | 0.62    | how big the heat blobs are               |
| `warp`   | 0..1  | 0.01 | 0.35    | how far the field is bent                |
| `detail` | 1..5  | 1    | 4       | noise octaves                            |
| `smear`  | 0..1  | 0.01 | 0.35    | how long a held colour runs along a row  |
| `drag`   | 0..1  | 0.01 | 0.22    | how far rows are pulled sideways         |
| `tear`   | 0..1  | 0.01 | 0.3     | how many bands of rows tear, and how far |
| `steps`  | 2..24 | 1    | 6       | posterise levels                         |
| `grain`  | 0..1  | 0.01 | 0.1     | the Tool's own grain, not the chassis    |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. sear is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off every orbit, phase and roll is 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default. `size` on this page is the export width, not a Parameter.

## Palette

Variable length, default 6 inks: `#03071e #370617 #9d0208 #e85d04 #faa307 #ffe066`. The Palette is a ramp, coolest first: the lowest heat takes ink 1 and the peaks the last.

## Noise

The chassis hash and value noise, `vn(x, y, s)`. Seeds are the Tool seed `t` read as signed 32-bit, plus an offset, wrapped to 32 bits.

`fbm(x, y, s, oct)` sums `oct` octaves: octave `i` adds `a·vn(f·x, f·y, s + 131·i)`, starting at `a = 0.5`, `f = 1`, then `f *= 2.05` and `a *= 0.58` each octave. The sum is divided by the sum of the `a`s.

No draw stream: sear makes no sequential random draws.

## Heat grid

A grid of `FW` by `FH` 32-bit floats, `aspect = H/W`:

- `FW = max(8, round(640/aspect))` when `aspect > 1`, else 640. `FH = max(8, round(FW·aspect))`. Rounding is half up.
- `oct = clamp(round(detail), 1, 5)`, `span = 0.7 + 7.5·(1 - scale)`, `k = 2.6·warp`.
- Node `(i, j)` sits at `u = (i/FW)·span`, `v = (j/FH)·span·aspect`.
- Two slow warps: `wx = fbm(0.55·u + 11.3, 0.55·v + 4.1, t + 7, 2) - 0.5` and `wy = fbm(0.55·u + 2.7, 0.55·v + 19.7, t + 13, 2) - 0.5`.
- Heat `h = fbm(u + k·wx, v + k·wy, t, oct)`, stored as f32. The lowest and highest heat, `lo` and `hi`, are tracked on the unrounded values.

### Flattening

- `r = 1/(hi - lo)` when `hi - lo > 1e-4`, else 1.
- Each node's bin is `trunc((h - lo)·r·511)`, from the stored f32 `h`, out of 512 bins. A histogram counts them.
- Walking bins in order with a running count `run`, bin `b` maps to `raw + (eq - raw)·0.88`, with `raw = b/511` and `eq = (run + count_b/2)/(FW·FH)`. The map is stored as f32. Then `run += count_b`.
- Each node is replaced by its bin's map value.

## Ramp

A 256-entry lookup of bytes:

- `steps = max(2, round(steps))`, `n` the ink count.
- Entry `i` posterises to `t = min(steps - 1, floor(i/255·steps)) / (steps - 1)`.
- `u = clamp(t, 0, 1)·(n - 1)`, `k = min(n - 2, floor(u))`, `fr = u - k`. Each channel is `round(ink_k + (ink_{k+1} - ink_k)·fr)`, rounding half up.

## Rows

Per frame:

- `maxRun = max(2, round((0.02 + 1.2·smear²)·W))`, `thresh = 0.0015 + 0.22·(1 - smear)^2.2`.
- `dragPx = 0.3·drag·W`, `bandH = max(2, round(0.028·H))`, `kx = FW/W`, `gain = 0.34·grain`.

For each row `y`:

- `band = floor(y/bandH)`. It is torn when `hash(band, 77, t + 3) < 0.62·tear`. A torn band shifts by `(hash(band, 91, t + 5) - 0.5)·W·1.4·tear`, holds runs as long as the frame, and needs a change `0.45·tear` larger to break one. An untorn row has no shift, `maxRun` and `thresh`.
- The grid row: `gy = (y/H)·FH`, `y0 = floor(gy)` clamped to `0..FH-1`, `fy = gy - floor(gy)`, `y1 = min(y0 + 1, FH - 1)`.
- `pull = dragPx·(0.15 + 0.85·vn(0.014·y, 0.37·band, t + 17))·(0.5 + 0.5·sin(0.031·y))`.

For each pixel `x` on the row:

- `sx = max(0, x - pull)`, `gx = (sx + shift)·kx`, wrapped into `[0, FW)` as `gx - floor(gx/FW)·FW`.
- `x0 = trunc(gx)`, `fx = gx - x0`, then `x0` clamped to `FW - 1`. `x1 = x0 + 1`, wrapping to 0 past the last column.
- `v` is the bilinear read: across `x0..x1` by `fx` on rows `y0` and `y1`, then down by `fy`.
- The held value starts unset each row. It takes `v` when unset, when `|v - held| > threshold`, or when the run has reached its length; the run then restarts at 0. Each pixel adds 1 to the run.
- When `gain > 0.001`, `q = held + (hash(x, y, t + 29) - 0.5)·gain`, else `held`.
- The ramp entry is `trunc(255·q)` clamped to `0..255`. Alpha is opaque.

## Fidelity notes

- The grid and the map are 32-bit floats; the port must round at the same points or the flat blocks shift by a level.
- `sin` and `pow` are called once per row, so last-bit differences from V8 are rare and move a whole row's pull by a fraction of a pixel.
