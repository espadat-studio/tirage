# mist

A neon spray over a soft pastel ground. Three ground inks blot into each other, then a curtain of tall, ragged fluorescent streaks in the first ink bleeds over them. The field is painted on a small buffer, sprinkled with a fine grain, and scaled up smooth.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id        | range     | step | default | role                                            |
| --------- | --------- | ---- | ------- | ----------------------------------------------- |
| `streaks` | 0.4..2    | 0.05 | 1       | how tightly the streaks pack across the frame   |
| `cover`   | 0.2..0.85 | 0.01 | 0.55    | how much of the frame the neon covers           |
| `soft`    | 0.1..1    | 0.01 | 0.5     | how wide the streak edges fade                  |
| `blot`    | 0.2..1    | 0.01 | 0.5     | how small the ground blots are                  |
| `grain`   | 0..1      | 0.01 | 0.4     | the Tool's own per-pixel grain, not the chassis |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. With motion off every motion offset is zero. The Loop turns motion on at site defaults, and motion is not a Parameter. See [Motion](#motion). Grain and dither are the shared [chassis](chassis.md) post-passes, off by default. Default Palette: `#dcff3a #1e1b3a #3b2a6e #5a2e7a`.

The site has 4 ink swatches with fixed roles: Ink 1 is the neon, Inks 2 to 4 are the ground. A Palette of more than 4 inks is refused, as with aura. A shorter Palette wraps by position over the 4 slots. The site fills missing slots with derived tints instead, so a Palette under 4 inks does not replay on the site.

## Random draws

The same sequential generator as [aura](aura.md): xorshift on 32 bits with the arithmetic middle shift, started from the Tool seed times `2654435761` as a 64-bit float product wrapped to 32 bits, 0 becoming 1.

Draws happen in this order:

1. For each ground ink `i` (Inks 2 to 4), 2 draws: an offset `ox = draw * 37`, then `oy = draw * 43`.
2. The streak offset: `sox = draw * 53`, then `soy = draw * 29`.

## Noise

The hash, value noise, `fbm` and `fbm2` are aura's, channel for channel, with the same noise seed: the Tool seed xor `0x51ed270b`.

## Buffer

As aura: width `max(90, min(700, round(W)))`, height `max(90, round(bw · H / W))`, 700 being the PNG export's budget. Up to 700 px wide the draw is a plain copy; wider, the buffer is scaled up with high-quality smoothing, which Chrome samples bicubic.

## Field

- `th = 1 - 0.9·cover`, `del = 0.06 + 0.24·soft`, `stf = 2.6·streaks + 1.2`, `bsc = 0.9 + 2.2·blot`, `gr = 13·grain`.
- `short = min(bw, bh)`. A buffer pixel `(x, y)` sits at `nx = (x - bw/2) / short · 1.7`, `ny = (y - bh/2) / short · 1.7`, sampled at its top-left corner.
- Ground: at `gx = bsc·nx`, `gy = bsc·ny`, each ground ink `i` reads `nz = fbm2(gx + ox_i, gy + oy_i, 60 + i)`, with `i` counting 0..2 over Inks 2 to 4. Its weight is `max(nz, 0.002) ^ 3.4`. The ground is the weighted mean of the squared inks, `Σ w·c² / Σ w`, summed in ink order, still squared.
- Curtain: `q = fbm(1.9·nx + 3.1, 1.9·ny + 8.7, 91)`. The streak point is `sx = stf·nx + 0.5·(q - 0.5) + sox` and `sy = 0.5·ny + soy`, summed left to right. Its strength is `st = 0.72·fbm(sx, sy, 77) + 0.28·fbm(2.4·sx, 2.1·sy, 78)`. Tall thin streaks come from stretching x by `stf` and squashing y by 0.5.
- The neon share is a smoothstep, `m = s²(3 - 2s)` with `s = clamp((st - e0) / (e1 - e0), 0, 1)`, `e0 = th - del` and `e1 = th + del`. The width is `e1 - e0` as computed, not `2·del`.
- Each squared channel moves toward the squared neon: `c = c + (neon - c)·m`.

## Pixel

- The grain is `g = (hash(x, y, 7, noise seed) - 0.5)·gr`, aura's hash at the integer buffer pixel, channel 7.
- Each channel is `sqrt(c) + g`, stored as a byte, clamped and rounded half to even. Alpha is opaque.
- One `g` per pixel, added to all three channels.

## Motion

The site has three motion modes: Bleed, Drift and Pulse. Bleed is the default and the only one ported: amount 0.8, 72 frames at 12 fps.

- Frame `f` sits at `T = f / frames`. The amount is capped at 1.6 before use, which 0.8 never reaches. The walk radius is `r = 0.45 amount = 0.36` noise units, with `(zx, zy) = (cos 2πT, sin 2πT) r`.
- Both layers walk round circles in noise space, at different reaches:
  - The ground reads at `gx = bsc·nx + zx / 2`, `gy = bsc·ny + zy / 2`: half the radius.
  - The curtain's streak point gains the offset after `sox` and `soy`: `sx = ... + sox + zx`, `sy = 0.5·ny + soy + 0.4 zy`. The full radius across, 0.4 of it down, so the streaks sway sideways more than they rise.
  - The curtain's warp `q` does not move.
- So the neon bleeds across a ground that drifts more slowly beneath it. The random draws, the threshold and the grain hold.
- The walk runs one lap over the Loop, whatever the amount. The amount only sets the radius.
- The Loop closes because the offsets have period 1 in `T`: frame `frames` is frame 0.
- At `f = 0` both layers sit shifted along `x` from their Still place, so frame 0 is not the Still and gets a reference export of its own.
- The mid frame is 36: half a lap, both layers twice their reach from where they are at frame 0, as far as the Loop takes them.
- A Loop frame uses the same 700 px budget as the Still: the PNG export draws the scrubbed frame at 700. The smaller budget the preview uses while playing is ignored.
- Drift slides the curtain alone on an ellipse and Pulse moves its threshold. Neither is ported. They are not reachable from a Recipe.

## Fidelity notes

- At the 540x960 fixture size the buffer is the frame, so the fixtures check the field and the mix, not the upscale.
- The site uses `pow` and `sqrt` only, no trig, so a byte flips by one at most where V8's `pow` differs in the last bit.
- Reference exports use a 4-ink Palette, so the site's swatches equal the Recipe's Palette.
