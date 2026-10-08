import { writeFileSync } from "node:fs";

import { chromium } from "playwright-core";

const [setName, outDir, family, colsArg, scaleArg] = process.argv.slice(2);
const SETS: Record<string, string> = {
  dos: "0369#%&@!;,'()",
  blocks: "░▒▓█■▪·",
  runes: "†‡§¶®©≠∞≈",
};
const STACK = "ui-monospace,SFMono-Regular,Menlo,Consolas,'DejaVu Sans Mono',monospace";
const W = 540;
const H = 960;
const PAPER = "#f3efe0";
const INKS = ["#141414", "#d7263d"];

let state = 20261008;
const rand = (): number => {
  state = (state + 0x6d2b79f5) | 0;
  let t = Math.imul(state ^ (state >>> 15), 1 | state);
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};

const chars = SETS[setName];
if (!chars || !outDir) throw new Error("usage: bun chrome.ts <dos|blocks|runes> <out-dir> [family] [cols] [size-scale]");
const cols = Number(colsArg ?? 13);
const k = Number(scaleArg ?? 1);
const cw = W / cols;
const rows = Math.round(H / cw);
const rh = H / rows;
const glyphs: [string, number, number, number, string][] = [];
for (const [scale, dx, dy] of [[0.5 * k, 0.5, 0.5], [0.44 * k, 1, 1]]) {
  for (let j = 0; j < rows; j++)
    for (let i = 0; i < cols; i++) {
      if (rand() > 0.8) continue;
      const ch = chars[Math.floor(rand() * chars.length)];
      const ink = INKS[rand() < 0.3 ? 1 : 0];
      glyphs.push([ch, (i + dx) * cw, (j + dy) * rh, Number((rh * scale).toFixed(1)), ink]);
    }
}

const browser = await chromium.launch({ executablePath: process.env.PG_CHROMIUM ?? "/usr/bin/chromium", headless: true });
const page = await browser.newPage();
const result = await page.evaluate(
  ({ glyphs, W, H, PAPER, STACK, family }) => {
    const draw = (face: string): HTMLCanvasElement => {
      const cv = document.createElement("canvas");
      cv.width = W;
      cv.height = H;
      const c = cv.getContext("2d")!;
      c.fillStyle = PAPER;
      c.fillRect(0, 0, W, H);
      c.textAlign = "center";
      c.textBaseline = "middle";
      for (const [ch, x, y, sz, ink] of glyphs) {
        c.fillStyle = ink;
        c.font = `700 ${sz}px ${face}`;
        c.fillText(ch, x, y);
      }
      return cv;
    };
    const stack = draw(STACK);
    const named = family ? draw(`'${family}'`).toDataURL() : null;
    return { png: stack.toDataURL(), stackEqualsNamed: named === null ? null : named === stack.toDataURL() };
  },
  { glyphs, W, H, PAPER, STACK, family },
);
await browser.close();

const tag = `${setName}${colsArg ? `-c${cols}x${k}` : ""}`;
writeFileSync(`${outDir}/chrome-${tag}.png`, Buffer.from(result.png.split(",")[1], "base64"));
writeFileSync(
  `${outDir}/glyphs-${tag}.tsv`,
  [`${W}\t${H}\t${PAPER}`, ...glyphs.map(([ch, x, y, sz, ink]) => [ch.codePointAt(0), x, y, sz, ink].join("\t"))].join("\n"),
);
console.log(`${tag}\tglyphs=${glyphs.length}\tstack==named(${family ?? "-"}): ${result.stackEqualsNamed}`);
