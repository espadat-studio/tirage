---
title: "Still and Loop"
description: "One Recipe gives one frame or a short video, and the first frame of the video is that frame."
---

A _Visual_ is what a Recipe renders. It is either a _Still_, a single frame, or a _Loop_, an animation that ends where it starts: after its last frame, frame 0 plays again. The Tool decides which you get. husk, vein and aura draw Stills. sonar, kiosk and frond draw Loops. `tirage tools` lists the frame count beside each Tool, and a Still Tool has 1.

## Frame 0 is the Still

A Loop's first frame is the Still of the same Recipe. `render` draws frame 0 unless you ask for another:

```sh
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o still.png
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 --frame 12 -o frame-12.png
```

sonar has 24 frames, so `--frame` runs from 0 to 23 and `render` stops on anything higher. A Still Tool accepts only frame 0.

Because the Still is frame 0, a page can show it while the video loads, or in place of the video when motion is off, and nothing jumps when playback starts.

## Loops are pre-rendered video

tirage does not animate in the browser. Drawing a 1080x1920 frame takes about 97 ms on a desktop CPU, which is 2 to 3 frames per second on a mid-range phone. Instead, tirage renders every frame of a Loop once and encodes them into an H.264 MP4. Any video element plays that with hardware decoding, and it caches as a file. The `tirage-encode` crate in the repository's `encode/` directory does the encoding: it takes a Recipe and returns the MP4 bytes.

A Loop is encoded at 720x1280 with the Tool's own frame count and frame rate: sonar is 24 frames at 10 fps, kiosk 24 at 6 fps, frond 36 at 12 fps. You render the Still at any size up to 8192 pixels an edge. Every Loop has to stay under 1.5 MB. The tests encode fixed Seeds of each Loop Tool and fail when one goes over.
