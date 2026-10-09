import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { parseArgs } from "node:util";

import type { Browser } from "playwright-core";

import { checkPage, parseManifest } from "./manifest";
import type { Fixture } from "./manifest";
import {
  downloadPng,
  launch,
  openTool,
  pngSize,
  setParam,
  setRange,
  setRatio,
  setSize,
  setToggle,
  sha256,
  sizeControl,
} from "./page";

const RATIO = "9:16";
const WIDTH = 540;
const HEIGHT = 960;
const FONT = join(import.meta.dir, "../fonts/DejaVuSansMono-Bold.ttf");

async function exportPng(
  browser: Browser,
  slug: string,
  { recipe, frame }: Fixture,
): Promise<{ png: Buffer; pageSha: string }> {
  const { context, page, pageSha } = await openTool(browser, slug, recipe.tool_seed, recipe.palette);
  try {
    await setRatio(page, slug, RATIO);
    const size = await sizeControl(page);
    if (size in recipe.params) throw new Error(`#${size} is the export size on this Tool, not a Parameter`);
    await setSize(page, size, WIDTH);

    await setToggle(page, "motionTog", frame !== undefined);
    for (const [id, value] of Object.entries(recipe.params)) await setParam(page, id, value);
    if (frame !== undefined) await setRange(page, "scrub", frame);

    const png = await downloadPng(page);
    const [w, h] = pngSize(png);
    if (w !== WIDTH || h !== HEIGHT) throw new Error(`${slug} exported ${w}x${h}, expected ${WIDTH}x${HEIGHT}`);
    return { png, pageSha };
  } finally {
    await context.close();
  }
}

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    update: { type: "boolean", default: false },
    manifest: { type: "string", default: join(import.meta.dir, "../tests/refs/manifest.json") },
  },
});

if (process.env.CI) throw new Error("refs drives the live site and never runs in CI");

const manifestPath = resolve(values.manifest);
const manifest = parseManifest(JSON.parse(readFileSync(manifestPath, "utf8")));
const slugs = positionals.length ? positionals : Object.keys(manifest.tools);
for (const slug of slugs) if (!manifest.tools[slug]) throw new Error(`${slug} has no entry in ${manifestPath}`);

const fontSha = sha256(readFileSync(FONT));
const browser = await launch();
try {
  for (const slug of slugs) {
    const entry = manifest.tools[slug];
    const pngs: [string, Buffer][] = [];
    let pageSha: string | undefined;
    for (const fixture of entry.fixtures) {
      const first = await exportPng(browser, slug, fixture);
      const second = await exportPng(browser, slug, fixture);
      for (const sha of [first.pageSha, second.pageSha]) {
        checkPage(slug, entry.page_sha256, sha, values.update);
        if (pageSha && sha !== pageSha) throw new Error(`${slug} page changed during the run`);
        pageSha = sha;
      }
      if (!first.png.equals(second.png)) {
        throw new Error(`${slug}/${fixture.name} exported different bytes twice: pin every Parameter and the frame`);
      }
      pngs.push([join(dirname(manifestPath), slug, `${fixture.name}.png`), first.png]);
    }
    for (const [path, png] of pngs) {
      mkdirSync(dirname(path), { recursive: true });
      writeFileSync(path, png);
      console.log(path);
    }
    Object.assign(entry, { page_sha256: pageSha, chromium: browser.version(), font_sha256: fontSha });
    writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
  }
} finally {
  await browser.close();
}
