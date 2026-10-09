---
title: "pith"
description: "Cut-tissue cells with veins carved through them. It renders a Still."
---

Cut-tissue cells on a paper ground. Each cell is a cluster of rounded boxes with a lumpy, ragged outline, banded from its edge inward: an outer band, a thin rim, then a mottled core. Branching veins carve the ground back through each cell, and speckle blurs the bands into one another.

pith renders a Still: one frame. Its site page is [playgrnd.tools/pith](https://www.playgrnd.tools/pith/).

## Still

![pith Still for Seed 14](../../../assets/tools/pith.png)

Seed 14, rendered by:

```sh
tirage derive --seed 14 --tool pith | tirage render --size 540x960 -o pith.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
pith  1 frame
  count       1..=60    step 1
  size        0..=1     step 0.01
  zoom        1..=8     step 0.1
  round       0..=1     step 0.01
  wobble      0..=1     step 0.01
  band        0..=1     step 0.01
  dither      0..=1     step 0.01
  veins       0..=1     step 0.01
  thick       0..=1     step 0.01
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

A Seed draws `count`, `size`, `zoom`, `round`, `wobble`, `band`, `veins` and `thick`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. `dither`, the speckle across band edges, stays at 0.55, and `grain`, the Tool's own flecks, at 0.4. The dither and grain post-passes stay off.

The Palette is read by position: ground, outer band, rim, core, then the grain flecks. A shorter Palette reuses the second ink for the missing ones.
