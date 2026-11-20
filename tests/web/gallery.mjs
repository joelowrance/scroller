// Plays Cabinet 1's debug enemy gallery in headless Chromium and screenshots
// each enemy type, for review. Not part of the smoke test.
// Usage: node gallery.mjs <base-url> <screenshot-dir>
// Normally run through scripts/gallery-web.sh.
import { mkdir } from "node:fs/promises";
import { chromium } from "playwright";

const [base, out] = process.argv.slice(2);
if (!base || !out) {
  console.error("usage: node gallery.mjs <base-url> <screenshot-dir>");
  process.exit(2);
}
await mkdir(out, { recursive: true });

// Game seconds run twice as fast as real ones.
const SCALE = 2;
const url = `${base}?gallery&invincible&time_scale=${SCALE}`;

// When to look, in game seconds after the Stage starts (see
// assets/stages/1-gallery.ron, which scrolls 30 px per second).
const moments = [
  [3.0, "skiff-v-peeling"],
  [13.5, "gull-snake"],
  [27.5, "bellbuoys-ringing"],
  [32.5, "lamplighter-hover"],
  [40.4, "hook-warning"],
  [42.0, "hooks-passing"],
  [51.0, "ferry-launching"],
  [54.5, "ferry-ring"],
  [68.0, "barnacles"],
  [75.8, "clams-open"],
  [86.0, "mote-cloud"],
  [96.0, "arc-gate"],
  [102.8, "gulls-firing"],
];

const errors = [];
const browser = await chromium.launch({
  args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"],
});
const page = await (await browser.newContext({ viewport: { width: 960, height: 720 } })).newPage();
page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
page.on("pageerror", (e) => errors.push(e.message));

const wait = (ms) => page.waitForTimeout(ms);
const tap = async (key) => {
  await page.keyboard.down(key);
  await wait(150);
  await page.keyboard.up(key);
  await wait(250);
};

await page.goto(url);
await page.waitForSelector("#loading", { state: "detached", timeout: 60_000 });
await page.click("#game");
await wait(500);
await tap("Space"); // NEW RUN
await wait(500);
await tap("Space"); // play 1-1: the gallery
const start = Date.now();
for (const [at, name] of moments) {
  const due = start + (at * 1000) / SCALE;
  await wait(Math.max(0, due - Date.now()));
  await page.screenshot({ path: `${out}/${String(at).padStart(5, "0")}-${name}.png` });
}
await browser.close();
if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log(`Gallery screenshots in ${out}`);
