import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { parseArgs } from "node:util";

import { chromium } from "playwright-core";
import type { Browser, Page } from "playwright-core";

import { checkPage, parseManifest } from "./manifest";
import type { Fixture, ParamValue } from "./manifest";

const BASE = "https://www.playgrnd.tools";
const RATIO = "9:16";
const WIDTH = 540;
const HEIGHT = 960;
const FONTCONFIG_FILE = join(import.meta.dir, "fonts.conf");
const FONT = join(import.meta.dir, "../fonts/DejaVuSansMono-Bold.ttf");

const sha256 = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");

async function setRange(page: Page, id: string, value: number): Promise<void> {
  const error = await page.evaluate(([id, value]) => {
    const el = document.getElementById(id);
    if (!(el instanceof HTMLInputElement) || el.type !== "range") return `#${id} is not a slider`;
    el.value = String(value);
    if (Number(el.value) !== value) return `#${id} rejected ${value} (min ${el.min}, max ${el.max}, step ${el.step})`;
    el.dispatchEvent(new Event("input", { bubbles: true }));
    el.dispatchEvent(new Event("change", { bubbles: true }));
    return null;
  }, [id, value] as const);
  if (error) throw new Error(error);
}

async function setToggle(page: Page, id: string, on: boolean): Promise<void> {
  const toggle = page.locator(`button#${id}[aria-pressed]`);
  if (!(await toggle.count())) throw new Error(`#${id} is not a toggle`);
  if ((await toggle.getAttribute("aria-pressed")) !== String(on)) await toggle.click({ force: true });
  if ((await toggle.getAttribute("aria-pressed")) !== String(on)) {
    throw new Error(`#${id} did not switch ${on ? "on" : "off"}`);
  }
}

async function setPick(page: Page, id: string, label: string): Promise<void> {
  const error = await page.evaluate(([id, label]) => {
    const host = document.getElementById(id) as (HTMLElement & { __pickMenu?: HTMLElement }) | null;
    if (!host?.__pickMenu) return `#${id} is not a picker`;
    const options = [...host.__pickMenu.querySelectorAll<HTMLElement>("[role=option]")];
    const option = options.find(o => o.textContent === label);
    if (!option) return `#${id} has no option ${label} (has ${options.map(o => o.textContent).join(", ")})`;
    option.click();
    return host.querySelector(".val")?.textContent === label ? null : `#${id} did not select ${label}`;
  }, [id, label] as const);
  if (error) throw new Error(error);
}

async function setParam(page: Page, id: string, value: ParamValue): Promise<void> {
  if (typeof value === "number") return setRange(page, id, value);
  if (typeof value === "boolean") return setToggle(page, id, value);
  return setPick(page, id, value);
}

async function setSize(page: Page, params: Record<string, ParamValue>): Promise<void> {
  const id = (await page.locator("#sizePx").count()) ? "sizePx" : "size";
  if (id in params) throw new Error(`#${id} is the export size on this Tool, not a Parameter`);
  await page.evaluate(id => {
    const el = document.getElementById(id) as HTMLInputElement;
    el.min = "1";
    el.step = "1";
  }, id);
  await setRange(page, id, WIDTH);
}

async function exportPng(
  browser: Browser,
  slug: string,
  { recipe, frame }: Fixture,
): Promise<{ png: Buffer; pageSha: string }> {
  const context = await browser.newContext({ acceptDownloads: true, viewport: { width: 1600, height: 1000 } });
  try {
    await context.addInitScript(
      v => localStorage.setItem("playgrnd.mycolors", v),
      JSON.stringify({ v: 1, cols: recipe.palette, on: true }),
    );
    const page = await context.newPage();
    const response = await page.goto(`${BASE}/${slug}/?seed=${recipe.tool_seed}`, { waitUntil: "networkidle" });
    if (!response?.ok()) throw new Error(`GET ${BASE}/${slug}/ -> ${response?.status()}`);
    const pageSha = sha256(await response.body());

    if ((await page.inputValue("#seed")) !== String(recipe.tool_seed)) {
      throw new Error(`${slug} did not apply seed ${recipe.tool_seed}`);
    }
    if ((await page.getAttribute("#mycTog", "aria-pressed")) !== "true") {
      throw new Error(`${slug} did not apply the Palette`);
    }

    const ratio = page.locator("#ratios button.ratio", { hasText: RATIO });
    if (!(await ratio.count())) throw new Error(`${slug} offers no ${RATIO} ratio`);
    await ratio.first().click({ force: true });
    await setSize(page, recipe.params);

    await setToggle(page, "motionTog", frame !== undefined);
    for (const [id, value] of Object.entries(recipe.params)) await setParam(page, id, value);
    if (frame !== undefined) await setRange(page, "scrub", frame);

    await page.click("#exportTog", { force: true });
    const [download] = await Promise.all([page.waitForEvent("download"), page.click("#expPng", { force: true })]);
    const png = readFileSync(await download.path());
    const [w, h] = [png.readUInt32BE(16), png.readUInt32BE(20)];
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
const browser = await chromium.launch({ channel: "chromium", env: { ...process.env, FONTCONFIG_FILE } });
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
