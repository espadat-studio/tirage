# Census of building blocks copied across Tools

Answers [#182](https://github.com/espadat-studio/tirage/issues/182). Source read at `b59309a`. Every claim cites `src/<file>.rs:<line>` at that commit.

`chassis` is not a Tool. `aura` and `terrain` are Tools whose helpers other Tools import. A Tool "uses" a shared block when it imports it (`use crate::…`) or calls it by path. Import counts come from parsing every `use crate::{chassis,aura,terrain}::…` in `src/`.

Groups:

- G1: identical up to renaming or a type spelling that does not change bits.
- G2: identical apart from constants that could become arguments.
- G3: near-copy. At least one op differs, so outputs differ.

## Summary

| Block | Definitions | Tools using any copy | Main split |
|---|---|---|---|
| Xorshift32 RNG | 3 (`chassis`, `aura`, `husk`) | 35 | logical vs arithmetic `>> 17`; seed scramble |
| Seed scramble before RNG | 5 families, inline | 31 | constant and integer vs f64 multiply |
| Integer hash | 5 (`chassis`, `sampler`, `chassis::noise_hash`, `aura`, `terrain`) | 39 | add-combine 1 round vs xor-combine 3 rounds |
| 2D value noise | 7 | 27 | hash family; mix vs lerp vs polynomial blend |
| fbm | 9 | 15 | octave seed step 131 vs 1319; gain and lacunarity; fixed vs loop |
| Domain-warped fbm level | 5 inline | 5 | constants |
| f32 grid + bilinear read | 8 inline reads | 8 | clamp vs wrap; frac before vs after clamp; blend form |
| Marching squares | 2 | 2 | `>=` vs `>`; saddle rule; chaining |
| `lerp` fn | 3 fns + 1 closure | 4 | none |
| Smoothstep `t*t*(3-2t)` | 12 inline + 1 fn | 13 | `mist` takes edges |
| Luma | 5 fns | 5 | Rec.601 vs Rec.709 |
| RGB mix and lighten | 9 | 10 | rounding: none, `store`, `round_half_up` |
| Fisher–Yates shuffle | 4 inline | 4 | draw source |
| Sub-seed `seed*7 + k` | 7 inline | 7 | none |
| Per-frame seed `seed + t*K` | 11 inline | 11 | `K` |
| Bayer 4x4 | 3 | 3 (incl. `chassis`) | table type |
| Field size from aspect | 3 inline | 3 | base edge 560 vs 640 |
| Loop drift circle `zw` | 5 inline | 5 | `AMOUNT` |

## Xorshift32 RNG

| Copy | Signature | Ops |
|---|---|---|
| `chassis.rs:43` `XorShift` | `new(seed: u32)`, `next(&mut self) -> f64` | `new` = `seed.max(1)`. `x ^= x<<13; x ^= x>>17` (logical, `u32`); `x ^= x<<5`; `/ 2^32` |
| `husk.rs:275` `Xorshift` (private) | tuple ctor, `next() -> f64` | Same three shifts, logical `>>`. Seed `match s*2654435761 {0=>1, s=>s}` at `husk.rs:296` equals `.max(1)` |
| `aura.rs:213` `Xorshift` | `new(tool_seed)`, `from_state(state)`, `next() -> f64` | `x ^= ((x as i32) >> 17) as u32` (arithmetic shift). `new` = `(f64(seed) * 2_654_435_761.0).rem_euclid(2^32) as u32`, then `.max(1)`. `from_state` = `.max(1)` |

Groups:

- G1: `husk::Xorshift` = `chassis::XorShift::new(tool_seed.wrapping_mul(2_654_435_761))`. That is the exact call in `culture.rs:311`, `splice.rs:389`, `strand.rs:383`, `whorl.rs:298` and `carve.rs:266`.
- G3: `aura::Xorshift` vs `chassis::XorShift`. Shift kind differs, so the streams split as soon as bit 31 is set. `new` also multiplies in f64. Above 2^53 that product rounds, so it is not `wrapping_mul`.

Users (Tools):

- `chassis::XorShift` (18): carve chaff cipher coral crowd culture dahlia fold frond hiss motley pith splice sprig stitch strand vein whorl.
- `aura::Xorshift` (17): aura fete filament mist modular optic parcel prism quilt riso specimen static tokens totem vee warp weave.
- `husk::Xorshift` (1): husk.
- Total: 36 Tools hold an Xorshift32. 35 use a shared one or the husk copy; no Tool uses both.

## Seed scramble before the RNG

Inline expressions fed to the RNG constructors.

| Family | Expression | Copies | Group |
|---|---|---|---|
| A: Knuth multiply | `XorShift::new(s.wrapping_mul(2_654_435_761))` | culture:311, splice:389, sprig:445, strand:383, whorl:298, carve:266, husk:296 (own RNG) | G1 |
| A' | `XorShift::new(s.wrapping_add(3).wrapping_mul(2_654_435_761))` | pith:374 | G2 (+3) |
| B: Murmur multiply + xor | `XorShift::new(s.wrapping_mul(2_246_822_519) ^ 0x27d4_eb2f)` | cipher:394, crowd:327, dahlia:309, fold:409, hiss:455, motley:347, stitch:359, vein:441 | G1 |
| B | `XorShift::new((s as i32).wrapping_mul(-2_048_144_777) as u32 ^ 0x27d4_eb2f)` | coral:376 | G1: `-2_048_144_777 ≡ 2_246_822_519 (mod 2^32)`, so the bits match |
| B' | `s.wrapping_mul(2_246_822_519)` without xor | chaff:340, carve:296 (per panel, after `+ i*7919`) | G2 |
| B'' | `s.wrapping_mul(3_266_489_917) ^ 0x1656_67b1` | hiss:551 | G2 |
| C: f64 golden + k | `(f64(s) * 2_654_435_761.0 + k).rem_euclid(2^32) as u32` into `aura::Xorshift::from_state` | aura:217 (k=0, inside `new`), filament:426 (k=13), specimen:497 (k=7), modular:342 (k=17) | G2 (k) |
| C | `(f64(s) * 2_654_435_761.0 + 17.0) as u64 as u32` | tokens:558 | G1 with modular. The value is integral and below 2^64, so `as u64 as u32` gives the same result as `rem_euclid(2^32)` |
| D: xor salt | `Xorshift::from_state(s ^ K)` | fete:316 (`0x51ed_270b`), fete:444 (`0x2545_f491`), static:391 | G2 |
| E | `base = (s+1)*2_654_435_761`; `XorShift::new(base ^ K)` for 4 streams | frond:587-591 | G3 vs A (+1, xor) |

Family C differs from A in arithmetic, not only in constants. A wraps in `u32`, C multiplies in f64, so they are G3 to each other.

## Integer hash

| Copy | Signature | Ops | Group |
|---|---|---|---|
| `chassis.rs:7` `hash` | `(x: i32, y: i32, seed: u32) -> f64` | `x*374761393 + y*668265263 + seed*1274126177` (wrapping add); `n = (n ^ n>>13) * 1274126177`; `n ^= n>>16`; `/ 2^32` | reference |
| `sampler.rs:316` `hash` | `(x: u32, y: u32, s: u32) -> f64` | Same ops on `u32` inputs. `chassis::hash(x as i32, y as i32, s)` gives the same bits | G1 |
| `chassis.rs:483` `noise_hash` | `(x: usize, y: usize) -> f64` | Same constants, no seed. JS-number emulation: products in f64, `to_int32` via `rem_euclid`, `>> 13` on `i32` (arithmetic), second multiply in f64 (loses bits above 2^53) | G3 |
| `aura.rs:235` `hash` | `(x: i32, y: i32, channel: u32, seed: u32) -> f64` | `x*374761393 ^ y*668265263 ^ channel*1440662683 ^ seed*1013904223` (xor-combine); three rounds `>>15 *2246822519`, `>>13 *3266489917`, `>>16` | reference |
| `terrain.rs:264` `hash` | `(x, y, z, w: i32, seed: u32) -> f64` | `aura::hash` plus `^ w*1274126177`. `terrain::hash(x, y, z, 0, s)` = `aura::hash(x, y, z as u32, s)` | G2 (w=0 collapses it) |

`chassis::hash` vs `aura::hash` is G3: different combine, constants and number of rounds.

Users:

- `chassis::hash` (23): atlas carve chaff cipher coral crowd culture dahlia frond hiss husk kiosk mosh motley pane pith relief sear sonar splice stitch strand vein.
- `aura::hash` (12): aura fete filament mist parcel prism quilt riso static tokens warp zig.
- `terrain::hash` (4): terrain oddgrid specimen stipple.
- Local `sampler::hash` (1). `noise_hash` runs only inside chassis dither.
- 39 Tools in total. None uses two families.

## 2D value noise

All copies use smoothstep fade `t*t*(3-2t)` and integer lattice corners from `floor`.

| Copy | Hash | Blend of corners a b / c d | Corner add | Group |
|---|---|---|---|---|
| `chassis.rs:17` `value_noise(x, y, seed: u32)` | `chassis::hash` | `(a(1-u) + b·u)(1-v) + (c(1-u) + d·u)v` | `wrapping_add` | ref "chassis" |
| `fete.rs:259` `value_noise(x, y, seed)` | `aura::hash(…, 0, seed)` | `lerp(lerp(a,b,u), lerp(c,d,u), v)`, `lerp = a+(b-a)t` | `+` | ref "lerp" |
| `filament.rs:395` `noise(x, y, seed)` | `aura::hash(…, 0, seed)` | same, lerp as closure | `wrapping_add` | G1 with fete |
| `tokens.rs:543` `field(x, y, seed)` | `aura::hash(…, 0, seed)` | `top = a+(b-a)u; top + ((c+(d-c)u) - top)v` (same ops as nested lerp) | `+` | G1 with fete |
| `terrain.rs:280` `noise(x, y, zw: Option<(f64,f64)>, seed)` | `terrain::hash(…, z, w, seed)` | nested lerp. With `zw=None` it is `face(0,0)`. With `Some` it adds a 4D lerp over `(z,w)` | `+` | `None`: G1 with fete (hash collapses). `Some`: G3 (4D) |
| `aura.rs:249` `Noise::value(&self, x, y, channel)` | `aura::hash(…, channel, self.0)` | `a + (b-a)u + (c-a)v + (a-b-c+d)uv` | `+` | ref "poly" |
| `parcel.rs:245` closure `vn(x, y, k)` | `aura::hash(…, k, ns)` | same poly | `+` | G1 with `Noise::value` (private, hence the copy) |

Across references everything is G3. "chassis" vs "lerp" use a different hash and a different blend. "lerp" vs "poly" use the same hash but the blend ops round differently.

`+` vs `wrapping_add` on corners: release builds wrap, so bits match. A dev build panics on overflow at `i32::MAX`, which real inputs never reach.

Users:

- `chassis::value_noise` (17): benday chaff cipher coral crowd frond hiss husk motley pith sear splice sprig stitch strand vein whorl. Also through `chassis::fbm`: atlas bloom sonar.
- Lerp family (5): fete filament tokens, terrain (and its importers oddgrid modular specimen stipple through `terrain::fbm`).
- Poly family (2): aura and mist (`aura::Noise`), parcel.
- 27 Tools in total.

## fbm

| Copy | Signature | Noise | Octaves | Lacunarity / gain | Octave seed | Normalised | Group |
|---|---|---|---|---|---|---|---|
| `chassis.rs:27` | `fbm(x, y, seed: u32, octaves: u32)` | chassis | param | 2.0 / 0.5, w0=0.5 | `seed + o*131` | `/ Σw` | ref |
| `husk.rs:451` | `fbm(x, y, seed)` | chassis | 3 | 2.07 / 0.55 | `+ o*131` | yes | G2 |
| `sear.rs:266` | `fbm(x, y, seed: i32, octaves)` | chassis | param | 2.05 / 0.58 | `i32 seed.wrapping_add(i*131) as u32` (same bits as `u32`) | yes | G2 |
| `terrain.rs:306` | `fbm(x, y, zw, seed, octaves)` | terrain lerp | param | 2.0 / 0.5 | `+ o*1319` | yes | ref "1319" |
| `filament.rs:409` | `fbm(x, y, seed, octaves)` | lerp | param | 2.0 / 0.5 | `+ o*1319` | yes | G1 with terrain at `zw=None` |
| `fete.rs:272` | `fbm(x, y, seed)` | lerp | 3 | 2.0 / 0.5 | `+ o*1319` | yes | G1 with filament `octaves=3` (G2 for the fixed count) |
| `aura.rs:260` | `Noise::fbm(&self, x, y, channel)` | poly | 3 fixed | ×2.13, ×4.31 / 0.55, 0.28, 0.17 | `channel + 11`, `+ 23` | no | G3 |
| `aura.rs:266` | `Noise::fbm2(&self, x, y, channel)` | poly | 2 fixed | ×2.13 / 0.62, 0.38 | `channel + 11` | no | ref "2-term" |
| `parcel.rs:255` | closure `fbm(x, y, k)` | poly (`vn`) | 2 fixed | ×2.17 / 0.62, 0.38 | `k + 9` | no | G2 with `fbm2` |

Loop update order (`weight` before or after `frequency`) differs between copies but does not change the result.

Loop fbm (131 step) vs loop fbm (1319 step) is G3: the noise differs as well as the step.

Inline fixed-weight octave sums over `chassis::value_noise`, all G3 to the above:

- `coral.rs:409-415`: 0.58 / 0.30 / 0.12 at ×1, ×2.3, ×6.1. Seeds `sd(2)`, `sd(3)`, `sd(3)`; the last two share a seed.
- `chaff.rs:483-485`: 0.66 / 0.34, two scales, two seeds.
- `frond.rs:552-553`: 0.62 / 0.38. The scale ratio is 1/0.34. Seed `+91`.
- `splice.rs:501-525`: 4-term table of `(size, weight, seed)`.

Users of any fbm (15): atlas bloom sonar (chassis); husk sear; terrain modular oddgrid specimen stipple (terrain); filament fete; aura mist (`Noise`); parcel.

## Domain-warped fbm level

The pattern: two low-octave fbm offsets, then one main fbm at the warped point, then contrast.

| Copy | Offsets | Seeds | Warp gain | Group |
|---|---|---|---|---|
| `terrain.rs:340-349` | `(u+5.2, v+1.3)`, `(u+9.1, v+7.7)`, 2 oct | `+11`, `+29` | `warp*(w-0.5)*2` then `((f-0.5)*contrast+0.5).clamp(0,1)` | ref |
| `stipple.rs:484-494` `Field::level` | same | same | same | G1. Input is first folded by symmetry and zoom |
| `sear.rs:304-316` | `(u*0.55+11.3+ox, …)`, `(u*0.55+2.7, …)`, 2 oct | `+7`, `+13` | `wx*k`, `k = warp*2.6`; no contrast | G2/G3 (offset scaling and shape) |
| `sonar.rs:271-278` | `(x*0.6+11.3, y*0.6+3.7)`, `(x*0.6+5.1, y*0.6+19.9)`, 3 oct | `+7`, `+13` | `warp*2.4` | G2 |
| `atlas.rs:291-298` | `(u*1.6+19, v*1.6)`, `(u*1.6, v*1.6+7)`, 3 oct | `+41`, `+83` | `warp*2.2`, contrast `0.7+mix*1.6`, clamp `0.999` | G2 |

`stipple::Field` is not a grid. It evaluates fbm on demand and stores no samples.

## f32 grid with bilinear read

| Copy | Storage | Index and frac | Edges | Blend | Group |
|---|---|---|---|---|---|
| `husk.rs:269` `Field`, read `husk.rs:398-412` | `Vec<f32>` | `g = px * (fw / w)`; `lo = (g as usize).min(last)`; `f = g - lo` (after clamp) | clamp both axes | `a+(b-a)fx`, then `+(b-a)fy` | ref |
| `culture.rs:297` `Plate`, read `culture.rs:390-409` | `Vec<f32>` | same, inline | clamp | same | G1 |
| `chaff.rs:468-481` | `Vec<u8>` mask, `/255` | same (`u32`), scale fixed 0.5 | clamp | same | G2 (source type, scale) |
| `sear.rs:277` `Heat`, read `sear.rs:407-428` | `Vec<f32>` | `gy = (y/h) * fh` (divide first); `fy = gy - gy.floor()` (before clamp); x: `gx -= floor(gx/fw)*fw` | y clamp, x wraps (`x1 = 0` past the edge) | same | G3 |
| `strand.rs:551-566` | RGBA `u8` channel, `/255` | scale fixed 0.5; `fy` after clamp; `fx` after clamp | y clamp, x wraps | same | G3 (wrap) |
| `terrain.rs:356-370` | `Vec<f32>`, padded +2 | `fx = bx / STEP`; `sx = fx - i0`; no clamp | padding | nested `lerp` (same ops) | G2/G3: no clamp |
| `coral.rs:324` `Grid::at` | `Vec<f32>`, padded +2 | `g = x / cell`; no clamp | padding | `(a(1-xf)+b·xf)(1-yf) + …` | G3 (blend form) |
| `crowd.rs:510-522` | `Vec<f32>` | `g = (px+cell)/cell - 0.5`; `f = g - g.trunc()` | clamp `+1` only | mix form, then `.min(1.0)` | G3 |

`vein.rs:296` `Grid` is not bilinear. `at_pixel` reads the nearest node with `round_half_up`, and the grid feeds marching squares.

Field size from aspect is also repeated: `w = aspect > 1 ? round_half_up(N/aspect).max(8) : N; h = round_half_up(w*aspect).max(8)`.

- husk:287 and culture:304 use `N=560` (G1).
- sear:284 uses `N=640` (G2).

## Marching squares

| Copy | Inside test | Saddle (5, 10) | Output | Group |
|---|---|---|---|---|
| `vein.rs:315` `Grid::cut` | `v >= level` | decided by centre mean | edge-indexed links into `Path2D`, coords rounded to 0.1 | G3 |
| `specimen.rs:580` `contours` | `v > level` | fixed pairing | segments chained through `HashMap<String>` keys `{:.4}` | G3 |

Both share the corner bit order (a=8, b=4, c=2, d=1), the 1e-9 zero-span guard, and the case table for 1–4 and 6–9 and 11–14.

## Small maths

| Block | Copies | Group |
|---|---|---|
| `lerp(a, b, t) = a + (b-a)*t` | fete:255, oddgrid:377, terrain:276 (fns); filament:401 (closure) | G1 |
| Smoothstep `t*t*(3.0-2.0*t)` | chassis:19, aura:252, fete:261, filament:397, terrain:282, tokens:545, parcel:249, coral:428, crowd:304, culture:432, husk:431, hiss:384 | G1 |
| `smoothstep(e0, e1, x)` with clamp and zero-width `1e-6` guard | mist:208 | G3 (GLSL form) |
| Luma Rec.601 `r*0.299 + g*0.587 + b*0.114` | mosh:254, sampler:330, totem:481 | G1 |
| Luma Rec.709 `0.2126r + 0.7152g + 0.0722b` | stitch:325, vein:292 | G1; G2 vs Rec.601 |

## Colour mixing

| Copy | Signature | Rounding |
|---|---|---|
| `fold.rs:397` `mix` | `([u8;3], [u8;3], t) -> [u8;3]` | `store` (clamp, ties-even) |
| `crowd.rs:270` `mix` | `(Rgb=[f64;3], Rgb, t) -> Rgb` | none |
| `whorl.rs:369` inline | f64 ground→ink | `store` |
| `carve.rs:480-483` inline | `ca + (cb-ca)*t + noise` | `store` |
| `sear.rs:350` `ramp` | quantised palette LUT, `steps` | `round_half_up(..) as u8`, no clamp |
| `culture.rs:430-434` inline | palette band `i0 = min(n-2)`, smoothstep `fr` | f64, `store` later |
| `hiss.rs:377` `marble` | palette band, smoothstep `w` | f64 |
| `benday.rs:329` inline | `[0,1,2].map` | f64 |
| `coral.rs:314` `lift(c, k)` | toward white `v + (255-v)k` | f64 |
| `vein.rs:280` `tint(ink, t)` | `t >= 0`: toward white. `t < 0`: `c*(1 - 0.9·|t|)` | `round_half_up`, clamp |

The per-channel blend `a + (b-a)t` is common to all of them. Rounding and types differ, so all copies are G3 against each other except `fold::mix` vs `whorl` (G1 ops on f64-converted input). `coral::lift` is the `t >= 0` branch of `vein::tint` without rounding (G3).

Palette band lookup (`k = floor(u).min(n-2)`, blend `inks[k]` to `inks[k+1]`) appears in sear:357, culture:430 and hiss:377. Fade and rounding differ (G3).

## Shuffles

Fisher–Yates `for i in (1..len).rev() { j = (draw * (i+1)) as usize; swap(i, j) }`:

| Copy | Draw | Group |
|---|---|---|
| `prism.rs:253` | `aura::Xorshift::next` | G1 |
| `riso.rs:441` | `aura::Xorshift::next` | G1 |
| `stitch.rs:364` | `chassis::XorShift::next` | G1 in ops; different RNG |
| `mosh.rs:489` | `chassis::hash(i, 7, seed+13)` | G3 (stateless draw) |

## Seed plumbing

| Block | Copies | Group |
|---|---|---|
| Sub-seed `seed.wrapping_mul(7).wrapping_add(k)` | vein:438, hiss:460, benday:331, stitch:338, crowd:457, cipher:347, coral:365 (`i32`, same bits) | G1 |
| Per-frame seed `seed + t*7919` (`SHUFFLE`) | atlas:16, kiosk:14, mosh:14, pane:14, relief:14, sampler:16, totem:15 | G1 |
| Per-frame `t*911` (`BOIL`/`FLICKER`) | pith:18, frond:17 (`i32`), motley:16 | G2 vs 7919 |
| Per-frame `t*2_654_435_761` | static:18 | G2 |

## Dither tables

Bayer 4x4 `[0,8,2,10,12,4,14,6,3,11,1,9,15,7,13,5]`:

- `frond.rs:330` (`[u8;16]`) and `quilt.rs:269` (`[f64;16]`): threshold `(B+0.5)/16`. G1, apart from the table type.
- `chassis.rs:469` `bayer(x, y, side)` builds the same matrix recursively for any power-of-two side. Checked by hand at (1,0), (2,0) and (0,1).

## Loop drift circle

`zw = drift.map(|t| ((TAU*t).cos()*AMOUNT, (TAU*t).sin()*AMOUNT))` appears at oddgrid:393, terrain:330, modular:442, specimen:679 and stipple:516. It is G2: `AMOUNT` is 0.8, 0.6, 0.6, 0.8 and 0.3. Other Loops build similar offsets with extra factors (sear:294-297, coral:395, culture:320-322); those are G3.

## Notes for the byte-identical plan

- Merge without changing output: `husk::Xorshift` into `chassis::XorShift`; `sampler::hash` into `chassis::hash`; `fete::value_noise`, `filament::noise` and `tokens::field` into one lerp-family noise (also `terrain::noise(.., None, ..)`); `fete::fbm` and `filament::fbm` into `terrain::fbm(.., None, ..)`; `parcel`'s `vn` into `aura::Noise::value` (needs `pub(crate)`); the 3 `lerp` fns; Rec.601 and Rec.709 `luma`; the Bayer tables.
- Merge with constants as arguments: `husk::fbm` and `sear::fbm` into `chassis::fbm` with lacunarity and gain; `parcel`'s `fbm` into `aura::Noise::fbm2` with lacunarity and channel step; the husk/culture/chaff bilinear read (source type aside); the seed scrambles A, B and C.
- Do not merge without new goldens: the two Xorshift32 families (shift kind); the chassis, lerp and poly noise blends; the 131 vs 1319 octave seeds; the wrapping bilinear reads (sear, strand); coral's and crowd's mix-form reads; the two marching squares; colour mixes with different rounding.
