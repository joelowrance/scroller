// Plays both Stages of one Cabinet unattended in headless Chromium, with the
// invincible Hero and a faster clock, and screenshots each Stage from start
// to end. Fails if a Stage doesn't clear in time or the page logs an error.
// Usage: node cabinet-stages.mjs <base-url> <screenshot-dir> <cabinet 1-4>
// Normally run through scripts/stages-web.sh.
import { mkdir } from "node:fs/promises";
import { chromium } from "playwright";

const [base, out, cabinetArg] = process.argv.slice(2);
const cabinet = Number(cabinetArg);
if (!base || !out || !(cabinet >= 1 && cabinet <= 4)) {
  console.error("usage: node cabinet-stages.mjs <base-url> <screenshot-dir> <cabinet 1-4>");
  process.exit(2);
}
await mkdir(out, { recursive: true });

// The time scale is capped per frame by the engine, so the real speed-up
// depends on the frame rate; a Stage clear is detected, not timed.
const url = `${base}?invincible&time_scale=10`;
// Wall-clock limit for one Stage (a 4-minute Stage at even 2x is 120 s).
const STAGE_LIMIT_MS = 240_000;
const SHOT_EVERY_MS = 4_000;

const failures = [];
const fail = (msg) => {
  failures.push(msg);
  console.error(`FAIL: ${msg}`);
};
const browser = await chromium.launch({
  args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"],
});
const page = await (await browser.newContext({ viewport: { width: 960, height: 720 } })).newPage();
page.on("console", (m) => m.type() === "error" && fail(`console error: ${m.text()}`));
page.on("pageerror", (e) => fail(`page error: ${e.message}`));

const wait = (ms) => page.waitForTimeout(ms);
const tap = async (key) => {
  await page.keyboard.down(key);
  await wait(150);
  await page.keyboard.up(key);
  await wait(250);
};
const shot = (name) => page.screenshot({ path: `${out}/${name}.png` });
const saved = () => page.evaluate(() => localStorage.getItem("scroller.save") !== null);

try {
  await page.goto(url);
  await page.waitForSelector("#loading", { state: "detached", timeout: 60_000 });
  await page.click("#game");
  await wait(500);
  await tap("Space"); // NEW RUN
  await wait(500);

  // Skip the earlier Cabinets with the developer key C (clear Stage).
  for (let c = 1; c < cabinet; c++) {
    for (let s = 1; s <= 2; s++) {
      await tap("Space"); // start the Stage
      await wait(800);
      await tap("KeyC");
      await wait(800);
    }
    await wait(4000); // the Cabinet Transition
  }

  for (let stage = 1; stage <= 2; stage++) {
    const name = `${cabinet}-${stage}`;
    // Start the Stage; its Save is used up. A Cabinet Transition may still
    // be playing (they vary in length), so keep trying for a while.
    for (let tries = 0; tries < 20 && (await saved()); tries++) {
      await tap("Space");
      await wait(600);
    }
    if (await saved()) fail(`${name}: the Save was not used up when the Stage began`);
    // Hold fire, so the screenshots show the Hero shooting.
    await page.keyboard.down("KeyZ");
    const start = Date.now();
    let n = 0;
    while (!(await saved())) {
      if (Date.now() - start > STAGE_LIMIT_MS) {
        fail(`${name} did not clear within ${STAGE_LIMIT_MS / 1000} s`);
        break;
      }
      n += 1;
      await shot(`${name}-${String(n).padStart(2, "0")}`);
      await wait(SHOT_EVERY_MS);
    }
    await page.keyboard.up("KeyZ");
    console.log(`${name} cleared after ${((Date.now() - start) / 1000).toFixed(0)} s (${n} screenshots)`);
    await shot(`${name}-cleared`);
    if (stage === 1) await wait(500);
  }
} catch (e) {
  fail(`run did not finish: ${e.message}`);
  await shot("failure").catch(() => {});
}

await browser.close();
console.log(`Screenshots in ${out}`);
if (failures.length > 0) {
  console.error(`Stage run FAILED (${failures.length} problem${failures.length === 1 ? "" : "s"}).`);
  process.exit(1);
}
console.log("Both Stages cleared.");
