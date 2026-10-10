---
title: "warp"
description: "Op-art checks or slanted bars, bent through a wave, a taper or a bulge. It renders a Still."
---

Op-art in one ink on one ground. A checkerboard or a field of slanted bars is bent through a wave, a taper or a bulge. The shapes are polygons, filled in one pass.

warp renders a Still: one frame. Its site page is [playgrnd.tools/warp](https://www.playgrnd.tools/warp/).

## Still

![warp Still for Seed 1](../../../assets/tools/warp.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool warp | tirage render --size 540x960 -o warp.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
warp  1 frame
  styles      Checker, Slash
  scale       0.5..=2   step 0.05
  warp        0..=1     step 0.01
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

A Seed draws `styles`, `scale` and `warp`, each across its full range. The site's Auto style is left out: it picks checks or bars from the Tool seed, so a Recipe names that style instead. Dither and grain stay off.

The Palette has 5 inks by role: a Base, then Ink 1 to Ink 4. warp takes at most 5. Each Still paints the Base and one of the four inks, and the Tool seed picks which ink and which of the two is the ground.
