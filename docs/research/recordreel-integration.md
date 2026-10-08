# recordreel integration points for Visuals

Resolves #5. Read-only survey of `espadat-studio/recordreel` (local checkout, 2026-10-08) and the takumi crates it locks: `takumi 2.14.0`, `takumi-core 0.25.0`, `takumi-raster 0.5.1`, `takumi-html 0.3.4` (`Cargo.lock`).

Paths starting `crates/`, `docs/`, `prototype/` or naming `DESIGN.md`/`PRODUCT.md` are in recordreel. Paths starting `takumi-*` are in `~/.local/share/cargo/registry/src/index.crates.io-*/`.

## Summary

- A Still enters as image bytes keyed by a string. takumi matches that key against `<img src>` or `url()`. Today recordreel feeds two kinds of image: static design assets compiled into the binary, and covers fetched at render time.
- A per-Reel Still fits the cover path, not the static-asset path. The Seed must appear in the key written in the HTML, because the card cache hash covers the HTML but not the dynamic image bytes.
- PNG, JPEG, WebP, GIF and SVG all decode. SVG is rasterised by resvg. Animated GIF/APNG/WebP are accepted, and a still render draws the frame at `time_ms`.
- Five of the seven shipped designs already use playgrnd rasters. Three use a full-Slide `url()` background; the rest use `<img>` inside an `overflow: hidden` wrapper.
- There is no Reel table. A Seed can be derived from `(collectors.id, reel)` with no migration. Storing one needs a new column or table.
- The Reel page has no CSP, and its JS is inline or same-origin. A WASM module can be served from the binary. The asset route needs an `application/wasm` MIME type. ADR-0001 rejected WASM for first paint, so a Loop has to be an add-on over the Still.
- Every motion rule sits under `prefers-reduced-motion: no-preference`. A Loop must follow the same rule and show the Still otherwise.

## 1. How takumi takes image assets

### recordreel's wrapper (`crates/card/src/lib.rs`)

| Step | Where | What |
|---|---|---|
| Static assets | `lib.rs:68-89` | `Renderer::new(css, fonts, assets: &[(&str, &[u8])], size, encoding)`. Each asset is decoded once, when the renderer is built. |
| Per-render images | `lib.rs:91-101` | `render(html, covers: Vec<(String, Vec<u8>)>)` decodes covers on every call, then merges in the static assets. |
| Decode | `lib.rs:167-178` | `ImageSource::from_bytes(bytes)` for each entry, keyed by `Arc<str>` = the src string. |
| Missing-key guard | `lib.rs:103-105`, `180-189` | Fails the render if an `<img src>` has no entry. It checks only `<img>` nodes, not CSS `url()`. |
| Render + encode | `lib.rs:106-138` | `takumi::render(RenderOptions{viewport, node, fonts, images, stylesheet})`, then JPEG q85 via `jpeg-encoder` or PNG via `image`. |
| Cover filter precedent | `lib.rs:142-165`, `crates/app/src/designs/mod.rs:241-248` | `dither()` is a per-design `CoverFilter` run on cover bytes before render (One-bit Crate). A Visual could get the same hook. |

How assets are keyed (`crates/app/src/designs/mod.rs`):

- Static assets are `include_bytes!` entries in each design's `assets` (e.g. `designs/wheatpaste_wall.rs:14-25`).
- They are registered as `/designs/<slug>/<file>` (`mod.rs:155-161`). `.ttf` files are filtered out of the image map.
- The same path is served over HTTP by `design_asset` (`crates/app/src/lib.rs:305-316`), so one path works for both the browser and takumi. DESIGN.md "Assets" (`DESIGN.md:314`) records the rule.
- The card cache key is `Built.inputs` (slug, both stylesheets, fonts, static assets and their bytes, encoding; `mod.rs:192-210`) plus the rendered HTML (`mod.rs:221-230`). **Per-render image bytes are not hashed.** Covers work because their Discogs URL is in the HTML. A seeded Still must also put its Seed and Recipe version in the src (e.g. `/visuals/aura-<seed>-v1-9x16.png`), or a changed Visual would be served from a stale cached card.
- Covers are fetched in `crates/app/src/share_cards.rs:84-93` (`state.discogs.cover(src)`), then rendered on a blocking thread. A Still generated in-process would slot in there as one more `(src, bytes)` pair, with no network fetch.

### takumi internals

| Fact | Source |
|---|---|
| Features on: recordreel uses `features = ["from-html"]` and keeps the defaults (recordreel `Cargo.toml`). Defaults: `raster-backend`, `svg-source`, `image-decoding` (= jpeg, webp, gif), `woff`, `woff2`, `rayon`. | `takumi-2.14.0/Cargo.toml` `[features]` |
| PNG and ICO are always on (`image` crate features `png`, `ico`). | `takumi-core-0.25.0/Cargo.toml` `[dependencies.image]` |
| `ImageSource::from_bytes`: if the bytes are UTF-8 containing `<svg`, parse as SVG (usvg). Otherwise, if GIF/APNG/animated WebP, make an `Animated` source. Otherwise decode as a bitmap. | `takumi-core-0.25.0/src/resources/image.rs:762-788`, `is_svg_like` `:951-953` |
| `ImageSource` variants: `Svg`, `Bitmap`, `Animated`, `Encoded` (lazy decode at draw size; recordreel never uses it). | `image.rs:70-80` |
| How a src resolves: `data:` URI, then inline SVG markup in the src string, then lookup in the `images` map, else `ImageError::Unknown`. | `takumi-core-0.25.0/src/layout/node/image.rs:194-211` |
| A CSS `url()` background goes through the same resolver. **A missing key is skipped silently** (`if let Ok`), so the `<img>` guard above does not catch it. | `takumi-raster-0.5.1/src/background_drawing.rs:413-414` |
| Animated sources draw the frame at the render context's `time_ms`. | `background_drawing.rs:424-454`, `takumi-core-0.25.0/src/context.rs:117` |
| takumi can also render timelines (`render_animation`, `write_animated_gif/png/webp`). This is a possible server-side Loop path, but DESIGN.md rejects WebP for sharing (`DESIGN.md:280`). | `takumi-2.14.0/src/lib.rs:86-89`, `takumi-raster-0.5.1/src/render.rs:621-633` |

Size limits:

- Bitmap decode is capped at 8192 px per edge and 8192x8192 px total. Source: `takumi-core-0.25.0/src/resources/image_decoder/mod.rs:42-46`.
- Animation: at most 1024 frames and 4x that pixel total across all frames. Source: `image_decoder/mod.rs:122-125`.
- SVG rasterisation: at most 16 Mi px. Source: `image.rs:57-62`.
- The 16 MiB `ResourceCache` budget (`image.rs:1151`) does not apply, because recordreel passes a pre-decoded map.
- Practical bound: a 1080x1920 Slide is 2.07 Mpx, so a full-Slide Still is far inside every limit. RGBA decode costs about 8.3 MB per full-Slide Still per render.

SVG support:

- SVG works through resvg.
- `currentColor` takes the host `color` (`image.rs:121-148`, `193-207`).
- `<text>` re-parses with the registered fonts (`image.rs:96-100`, `183-185`).
- Intrinsic size comes from `width`/`height`/`viewBox` and is used for `background-size` (`image.rs:93-95`, `210-217`).
- Caveat: SVG is rasterised by resvg, not by the browser. The fidelity check has to compare takumi's raster, not Chromium's.
- Gap: `design_asset` maps only `jpg`, `png` and `ttf` to a MIME type. Anything else, `svg` included, is served as `application/octet-stream` (`crates/app/src/lib.rs:309-314`). `/rise.svg` has its own route (`lib.rs:238-241`).

## 2. Backgrounds and textures in Slides today

DESIGN.md "Synthetic covers" (`DESIGN.md:282`): prototype covers are playgrnd rasters (aura 7, optic 11, rise 3, mist 3, riso 3), stand-ins only (`prototype/directions/covers/`). Real covers come from Discogs. Shipped design textures are also playgrnd rasters, named `<tool>-<seed>-<ratio>` (`DESIGN.md:314`).

| Design | playgrnd assets | How used | Source |
|---|---|---|---|
| 00 Sealed Release | none | Gradients only (cellophane wrap and barcode) | `designs/sealed_release.rs:14`, `DESIGN.md:205` |
| 01 Variant Press | `vein-3`, `vein-8`, `mist-2`, `aura-4` (1x1) | Disc pressing `<img>` in a round `overflow: hidden` wrapper; picked per Slide kind | `templates/designs/variant-press/slide.html:3`, `src/designs/variant_press.rs:85-92`, `DESIGN.md:327` |
| 02 Photocard Binder | `aura-6-3x4` | Holo foil `<img>` in wrappers (bands, RARE stamp) | `templates/designs/photocard-binder/slide.html:2,17,89,130,209,243`, `DESIGN.md:337` |
| 03 Wheatpaste Wall | `kiosk-3`, `riso-3`, `splice-3` (9x16) | `kiosk` is the only full-Slide `url()` (`background-size: 100% 100%`); `riso` old sheet and `splice` grain are `<img>` | `static/designs/wheatpaste-wall/slide.css:12-13`, `slide.html:4,8`, `DESIGN.md:347` |
| 05 Test Pressing | `husk-21-1x1` | Disc surface `<img>` under lacquer | `templates/designs/test-pressing/slide.html:3,80`, `DESIGN.md:356` |
| 07 One-bit Crate | `sonar-11` (9x16, 16x9 PNG) | Full-Slide `url()` desktop | `static/designs/one-bit-crate/slide.css:8`, `link_card.css:3`, `DESIGN.md:362` |
| 08 Paste-up Tropical | `frond-4` (9x16, 16x9), `coral-5-9x16` | `frond` full-Slide `url()` with `cover`; `coral` leaf panels as `<img>` | `static/designs/paste-up-tropical/slide.css:8-10`, `slide.html:12,106`, `DESIGN.md:373,377` |

Unshipped directions in `prototype/directions/*/assets/` add whorl, pith, stipple, pane and culture (`prototype/directions/README.md`). The landing `.close` section uses `/rise.svg` as a CSS background (`static/site.css:496`).

takumi caveats on backgrounds (DESIGN.md "Build Rules", `DESIGN.md:270-283`, plus per-design notes):

- A `url()` background with `background-size: cover` is not clipped to its box and can flood the Slide. Use `<img>` + `object-fit: cover` in an `overflow: hidden` wrapper, unless it covers the whole Slide (`:274`, `:377`).
- `border-radius: 50%` on an image wrapper renders as an egg. Use a fixed radius in `m` (`:275`, `:327`).
- An `<img>` that shares a box with siblings gets stretched. Give it its own absolutely positioned wrapper (`:277`).
- A `repeating-linear-gradient` on a bordered or shadowed box floods outside it. Put it on a child in an `overflow: hidden` parent (`:367`).
- `background-clip: text` works only on the element that holds the text (`:276`). Test Pressing swapped its husk ink texture for a gradient because light patches failed contrast (`:356`).
- `slide.css` has to parse strictly with `StyleSheet::parse`: no `clamp()`, `min()`, `max()`, `@font-face` or 3D transforms (`:272`, `:301`, `PRODUCT.md:35`).
- Fidelity: Chromium vs takumi differ by 1.1-2.2% of pixels, 0.3-1.0% after 1px erosion (`:281`). Per-design diffs run 0.18-1.64% (`:327-377`).
- Size budgets: Share card under 1 MB, Link card under 300 KB (`:280`). Busy full-bleed textures push JPEG size: Wheatpaste is 218-275 KB and Paste-up 220-321 KB, against 107-204 KB elsewhere.
- Contrast is measured behind glyph pixels. Text must never sit straight on a texture: Paste-up puts every run on a plate (`:376`), and Wheatpaste allows no text on the wall (`:343`). A generated Still has to stay within the same contrast rules, which is part of "taste bounds".

Implication: the fitting entry points are (a) a full-Slide `url()` on `.slide`/`.link-card` and (b) an `<img>` texture in a clipped wrapper. Both exist and are takumi-checked. A Still is a drop-in replacement for any row above, as long as the Seed is in the key.

## 3. Where a per-Reel Seed could live

The data model (`crates/app/migrations/`):

- There is no Reel row. A Reel is `(collector, reel)` with `reel ∈ {"all", "<current year>"}`, resolved by `reels::year` (`crates/app/src/reels.rs:80-87`).
- Slides are derived from `records` on every request (`reels.rs:89-132`). `reel::Kind` is `Year(i16) | AllTime` (`crates/reel/src/lib.rs:19-22`).
- `collectors.id` is the Discogs identity id (`crates/app/src/auth.rs:101-110`), so it is stable across sign-ins. `collectors.design` holds the chosen design slug (`migrations/20261008120000_designs.sql`).
- `shares (collector_id, reel)` exists only for Shared Reels (`migrations/20261007140000_shares.sql`).
- Erasure cascades from `collectors` (`crates/app/src/erasure.rs:62`).
- A determinism precedent already exists: Paste-up Tropical seeds its layout with FNV-1a over the text plus a splitmix-style `unit(seed, i)` (`crates/app/src/designs/paste_up_tropical.rs:205-220`, `354-359`).

| Option | Change | Pros | Cons |
|---|---|---|---|
| A. Derive | `seed = H(collectors.id, reel, design.slug)` in Rust, no migration | Stable, free, follows the Paste-up precedent, nothing new to erase | No re-roll unless an epoch is mixed in; the Seed is guessable (harmless) |
| B. Column on `collectors` | `ALTER TABLE collectors ADD COLUMN visual_seed INTEGER` | Re-roll possible; erased by cascade | One Seed for both Reels unless mixed with `reel` |
| C. Table | `reel_visuals (collector_id REFERENCES collectors ON DELETE CASCADE, reel TEXT, seed INTEGER, PRIMARY KEY (collector_id, reel))`, like `shares` | Per-Reel, re-rollable, erased by cascade | A migration and a write path for data that A gives for free |

Lean: A, with `reel` and `slug` in the hash, so switching design gives a new Visual (switching design already purges cards, `DESIGN.md:315`). Move to C only if Collectors get a "re-roll" control.

## 4. Reel page asset loading, and whether WASM fits

Current loading (`crates/app/templates/reel.html`):

- All CSS is inlined in one `<style>`: `slide_css + reel.css + page_css` (`:24`).
- htmx (`/htmx-2.0.11.min.js`, 51 KB raw, 16.8 KB gzip, 15.3 KB brotli, measured) loads `defer`, and only on owner and updating views. **Shared public Reels load no htmx** (`:25`).
- `share.js` is inlined (`:74`, `crates/app/src/reels.rs:22`). The landing page inlines `pick.js` (`templates/landing.html:37`).
- Slide images are same-origin `/designs/<slug>/<file>` `<img>`/`url()` served from `include_bytes!` with `max-age=86400` (`crates/app/src/lib.rs:305-325`). htmx and Archivo are `immutable` for one year (`lib.rs:275-289`).
- htmx swaps only the `#sync`/`#reel-status` fragments every 5 s (`templates/sync_status.html:3`, `templates/reel_status.html:2`). A fresh Shared Reel answers with `hx-refresh` (`crates/app/src/share.rs:305-307`). Slides are never swapped, so a Loop mounted on page load will not be torn down by htmx.

Hosting a WASM module:

- **CSP: none.** No `Content-Security-Policy` header exists anywhere in `crates/` or `deploy/`. WASM needs no `wasm-unsafe-eval` today. If a CSP is added later, it needs `script-src 'wasm-unsafe-eval'` plus hashes for the inline `share.js`/`pick.js`.
- **Serving:** add a route like htmx's (`lib.rs:275-281`) with `Content-Type: application/wasm` and `immutable` caching under a versioned name. `instantiateStreaming` requires that MIME type. The generic `design_asset` route would send `application/octet-stream` (`lib.rs:309-314`).
- **Origin guard:** `require_ours` blocks only unsafe methods without our Origin (`crates/app/src/origin.rs:10-18`), so a GET for `.wasm` passes.
- **Edge cache:** the Cloudflare Cache Rule lists `/r/`, `/c/`, `/leaderboards`, `/privacy` and the signed-out `/` (`docs/ops.md:21`). Unverified: whether Cloudflare's default cached extensions include `.wasm`. If not, add the path to the rule.
- **Budget:** the ADR rejected Leptos/WASM because "Reels arrive as shared links, often in Instagram/TikTok in-app browsers on phones. First paint is the product; a WASM bundle delays it" (`docs/adr/0001-single-rust-binary-on-sqlite.md:11`, `docs/agents/research/stack-selection.md:197`). No explicit byte budget is written down. Proposal (not from the repo): keep the Loop module at or under htmx's ~16 KB compressed, and load it after first paint.
- **Shape that fits:** server renders the Still as `<img>` (so takumi and the browser match and crawlers see it). Then a small inline script, gated on reduced motion, lazily fetches the `.wasm` for the Slide in view (IntersectionObserver) and swaps a `<canvas>` over the `<img>`. The Share card stays the Still. A Loop is a page-only element: `display: none` in `slide.css`, shown under `.reel` by `page.css` (`DESIGN.md:273`).

## 5. prefers-reduced-motion rules already in place

- DESIGN.md "Motion (page only)" (`DESIGN.md:264-268`): all motion is scroll-driven, inside `prefers-reduced-motion: no-preference` and `@supports (animation-timeline: view())`. Print order: glare, then sleeves, then stickers. The one looping motion is the Sync bar; under reduced motion its track stays empty and the label carries the state.
- Code matches. Every design's `page.css` opens its animations with `@media (prefers-reduced-motion: no-preference)`:
  - `static/designs/sealed-release/page.css:11`
  - `variant-press/page.css:259`
  - `photocard-binder/page.css:312`
  - `wheatpaste-wall/page.css:197`
  - `test-pressing/page.css:208`
  - `one-bit-crate/page.css:295`
  - `paste-up-tropical/page.css:210`
  - Site-wide: `static/site.css:580`
- Infinite loops exist only inside that guard: One-bit marching ants (`one-bit-crate/page.css:297,304`) and the Sync bar (`site.css:591`). Paste-up also sets `transition: none` under `reduce` (`paste-up-tropical/page.css:242-246`).
- Rule for a Loop: start only when `matchMedia('(prefers-reduced-motion: no-preference)').matches`, and listen for changes. Otherwise leave the Still `<img>` in place. A Loop has to end where it starts (glossary), so pausing on frame 0 equals the Still.

## Open points for the spec

- Still format: PNG vs JPEG for the generated bytes. They are decoded to RGBA before render either way, so this matters only for HTTP size on the page. SVG suits vector Tools but is rasterised by resvg, which needs its own fidelity check.
- Generation cost per render: covers are already fetched per card render. Adding Still generation in `share_cards::render` keeps one render path. It can be cached like cards (`share_cards.rs:95-110`).
- Card-guard gap: `card::unmatched` does not check `url()` keys. A seeded `url()` background with a typo renders blank without an error. Either use `<img>` for seeded Visuals or extend the guard.
