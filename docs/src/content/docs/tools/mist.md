---
title: "mist"
description: "A neon spray over a soft pastel ground. It renders a Still."
---

A neon spray over a soft pastel ground. Three ground inks blot into each other, then a curtain of tall, ragged streaks in the first ink bleeds over them. The field is painted on a small buffer with a fine grain and scaled up smooth.

mist renders a Still: one frame. Its site page is [playgrnd.tools/mist](https://www.playgrnd.tools/mist/).

## Still

![mist Still for Seed 6](../../../assets/tools/mist.png)

Seed 6, rendered by:

```sh
tirage derive --seed 6 --tool mist | tirage render --size 540x960 -o mist.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
mist  1 frame
  streaks     0.4..=2   step 0.05
  cover       0.2..=0.85 step 0.01
  soft        0.1..=1   step 0.01
  blot        0.2..=1   step 0.01
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

A Seed draws `streaks`, `cover`, `soft`, `blot` and `grain`, the Tool's own per-pixel noise. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. Dither and the grain post-pass stay off. mist takes at most 4 inks: the first is the neon, the other three the ground.
