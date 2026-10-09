---
title: "tirage"
description: "Original generative artwork and animations, rendered from code."
---

> Original generative artwork and animations, rendered from code.

It reproduces the look of playgrnd.tools generators; not affiliated.

Each generator is a _Tool_, named by its site slug: aura, sonar, kiosk and more. A Tool paints with a _Palette_ and takes a few _Parameters_, one per slider on its site page.

A single integer, the _Seed_, derives a whole _Recipe_: the Tool, a value for each Parameter, a Palette and a _Tool seed_. The same Seed with the same _Pins_ always gives the same Recipe. Any Seed gives an on-brand result, because each Parameter is drawn from its _Taste bounds_, a hand-picked sub-range.

A Recipe renders a _Visual_. That is either a _Still_, one frame, or a _Loop_, an animation that ends where it starts. A Loop's first frame is the Still of the same Recipe.

## Install and render

```sh
cargo install --locked --git https://github.com/espadat-studio/tirage --tag vX.Y.Z tirage-cli
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png
```

`derive` prints the Recipe for Seed 42, with the Tool pinned to sonar, as JSON. `render` reads it and writes a 1080x1920 PNG. `tirage tools` lists every Tool. The [CLI Reference](/cli-reference/tirage/) covers every command and flag.

[Quick Start](/getting-started/quick-start/) goes on from here to a Loop and its MP4.

[Concepts](/concepts/seed-and-recipe/) explains why the same Seed always gives the same Visual, and how to steer it.
