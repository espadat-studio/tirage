# Rust core on tiny-skia

Every Tool is written in Rust and drawn through tiny-skia 0.12 + kurbo 0.13, not in TypeScript on Canvas 2D like the playgrnd.tools pages we reproduce. The first consumer, recordreel, is a single Rust binary that needs Stills in-process next to takumi; a TypeScript core would need a JS runtime plus a native canvas on the server. tiny-skia and kurbo already sit in recordreel's lockfile, so the server binary grows by nothing, and tiny-skia's anti-aliasing is kin to the Skia that Chrome uses for the Reference exports.

## Considered Options

- **TypeScript on Canvas 2D**: closest to the source and a free browser Loop, but breaks recordreel's single-binary shape and puts server and browser on different rasterizers.
- **vello_cpu 0.3**: ~5x faster as WASM (20 ms vs 97 ms per 1080x1920 frame) and covers 49/52 Tools unaided, but younger, ~2x the gzip size, and a second rasterizer in recordreel. Neither engine reaches a live full-res 30 fps Loop on a phone, so speed did not decide it.

## Consequences

- Tools draw against our own surface mirroring the Canvas 2D subset; it owns Canvas semantics (straight vs premultiplied alpha) and is the seam if a live WASM Loop forces a rasterizer swap.
- We hand-write arcs/arcTo, blur and the save/restore stack that tiny-skia lacks.
- The core stays `wasm32-unknown-unknown`-clean (no threads, filesystem, `std::time` or OS RNG), enforced in CI.
- Toolchain tracks recordreel: edition 2024, Rust 1.99.0.
