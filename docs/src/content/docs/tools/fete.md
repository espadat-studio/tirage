---
title: "fete"
description: "A festival poster of pixel noise patches under a white doodle and black dots. It renders a Still."
---

A festival poster. Noise patches in bright inks fill a coarse grid that is blown up with hard pixel edges. One big white doodle sits on top, with black dots scattered over the frame and along the line.

fete renders a Still: one frame. Its site page is [playgrnd.tools/fete](https://www.playgrnd.tools/fete/).

## Still

![fete Still for Seed 1](../../../assets/tools/fete.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool fete | tirage render --size 540x960 -o fete.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
fete  1 frame
  motifs      Auto, Rings, Spiral, Burst, Atom, Wave, Scribble
  scale       1.2..=6   step 0.1
  res         36..=120  step 4
  lw          0.4..=2.2 step 0.05
  dots        0..=80    step 2
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

A Seed draws `motifs`, `scale`, `res`, `lw` and `dots`, each across its full range. `motifs` picks the doodle: rings, a spiral, a burst of rays, an atom, a zigzag wave or a looping scribble. Auto lets the Tool seed pick one. `res` is how many grid columns span the frame, and `scale` how many patches fit across it. Dither and grain stay off.

The Palette takes any number of inks from 2 and colours the patches. The line is always white and the dots near black; neither is part of the Palette.
