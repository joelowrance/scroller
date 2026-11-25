// Plays a full Run to the Ending in headless Chromium with the developer
// keys: every Stage is cleared with C, and P adds points in each Cabinet so
// the Ending's per-Cabinet tally has something to show. Screenshots the
// Ending's summit, its look-back panels and the tally, then checks the Run
// is over: no Save left, and the score (with the lives bonus) on the High
// Score Table after initials.
//
// Usage: node ending.mjs <base-url> <screenshot-dir>
// Run by scripts/smoke-web.sh.
import { mkdir } from "node:fs/promises";
import { chromium } from "playwright";

const [base, out] = process.argv.slice(2);
if (!base || !out) {
  console.error("usage: node ending.mjs <base-url> <screenshot-dir>");
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
const page = await (await browser.newContext({ viewport: { width: 960, height: 720 } })).newPage();
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
const read = (key) =>
  page.evaluate((k) => JSON.parse(localStorage.getItem(`scroller.${k}`) ?? "null"), key);
const shot = (name) => page.screenshot({ path: `${out}/ending-${name}.png` });

// Presses A until Cabinet `c`'s (1-based) Stage `s` (0-based) is playing, as
// the game's Stage clock shows, so Stage Cards and Cabinet Transitions of
// any length are waited out rather than timed.
const startStage = async (c, s) => {
  for (let tries = 0; tries < 60; tries++) {
    const k = await read("debug.stage");
    if (k && k.cabinet === c - 1 && k.stage === s) return true;
    await tap("Space");
    await wait(350);
  }
  return false;
};
// The Save a clear of Cabinet `c`'s Stage `s` writes; none after the last.
const nextSave = (c, s) => (s === 0 ? { cabinet: c - 1, stage: 1 } : c < 4 ? { cabinet: c, stage: 0 } : null);

try {
  await page.goto(`${base}?debug&invincible`);
  await page.waitForSelector("#loading", { state: "detached", timeout: 60_000 });
  await page.click("#game");
  await wait(500);
  await tap("Space"); // NEW RUN
  await wait(500);

  for (let c = 1; c <= 4; c++) {
    for (let s = 0; s < 2; s++) {
      if (!(await startStage(c, s))) throw new Error(`could not start Stage ${c}-${s + 1}`);
      await wait(400);
      if (s === 0) await tap("KeyP", c); // a different tally for each Cabinet
      const want = nextSave(c, s);
      if (want === null) {
        await tap("KeyC"); // the last Stage: on to the Ending
        break;
      }
      let saved = false;
      for (let tries = 0; tries < 10 && !saved; tries++) {
        await tap("KeyC");
        await wait(400);
        const v = await read("save");
        saved = v && v.cabinet === want.cabinet && v.stage === want.stage;
      }
      if (!saved) throw new Error(`Stage ${c}-${s + 1} did not clear`);
    }
  }
  const before = (await read("debug.stage")) ?? {};
  console.log(`Last Stage cleared: Cabinet ${before.cabinet + 1}, Stage ${before.stage + 1}`);

  // The final Stage's walk-off, then the Ending's own timeline (13 s). How
  // long the walk-off takes in a browser varies, so shoot every second
  // rather than aim at the beats.
  for (let second = 1; second <= 17; second++) {
    await wait(1_000);
    await shot(String(second).padStart(2, "0") + "s");
  }

  if ((await read("save")) !== null) fail("a Save exists after the Ending: the Run should be over");

  await tap("Space"); // initials (the score should qualify)
  await wait(500);
  await shot("name-entry");
  await tap("ArrowUp", 4); // E
  await tap("Space", 3);
  await wait(600);
  await shot("high-scores");
  const entries = (await read("high_scores"))?.entries ?? [];
  // 10,000 a life left; the invincible Hero keeps all three.
  if (!entries.some((e) => e.initials.startsWith("E") && e.score >= 30_000)) {
    fail(`no Ending score on the High Score Table: ${JSON.stringify(entries.map((e) => `${e.initials} ${e.score}`))}`);
  }
} catch (e) {
  fail(`flow did not finish: ${e.message}`);
  await shot("failure").catch(() => {});
}

await browser.close();
if (failures.length > 0) {
  console.error(`Ending test FAILED (${failures.length} problem${failures.length === 1 ? "" : "s"}).`);
  process.exit(1);
}
console.log("Reached the Ending through a full Run.");
