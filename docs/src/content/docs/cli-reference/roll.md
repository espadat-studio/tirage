---
title: "roll"
description: "Roll a random Seed and write its Visual to a file."
---

```sh
tirage roll [OPTIONS]
```

Deals a random Seed with no Tool pinned, so the Seed picks the Tool and every Parameter, and writes one file in the current directory. roll takes no input and never prompts.

## Options

roll has no options of its own. `-h`, `--help`, `--no-color` and `--no-input` work here as on every command, and `--no-input` changes nothing. See [tirage](/cli-reference/tirage/).

## Output

A Still is rendered at 1080x1920 and written to `<tool>-<seed>.png`. A Loop is encoded as [encode](/cli-reference/encode/) does and written to `<tool>-<seed>.mp4`. Without the `encode` feature, roll deals Seeds until one gives a Still, so it never writes an MP4.

stdout stays empty. stderr gets the line that makes the same file again:

```
→ tirage derive --seed 8812 | tirage render --size 1080x1920 -o vein-8812.png
```

Running that line writes a file with the same bytes. Keep it if you like the roll: the Seed is random, so a second roll gives another file.

roll never overwrites. If the target file exists, it exits 1 and writes nothing.

## Exit codes

| Code | When                                                                                      |
| ---- | ----------------------------------------------------------------------------------------- |
| `0`  | The file was written                                                                      |
| `1`  | The target file exists, a Loop went over the 1.5 MB cap, or the file could not be written |
| `2`  | A bad flag                                                                                |

## Examples

```sh
tirage roll
tirage roll 2>> rolls.log
```
