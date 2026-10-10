---
title: "optic"
description: "Even vertical bars that switch direction inside up to three nested figures. It renders a Still."
---

Even vertical bars fill the frame. Up to three nested figures sit on its centre, and inside each one the bars run again, moved against the bars around it. They shift half a bar, turn to horizontal, or tilt 45° either way.

optic renders a Still: one frame. Its site page is [playgrnd.tools/optic](https://www.playgrnd.tools/optic/).

## Still

![optic Still for Seed 1](../../../assets/tools/optic.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool optic | tirage render --size 540x960 -o optic.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
optic  1 frame
  styles      Auto, Diamond, Circle, Peak, Square
  count       6..=28    step 1
  weight      0.3..=0.7 step 0.01
  sizeF       0.4..=1   step 0.01
  levels      1..=3     step 1
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

A Seed draws every Parameter above `ditherTog`, each across its full range. `styles` is the figure shape. Auto lets the Tool seed pick the shape, and it also changes how the bars move inside each figure, so Auto gives a different Still from naming the same shape. `count` sets how many bars cross the short edge and `weight` how much of each step is bar. `sizeF` sets the size of the outer figure and `levels` how many figures nest inside one another. Dither and grain stay off.

The Palette has 2 inks by role: Base and Ink. optic takes at most 2. The Base is the ground and the Ink is every bar.
