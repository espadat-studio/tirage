---
title: "parcel"
description: "A two-tone block field with hairline survey grids over it. It renders a Still."
---

A survey map. Blocks of one ink spread across a plain ground on a coarse cell grid, the way plots sit on a land registry sheet. A few small grids of hairlines float on top, each a ragged cluster of cells. By default the lines multiply onto what they cross, so they read darker over the blocks than over the ground.

parcel renders a Still: one frame. Its site page is [playgrnd.tools/parcel](https://www.playgrnd.tools/parcel/).

## Still

![parcel Still for Seed 1](../../../assets/tools/parcel.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool parcel | tirage render --size 540x960 -o parcel.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
parcel  1 frame
  cells       8..=28    step 1
  cover       0.2..=0.8 step 0.01
  chunk       0.5..=2   step 0.05
  grids       0..=8     step 1
  blends      Multiply, Normal
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

A Seed draws `cells`, `cover`, `chunk` and `grids` across their full range. `cells` is how many cells span the frame, `cover` is the share of them the blocks take, and `chunk` decides how big the blobs grow. `grids` is how many hairline clusters there are, so Seed 1, which draws 0, has none. `blends` stays at Multiply. Switch it to Normal and the lines are drawn flat over the blocks. Dither and grain stay off.

The Palette has three inks, one per role: the ground, the blocks and the lines. parcel takes at most 3.
