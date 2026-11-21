// Plays Cabinet 1's two Stages unattended in headless Chromium, with the
// invincible Hero and the game sped up, and fails unless both clear.
// Screenshots the start, middle and end of each Stage.
// Usage: node stages.mjs <base-url> <screenshot-dir>
// Run by scripts/smoke-web.sh after smoke.mjs.
import { mkdir } from "node:fs/promises";
import { chromium } from "playwright";

const [base, out] = process.argv.slice(2);
if (!base || !out) {
  console.error("usage: node stages.mjs <base-url> <screenshot-dir>");
  process.exit(2);
}
await mkdir(out, { recursive: true });

const SCALE = 8;
// Stage lengths at normal speed (see docs/cabinets/1-side-shooter.md).
const STAGES = [
  { name: "1-1", seconds: 210 },
  { name: "1-2", seconds: 225 },
];

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
// With debug_keys, Cabinet 1 records the Stage's real progress here.
const clock = () => read("debug.stage");
const shot = (name) => page.screenshot({ path: `${out}/stages-${name}.png` });
// Waits until the game itself has scrolled `fraction` of Stage `stage`,
// however fast or slow the browser runs it.
const reach = async (stage, fraction, timeoutMs) => {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const c = await clock();
    if (c && c.stage === stage && c.progress >= fraction) return c;
    await wait(100);
  }
  return null;
};

try {
  await page.goto(`${base}?invincible&time_scale=${SCALE}`);
  await page.waitForSelector("#loading", { state: "detached", timeout: 60_000 });
  await page.click("#game");
  await wait(500);
  await tap("Space"); // NEW RUN
  await wait(500);

  for (const [i, stage] of STAGES.entries()) {
    await tap("Space"); // play the Stage
    // Hold fire, so the screenshots show the Hero shooting.
    await page.keyboard.down("KeyZ");
    // Generous: the browser may run well below the requested speed.
    const budget = ((stage.seconds * 3) / SCALE) * 1000 + 30_000;
    let stuck = false;
    for (const [fraction, label] of [[0.02, "start"], [0.5, "middle"], [0.97, "end"]]) {
      if (!(await reach(i, fraction, budget))) {
        fail(`Stage ${stage.name} never reached ${label} (${fraction})`);
        stuck = true;
        break;
      }
      await shot(`${stage.name}-${label}`);
    }
    const end = stuck ? null : await reach(i, 1.0, budget);
    await page.keyboard.up("KeyZ");
    // Clearing writes a Save for the next Stage (Cabinet 2 after 1-2).
    const want = i === 0 ? { cabinet: 0, stage: 1 } : { cabinet: 1, stage: 0 };
    let cleared = false;
    for (let n = 0; n < 50 && end; n++) {
      const s = await save();
      if (s && s.cabinet === want.cabinet && s.stage === want.stage) {
        cleared = true;
        break;
      }
      await wait(100);
    }
    if (!cleared) {
      fail(`Stage ${stage.name} did not clear`);
      await shot(`${stage.name}-stuck`);
      break;
    }
    const seconds = end.seconds.toFixed(1);
    console.log(`Stage ${stage.name} cleared after ${seconds} game seconds (designed: ${stage.seconds})`);
    // One long frame (up to 2 game seconds at 8x) may overshoot the end.
    if (Math.abs(end.seconds - stage.seconds) > 3) {
      fail(`Stage ${stage.name} took ${seconds} game seconds, not about ${stage.seconds}`);
    }
    await wait(600);
    await shot(`${stage.name}-cleared`);
    if (i === 0 && (await save()).score < 3000) fail("no Stage clear bonus in the score");
  }
} catch (e) {
  fail(`flow did not finish: ${e.message}`);
  await shot("failure").catch(() => {});
}

await browser.close();
if (failures.length > 0) {
  console.error(`Stage test FAILED (${failures.length} problem${failures.length === 1 ? "" : "s"}).`);
  process.exit(1);
}
console.log("Both Cabinet 1 Stages cleared unattended.");
