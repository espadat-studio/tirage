---
title: "vee"
description: "Stacked bands of angled poster stripes, laid out as chevrons, quarters, one field or two crossing layers. It renders a Still."
---

Horizontal bands stack down the frame, and each one fills with straight stripes at an angle. A band lays them out one of four ways: one diagonal field, a chevron of two mirrored halves, four mirrored quarters, or two layers of stripes that cross. Where crossing stripes overlap, a third ink shows.

vee renders a Still: one frame. Its site page is [playgrnd.tools/vee](https://www.playgrnd.tools/vee/).

## Still

![vee Still for Seed 1](../../../assets/tools/vee.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool vee | tirage render --size 540x960 -o vee.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
vee  1 frame
  styles      Auto, Chevron, Quad, Diagonal, Cross
  angle       15..=75   step 1
  width       0.4..=2.2 step 0.05
  weight      0.15..=0.85 step 0.01
  bands       1..=4     step 1
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

A Seed draws every Parameter above `ditherTog`, each across its full range. `styles` picks the layout for every band. Auto lets the Tool seed pick one per band instead. `angle` is the stripe angle in degrees, `width` scales the stripe spacing, and `weight` sets how much of each step is stripe. `bands` sets how many bands stack down the frame. Dither and grain stay off.

The Palette has 4 inks by role: Base, Ink 1, Ink 2 and Cross. vee takes at most 4. Base is the ground. The stripe inks cycle Ink 1, Ink 2, Cross. Each band starts on Ink 1 or Ink 2, and a crossing band takes the next two in the cycle for its second layer and for the overlap.
