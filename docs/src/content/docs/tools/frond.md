---
title: "frond"
description: "A paste-up of a plant. It renders a Loop of 36 frames at 12 fps."
---

A paste-up of a plant. Soft shadows and a few rounded paper cut-outs sit on a sheet. Torn blocks of ordered 1-bit noise bite in from the edges. Over them a plant is drawn as a broken line of tiny squares and diamonds, then a few thin registration circles and rules go over the top.

frond renders a Loop: 36 frames at 12 fps. Each frame deals the marks of the plant and the noise blocks again, while the cut-outs, the pot and the registration stay put. Frame 0 is the Still. Its site page is [playgrnd.tools/frond](https://www.playgrnd.tools/frond/).

## Still

![frond Still for Seed 5](../../../assets/tools/frond.png)

Seed 5, rendered by:

```sh
tirage derive --seed 5 --tool frond | tirage render --size 540x960 -o frond.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
frond  36 frames
  dirs        Potted, Bouquet, Fronds
  masses      0..=6     step 1
  size        0..=1     step 0.01
  round       0..=1     step 0.01
  growth      0..=1     step 0.01
  detail      0..=1     step 0.01
  coarse      0..=1     step 0.01
  breakup     0..=1     step 0.01
  noise       0..=1     step 0.01
  patch       0..=1     step 0.01
  circles     0..=1     step 0.01
  rules       0..=1     step 0.01
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

A Seed draws `dirs`, `masses`, `size`, `round`, `growth`, `detail`, `coarse`, `breakup`, `noise`, `patch`, `circles` and `rules`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. Dither and grain stay off.
