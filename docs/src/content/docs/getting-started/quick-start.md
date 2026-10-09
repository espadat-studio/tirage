---
title: "Quick Start"
description: "Derive a Recipe from a Seed, render a Still, then render the Loop and encode it to MP4."
---

This page goes from a Seed to a Still, then to a Loop as an MP4. It needs the CLI from [Installation](/getting-started/installation/).

## Derive a Recipe

A Seed is one integer. `derive` turns it into a Recipe: a Tool, a value for each of its Parameters, a Palette and a Tool seed.

```sh
tirage derive --seed 42 --tool sonar > recipe.json
```

`--tool sonar` is a Pin. Without it the Seed derives the Tool as well. `derive` prints the Recipe on one line. Spread out, this is what `recipe.json` holds:

```json
{
  "tirage": 0,
  "tool": "sonar",
  "tool_seed": 4243409203,
  "palette": ["#0a0f1c", "#3ddc97", "#4361ee", "#ffd166", "#ef476f", "#f1faee"],
  "params": {
    "level": 0.64,
    "scale": 6.3,
    "warp": 0.14,
    "grid": 286,
    "depth": 0.58,
    "fringe": 0.68,
    "spark": 0.09,
    "ditherTog": false,
    "dthKinds": "Bayer 8",
    "dthSize": 2,
    "dthLevels": 3,
    "dthAmount": 1.0,
    "grainTog": false,
    "grnBlends": "Add",
    "grnAmount": 0.55,
    "grnSize": 1.0,
    "grnSpecks": 0.5,
    "grnVignette": 0.5
  }
}
```

Seed 42 with this Pin always gives this Recipe. Each Parameter was drawn from its Taste bounds, so the result is on brand without any tuning.

## Render a Still

```sh
tirage render recipe.json --size 1080x1920 -o out.png
```

`out.png` is a 1080x1920 Still: sonar's one-bit chart of a coastline, in the Recipe's Palette. Any size works, each edge up to 8192 pixels. The two steps also go in one pipe, with the same bytes out:

```sh
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png
```

## Render a Loop

Some Tools animate. `tirage tools` lists every Tool with its frame count and Parameters. Its first line is:

```
sonar  24 frames
```

sonar's Loop is 24 frames at 10 fps: the land keeps its shape while the water level rises and falls. `--frame` renders one frame of it:

```sh
tirage render recipe.json --size 720x1280 --frame 6 -o frame-6.png
```

Frame 6 is high tide. Frame 0 is the Still of the same Recipe, so leaving out `--frame` gives the same image as `--frame 0`. The tide runs one full cycle per Loop, so the last frame leads back into the first.

## Encode the Loop to MP4

The CLI renders one frame at a time. A whole Loop as a video comes from the `tirage-encode` crate: it renders every frame at 720x1280 and encodes them with openh264 into an H.264 MP4. Make a small program for it:

```sh
cargo new first-loop && cd first-loop
```

Add both crates to `Cargo.toml`:

```toml
[dependencies]
tirage = { git = "https://github.com/espadat-studio/tirage", rev = "e43e5b8" }
tirage-encode = { git = "https://github.com/espadat-studio/tirage", rev = "e43e5b8" }
```

Replace `src/main.rs`:

```rust
use std::fs;

use tirage::Recipe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let recipe = Recipe::from_json(&fs::read_to_string("recipe.json")?)?;
    fs::write("out.mp4", tirage_encode::encode(&recipe)?)?;
    Ok(())
}
```

Copy `recipe.json` next to it and run:

```sh
cargo run --release
```

openh264 prints one warning, that it cannot hold the bitrate without skipping frames. That is expected: a Loop keeps every frame. `out.mp4` is the Loop: H.264, 720x1280, 24 frames at 10 fps, 2.4 seconds, 1.15 MB. A Loop has to stay under 1.5 MB, and `encode` returns an error for one that does not. Its first frame is the Still, so `out.png` can be the video's poster. If the video does not play (reduced motion, blocked autoplay), the viewer sees the Still instead.
