---
title: "bloom"
description: "Pixel rings spreading from a quiet centre. It renders a Still."
---

Pixel rings. A grid of square cells is coloured by its distance from the middle of the frame, cut into bands that step through the Palette. Noise pushes the rings out of round and roughens their edges, and a quiet disc in the middle keeps the first ink. The grid is mirrored left to right and top to bottom.

bloom renders a Still: one frame. Its site page is [playgrnd.tools/bloom](https://www.playgrnd.tools/bloom/).

## Still

![bloom Still for Seed 1](../../../assets/tools/bloom.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool bloom | tirage render --size 540x960 -o bloom.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
bloom  1 frame
  cols        8..=96    step 1
  rings       1..=14    step 1
  warp        0..=1     step 0.01
  grain       0..=1     step 0.01
  steps       2..=14    step 1
  calm        0..=1     step 0.01
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

A Seed draws `cols`, `rings`, `warp`, `steps` and `calm`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed: `rings` stays within 4..=14, `steps` within 2..=8 and `calm` within 0..=0.5. `grain`, bloom's own roughness on the band edges, stays at 0.3. Dither and the grain pass stay off. The Palette is a ramp from the deepest band to the palest, and every ink is used whatever `steps` is.
