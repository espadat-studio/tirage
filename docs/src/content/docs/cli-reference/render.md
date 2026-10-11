---
title: "render"
description: "Render a Recipe to a PNG or an SVG."
---

```sh
tirage render [OPTIONS] [FILE]
tirage r [OPTIONS] [FILE]
```

Reads a Recipe and writes one frame of its Visual as a PNG, or its Still as an SVG. A Still has one frame. A Loop has as many as [tools](/cli-reference/tools/) lists for its Tool, and `--frame` picks one.

## Argument

`FILE` is the Recipe JSON that [derive](/cli-reference/derive/) printed. `-` reads it from stdin, and so does leaving it out. On a terminal with nothing piped in, render asks for a Seed and a Tool and derives the Recipe itself.

## Options

| Option              | Value                                          | Without it                                                                                                                                                                                                                                |
| ------------------- | ---------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--size <W>x<H>`    | Frame size in pixels, each edge from 1 to 8192 | Prompted on a terminal with a pick of 1080x1920 story, 1080x1350 portrait, 1080x1080 square, 1920x1080 landscape, or a custom size                                                                                                        |
| `--frame <T>`       | A frame of a Loop, counted from 0              | The Still                                                                                                                                                                                                                                 |
| `--format <FORMAT>` | `png` or `svg`                                 | Inferred from `-o`: a path ending in `.svg` writes an SVG, and anything else, stdout included, writes a PNG                                                                                                                               |
| `-o <FILE>`         | PNG or SVG to write, or `-` for stdout         | Prompted on a terminal, with `<tool>-<seed>.png` as default when render derived the Recipe itself and `out.png` otherwise, `.svg` with `--format svg`. The prompt asks before overwriting. An explicit path is overwritten without asking |

`-h`, `--help`, `--no-color` and `--no-input` work here as on every command. See [tirage](/cli-reference/tirage/).

## Input

render takes a Recipe from a path, as in `tirage render recipe.json`, or from stdin, as in `tirage derive --seed 42 | tirage render`, which is the usual pipeline. Alone on a terminal it asks for a Seed, with a random one as default, then for a Tool, with "deal from Seed" first in the list.

When render derived the Recipe by prompts, it prints the whole pipeline on stderr so you can rerun it without the prompts:

```
→ tirage derive --seed 1234 --tool sonar | tirage render --size 1080x1920 -o sonar-1234.png
```

With prompts off and nothing to read, render fails: exit 2 when stdin is a terminal, exit 1 when stdin is empty.

## Size

Any size from 1x1 to 8192x8192 pixels. The prompt offers the four common social formats and a custom entry.

## Frame

`--frame` picks one frame of a Loop, from 0 to one less than the Tool's frame count. Without it, render draws the Still, the Tool with motion off. Frame 0 is the Loop's first frame with motion on, so on some Tools it differs from the Still. A frame past the end exits 1, and the hint names the last valid frame. To turn every frame into an MP4, see [encode](/cli-reference/encode/).

## SVG

`--format svg`, or an `-o` path ending in `.svg`, writes the Still as an SVG instead of a PNG. It draws the same shapes as the PNG, as vector elements, with two differences:

- Still only. An SVG is the Still, so any `--frame` exits 2.
- No grain or dither. Those passes edit pixels, so the SVG leaves them out even when the Recipe turns them on.

Only some Tools write SVG. [tools](/cli-reference/tools/) marks them `svg`. For any other Tool, render exits 1 before it prompts, and the hint points to `tirage tools`.

## Output

`-o FILE` writes the PNG or SVG and leaves stdout empty. `-o -` streams it to stdout for the next command in a pipe, as a PNG unless `--format svg` is passed. render refuses to write PNG bytes to a terminal and exits 2 instead. The PNG is 8-bit RGBA at the exact size requested. The SVG's `width`, `height` and `viewBox` are that size in pixels.

## Exit codes

| Code | When                                                                                                                                                                                                                                            |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `0`  | The PNG or SVG was written                                                                                                                                                                                                                      |
| `1`  | The Recipe could not be read or is not valid: bad JSON, another derivation major, a value out of range, an unknown Parameter. Also a frame size or frame number out of range, an SVG of a Tool without SVG, or a file that could not be written |
| `2`  | A missing size or output path with prompts off, no Recipe on a terminal with prompts off, a bad flag value, an SVG with `--frame`, or a PNG aimed at the terminal                                                                               |

## Examples

```sh
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png
tirage render recipe.json --size 540x960 -o - > still.png
tirage render recipe.json --size 540x960 --frame 6 -o frame-6.png
tirage render recipe.json --size 540x960 -o still.svg
tirage render recipe.json --size 540x960 --format svg -o - > still.svg
```
