---
title: "quilt"
description: "Pixel quilt cloth in four inks, mirrored about the centre. It renders a Still."
---

Pixel quilt cloth. One pattern fills a chunky grid, measured from the centre so the sheet mirrors both ways, and the grid is blown up with hard edges. The patterns are bands, tabs, plaid, dither, steps, zigzag, diamond, cross, basket, rings, star, waves, gingham and burst.

quilt renders a Still: one frame. Its site page is [playgrnd.tools/quilt](https://www.playgrnd.tools/quilt/).

## Still

![quilt Still for Seed 1](../../../assets/tools/quilt.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool quilt | tirage render --size 540x960 -o quilt.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
quilt  1 frame
  styles      Auto, Bands, Tabs, Plaid, Dither, Steps, Zigzag, Diamond, Cross, Basket, Rings, Star, Waves, Gingham, Burst
  cells       28..=72   step 2
  chunk       0.7..=1.8 step 0.05
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

A Seed draws `styles`, `cells` and `chunk`, each across its full range. `styles` picks the pattern, and Auto lets the Tool seed pick one. `cells` is how many grid columns span the frame, and `chunk` scales every motif. Dither and grain stay off.

The Palette is four inks by role, in order: base cloth, two weave colours and a pop accent. quilt takes at most 4.
