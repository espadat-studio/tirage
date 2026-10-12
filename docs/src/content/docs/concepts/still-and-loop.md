---
title: "Still and Loop"
description: "One Recipe gives one frame or a short video: the Tool with motion off or on."
---

A _Visual_ is what a Recipe renders. It is either a _Still_, a single frame with the Tool's motion off, or a _Loop_, an animation with motion on that ends where it starts: after its last frame, frame 0 plays again. The Tool decides which you get. sonar, husk, kiosk, frond, pith, bloom, warp, mosh, relief, atlas, motley, static, fete, pane, sampler, totem, whorl, weave, zig, quilt, dahlia, carve, tokens, optic, vee, riso, rise, prism, filament, culture, sprig, fold, splice, crowd, parcel, chaff, strand, vein, aura, benday, stitch, mist, coral, sear, cipher, hiss, terrain, stipple, oddgrid and specimen draw Loops. Every other Tool draws a Still. `tirage tools` lists the frame count beside each Tool, and a Still Tool has 1.

## The Still and frame 0

`render` draws the Still unless you ask for a frame of the Loop:

```sh
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o still.png
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 --frame 12 -o frame-12.png
```

sonar has 24 frames, so `--frame` runs from 0 to 23 and `render` stops on anything higher. A Still Tool accepts only frame 0.

On sonar, kiosk, frond, pith, bloom, mosh, relief, atlas, motley, pane, sampler, totem, whorl, weave, zig, quilt, dahlia, carve, optic and vee, frame 0 is the Still. A page can show the Still while the video loads, or in place of the video when motion is off, and nothing jumps when playback starts. Other Tools start their motion mid-swing, glitch harder with motion on as static does, or start before the doodle is drawn as fete does, so their frame 0 differs from the Still. Each Tool's page says which kind it is.

## Loops are pre-rendered video

tirage does not animate in the browser. Drawing a 1080x1920 frame takes about 97 ms on a desktop CPU, which is 2 to 3 frames per second on a mid-range phone. Instead, tirage renders every frame of a Loop once and encodes them into an H.264 MP4. Any video element plays that with hardware decoding, and it caches as a file. The `tirage-encode` crate in the repository's `encode/` directory does the encoding: it takes a Recipe and returns the MP4 bytes. `tirage encode` does the same from the command line, in a CLI built with the `encode` feature.

A Loop is encoded at 720x1280 with the Tool's own frame count and frame rate, which each Tool's page gives. You render the Still at any size up to 8192 pixels an edge. `encode` refuses a Loop of 1.5 MB or more. The tests encode fixed Seeds of sonar, kiosk and frond, the Loops recordreel plays, and fail when one goes over.
