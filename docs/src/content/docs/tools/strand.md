---
title: "strand"
description: "Chains of torn rods printed in two plates that slipped apart, so a dark rim shows beside each one. It renders a Still."
---

Chains of short rods, printed as if by two plates that slipped apart. Each chain wanders across the sheet and now and then splits in two. Its rods are torn, lopsided shapes with a notch at every joint. The plate underneath shows as a thick rim on one side of each rod and a hairline on the other. The fill on top breaks up near its edges into specks, streaks or a dot screen.

strand renders a Still: one frame. Its site page is [playgrnd.tools/strand](https://www.playgrnd.tools/strand/).

## Still

![strand Still for Seed 1](../../../assets/tools/strand.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool strand | tirage render --size 540x960 -o strand.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
strand  1 frame
  count       1..=40    step 1
  len         3..=90    step 1
  wander      0..=1     step 0.01
  branch      0..=1     step 0.01
  thick       0..=1     step 0.01
  rod         0..=1     step 0.01
  notch       0..=1     step 0.01
  rough       0..=1     step 0.01
  offset      0..=1     step 0.01
  edge        0..=1     step 0.01
  tex         0..=1     step 0.01
  texKinds    Stipple, Drag, Screen
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

A Seed draws every Parameter above `grain`. `count` sets how many chains there are and `len` how many rods each one lays. `wander` is how far a chain turns at each rod and `branch` how often it splits. `thick` sets the rod width, `rod` its length and `notch` the gap between rods. `rough` tears the edges and makes each rod lopsided. `offset` is how far the plate slid and `edge` how much fatter than the fill it is. `tex` is how much of the fill fails to print, and `texKinds` the shape of the gaps: loose specks, streaks dragged one way, or a dot screen. `grain` is strand's own noise along the edges and over every pixel, not the shared grain pass. It stays at 0.42.

A Seed keeps `count` between 4 and 20, `len` between 6 and 32 and `thick` up to 0.8. Long chains of thick rods cover the whole frame in fill. Every other Parameter can take its full range. The shared grain and dither stay off.

The Palette has 3 inks by role: Ground, Plate and Fill. strand takes at most 3. With 2 inks, the fill prints in the plate ink, as it does on the site.
