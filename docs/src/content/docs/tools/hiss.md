---
title: "hiss"
description: "Pointed leaves full of television snow over a marbled colour field. It renders a Still."
---

Pointed leaves full of television snow, stacked down a marbled colour field. Black blots break up the marble, thin comb stripes fringe two corners, checkered lenses sit on the seams between leaves, and small white asterisks are scattered on top.

hiss renders a Still: one frame. Its site page is [playgrnd.tools/hiss](https://www.playgrnd.tools/hiss/).

## Still

![hiss Still for Seed 1](../../../assets/tools/hiss.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool hiss | tirage render --size 540x960 -o hiss.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
hiss  1 frame
  leaves      1..=5     step 1
  size        0.4..=1.4 step 0.01
  overlap     0..=0.5   step 0.01
  tilt        -1..=1    step 0.01
  shift       0..=1     step 0.01
  swirl       0..=1     step 0.01
  black       0..=1     step 0.01
  scale       0..=1     step 0.01
  checkers    0..=4     step 1
  steps       0..=1     step 0.01
  comb        0..=1     step 0.01
  marks       0..=24    step 1
  flecks      0..=1     step 0.01
  coarse      0..=1     step 0.01
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

A Seed draws every one of the 14 sliders above the chassis across its full range. `leaves` is how many leaves stack down the frame, and `size` how wide they are. `swirl`, `black` and `scale` shape the marble. `checkers` is how many lenses, `comb` how far the corner stripes reach, and `marks` how many asterisks. `steps` cuts every edge into stair steps. `flecks` prints some snow grains in an ink, and `coarse` makes the snow grains bigger. Dither and grain stay off.

The Palette takes any number of inks from 2 and colours the marble, the comb stripes and the flecks. The snow is grey, the lenses black and white, and the asterisks off-white; none of them is part of the Palette.
