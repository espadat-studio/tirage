import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { chromium } from "playwright-core";
import type { Browser, BrowserContext, Page } from "playwright-core";

import type { ParamValue } from "./manifest";

export const BASE = "https://www.playgrnd.tools";
export const RATIO = "9:16";
const FONTCONFIG_FILE = join(import.meta.dir, "fonts.conf");

const sha256 = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");

export function launch(): Promise<Browser> {
  return chromium.launch({ channel: "chromium", env: { ...process.env, FONTCONFIG_FILE } });
}

export async function openTool(
  browser: Browser,
  slug: string,
  toolSeed: number,
  palette?: string[],
): Promise<{ context: BrowserContext; page: Page; pageSha: string }> {
  const context = await browser.newContext({ acceptDownloads: true, viewport: { width: 1600, height: 1000 } });
  try {
    if (palette) {
      await context.addInitScript(
        v => localStorage.setItem("playgrnd.mycolors", v),
        JSON.stringify({ v: 1, cols: palette, on: true }),
      );
    }
    const page = await context.newPage();
    const response = await page.goto(`${BASE}/${slug}/?seed=${toolSeed}`, { waitUntil: "networkidle" });
    if (!response?.ok()) throw new Error(`GET ${BASE}/${slug}/ -> ${response?.status()}`);
    const pageSha = sha256(await response.body());
    if ((await page.inputValue("#seed")) !== String(toolSeed)) {
      throw new Error(`${slug} did not apply tool seed ${toolSeed}`);
    }
    if (palette && (await page.getAttribute("#mycTog", "aria-pressed")) !== "true") {
      throw new Error(`${slug} did not apply the Palette`);
    }
    if (palette) {
      const swatches = await page.evaluate(() => {
        const row = [...document.querySelectorAll<HTMLInputElement>("#swatches input[type=color]")];
        const rail = document.querySelector(".z-left") ?? document.body;
        const loose = [...rail.querySelectorAll<HTMLInputElement>("input[type=color]")].filter(
          input => !input.closest(".cpick") && !input.closest("#myc"),
        );
        return (row.length ? row : loose).map(input => input.value.toLowerCase());
      });
      const want = palette.map(ink => ink.toLowerCase());
      if (swatches.join() !== want.join()) {
        throw new Error(`${slug} shows swatches ${swatches.join(" ")}, expected the Palette ${want.join(" ")}`);
      }
    }
    return { context, page, pageSha };
  } catch (error) {
    await context.close();
    throw error;
  }
}

export async function setRatio(page: Page, slug: string, ratio: string): Promise<void> {
  const button = page.locator("#ratios button.ratio", { hasText: ratio });
  if (!(await button.count())) throw new Error(`${slug} offers no ${ratio} ratio`);
  await button.first().click({ force: true });
}

export async function sizeControl(page: Page): Promise<string> {
  return (await page.locator("#sizePx").count()) ? "sizePx" : "size";
}

export async function setSize(page: Page, id: string, width: number): Promise<void> {
  await page.evaluate(id => {
    const el = document.getElementById(id) as HTMLInputElement;
    el.min = "1";
    el.step = "1";
  }, id);
  await setRange(page, id, width);
}

export async function setRange(page: Page, id: string, value: number): Promise<void> {
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

export async function setToggle(page: Page, id: string, on: boolean): Promise<void> {
  const toggle = page.locator(`button#${id}[aria-pressed]`);
  if (!(await toggle.count())) throw new Error(`#${id} is not a toggle`);
  if ((await toggle.getAttribute("aria-pressed")) !== String(on)) await toggle.click({ force: true });
  if ((await toggle.getAttribute("aria-pressed")) !== String(on)) {
    throw new Error(`#${id} did not switch ${on ? "on" : "off"}`);
  }
}

export async function setPick(page: Page, id: string, label: string): Promise<void> {
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

export async function setParam(page: Page, id: string, value: ParamValue): Promise<void> {
  if (typeof value === "number") return setRange(page, id, value);
  if (typeof value === "boolean") return setToggle(page, id, value);
  return setPick(page, id, value);
}

export async function downloadPng(page: Page, size?: { width: number; height: number }): Promise<Buffer> {
  await page.click("#exportTog", { force: true });
  const [download] = await Promise.all([page.waitForEvent("download"), page.click("#expPng", { force: true })]);
  const png = readFileSync(await download.path());
  if (!size) return png;
  const [w, h] = [png.readUInt32BE(16), png.readUInt32BE(20)];
  if (w !== size.width || h !== size.height) {
    throw new Error(`${page.url()} exported ${w}x${h}, expected ${size.width}x${size.height}`);
  }
  return png;
}
