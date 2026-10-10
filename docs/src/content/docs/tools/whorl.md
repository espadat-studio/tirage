---
title: "whorl"
description: "Op-art stripes round pushing and pulling centres. It renders a Still."
---

Op-art stripes. A handful of centres each add their distance to one field, some counted positive and some negative. The bands of equal total are rings round the centres, curves between them, and pinched eyes where two families meet. A warp bends the field before the bands are read off it.

whorl renders a Still: one frame. Its site page is [playgrnd.tools/whorl](https://www.playgrnd.tools/whorl/).

## Still

![whorl Still for Seed 9](../../../assets/tools/whorl.png)

Seed 9, rendered by:

```sh
tirage derive --seed 9 --tool whorl | tirage render --size 540x960 -o whorl.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
whorl  1 frame
  centres     1..=14    step 1
  pull        0..=1     step 0.01
  push        0..=1     step 0.01
  stripes     2..=90    step 1
  weight      0..=1     step 0.01
  dirs        Smooth, Turbulent, Ripple
  warp        0..=1     step 0.01
  detail      0..=1     step 0.01
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

A Seed draws every Parameter above `ditherTog`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed: `stripes` stays within 2..=30 and `weight` within 0.2..=0.8. Dither and grain stay off. The first ink is the ground, and every later ink is a stripe colour, taken in turn.
