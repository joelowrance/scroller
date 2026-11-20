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
const save = () =>
  page.evaluate(() => JSON.parse(localStorage.getItem("scroller.save") ?? "null"));
const shot = (name) => page.screenshot({ path: `${out}/stages-${name}.png` });

try {
  await page.goto(`${base}?invincible&time_scale=${SCALE}`);
  await page.waitForSelector("#loading", { state: "detached", timeout: 60_000 });
  await page.click("#game");
  await wait(500);
  await tap("Space"); // NEW RUN
  await wait(500);

  for (const [i, stage] of STAGES.entries()) {
    await tap("Space"); // play the Stage
    const started = Date.now();
    const game = () => ((Date.now() - started) / 1000) * SCALE;
    // Hold fire, so the screenshots show the Hero shooting.
    await page.keyboard.down("KeyZ");
    for (const [at, label] of [[6, "start"], [stage.seconds / 2, "middle"], [stage.seconds - 8, "end"]]) {
      await wait(Math.max(0, ((at - game()) * 1000) / SCALE));
      await shot(`${stage.name}-${label}`);
    }
    // Clearing writes a Save for the next Stage (Cabinet 2 after 1-2).
    const want = i === 0 ? { cabinet: 0, stage: 1 } : { cabinet: 1, stage: 0 };
    const deadline = Date.now() + ((stage.seconds * 1.5) / SCALE) * 1000 + 30_000;
    let cleared = false;
    while (Date.now() < deadline) {
      const s = await save();
      if (s && s.cabinet === want.cabinet && s.stage === want.stage) {
        cleared = true;
        break;
      }
      await wait(500);
    }
    await page.keyboard.up("KeyZ");
    const took = game();
    if (!cleared) {
      fail(`Stage ${stage.name} did not clear`);
      await shot(`${stage.name}-stuck`);
      break;
    }
    console.log(`Stage ${stage.name} cleared after about ${Math.round(took)} game seconds`);
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
