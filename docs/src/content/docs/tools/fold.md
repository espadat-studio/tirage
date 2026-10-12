---
title: "fold"
description: "Pixel op-art patches folded across the sheet like a kaleidoscope. It renders a Loop of 36 frames at 12 fps, the patches sliding under fixed folds."
---

A collage of pixel op-art patches over a stepped burst ground, folded across the sheet so it mirrors like a kaleidoscope. Patches come as zebra stripes, staircases, dot rows, bursts, chevrons or checks.

fold renders a Loop: 36 frames at 12 fps, in which the patches and the ground slide once round a small circle under the folded view. The fold lines stay put, so the patches flow into and out of each mirror. The patches are already off their places at frame 0, so frame 0 differs from the Still. Its site page is [playgrnd.tools/fold](https://www.playgrnd.tools/fold/).

## Still

![fold Still for Seed 2](../../../assets/tools/fold.png)

Seed 2, rendered by:

```sh
tirage derive --seed 2 --tool fold | tirage render --size 540x960 -o fold.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
fold  36 frames
  folds       Four ways, Kaleidoscope, Across, Down, None
  kinds       Mixed, Zebra, Staircases, Dot rows, Bursts, Chevrons, Checks
  foldx       0.2..=0.8 step 0.01
  foldy       0.2..=0.8 step 0.01
  patches     1..=16    step 1
  size        0..=1     step 0.01
  scale       0..=1     step 0.01
  wave        0..=1     step 0.01
  levels      2..=8     step 1
  tilt        0..=1     step 0.01
  px          0..=1     step 0.01
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

A Seed draws every Parameter above the dither ones, each across its full range. `folds` picks how the sheet mirrors and `foldx` and `foldy` where the fold lines sit. `kinds` picks one patch kind for every patch, or Mixed for a different kind each. `px` sets the size of the pixel grid, and `levels` how many tones a burst steps through. Dither and grain stay off.

The Palette takes any number of inks from 2. Ink 0 is the ground. Zebra stripes use the darkest and lightest inks, and the other patches take two inks from the third ink on.
