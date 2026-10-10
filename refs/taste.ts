import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { parseArgs } from "node:util";

import type { Browser, Page } from "playwright-core";

import { downloadPng, launch, openTool, RATIO, setRange, setRatio, setSize, sizeControl } from "./page";
import { draw, DRAWS, parseBounds, sweep } from "./triage";
import type { Bounds, Slider } from "./triage";

const WIDTH = 270;
const HEIGHT = 480;
const WORKERS = 4;
const SHIPPED_TOOL_SEED = 9611518;
const PANELS = "#motionBody, #ditherBody, #grainBody, #exportBody";
const PREFIX = "window.DATA = ";

type Kind = "sweep" | "draw" | "check";

interface Job {
  file: string;
  kind: Kind;
  param?: string;
  toolSeed: number;
  sets: Record<string, number>;
}

interface Tile extends Omit<Job, "sets"> {
  values: Record<string, number>;
}

interface Sheet {
  slug: string;
  sliders: Slider[];
  bounds: Bounds | null;
  tiles: Tile[];
}

const { values: flags, positionals } = parseArgs({
  allowPositionals: true,
  options: { bounds: { type: "string" } },
});

if (process.env.CI) throw new Error("taste drives the live site and never runs in CI");
const [slug, extra] = positionals;
if (!slug || extra) throw new Error("usage: taste <slug> [--bounds FILE]");

const dir = join(import.meta.dir, "../target/taste", slug);
const dataPath = join(dir, "data.js");

async function artSliders(page: Page): Promise<Slider[]> {
  const sizeId = await sizeControl(page);
  const sliders = await page.evaluate(
    ([panels, sizeId]) => {
      return [...document.querySelectorAll<HTMLInputElement>("input[type=range]")]
        .filter(el => el.id !== "scrub" && el.id !== sizeId && !el.closest(panels))
        .map(el => ({
          id: el.id,
          min: Number(el.min),
          max: Number(el.max),
          step: Number(el.step),
          value: Number(el.value),
        }));
    },
    [PANELS, sizeId] as const,
  );
  for (const s of sliders) {
    if (!s.id || !(s.step > 0) || !(s.min < s.max)) throw new Error(`${slug}: #${s.id} is not a bounded slider`);
  }
  if (!sliders.length) throw new Error(`${slug} has no art sliders`);
  return sliders;
}

async function exportTile(browser: Browser, job: Job, ids: string[]): Promise<Tile> {
  const { context, page } = await openTool(browser, slug, job.toolSeed);
  try {
    await setRatio(page, slug, RATIO);
    await setSize(page, await sizeControl(page), WIDTH);
    for (const [id, value] of Object.entries(job.sets)) await setRange(page, id, value);
    const values = await page.evaluate(
      ids => Object.fromEntries(ids.map(id => [id, Number((document.getElementById(id) as HTMLInputElement).value)])),
      ids,
    );
    const png = await downloadPng(page);
    const [w, h] = [png.readUInt32BE(16), png.readUInt32BE(20)];
    if (Math.abs(w / h - WIDTH / HEIGHT) > 0.02) {
      throw new Error(`${slug} exported ${w}x${h}, not at the ${WIDTH}x${HEIGHT} aspect`);
    }
    writeFileSync(join(dir, job.file), png);
    const { sets: _, ...tile } = job;
    return { ...tile, values };
  } finally {
    await context.close();
  }
}

function previousTiles(): Tile[] {
  if (!existsSync(dataPath)) return [];
  const source = readFileSync(dataPath, "utf8");
  if (!source.startsWith(PREFIX) || !source.endsWith(";\n")) throw new Error(`${dataPath} was not written by taste`);
  return (JSON.parse(source.slice(PREFIX.length, -2)) as Sheet).tiles;
}

const browser = await launch();
try {
  const { context, page } = await openTool(browser, slug, SHIPPED_TOOL_SEED);
  const sliders = await artSliders(page).finally(() => context.close());
  const ids = sliders.map(s => s.id);

  const bounds = flags.bounds
    ? parseBounds(
      JSON.parse(readFileSync(resolve(process.env.MISE_ORIGINAL_CWD ?? ".", flags.bounds), "utf8")),
      slug,
      sliders,
    )
    : null;

  const drawJob = (kind: "draw" | "check", i: number): Job => {
    const { toolSeed, values } = draw(sliders, `${slug}/${kind}/${i}`, bounds ?? {});
    return { file: `${kind}-${i}.png`, kind, toolSeed, sets: values };
  };
  const sweeps = sliders.flatMap(s =>
    sweep(s).map((value, i): Job => ({
      file: `sweep-${s.id}-${i}.png`,
      kind: "sweep",
      param: s.id,
      toolSeed: SHIPPED_TOOL_SEED,
      sets: { [s.id]: value },
    }))
  );
  const draws = Array.from({ length: DRAWS }, (_, i) => drawJob(bounds ? "check" : "draw", i));
  const jobs = bounds ? draws : [...sweeps, ...draws];

  mkdirSync(dir, { recursive: true });
  const tiles: Tile[] = [];
  let next = 0;
  let done = 0;
  await Promise.all(
    Array.from({ length: WORKERS }, async () => {
      for (let i = next++; i < jobs.length; i = next++) {
        tiles[i] = await exportTile(browser, jobs[i], ids);
        console.log(`${++done}/${jobs.length} ${jobs[i].file}`);
      }
    }),
  );

  const kinds = new Set(jobs.map(j => j.kind));
  const sheet: Sheet = { slug, sliders, bounds, tiles: [...previousTiles().filter(t => !kinds.has(t.kind)), ...tiles] };
  writeFileSync(dataPath, `${PREFIX}${JSON.stringify(sheet)};\n`);
  copyFileSync(join(import.meta.dir, "taste.html"), join(dir, "index.html"));
  console.log(join(dir, "index.html"));
} finally {
  await browser.close();
}
