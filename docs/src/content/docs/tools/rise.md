---
title: "rise"
description: "Even vertical bars under large concentric bands that rise from one anchor. It renders a Still."
---

Even vertical bars fill the frame. Large concentric bands rise from one anchor over them, and each band trades which two inks are its ground and its bars. Every band shares the same bar grid, so the stripes line up from one band to the next.

rise renders a Still: one frame. Its site page is [playgrnd.tools/rise](https://www.playgrnd.tools/rise/).

## Still

![rise Still for Seed 1](../../../assets/tools/rise.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool rise | tirage render --size 540x960 -o rise.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
rise  1 frame
  styles      Bottom, Top, Left, Right, Centre
  count       6..=28    step 1
  weight      0.3..=0.7 step 0.01
  bands       1..=8     step 1
  depth       0.1..=0.4 step 0.01
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

A Seed draws every Parameter above `ditherTog`, each across its full range. `styles` is the anchor the bands rise from: the middle of an edge, or the centre of the frame. The site's Auto anchor is left out: it picks one from the Tool seed, so a Recipe names that anchor instead. `count` sets how many bars cross the short edge and `weight` how much of each step is bar. `bands` sets how many bands there are and `depth` how thick each one is. Dither and grain stay off.

The Palette has 3 inks by role: Field, Bars and Accent. rise takes at most 3. The Field and Bars inks paint the ground outside the bands, and the bands cycle through the other pairings of the three.
