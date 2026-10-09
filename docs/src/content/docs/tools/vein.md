---
title: "vein"
description: "Poured paint, cut flat into stacked levels. It renders a Still."
---

Poured paint, cut flat. A smooth stripy field is sliced at evenly spaced levels. Each level is the outline of everything at or above it, filled as one shape over the levels below, so the sheet reads as stacked layers of marbled paint. The lowest levels are a dark ground, and stars sit on that ground only.

vein renders a Still: one frame. Its site page is [playgrnd.tools/vein](https://www.playgrnd.tools/vein/).

## Still

![vein Still for Seed 3](../../../assets/tools/vein.png)

Seed 3, rendered by:

```sh
tirage derive --seed 3 --tool vein | tirage render --size 540x960 -o vein.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
vein  1 frame
  flows       Marble, Swirl, Ripple
  scale       0..=1     step 0.01
  curve       0..=1     step 0.01
  levels      3..=14    step 1
  tiger       0..=1     step 0.01
  edges       0..=1     step 0.01
  stars       0..=1     step 0.01
  tints       0..=1     step 0.01
  runs        0..=1     step 0.01
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

A Seed draws `flows`, `scale`, `curve`, `levels`, `tiger`, `edges`, `stars`, `tints` and `runs`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. Grain stays on, as the site page opens: Overlay at 0.4, size 1, specks 0.4, vignette 0.25. Dither stays off.
