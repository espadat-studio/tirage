# culture

Colonies on a culture plate. Each cell drops a soft bump into one field and the bumps add up, so cells that meet fuse into one colony. The field is cut into rings, the plate outside them and each later ink a ring further in, with a grain that bites into the ring edges.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id         | range         | step | default | role                                   |
| ---------- | ------------- | ---- | ------- | -------------------------------------- |
| `count`    | 1..90         | 1    | 36      | how many cells                         |
| `size`     | 0..1          | 0.01 | 0.4     | how big a cell is                      |
| `fuse`     | 0..1          | 0.01 | 0.62    | how far cells widen into each other    |
| `cover`    | 0..1          | 0.01 | 0.72    | how far the outer ring reaches         |
| `spread`   | 0.05..2       | 0.01 | 1.7     | how the inner rings stack              |
| `soft`     | 0.02..1       | 0.01 | 0.3     | how wide the crossing between rings is |
| `textures` | Fine, Stipple |      | Fine    | grain per pixel, or per dot            |
| `grain`    | 0..1          | 0.01 | 0.62    | the Tool's own grain, not the chassis  |
| `dot`      | 1..10         | 1    | 2       | the stipple dot size, in pixels        |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. culture is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off the drift, growth and colour roll are 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

## Palette

Variable length, default 3 inks: `#f6d8b0 #e4572e #17bebb`. Ink 1 is the plate; every later ink is a ring further in.

## Noise and draws

The chassis hash, on seeds `t + 29` and `t + 71`, with `t` the Tool seed read as signed 32-bit and the sums wrapped to 32 bits.

One draw stream, the chassis xorshift with logical shifts, started from `t·2654435761` as a 32-bit wrapping multiply, 0 becoming 1. For each of `max(1, count)` cells, in order: `cx = (-0.15 + 1.3·draw)·FW`, `cy = (-0.15 + 1.3·draw)·FH`, a motion phase `draw·2π`, then a radius factor `0.55 + 0.9·draw`. The phase is drawn whatever the motion.

## Field

A grid of `FW` by `FH` 32-bit floats, zeroed, `aspect = H/W`:

- `FW = max(8, round(560/aspect))` when `aspect > 1`, else 560. `FH = max(8, round(FW·aspect))`. Rounding is half up.
- `base = (0.05 + 0.2·size)·(0.75 + 0.8·fuse)·sqrt(FW·FH)·0.87`, the products taken left to right.
- Each cell has radius `r = base·factor`. It touches columns `max(0, floor(cx - r))` to `min(FW - 1, ceil(cx + r))` and the rows alike, and is skipped when that box is empty.
- Each node `(i, j)` in the box with `d² = ((i - cx)² + (j - cy)²)/r² < 1` adds `(1 - d²)²`. The sum is rounded to f32 after every add.

## Rings

`n` is the ink count. Boundaries are placed by radius and turned back into field values:

- `t0 = clamp(0.9·(1 - cover), 0.002, 0.998)`, `dEdge = sqrt(max(0, 1 - sqrt(t0)))`.
- `TH[0] = 0`. For `k` in `1..n-1`: `D = dEdge·((n - k)/(n - 1))^spread`, `TH[k] = (1 - D²)²`.
- `TH[n] = max(TH[n-1] + 1e-4, 1.55)`.

## Per pixel

- `soft' = max(0.001, soft)`, `gq = 1.05·grain`, `gl = 46·grain`, `dot' = max(1, round(dot))`.
- The field is read bilinear at `(x·FW/W, y·FH/H)`: the cell index truncated and clamped to the last node first, the fraction taken from the clamped index, the far node clamped too.
- The ring position `u`: below `TH[1]` it is `v/TH[1]`, or 0 when `TH[1]` is 0. Otherwise step `k` up from 1 while `k < n - 1` and `v ≥ TH[k+1]`; then `u = k + (v - TH[k])/(TH[k+1] - TH[k])`, the fraction 0 when the gap is not positive.
- The grain position is the pixel for Fine, or `(trunc(x/dot'), trunc(y/dot'))` for Stipple.
- When `gq > 0.002`, `u += (hash(gx, gy, t + 29) - 0.5)·gq`.
- `u` is clamped to `0..n-1`. `i0 = min(n - 2, trunc(u))`, `fr = u - i0`.
- The crossing sits at the top of each band: `fr = clamp((fr - (1 - soft'))/soft', 0, 1)`, then smoothstepped.
- The colour mixes ink `i0` toward ink `i0 + 1` by `fr`, per channel.
- When `gl > 0.002`, one value `(hash(gx, gy, t + 71) - 0.5)·gl` is added to all three channels.
- Channels are stored as bytes, clamped and rounded half to even. Alpha is opaque.

## Fidelity notes

- The field is a 32-bit float sum; the port must round after every add or ring edges move.
- The per-pixel path has no trig, only `pow` per ring and `sqrt`, so it should match to the byte.
