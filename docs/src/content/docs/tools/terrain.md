---
title: "terrain"
description: "A warped noise field cut into flat bands of colour. It renders a Still."
---

A warped noise field cut into flat bands, one band per ink, like a contour map with no lines. Grain shakes each pixel across band edges, and a block size can coarsen the map into square cells.

terrain renders a Still: one frame. Its site page is [playgrnd.tools/terrain](https://www.playgrnd.tools/terrain/).

## Still

![terrain Still for Seed 7](../../../assets/tools/terrain.png)

Seed 7, rendered by:

```sh
tirage derive --seed 7 --tool terrain | tirage render --size 540x960 -o terrain.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
terrain  1 frame
  scale       0.6..=9   step 0.1
  warp        0..=3     step 0.02
  oct         1..=7     step 1
  contrast    0.5..=4   step 0.05
  balance     -1.2..=1.2 step 0.05
  grain       0..=1.2   step 0.01
  block       0..=14    step 1
  ditherTog   on/off
  dthKinds    Bayer 8, Bayer 4, Noise
  dthSize     1..=10    step 1
  dthLevels   2..=8     step 1
  dthAmount   0..=1     step 0.05
  grainTog    on/off
  grnBlends   Add, Overlay, Soft light, Multiply, Screen
  grnAmount   0..=1     step 0.05
  grnSize     0.5..=4   step 0.1
  grnSpecks   0..=1     step 0.05
  grnVignette 0..=1     step 0.05
```

A Seed draws `scale`, `warp`, `oct`, `contrast`, `balance` and `block`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. `grain`, the Tool's own per-pixel noise, stays at 0.3. Dither and the grain post-pass stay off.

The Palette takes any number of inks from 2, one band each, read in order.
