# Text rendering for kiosk

Resolves #20. Measured 2026-10-08. Context: [Canvas 2D census](https://github.com/espadat-studio/tirage/blob/research/canvas2d-census/docs/research/canvas2d-census.md), [WASM size and speed](https://github.com/espadat-studio/tirage/blob/research/wasm-size-speed/docs/research/wasm-size-speed.md), [Reference export determinism](https://github.com/espadat-studio/tirage/blob/research/reference-export-determinism/docs/research/reference-export-determinism.md).

## Verdict

- **Use `skrifa` 0.44 outlines filled as tiny-skia paths.** skrifa is already in recordreel's lock through takumi, and takumi draws glyphs the same way. No shaping crate is needed.
- **Match Chrome's glyph pipeline, not just the outline.** For text up to 256 px, use the skrifa autohinter in light mode, snap the baseline to a whole pixel, and round x to 1/4 px. This gives **0.000-0.004% luma>48** against Chromium, for 2 fonts and 3 character sets at 540x960.
- **Above 256 px, Chrome switches to unhinted paths at float positions.** Plain unhinted skrifa outlines then match at 0.003-0.009%.
- A plain unhinted render lands at 1.6-3.4%. That alone eats or breaks the 1.6% ceiling from #13, before any background diff.
- **Pin the Reference font.** kiosk asks for a system monospace stack. On this machine Chromium resolves it to the user's fontconfig default, JetBrainsMono Nerd Font. Reference exports must run with a `FONTCONFIG_FILE` that holds only the bundled font.
- **Bundle DejaVu Sans Mono Bold 2.37** (Bitstream Vera licence plus public-domain DejaVu changes, 331,992 B, 199,402 B gzipped). JetBrains Mono Bold 2.304 (OFL 1.1, 277,828 B) is the runner-up. Both match equally well.
- WASM cost (whole cdylib: tiny-skia + skrifa, opt 3, simd128): 169 KB gzipped unhinted, **246 KB with the autohinter**. tiny-skia alone was about 75 KB in #11. The build has zero imports and runs in node.

## What kiosk draws

Source: `https://www.playgrnd.tools/kiosk/` (page SHA-256 prefix `485f066f17e4`, fetched 2026-10-08). Behaviour below is described in prose. No code is copied.

| Question | Answer |
|---|---|
| Canvas text calls | `fillText` (1 call site in the painter), `font`, `textAlign = "center"`, `textBaseline = "middle"`. `measureText` only in the SVG exporter, to find the middle-to-alphabetic shift for `<text>`. |
| Not used | `strokeText`, `letterSpacing`, `direction`, `fontKerning`, transforms around text, `maxWidth`. The painter never translates, rotates or scales. |
| String per call | Always 1 character, picked with `charAt`. So no kerning, ligatures or multi-glyph layout apply. |
| Font | `700 <size>px` plus the stack `ui-monospace,SFMono-Regular,Menlo,Consolas,'DejaVu Sans Mono',monospace`. The size is rounded to 0.1 px (`toFixed(1)`). Weight is fixed at 700. |
| Web fonts | The page loads DM Mono from Google Fonts, but only for the UI chrome (`--mono` CSS variable). The painter never names it. No `@font-face`. |
| Layout | A grid with `cols = grid` and `rows = round(H / (W / cols))`. Pass 1 draws big letters at cell centres (y at 0.52 of the cell), size `rh * bigSize`, kept when a hash is below `density`. Pass 2 draws small letters at cell corners, size `rh * smallSize`, kept when below `small`. |
| Parameters | `grid` 4-30, `bigSize` 0.3-1.6, `smallSize` 0.1-0.9, `density` 0-1, `small` 0-1, plus the character set chip and a free Custom text field. |
| Character sets | DOS `0369#%&@!;,'()`, Stipple `.,;:'"^~*`, Blocks `░▒▓█■▪·`, Code (13 ASCII slashes, brackets and operators), Digits `0-9`, Runes `†‡§¶®©≠∞≈`, Custom (default `.:-=+*#%@`, empty falls back to DOS). The page says atlas shares the stack and the sets. |
| Palette | Each glyph is filled with the Palette's darkest ink (hash < 0.28) or the first remaining ink. Both are roles, not fixed colours. |
| Seeded | The cell kept, the character and the colour each come from an integer hash of (column, row, seed + salt). |
| Loop | In `shuffle` mode the text seed becomes `seed + frame * 7919`, so the characters, colours and kept cells change every frame. Grid, sizes and positions do not change. In `bloom` mode the text is identical on every frame, and only the band colours step. |
| Glyph count | Defaults at 540x960 (9:16): 13 x 23 cells, about 250 big + 180 small = about 430 `fillText` per frame, at 20.9 px and 18.4 px. |
| Size range | 540 px wide: up to about 219 px (grid 4, bigSize 1.6). 2400 px wide, the default: about 94 px at the defaults and up to 960 px. Both Chrome regimes (≤256 px and >256 px) are reachable. |

## How Chrome rasterises canvas text (Linux)

| Step | Behaviour | Source |
|---|---|---|
| Font choice | Through fontconfig. `ui-monospace` is not a Linux family, and SF Mono, Menlo and Consolas are absent, so the first hit is DejaVu Sans Mono, then the generic `monospace`. Measured: with the default config here, the stack renders byte-identical to `'JetBrainsMono Nerd Font'`. | `bench/kiosk-text/chrome.ts` (`stack==named`) |
| `textBaseline = "middle"` | Offset = (normAscent - normDescent) / 2. `normAscent` = round to 1/64 px of (sTypoAscender × size / (sTypoAscender - sTypoDescender)). `normDescent` = round(size) - normAscent. These are OS/2 typo metrics, not hhea. | [text_metrics.cc `GetFontBaseline`](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/third_party/blink/renderer/core/html/canvas/text_metrics.cc), [simple_font_data.cc `TrySetNormalizedTypoAscentAndDescent`](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/third_party/blink/renderer/platform/fonts/simple_font_data.cc) |
| `textAlign = "center"` | x minus half the advance width. For one glyph that is the `hmtx` advance at the requested size. | measured, 0.000% |
| Hinting | Fontconfig on this host: `hintstyle` 1 (slight), antialias on, grayscale. Skia maps slight to `FT_LOAD_TARGET_LIGHT`, which "implies FORCE_AUTOHINT". So FreeType's light autohinter runs, not the font's TrueType bytecode. | `fc-match -v monospace:bold`, [SkFontHost_FreeType.cpp](https://skia.googlesource.com/skia/+/refs/heads/main/src/ports/SkFontHost_FreeType.cpp) |
| Glyph position | x is quantised to 1/4 px (2 subpixel bits), and the baseline y is rounded to a whole pixel. | [SkGlyph.h `kSubPixelPosLen = 2`](https://skia.googlesource.com/skia/+/refs/heads/main/src/core/SkGlyph.h), measured |
| Large text | When the text size exceeds 256 px (`memoryLimit`), Skia draws the glyph as a path. It is unhinted and not snapped. | [SkStrikeSpec.cpp `ShouldDrawAsPath`](https://skia.googlesource.com/skia/+/refs/heads/main/src/core/SkStrikeSpec.cpp), measured |
| Gamma | Total ink matches within 0.02% (mean grey 0.94666 vs 0.94653), so no extra mask gamma needs to be modelled at this bar. | measured |

## Measurement

Setup: Chromium 152.0.7977.82 headless (playwright-core), skrifa 0.44.0, tiny-skia 0.12.0. The canvas is 540x960 on paper `#f3efe0`, with inks `#141414` and `#d7263d`. The grid is kiosk-like but my own: 13 columns, 2 passes at 0.5 and 0.44 of the row height, 496 glyphs, mulberry32 seed 20261008. Chromium runs with a `FONTCONFIG_FILE` that holds only the font under test, plus slight hinting, so the Rust side reads the exact file Chromium used. Metric: recordreel's, the share of pixels whose absolute RGB difference has 601-luma > 48, over the full frame.

| Font | Set | blank | unhinted | unhinted, snap | bytecode light, snap | **autohint light, snap** |
|---|---|---|---|---|---|---|
| DejaVu Sans Mono Bold | DOS | 6.532% | 1.708% | 1.415% | 1.401% | **0.004%** |
| DejaVu Sans Mono Bold | Blocks | 13.376% | 2.144% | 0.996% | 1.498% | **0.000%** |
| DejaVu Sans Mono Bold | Runes | 7.296% | 2.556% | 1.975% | 2.502% | **0.004%** |
| JetBrains Mono Bold | DOS | 6.322% | 1.580% | 1.193% | 0.348% | **0.000%** |
| JetBrains Mono Bold | Blocks | 15.024% | 3.353% | 1.116% | 2.376% | **0.000%** |
| JetBrains Mono Bold | Runes | 7.492% | 2.245% | 1.780% | 0.932% | **0.001%** |

"blank" is paper only, which shows how much of the frame the text covers. "snap" means x rounded to 1/4 px and y rounded to a whole pixel. Snapping x to a whole pixel instead made every variant worse (1.4-3.7%). A ±0.25 px global offset sweep of the unhinted render had its minimum at (0, 0). The error is in the per-glyph pipeline, not the layout.

Large glyphs (DejaVu, DOS, 4 columns):

| Sizes | unhinted, float | autohint light, snap |
|---|---|---|
| 72-82 px | 0.470% | **0.002%** |
| 272-309 px | **0.009%** | 0.709% |

Unpinned run on this host (default fontconfig → JetBrainsMono Nerd Font Bold, 2.5 MB at `/usr/share/fonts/TTF/`): upstream JetBrains Mono Bold 2.304 renders it at 0.000% (DOS, Blocks) and 0.001% (Runes). For these characters the Nerd patch does not change the glyphs.

WASM (`bench/kiosk-text/wasm`, one exported function that hints and fills one glyph; `lto`, `codegen-units = 1`, `panic = "abort"`, `strip`, opt 3, `+simd128`):

| Build | raw | gzip -9 | imports |
|---|---|---|---|
| skrifa unhinted + tiny-skia | 493,209 | 169,127 | 0 |
| skrifa autohinter + tiny-skia | 740,438 | 246,011 | 0 |

Setting `opt-level = "z"` for only skrifa and read-fonts (through `CARGO_PROFILE_RELEASE_PACKAGE_*` env vars) changed nothing under this LTO setup. Not pursued.

## Crate choice

| Option | In recordreel lock | wasm32 | Pipeline match | Verdict |
|---|---|---|---|---|
| **skrifa 0.44** outlines → tiny-skia `fill_path` | yes (via takumi, parley, harfrust) | yes, 0 imports | yes: has a FreeType-compatible autohinter (`Engine::Auto`, `SmoothMode::Light`) and an unhinted path | **pick** |
| parley 0.11 / harfrust 0.12 (shaping) | yes | yes | not needed: kiosk draws 1 character per call | skip until a Tool draws strings |
| swash | no | yes | own rasteriser and hinter; duplicates skrifa | skip |
| ab_glyph / fontdue | no | yes | own rasteriser, no autohinter; cannot reproduce Skia's AA plus hinting | skip |
| cosmic-text / fontdb / fontique | fontique only | fontique memory-maps system fonts | system font discovery is the wrong model for a bundled, pinned font | skip |

takumi uses `DrawSettings::unhinted` into tiny-skia (`takumi-core-0.25.0/src/resources/glyph.rs:545`). Its crates are the same, but we need the hinted branch for text ≤256 px.

Renderer rules derived from this study, for the Canvas 2D-subset surface:

1. Parse `font` as weight, px size and family list. Map the whole monospace stack to the one bundled face.
2. `measureText` and centring use the `hmtx` advance at the requested size. `middle` uses normalised OS/2 typo metrics, rounded as Blink rounds them.
3. Size ≤ 256 px: autohint light, x rounded to 1/4 px, baseline y rounded to a whole pixel. Size > 256 px: unhinted, float position.
4. Fill with nonzero winding and anti-aliasing on.

Optional speed-up, not measured: per frame, kiosk uses 2 sizes, at most 14 characters for the fixed sets, and 4 x phases. Glyph coverage masks can be cached by (char, size, phase) and composited, as Skia's strike cache does. Hinting then runs once per glyph, not about 430 times per frame.

## Fonts to bundle

| Font | Source (exact) | File SHA-256 | Size | Licence | Glyphs | Notes |
|---|---|---|---|---|---|---|
| **DejaVu Sans Mono Bold 2.37** | [dejavu-fonts-ttf-2.37.tar.bz2](https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.tar.bz2), member `ttf/DejaVuSansMono-Bold.ttf` | `bce60f1b…6bf8f769` | 331,992 B, 199,402 B gzipped | Bitstream Vera Fonts licence (Bitstream glyphs) + Arev licence (Tavmjong Bah glyphs); DejaVu changes are public domain. Text: `LICENSE` in the same tarball. | 3,316 | Named in kiosk's own stack, so any Linux Chromium with DejaVu resolves it. Typo metrics 1556/-492 at 2048 upm. |
| JetBrains Mono Bold 2.304 | [JetBrainsMono-2.304.zip](https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip), member `fonts/ttf/JetBrainsMono-Bold.ttf` | `5590990c…142479dcb` | 277,828 B, 130,450 B gzipped | SIL OFL 1.1, no Reserved Font Name. Text: `OFL.txt` in the zip. | 1,743 | What this host resolves today, through the user's fontconfig default. Typo metrics 1020/-300 at 1000 upm. |

Licence obligations:

- Bitstream Vera licence: keep the copyright and permission notice with every copy. Do not sell the font on its own. A modified font, subsets included, must drop the names "Bitstream" and "Vera". "DejaVu" contains neither, so a subset may keep it. The Arev section has the same terms, with "Arev" as the reserved word.
- OFL 1.1: ship `OFL.txt`. Do not sell the font on its own. Bundling in software is allowed. With no Reserved Font Name, a subset may keep its name.

Why DejaVu over JetBrains Mono:

- The Tool itself names DejaVu, so a stock Linux Reference box reproduces it without a custom fontconfig.
- It has twice the coverage, which matters for the Custom set.
- The cost is +69 KB gzipped and a non-OFL licence, which is still permissive.

SF Mono, Menlo and Consolas are proprietary and cannot be bundled.

Reference export recipe addition (to ticket #13):

- Run `pg.ts` with `FONTCONFIG_FILE` pointing at a config whose only `<dir>` holds the bundled TTF, with slight hinting and grayscale. `bench/kiosk-text/run.sh` writes one.
- Record the font SHA-256 next to the page hash and the Chromium build.

## Reproduce

`bench/kiosk-text/run.sh` downloads both fonts and checks their SHA-256. It then renders each set in Chromium (`chrome.ts`) and with skrifa (`src/main.rs`), prints every variant's share, and builds the 2 WASM variants. It needs bun, cargo with `wasm32-unknown-unknown`, and `/usr/bin/chromium`. Output goes to `bench/kiosk-text/out/` (gitignored).

## Open questions

Sharp:

1. Which font: DejaVu Sans Mono Bold (recommended) or JetBrains Mono Bold? Either way, Reference exports must pin it through `FONTCONFIG_FILE`.
2. Custom set coverage: the Custom field accepts any character. With a single-font fontconfig, what does Chromium draw for a character the font lacks: `.notdef`, nothing, or a fallback? Our renderer must do the same. Not tested.
3. Astral characters in Custom: `charAt` splits surrogate pairs, so kiosk draws half an emoji. Chromium's output for a lone surrogate (likely U+FFFD or `.notdef`) is not tested.
4. Can we accept +77 KB gzipped for the autohinter? It is needed below 256 px, and at 540 px wide every Parameter value stays below 256 px.
5. SVG export: kiosk emits `<text>` with the font stack, so the result depends on the viewer's fonts. Do we emit `<text>` to match, or glyph outlines for stable pixels?

Fog:

- The match was measured on one Chromium build (152) and one host. Skia's autohinter or subpixel rules may drift across builds, as the determinism study warned for AA in general.
- Exports at 2400 px and above were not measured against the real kiosk. The 256 px switch is measured, but no glyph above 480 px was tested.
- Grain and dither run after text in kiosk. Their effect on the text diff is not measured here.
