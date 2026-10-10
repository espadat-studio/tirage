---
title: "culture"
description: "Colonies on a culture plate. It renders a Still."
---

Colonies on a culture plate. Each cell drops a soft bump into one field and the bumps add up, so cells that meet fuse into one colony. The field is cut into rings, the plate outside them and each later ink a ring further in, with a grain that bites into the ring edges.

culture renders a Still: one frame. Its site page is [playgrnd.tools/culture](https://www.playgrnd.tools/culture/).

## Still

![culture Still for Seed 14](../../../assets/tools/culture.png)

Seed 14, rendered by:

```sh
tirage derive --seed 14 --tool culture | tirage render --size 540x960 -o culture.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
culture  1 frame
  count       1..=90    step 1
  size        0..=1     step 0.01
  fuse        0..=1     step 0.01
  cover       0..=1     step 0.01
  spread      0.05..=2  step 0.01
  soft        0.02..=1  step 0.01
  textures    Fine, Stipple
  grain       0..=1     step 0.01
  dot         1..=10    step 1
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

A Seed draws every Parameter above `ditherTog` except `grain`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed: `count` stays within 15..=70, `size` within 0.15..=0.6 and `fuse` within 0..=0.8. `grain`, the Tool's own grain, stays at 0.62. Dither and the grain post-pass stay off. The first ink is the plate, and every later ink is a ring further in.
