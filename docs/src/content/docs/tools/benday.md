---
title: "benday"
description: "A heat map printed through a screen of ringed dots. It renders a Still."
---

A turbulent heat field cut into flat bands along a ramp of inks, seen through a turned screen of ringed dots. Dots grow where the field is cold. Each dot is a dark ring around a pale core, so the sheet reads like a cheap thermal print.

benday renders a Still: one frame. Its site page is [playgrnd.tools/benday](https://www.playgrnd.tools/benday/).

## Still

![benday Still for Seed 6](../../../assets/tools/benday.png)

Seed 6, rendered by:

```sh
tirage derive --seed 6 --tool benday | tirage render --size 540x960 -o benday.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
benday  1 frame
  turb        0..=1     step 0.01
  streak      0..=1     step 0.01
  dir         -1..=1    step 0.01
  scale       0..=1     step 0.01
  black       0..=1     step 0.01
  bands       2..=16    step 1
  rims        0..=1     step 0.01
  dot         0..=1     step 0.01
  ring        0..=1     step 0.01
  angle       0..=1     step 0.01
  bite        0..=1     step 0.01
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

A Seed draws all 11 art Parameters. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. Dither and the grain post-pass stay off.

The Palette is a ramp read cold to hot and takes any number of inks from 2. The first ink is also the dot ring.
