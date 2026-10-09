---
title: "encode"
description: "Encode a Loop to an H.264 MP4."
---

```sh
tirage encode [OPTIONS] [FILE]
tirage e [OPTIONS] [FILE]
```

Reads a Recipe, renders every frame of its Loop at 720x1280 and writes them as one H.264 MP4 at the Tool's own frame rate. sonar gives 24 frames at 10 fps, kiosk 24 at 6 fps, frond 36 at 12 fps. The bytes are what the `tirage-encode` library returns for the same Recipe.

encode is in the binary only when it was built with the `encode` feature. See [Installation](/getting-started/installation/#the-encode-feature).

## Argument

`FILE` is the Recipe JSON that [derive](/cli-reference/derive/) printed. `-` reads it from stdin, and so does leaving it out. On a terminal with nothing piped in, encode asks for a Seed and a Loop Tool and derives the Recipe itself.

## Options

| Option      | Value                           | Without it                                                                                                                                                                                                    |
| ----------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-o <FILE>` | MP4 to write, or `-` for stdout | Prompted on a terminal, with `<tool>-<seed>.mp4` as default when encode derived the Recipe itself and `out.mp4` otherwise. The prompt asks before overwriting. An explicit path is overwritten without asking |

`-h`, `--help`, `--no-color` and `--no-input` work here as on every command. See [tirage](/cli-reference/tirage/).

## Input

encode takes a Recipe from a path, as in `tirage encode recipe.json`, or from stdin, as in `tirage derive --seed 42 --tool sonar | tirage encode`. Alone on a terminal it asks for a Seed, with a random one as default, then for a Tool. That list has only the Tools with a Loop: sonar, kiosk and frond.

A Recipe of a Still Tool, husk, vein or aura, has no Loop to encode. encode exits 1 and the hint says to render it as a PNG.

When encode derived the Recipe by prompts, it prints the whole pipeline on stderr so you can rerun it without the prompts:

```
→ tirage derive --seed 1234 --tool sonar | tirage encode -o sonar-1234.mp4
```

With prompts off and nothing to read, encode fails: exit 2 when stdin is a terminal, exit 1 when stdin is empty.

## Output

`-o FILE` writes the MP4 and leaves stdout empty. `-o -` streams the MP4 to stdout for the next command in a pipe. encode refuses to write MP4 bytes to a terminal and exits 2 instead.

A Loop has to stay under 1.5 MB. One that does not exits 1, and the message names its size and the cap. Nothing is written in that case.

openh264 prints one warning on stderr while it encodes, that it cannot hold the bitrate without skipping frames. A Loop keeps every frame, so the warning is expected.

## Exit codes

| Code | When                                                                                                                                                                                                      |
| ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `0`  | The MP4 was written                                                                                                                                                                                       |
| `1`  | The Recipe could not be read or is not valid: bad JSON, another derivation major, a value out of range, an unknown Parameter. Also a Still Tool, a Loop over the cap, or an MP4 that could not be written |
| `2`  | A missing output path with prompts off, no Recipe on a terminal with prompts off, a bad flag value, or an MP4 aimed at the terminal                                                                       |

## Examples

```sh
tirage derive --seed 42 --tool sonar | tirage encode -o out.mp4
tirage encode recipe.json -o - > loop.mp4
```
