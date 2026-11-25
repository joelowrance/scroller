// Plays both Stages of one Cabinet unattended in headless Chromium, with the
// invincible Hero and the game sped up, and fails unless both clear in about
// their designed time with no page or console errors. Screenshots the start,
// middle and end of each Stage. Earlier Cabinets are skipped with the
// developer key C (clear Stage).
//
// Progress is read from the game's own Stage clock (`scroller.debug.stage`,
// written by the gameplay kit while debug_keys is on), so a slow browser
// that can't keep up with the time scale doesn't fail the test.
//
// Usage: node stages.mjs <base-url> <screenshot-dir> <cabinet 1-4>
// Run by scripts/smoke-web.sh (every Cabinet with Stages) and
// scripts/stages-web.sh (one Cabinet).
import { mkdir } from "node:fs/promises";
import { chromium } from "playwright";

const [base, out, cabinetArg] = process.argv.slice(2);
const cabinet = Number(cabinetArg);
if (!base || !out || !(cabinet >= 1 && cabinet <= 4)) {
  console.error("usage: node stages.mjs <base-url> <screenshot-dir> <cabinet 1-4>");
  process.exit(2);
}
await mkdir(out, { recursive: true });

const SCALE = 8;
// Longest a Stage may be designed to last (docs/DESIGN.md: 3-4 minutes).
const MAX_STAGE_SECONDS = 240;
// One long frame (up to 2 game seconds at 8x) may overshoot the end.
const SECONDS_SLACK = 3;
// Score a Stage clear must have reached, where a Cabinet pays a clear bonus.
const CLEAR_BONUS = { 1: 3000 };

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
const read = (key) =>
  page.evaluate((k) => JSON.parse(localStorage.getItem(`scroller.${k}`) ?? "null"), key);
const save = () => read("save");
const clock = () => read("debug.stage");
const shot = (name) => page.screenshot({ path: `${out}/stages-${name}.png` });
// Waits until the game itself has scrolled `fraction` of this Cabinet's
// Stage `stage` (0-based), however fast or slow the browser runs it.
const reach = async (stage, fraction, timeoutMs) => {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const c = await clock();
    if (c && c.cabinet === cabinet - 1 && c.stage === stage && c.progress >= fraction) return c;
    await wait(100);
  }
  return null;
};
// The Save a clear of Cabinet `c`'s (1-based) Stage `s` (0-based) writes:
// the next Stage of that Cabinet, or the first of the next. The last Stage
// of the Run has none.
const nextSave = (c, s) =>
  s === 0 ? { cabinet: c - 1, stage: 1 } : c < 4 ? { cabinet: c, stage: 0 } : null;
const isSave = (v, want) => v && want && v.cabinet === want.cabinet && v.stage === want.stage;
// Presses A until Cabinet `c`'s Stage `s` is playing, as the game's Stage
// clock shows, so Stage Cards and Cabinet Transitions of any length are
// waited out rather than timed.
const startStage = async (c, s) => {
  for (let tries = 0; tries < 60; tries++) {
    const k = await clock();
    if (k && k.cabinet === c - 1 && k.stage === s) return true;
    await tap("Space");
    await wait(350);
  }
  return false;
};

try {
  // The autopilot runs Cabinet 3's Hero, whose run drives the scroll.
  await page.goto(`${base}?invincible&autopilot&time_scale=${SCALE}`);
  await page.waitForSelector("#loading", { state: "detached", timeout: 60_000 });
  await page.click("#game");
  await wait(500);
  await tap("Space"); // NEW RUN
  await wait(500);

  // Skip the earlier Cabinets with the developer key C (clear Stage).
  for (let c = 1; c < cabinet; c++) {
    for (let s = 0; s < 2; s++) {
      if (!(await startStage(c, s))) throw new Error(`could not start Stage ${c}-${s + 1} to skip it`);
      for (let tries = 0; tries < 10 && !isSave(await save(), nextSave(c, s)); tries++) {
        await tap("KeyC");
        await wait(400);
      }
    }
  }

  for (let i = 0; i < 2; i++) {
    const name = `${cabinet}-${i + 1}`;
    if (!(await startStage(cabinet, i))) {
      fail(`Stage ${name} did not start`);
      await shot(`${name}-stuck`);
      break;
    }
    // Hold fire, so the screenshots show the Hero shooting.
    await page.keyboard.down("KeyZ");
    // Generous: the browser may run well below the requested speed.
    const budget = ((MAX_STAGE_SECONDS * 3) / SCALE) * 1000 + 30_000;
    let stuck = false;
    for (const [fraction, label] of [[0.02, "start"], [0.5, "middle"], [0.97, "end"]]) {
      if (!(await reach(i, fraction, budget))) {
        fail(`Stage ${name} never reached ${label} (${fraction})`);
        stuck = true;
        break;
      }
      await shot(`${name}-${label}`);
      if (label === "start" && (await save())) {
        fail(`Stage ${name}: the Save was not used up when the Stage began`);
      }
    }
    const end = stuck ? null : await reach(i, 1.0, budget);
    await page.keyboard.up("KeyZ");
    const want = nextSave(cabinet, i);
    let cleared = end !== null && want === null;
    for (let n = 0; n < 50 && end && want; n++) {
      const s = await save();
      if (s && s.cabinet === want.cabinet && s.stage === want.stage) {
        cleared = true;
        break;
      }
      await wait(100);
    }
    if (!cleared) {
      fail(`Stage ${name} did not clear`);
      await shot(`${name}-stuck`);
      break;
    }
    const seconds = end.seconds.toFixed(1);
    const designed = end.designed.toFixed(1);
    console.log(`Stage ${name} cleared after ${seconds} game seconds (designed: ${designed})`);
    if (Math.abs(end.seconds - end.designed) > SECONDS_SLACK) {
      fail(`Stage ${name} took ${seconds} game seconds, not about ${designed}`);
    }
    await wait(600);
    await shot(`${name}-cleared`);
    const bonus = CLEAR_BONUS[cabinet];
    if (i === 0 && bonus && (await save()).score < bonus) {
      fail(`Stage ${name}: no Stage clear bonus in the score`);
    }
  }
} catch (e) {
  fail(`flow did not finish: ${e.message}`);
  await shot(`${cabinet}-failure`).catch(() => {});
}

await browser.close();
if (failures.length > 0) {
  console.error(`Stage test FAILED (${failures.length} problem${failures.length === 1 ? "" : "s"}).`);
  process.exit(1);
}
console.log(`Both Cabinet ${cabinet} Stages cleared unattended.`);
