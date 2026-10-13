---
title: "filament"
description: "Bundles of fine strands traced through a curling flow field, with small symbols riding them. It renders a Loop of 30 frames at 16 fps, a dash running along about half the strands."
---

Bundles of fine strands on a dark ground. Each bundle starts from one spot and follows a curling flow field, so its strands run together like a rope and then fray apart. Most strands are fine and a few are heavy. Small arrows, rings, crosses, hooks and ticks ride along the strands in three accent colours, more of them near the far end. Some strands can also carry a chain of beads.

filament renders a Loop: 30 frames at 16 fps. About half the strands show only a third of their length at a time, and that piece runs along the strand twice per Loop. The other strands, the symbols and the beads hold still. The moving strands are already cut short at frame 0, so frame 0 differs from the Still. Its site page is [playgrnd.tools/filament](https://www.playgrnd.tools/filament/).

## Still

![filament Still for Seed 2](../../../assets/tools/filament.png)

Seed 2, rendered by:

```sh
tirage derive --seed 2 --tool filament | tirage render --size 540x960 -o filament.png
```

## Loop

<video src="/tools/filament.mp4" width="360" height="640" autoplay loop muted playsinline></video>

The same Seed 2, encoded at 360x640 by:

```sh
tirage derive --seed 2 --tool filament | tirage encode --size 360x640 -o filament.mp4
```

## Parameters

From [`tirage tools`](/cli-reference/tools/), without the 11 shared [dither and grain](/concepts/dither-and-grain/) Parameters:

```
filament  30 frames
  zoom        0.3..=6   step 0.05
  turn        0.2..=5   step 0.05
  oct         1..=6     step 1
  curl        0..=1     step 0.01
  tangle      0..=1     step 0.01
  bundles     1..=90    step 1
  per         1..=40    step 1
  clump       0..=1     step 0.01
  tight       0.005..=0.2 step 0.005
  len         20..=600  step 10
  step        1..=10    step 0.2
  wgt         0.2..=8   step 0.1
  wvar        0..=1     step 0.01
  hier        0..=1     step 0.01
  hair        0..=1     step 0.01
  mstyles     Symbols, Beads, Both
  mark        0..=1     step 0.01
  bead        0..=1     step 0.01
  bgap        2..=14    step 1
  msize       2..=26    step 0.5
```

A Seed draws all twenty of filament's own Parameters. `zoom`, `turn`, `oct` and `curl` shape the flow field, and `tangle` lets strands leave it and cross each other. `bundles` sets how many ropes there are, `per` how many strands each one has, `tight` how close they start, and `clump` how much the ropes gather in one part of the frame. `len` and `step` set how far a strand walks. `wgt`, `wvar` and `hier` set the stroke weights, and `hair` is the share of strands drawn as hairlines. `mstyles` picks symbols, beads or both. `mark` sets how many symbols there are, `bead` how many strands carry beads, `bgap` the spacing of the beads, and `msize` the size of every mark.

A Seed keeps `bundles` from 23 and `len` from 160. With a single bundle or a very short walk, the frame is almost empty. Every other Parameter can take its full range.

The Palette takes any number of inks from 2 and colours the strands and beads. The last ink draws the hairlines. The dark ground and the three symbol accents are fixed by filament itself.
