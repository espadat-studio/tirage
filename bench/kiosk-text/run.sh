#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
FONTS="$PWD/fonts"
OUT="$PWD/out"
mkdir -p "$FONTS" "$OUT"

fetch() {
  local name=$1 url=$2 member=$3 sha=$4
  local dir="$FONTS/$name"
  [ -f "$dir/font.ttf" ] && return
  mkdir -p "$dir/dl"
  curl -sL "$url" -o "$dir/dl/archive"
  case "$url" in
    *.zip) unzip -q -o "$dir/dl/archive" "$member" -d "$dir/dl" ;;
    *) tar xjf "$dir/dl/archive" -C "$dir/dl" "$member" ;;
  esac
  cp "$dir/dl/$member" "$dir/font.ttf"
  echo "$sha  $dir/font.ttf" | sha256sum -c --quiet
  cat > "$dir/fonts.conf" <<EOF
<?xml version="1.0"?>
<fontconfig>
  <dir>$dir</dir>
  <cachedir>$dir/cache</cachedir>
  <match target="font">
    <edit name="antialias"><bool>true</bool></edit>
    <edit name="hinting"><bool>true</bool></edit>
    <edit name="hintstyle"><const>hintslight</const></edit>
    <edit name="rgba"><const>none</const></edit>
  </match>
</fontconfig>
EOF
}

fetch dejavu https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.tar.bz2 \
  dejavu-fonts-ttf-2.37/ttf/DejaVuSansMono-Bold.ttf bce60f1b4421acd9ea51ba6623d7024ecbe6817a953e3654df62a5e6bdf8f769
fetch jetbrains https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip \
  fonts/ttf/JetBrainsMono-Bold.ttf 5590990c82e097397517f275f430af4546e1c45cff408bde4255dad142479dcb

cargo build --release -q
for entry in "dejavu:DejaVu Sans Mono" "jetbrains:JetBrains Mono"; do
  name=${entry%%:*} family=${entry#*:}
  mkdir -p "$OUT/$name"
  for set in dos blocks runes; do
    FONTCONFIG_FILE="$FONTS/$name/fonts.conf" bun chrome.ts "$set" "$OUT/$name" "$family"
    ./target/release/kiosk-text-bench "$FONTS/$name/font.ttf" "$OUT/$name/glyphs-$set.tsv" \
      "$OUT/$name/chrome-$set.png" "$OUT/$name/rust-$set-" | sed "s/^/$name\t$set\t/"
  done
done

mkdir -p "$OUT/large"
for c in "4 1.2" "4 4.5"; do
  set -- $c
  FONTCONFIG_FILE="$FONTS/dejavu/fonts.conf" bun chrome.ts dos "$OUT/large" "DejaVu Sans Mono" "$1" "$2"
  ./target/release/kiosk-text-bench "$FONTS/dejavu/font.ttf" "$OUT/large/glyphs-dos-c$1x$2.tsv" \
    "$OUT/large/chrome-dos-c$1x$2.png" "$OUT/large/rust-c$1x$2-" | sed "s/^/large\tc$1x$2\t/"
done

for feat in "" "--features autohint"; do
  (cd wasm && RUSTFLAGS="-C target-feature=+simd128" cargo build -q --release --target wasm32-unknown-unknown $feat)
  w=wasm/target/wasm32-unknown-unknown/release/kiosk_text_wasm.wasm
  echo "wasm ${feat:-unhinted}: raw=$(stat -c%s $w) gzip9=$(gzip -9c $w | wc -c)"
done
