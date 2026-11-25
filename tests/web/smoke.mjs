// Drives the web build through a short Run in headless Chromium, screenshots
// each screen, and exits non-zero if anything goes wrong.
// Usage: node smoke.mjs <base-url> <screenshot-dir>
// Normally run through scripts/smoke-web.sh.
import { mkdir } from "node:fs/promises";
import { chromium } from "playwright";

const [url, out] = process.argv.slice(2);
if (!url || !out) {
  console.error("usage: node smoke.mjs <base-url> <screenshot-dir>");
  process.exit(2);
}
await mkdir(out, { recursive: true });

const failures = [];
const fail = (msg) => {
  failures.push(msg);
  console.error(`FAIL: ${msg}`);
};

const browser = await chromium.launch({
  args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"],
});
// A fresh context has empty localStorage, so every run starts from no Save.
const context = await browser.newContext({ viewport: { width: 960, height: 720 } });
const page = await context.newPage();
page.on("console", (m) => {
  if (m.type() === "error") fail(`console error: ${m.text()}`);
  else if (m.type() === "warning") fail(`console warning: ${m.text()}`);
});
page.on("pageerror", (e) => fail(`page error: ${e.message}`));

const wait = (ms) => page.waitForTimeout(ms);
const tap = async (key, times = 1) => {
  for (let i = 0; i < times; i++) {
    await page.keyboard.down(key);
    await wait(150);
    await page.keyboard.up(key);
    await wait(250);
  }
};
let n = 0;
const shot = async (name) => {
  n += 1;
  await page.screenshot({ path: `${out}/${String(n).padStart(2, "0")}-${name}.png` });
};
const stored = (key) => page.evaluate((k) => localStorage.getItem(`scroller.${k}`), key);

try {
  // Release behaviour first: without ?debug the developer keys do nothing.
  await page.goto(url);
  await page.waitForSelector("#loading", { state: "detached", timeout: 60_000 });
  await page.click("#game");
  await wait(500);
  await shot("title-release");
  await tap("Space"); // NEW RUN
  await tap("Space"); // play 1-1
  await wait(1500);
  await tap("KeyC"); // developer key: must do nothing
  await tap("KeyP");
  await wait(800);
  if ((await stored("debug.stage")) !== null) fail("debug.stage was written without ?debug");
  if ((await stored("save")) !== null) fail("the C key cleared a Stage without ?debug");
  await shot("release-no-debug-keys");

  // Debug mode: the rest of the flow uses the developer keys.
  await context.clearCookies();
  await page.evaluate(() => localStorage.clear());
  await page.goto(`${url}?debug`);
  // index.html removes the loading text once the game has drawn.
  await page.waitForSelector("#loading", { state: "detached", timeout: 60_000 });
  await page.click("#game");
  await wait(500);
  await shot("title");

  await tap("Space"); // NEW RUN
  await wait(500);
  await shot("stage-card-new-run");

  await tap("Space"); // play 1-1
  // Hold fire on the starting row through Stage 1-1's first Skiff V.
  await page.keyboard.down("Space");
  await wait(8_000);
  await shot("wren-firing");
  await page.keyboard.up("Space");
  await tap("KeyP", 6); // developer key: +6000 points, beats the seeded table
  await tap("KeyK"); // developer key: lose a life
  await tap("ArrowUp");
  await wait(500);
  await shot("playing-1-1");

  await tap("KeyC"); // developer key: clear Stage 1-1
  await wait(600);
  await shot("stage-card-cleared");
  if ((await stored("save")) === null) fail("no Save in localStorage after clearing Stage 1-1");

  await tap("Space"); // play 1-2
  await wait(800);
  if ((await stored("save")) !== null) fail("the Save was not used up when Stage 1-2 began");
  await tap("KeyC"); // clear 1-2: the 1 -> 2 Cabinet Transition (5 s)
  await wait(1800);
  await shot("cabinet-transition");
  // The Stage Card waits for input, so a generous wait is safe on a slow
  // browser.
  await wait(5500);
  await shot("stage-card-cabinet-2");

  await tap("Space"); // play 2-1
  await wait(1500);
  await shot("playing-2-1");
  await tap("Escape"); // pause
  await wait(400);
  await shot("paused");
  await tap("ArrowDown");
  await tap("Space"); // QUIT RUN
  await wait(1500);
  await shot("game-over");

  await tap("Space"); // name entry
  await wait(500);
  await tap("ArrowUp", 3); // first letter: D
  await tap("Space");
  await tap("Space");
  await wait(300);
  await shot("name-entry");
  await tap("Space"); // confirm: title on the High Score Table
  await wait(600);
  await shot("high-scores");

  const table = JSON.parse((await stored("high_scores")) ?? "null");
  const entries = table?.entries ?? [];
  if (!entries.some((e) => e.initials.startsWith("D") && e.score >= 6000)) {
    fail(`no new high score written; table is ${JSON.stringify(entries.map((e) => `${e.initials} ${e.score}`))}`);
  }

  await tap("Space"); // back to the title panel
  await wait(500);
  await shot("title-with-practice");
} catch (e) {
  fail(`flow did not finish: ${e.message}`);
  await shot("failure").catch(() => {});
}

await browser.close();
console.log(`Screenshots in ${out}`);
if (failures.length > 0) {
  console.error(`Smoke test FAILED (${failures.length} problem${failures.length === 1 ? "" : "s"}).`);
  process.exit(1);
}
console.log("Smoke test passed.");
