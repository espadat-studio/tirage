---
title: "render"
description: "Render a Recipe to a PNG."
---

```sh
tirage render [OPTIONS] [FILE]
tirage r [OPTIONS] [FILE]
```

Reads a Recipe and writes one frame of its Visual as a PNG. A Still has one frame. A Loop has as many as [tools](/cli-reference/tools/) lists for its Tool, and `--frame` picks one.

## Argument

`FILE` is the Recipe JSON that [derive](/cli-reference/derive/) printed. `-` reads it from stdin, and so does leaving it out. On a terminal with nothing piped in, render asks for a Seed and a Tool and derives the Recipe itself.

## Options

| Option           | Value                                          | Without it                                                                                                                                                  |
| ---------------- | ---------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--size <W>x<H>` | Frame size in pixels, each edge from 1 to 8192 | Prompted on a terminal with a pick of 1080x1920 story, 1080x1350 portrait, 1080x1080 square, 1920x1080 landscape, or a custom size                          |
| `--frame <T>`    | A frame of a Loop, counted from 0              | Frame 0, the Still                                                                                                                                          |
| `-o <FILE>`      | PNG to write, or `-` for stdout                | Prompted on a terminal, with `<tool>-<seed>.png` as default when render derived the Recipe itself and `out.png` otherwise. Asks before it overwrites a file |

`-h`, `--help`, `--no-color` and `--no-input` work here as on every command. See [tirage](/cli-reference/tirage/).

## Input

render takes a Recipe three ways:

- A path: `tirage render recipe.json`.
- stdin: `tirage derive --seed 42 | tirage render`. This is the usual pipeline.
- Prompts: `tirage render` alone on a terminal asks for a Seed, with a random one as default, then for a Tool, with "deal from Seed" first in the list.

When render derived the Recipe by prompts, it prints the whole pipeline on stderr so you can rerun it without the prompts:

```
→ tirage derive --seed 1234 --tool sonar | tirage render --size 1080x1920 -o sonar-1234.png
```

With prompts off and nothing to read, render fails: exit 2 when stdin is a terminal, exit 1 when stdin is empty.

## Size

Any size from 1x1 to 8192x8192 pixels. The prompt offers the four common social formats and a custom entry.

## Frame

`--frame` picks one frame of a Loop, from 0 to one less than the Tool's frame count. Frame 0 is the Still. A frame past the end exits 1, and the hint names the last valid frame. To turn every frame into an MP4, see the [encode](https://github.com/espadat-studio/tirage/tree/master/encode) crate.

## Output

`-o FILE` writes the PNG and leaves stdout empty. `-o -` streams the PNG to stdout for the next command in a pipe. render refuses to write PNG bytes to a terminal and exits 2 instead. The PNG is 8-bit RGBA at the exact size requested.

## Exit codes

| Code | When                                                                                                                                                                                                             |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `0`  | The PNG was written                                                                                                                                                                                              |
| `1`  | The Recipe could not be read or is not valid: bad JSON, another derivation major, a value out of range, an unknown Parameter. Also a frame size or frame number out of range, or a PNG that could not be written |
| `2`  | A missing size or output path with prompts off, no Recipe on a terminal with prompts off, a bad flag value, or a PNG aimed at the terminal                                                                       |

## Examples

```sh
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png
tirage render recipe.json --size 540x960 -o - > still.png
tirage render recipe.json --size 540x960 --frame 12 -o frame-12.png
```
