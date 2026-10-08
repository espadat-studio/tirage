import { mkdirSync, writeFileSync } from "fs";
import { execFileSync } from "child_process";

import { chromium, type Browser } from "playwright-core";

const BASE = "https://www.playgrnd.tools";
const DIR = new URL(".", import.meta.url).pathname;
const SIZE = "600";
const RANDOM_PER_TOOL = 24;
const SWEEP_POINTS = 5;
const WORKERS = 6;

type Range = [string, number, number, number];
type ToolSpec = { tool: string; design: string; ratio: string; seed: number; shipped: string; inks: string[]; params: Range[] };

const TOOLS: ToolSpec[] = [
  { tool: "vein", design: "Variant Press", ratio: "1:1", seed: 3, shipped: "variant-press/vein-3-1x1.jpg",
    inks: ["#ff3fa4", "#6b2bff", "#ffffff", "#19c3b4", "#1f3bff", "#1a0638"],
    params: [["scale", 0, 1, 0.5], ["curve", 0, 1, 0.6], ["levels", 3, 14, 9], ["tiger", 0, 1, 0.5], ["edges", 0, 1, 0.4], ["stars", 0, 1, 0.5], ["tints", 0, 1, 0.5], ["runs", 0, 1, 0.5]] },
  { tool: "aura", design: "Photocard Binder", ratio: "3:4", seed: 6, shipped: "photocard-binder/aura-6-3x4.jpg",
    inks: ["#ff8fcb", "#b79cff", "#ffe066", "#6ee7c8", "#7cc6ff"],
    params: [["scale", 0.5, 2, 2], ["churn", 0, 1, 1], ["punch", 0, 1, 1]] },
  { tool: "kiosk", design: "Wheatpaste Wall", ratio: "9:16", seed: 3, shipped: "wheatpaste-wall/kiosk-3-9x16.jpg",
    inks: ["#f7f5ef", "#ff48b0", "#0078bf", "#d1007a", "#0068a8"],
    params: [["split", 0.15, 0.95, 0.62], ["rings", 4, 40, 15], ["stripes", 2, 24, 9], ["grid", 4, 30, 13], ["bigSize", 0.3, 1.6, 0.5], ["density", 0, 1, 0.84], ["smallSize", 0.1, 0.9, 0.44], ["small", 0, 1, 0.6], ["blocks", 0, 1, 0.62]] },
  { tool: "husk", design: "Test Pressing", ratio: "1:1", seed: 21, shipped: "test-pressing/husk-21-1x1.jpg",
    inks: ["#6b4fc4", "#4a2a9c", "#d9d9e0", "#0f0f11"],
    params: [["count", 1, 70, 30], ["size", 0, 1, 0.62], ["vary", 0, 1, 0.5], ["lump", 0, 1, 0.42], ["eat", 0, 1, 0.55], ["tex", 0, 1, 0.45], ["grain", 0, 1, 0.3]] },
  { tool: "sonar", design: "One-bit Crate", ratio: "9:16", seed: 11, shipped: "one-bit-crate/sonar-11-9x16.png",
    inks: ["#000000", "#ffffff"],
    params: [["level", 0, 1, 0.5], ["scale", 1, 10, 2.2], ["warp", 0, 1, 0.4], ["grid", 40, 320, 150], ["depth", 0, 1, 0.7], ["fringe", 0, 1, 0.45], ["spark", 0, 1, 0.6]] },
  { tool: "frond", design: "Paste-up Tropical", ratio: "9:16", seed: 4, shipped: "paste-up-tropical/frond-4-9x16.jpg",
    inks: ["#1fa83a", "#ff2d8a", "#ffb000", "#0b6b3a", "#fff7e6", "#17120e"],
    params: [["masses", 0, 6, 3], ["size", 0, 1, 0.62], ["round", 0, 1, 0.55], ["growth", 0, 1, 0.55], ["detail", 0, 1, 0.5], ["coarse", 0, 1, 0.42], ["breakup", 0, 1, 0.35], ["noise", 0, 1, 0.45], ["patch", 0, 1, 0.5], ["circles", 0, 1, 0.5], ["rules", 0, 1, 0.45]] },
];

type Job = { id: string; tool: string; kind: "calib" | "ink" | "sweep" | "random"; seed: number; inks: string[]; sets: Record<string, number>; param?: string };

function mulberry32(a: number) {
  return () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function rotate<T>(xs: T[], n: number): T[] {
  return [...xs.slice(n), ...xs.slice(0, n)];
}

function plan(): Job[] {
  const rand = mulberry32(15);
  const jobs: Job[] = [];
  for (const t of TOOLS) {
    jobs.push({ id: `${t.tool}-calib`, tool: t.tool, kind: "calib", seed: t.seed, inks: t.inks, sets: {} });
    for (let r = 1; r < Math.min(t.inks.length, 4); r++)
      jobs.push({ id: `${t.tool}-ink${r}`, tool: t.tool, kind: "ink", seed: t.seed, inks: rotate(t.inks, r), sets: {} });
    for (const [id, min, max] of t.params)
      for (let i = 0; i < SWEEP_POINTS; i++)
        jobs.push({ id: `${t.tool}-sw-${id}-${i}`, tool: t.tool, kind: "sweep", param: id, seed: t.seed, inks: t.inks, sets: { [id]: min + ((max - min) * i) / (SWEEP_POINTS - 1) } });
    for (let i = 0; i < RANDOM_PER_TOOL; i++) {
      const sets: Record<string, number> = {};
      for (const [id, min, max] of t.params) sets[id] = min + (max - min) * rand();
      jobs.push({ id: `${t.tool}-rnd-${i}`, tool: t.tool, kind: "random", seed: 1 + Math.floor(rand() * 99998), inks: t.inks, sets });
    }
  }
  return jobs;
}

async function render(browser: Browser, job: Job, ratio: string) {
  const ctx = await browser.newContext({ acceptDownloads: true, viewport: { width: 1600, height: 1000 } });
  try {
    const page = await ctx.newPage();
    await page.addInitScript(v => localStorage.setItem("playgrnd.mycolors", v), JSON.stringify({ v: 1, cols: job.inks, on: true }));
    await page.goto(`${BASE}/${job.tool}/?seed=${job.seed}`, { waitUntil: "networkidle" });
    if ((await page.getAttribute("#mycTog", "aria-pressed")) !== "true") throw new Error(`${job.id}: custom inks not applied`);
    await page.locator("#ratios button.ratio", { hasText: ratio }).first().click({ force: true });
    const sizeId = (await page.locator("#sizePx").count()) ? "sizePx" : "size";
    const actual = await page.evaluate(([sets, sizeId, size]) => {
      const out: Record<string, number> = {};
      for (const [id, v] of [...Object.entries(sets), [sizeId, Number(size)]] as [string, number][]) {
        const el = document.getElementById(id) as HTMLInputElement | null;
        if (!el) throw new Error(`no #${id}`);
        el.value = String(v);
        el.dispatchEvent(new Event("input", { bubbles: true }));
        el.dispatchEvent(new Event("change", { bubbles: true }));
      }
      const ids = [...document.querySelectorAll<HTMLInputElement>('input[type="range"]')].map(e => e.id);
      for (const id of ids) out[id] = Number((document.getElementById(id) as HTMLInputElement).value);
      return out;
    }, [job.sets, sizeId, SIZE] as const);
    await page.click("#exportTog", { force: true });
    const [dl] = await Promise.all([page.waitForEvent("download", { timeout: 120_000 }), page.click("#expPng", { force: true })]);
    const png = `${DIR}img/${job.id}.png`;
    await dl.saveAs(png);
    execFileSync("magick", [png, "-quality", "80", png.replace(/\.png$/, ".webp")]);
    execFileSync("rm", [png]);
    return actual;
  } finally {
    await ctx.close();
  }
}

const jobs = plan();
mkdirSync(`${DIR}img`, { recursive: true });
const browser = await chromium.launch({ executablePath: process.env.PG_CHROMIUM ?? "/usr/bin/chromium", headless: true });
const results: (Job & { values: Record<string, number> })[] = [];
let next = 0;
await Promise.all(Array.from({ length: WORKERS }, async () => {
  while (next < jobs.length) {
    const job = jobs[next++];
    const spec = TOOLS.find(t => t.tool === job.tool)!;
    const values = await render(browser, job, spec.ratio);
    results.push({ ...job, values });
    console.log(`${results.length}/${jobs.length} ${job.id}`);
  }
}));
await browser.close();
results.sort((a, b) => jobs.indexOf(jobs.find(j => j.id === a.id)!) - jobs.indexOf(jobs.find(j => j.id === b.id)!));
writeFileSync(`${DIR}data.js`, `window.TOOLS=${JSON.stringify(TOOLS)};\nwindow.TILES=${JSON.stringify(results)};\n`);
