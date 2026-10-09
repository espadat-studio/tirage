---
title: "tools"
description: "List Tools with their frame counts and Parameters."
---

```sh
tirage tools [OPTIONS]
tirage t
tirage ls
```

Lists every Tool with its frame count and its Parameters. For each Parameter it prints the id, then the range and step of a slider, `on/off` for a toggle, or the choices of a picker. The slugs are what [derive](/cli-reference/derive/) pins a Tool by, and the ids are what a Taste bounds file names.

## Options

| Option   | Does                       |
| -------- | -------------------------- |
| `--json` | Print JSON instead of text |

`-h`, `--help`, `--no-color` and `--no-input` work here as on every command. See [tirage](/cli-reference/tirage/).

## Tools

| Tool    | Frames | Visual |
| ------- | ------ | ------ |
| `sonar` | 24     | Loop   |
| `husk`  | 1      | Still  |
| `vein`  | 1      | Still  |
| `aura`  | 1      | Still  |
| `kiosk` | 24     | Loop   |
| `frond` | 36     | Loop   |

## Text output

Each Tool heads a block, then one line per Parameter:

```
sonar  24 frames
  level       0..=1     step 0.01
  scale       1..=10    step 0.1
  warp        0..=1     step 0.01
  grid        40..=320  step 2
  depth       0..=1     step 0.01
  fringe      0..=1     step 0.01
  spark       0..=1     step 0.01
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

A range like `0..=1 step 0.01` means any value from 0 to 1 in steps of 0.01, both ends included. The last eleven Parameters, from `ditherTog` to `grnVignette`, are the dither and grain passes. Every Tool has them.

## JSON output

`--json` prints one array with one object per Tool, in the same order as the text. Each object has `slug`, `frames` and `params`. Each Parameter has an `id` and a `kind`: `range` carries `min`, `max` and `step`, `choice` carries `choices`, and `toggle` carries nothing more. The real output is one line and lists every Parameter. Trimmed and pretty printed:

```json
[
  {
    "slug": "sonar",
    "frames": 24,
    "params": [
      { "id": "level", "kind": "range", "min": 0.0, "max": 1.0, "step": 0.01 },
      { "id": "ditherTog", "kind": "toggle" },
      {
        "id": "dthKinds",
        "kind": "choice",
        "choices": ["Bayer 8", "Bayer 4", "Noise"]
      }
    ]
  }
]
```

## Exit codes

| Code | When                    |
| ---- | ----------------------- |
| `0`  | The listing was printed |
| `2`  | An unknown option       |

A closed pipe, as when piping into `head`, is not an error.

## Examples

```sh
tirage tools
tirage tools --json
tirage tools --json | jq -r '.[].slug'
```
