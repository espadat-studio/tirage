---
title: "aura"
description: "A soft glow of melting inks. It renders a Loop of 72 frames at 12 fps, the inks sliding round a circle over a fixed warp."
---

A soft glow of melting inks. Each ink owns a warped noise field. Every pixel mixes all inks, weighted by how strong each field is there, so the inks bleed into tints with no hard edge. The field is painted on a small buffer and scaled up smooth.

aura renders a Loop: 72 frames at 12 fps, in which the point every ink field reads drifts once round a circle, so all the inks slide together. The warp under them holds still, so the blots bend as they pass through it. The inks are already shifted at frame 0, so frame 0 differs from the Still. Its site page is [playgrnd.tools/aura](https://www.playgrnd.tools/aura/).

## Still

![aura Still for Seed 4](../../../assets/tools/aura.png)

Seed 4, rendered by:

```sh
tirage derive --seed 4 --tool aura | tirage render --size 540x960 -o aura.png
```

## Loop

<video src="/tools/aura.mp4" width="360" height="640" autoplay loop muted playsinline></video>

The same Seed 4, encoded at 360x640 by:

```sh
tirage derive --seed 4 --tool aura | tirage encode --size 360x640 -o aura.mp4
```

## Parameters

From [`tirage tools`](/cli-reference/tools/), without the 11 shared [dither and grain](/concepts/dither-and-grain/) Parameters:

```
aura  72 frames
  styles      Auto, Clouds, Mesh, Sweep [no Taste]
  scale       0.5..=2   step 0.05
  churn       0..=1     step 0.01
  punch       0..=1     step 0.01
```

A Seed draws `styles`, `scale`, `churn` and `punch`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed.
