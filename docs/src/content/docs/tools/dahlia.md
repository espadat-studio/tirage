---
title: "dahlia"
description: "A firework burst of dashed rays in stacked inks on a printed paper ground. It renders a Still."
---

A firework burst on paper. Rays leave one centre and bow a little as they go. Each ray is a chain of short dashes, often two or three inks laid side by side, and many dashes end in a round bead of the accent ink.

dahlia renders a Still: one frame. Its site page is [playgrnd.tools/dahlia](https://www.playgrnd.tools/dahlia/).

## Still

![dahlia Still for Seed 1](../../../assets/tools/dahlia.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool dahlia | tirage render --size 540x960 -o dahlia.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
dahlia  1 frame
  rays        8..=300   step 1
  size        0..=1     step 0.01
  ragged      0..=1     step 0.01
  cx          0.1..=0.9 step 0.01
  cy          0.1..=0.9 step 0.01
  width       0..=1     step 0.01
  dash        0..=1     step 0.01
  gap         0..=1     step 0.01
  bend        0..=1     step 0.01
  caps        0..=1     step 0.01
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

A Seed draws every Parameter above `ditherTog`, each across its full range. `size` sets how far the burst reaches and `ragged` how unevenly its rays end. `cx` and `cy` place the centre. `width`, `dash` and `gap` shape the dashes, `bend` bows the rays, and `caps` is the chance that a dash ends in a bead.

Grain is on, as the site page opens it: multiply at 0.22, with specks at 0.5 and vignette at 0.15. Its `grnAmount` of 0.22 sits between the slider's steps. Dither stays off.

The Palette takes any number of inks from 2, read by role. The first ink is the paper and the last is the accent for the beads. The inks between colour the rays. With only 2 inks, the rays use the accent too.
