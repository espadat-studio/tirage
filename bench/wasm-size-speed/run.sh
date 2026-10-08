#!/usr/bin/env bash
set -euo pipefail
B="$(cd "$(dirname "$0")" && pwd)"
OUT="${OUT:-$B/out}"
OPT="${WASM_OPT:-$OUT/node_modules/.bin/wasm-opt}"
VARIANTS="${VARIANTS:-scalar simd}"
PROFILES="${PROFILES:-release speed o2 os}"
cd "$B"

CARGO_TARGET_DIR="$OUT/native" cargo build -q --profile speed --examples
echo "native speed ts $("$OUT/native/speed/examples/bench_ts")"
echo "native speed vc $("$OUT/native/speed/examples/bench_vc")"

for v in $VARIANTS; do
  flags=""
  feats=(--enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-multivalue --enable-reference-types)
  if [ "$v" = simd ]; then flags="-C target-feature=+simd128"; feats+=(--enable-simd); fi
  for p in $PROFILES; do
    CARGO_TARGET_DIR="$OUT/wt-$v" RUSTFLAGS="$flags" cargo build -q --target wasm32-unknown-unknown --profile "$p" --lib
    case $p in release) o=-Oz ;; os) o=-Os ;; o2) o=-O2 ;; *) o=-O3 ;; esac
    for c in ts vc; do
      d="$OUT/pkg/$v-$p-$c"
      wasm-bindgen --target web --out-dir "$d/web" "$OUT/wt-$v/wasm32-unknown-unknown/$p/$c.wasm"
      wasm-bindgen --target nodejs --out-dir "$d/node" "$OUT/wt-$v/wasm32-unknown-unknown/$p/$c.wasm"
      bg_gz=$(gzip -9c "$d/web/${c}_bg.wasm" | wc -c)
      "$OPT" "$o" "${feats[@]}" "$d/node/${c}_bg.wasm" -o "$d/node/${c}_bg.wasm"
      opt_gz=$(gzip -9c "$d/node/${c}_bg.wasm" | wc -c)
      glue_gz=$(gzip -9c "$d/web/$c.js" | wc -c)
      for rt in node bun; do
        echo "$v $p $c $rt bindgen_gz=$bg_gz opt_gz=$opt_gz glue_gz=$glue_gz $($rt "$B/run.cjs" "$d/node/$c.js")"
      done
    done
  done
done
