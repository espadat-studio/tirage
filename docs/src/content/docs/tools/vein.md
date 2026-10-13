---
title: "vein"
description: "Poured paint, cut flat into stacked levels. It renders a Loop of 36 frames at 12 fps, the levels flowing as the warp drifts round a circle."
---

Poured paint, cut flat. A smooth stripy field is sliced at evenly spaced levels. Each level is the outline of everything at or above it, filled as one shape over the levels below, so the sheet reads as stacked layers of marbled paint. The lowest levels are a dark ground, and stars sit on that ground only.

vein renders a Loop: 36 frames at 12 fps, in which the warp under the stripes drifts once round a small circle, so the levels bend and flow back. The level inks and the star places hold still, and stars show or hide as the dark ground passes them. The warp is already shifted at frame 0, so frame 0 differs from the Still. Its site page is [playgrnd.tools/vein](https://www.playgrnd.tools/vein/).

## Still

![vein Still for Seed 3](../../../assets/tools/vein.png)

Seed 3, rendered by:

```sh
tirage derive --seed 3 --tool vein | tirage render --size 540x960 -o vein.png
```

SVG: yes. `-o vein.svg` writes this Still as an [SVG](/cli-reference/render/#svg).

## Parameters

From [`tirage tools`](/cli-reference/tools/), without the 11 shared [dither and grain](/concepts/dither-and-grain/) Parameters:

```
vein  36 frames, svg
  flows       Marble, Swirl, Ripple
  scale       0..=1     step 0.01
  curve       0..=1     step 0.01
  levels      3..=14    step 1
  tiger       0..=1     step 0.01
  edges       0..=1     step 0.01
  stars       0..=1     step 0.01
  tints       0..=1     step 0.01
  runs        0..=1     step 0.01
```

A Seed draws `flows`, `scale`, `curve`, `levels`, `tiger`, `edges`, `stars`, `tints` and `runs`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. Grain stays on, as the site page opens: Overlay at 0.4, size 1, specks 0.4, vignette 0.25.
