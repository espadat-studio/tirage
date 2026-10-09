---
title: "husk"
description: "Scattered seed husks on a plain ground. It renders a Still."
---

Scattered seed husks on a plain ground. Each husk is a lumpy oval with a dark rim. Inside the rim a light fill is bitten away, either into ragged islands or into a dot screen that thins toward the edge. Where husks overlap, their dark parts merge but their fills stay apart.

husk renders a Still: one frame. Its site page is [playgrnd.tools/husk](https://www.playgrnd.tools/husk/).

## Still

![husk Still for Seed 2](../../../assets/tools/husk.png)

Seed 2, rendered by:

```sh
tirage derive --seed 2 --tool husk | tirage render --size 540x960 -o husk.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
husk  1 frame
  count       1..=70    step 1
  size        0..=1     step 0.01
  vary        0..=1     step 0.01
  lump        0..=1     step 0.01
  bites       Crumble, Dots
  eat         0..=1     step 0.01
  tex         0..=1     step 0.01
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

A Seed draws `count`, `size`, `vary`, `lump`, `eat` and `tex`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. `bites` stays at Crumble, and `grain`, the Tool's own per-pixel noise, at 0.3. Dither and the grain post-pass stay off.
