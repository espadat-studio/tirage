---
title: "riso"
description: "Bands of torn paper shapes printed in two inks, with a big dot, pen scribbles and specks. It renders a Still."
---

A risograph collage. The frame is stacked into horizontal bands, and each band is printed in two inks: a ground and a pattern of torn paper on it. The pattern is ragged blocks, wavy streaks or rows of short scratchy dashes. A big dot with a torn edge may sit over one band, thin pen lines wander across the sheet, and light and dark specks lie over everything.

riso renders a Still: one frame. Its site page is [playgrnd.tools/riso](https://www.playgrnd.tools/riso/).

## Still

![riso Still for Seed 1](../../../assets/tools/riso.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool riso | tirage render --size 540x960 -o riso.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
riso  1 frame
  bands       2..=5     step 1
  rough       0.1..=1   step 0.01
  scrib       0..=1     step 0.01
  dotp        0..=1     step 0.01
  grain       0..=1     step 0.01
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

A Seed draws `bands`, `rough`, `scrib` and `dotp`, each across its full range. `bands` sets how many bands there are and `rough` how torn the edges look. `scrib` sets how many pen lines cross the sheet, up to 3, and `dotp` is the chance of the big dot.

`grain` is riso's own layer of specks, not the shared grain pass. A Seed leaves it at 0.5, as the site page opens it. The shared grain and dither stay off.

The Palette takes any number of inks from 2. Each band picks its ground from it, then a pattern ink that stands out from the ground in lightness. The pen ink and the specks are riso's own colours, not part of the Palette.
