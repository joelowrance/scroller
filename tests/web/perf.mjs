// Measures frame times while a Cabinet's Stages play unattended in headless
// Chromium (invincible, autopiloted Hero holding fire), so the busiest moments
// of every Stage are exercised. Samples requestAnimationFrame intervals in the
// page, tagged with the Stage's progress read from the game's own clock
// (`scroller.debug.stage`), and prints percentiles plus the worst 5 s window.
//
// Software rendering (SwiftShader) makes absolute numbers pessimistic; compare
// runs of this script with each other, on the same machine, not with a GPU.
//
// Usage: node perf.mjs <base-url> <out-dir> <cabinet 1-4> [time-scale]
// PERF_VIEWPORT=320x240 shrinks the window (see above). Normally run through scripts/perf-web.sh. For the in-game log lines too,
// build with CARGO_FLAGS="--features fps-log" (they are echoed to the console).
import { mkdir, writeFile } from "node:fs/promises";
import { chromium } from "playwright";

const [base, out, cabinetArg, scaleArg] = process.argv.slice(2);
const cabinet = Number(cabinetArg);
if (!base || !out || !(cabinet >= 1 && cabinet <= 4)) {
  console.error("usage: node perf.mjs <base-url> <out-dir> <cabinet 1-4> [time-scale]");
  process.exit(2);
}
await mkdir(out, { recursive: true });
const SCALE = Number(scaleArg ?? process.env.PERF_SCALE ?? 4);
const MAX_STAGE_SECONDS = 240;
const WINDOW_MS = 5000;
// A tiny window makes software rendering cheap, so frame times show the
// game's own CPU cost; the default size shows the two together.
const [VIEW_W, VIEW_H] = (process.env.PERF_VIEWPORT ?? "960x720").split("x").map(Number);

const browser = await chromium.launch({
  args: [
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
    "--ignore-gpu-blocklist",
    // PERF_UNCAPPED=1 lifts the 60 Hz cap, so frame times show real cost
    // (and headroom) instead of snapping to multiples of 16.7 ms.
    ...(process.env.PERF_UNCAPPED ? ["--disable-frame-rate-limit", "--disable-gpu-vsync"] : []),
  ],
});
const page = await (await browser.newContext({ viewport: { width: VIEW_W, height: VIEW_H } })).newPage();
const failures = [];
const gameLog = [];
page.on("console", (m) => {
  if (m.type() === "error") failures.push(`console error: ${m.text()}`);
  else if (/diagnostic|fps|frame_time|entity/i.test(m.text())) gameLog.push(m.text());
});
page.on("pageerror", (e) => failures.push(`page error: ${e.message}`));

// In-page sampler: one entry per animation frame, [timestamp ms, dt ms, stage, progress].
await page.addInitScript(() => {
  window.__perf = { on: false, frames: [], stage: -1, progress: 0 };
  let last = performance.now();
  const tick = (now) => {
    const p = window.__perf;
    if (p.on) p.frames.push([now, now - last, p.stage, p.progress]);
    last = now;
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
  setInterval(() => {
    try {
      const c = JSON.parse(localStorage.getItem("scroller.debug.stage") ?? "null");
      if (c) {
        window.__perf.stage = c.stage;
        window.__perf.progress = c.progress;
        window.__perf.cabinet = c.cabinet;
      }
    } catch {}
  }, 250);
});

// Main-thread CPU seconds so far (the game's own work, not waiting for the
// machine), so runs on a busy shared box can still be compared.
const cdp = await page.context().newCDPSession(page);
await cdp.send("Performance.enable");
const cpuSeconds = async () => {
  const { metrics } = await cdp.send("Performance.getMetrics");
  return metrics.find((m) => m.name === "TaskDuration").value;
};

const wait = (ms) => page.waitForTimeout(ms);
const tap = async (key) => {
  await page.keyboard.down(key);
  await wait(150);
  await page.keyboard.up(key);
  await wait(250);
};
const read = (key) =>
  page.evaluate((k) => JSON.parse(localStorage.getItem(`scroller.${k}`) ?? "null"), key);
const clock = () => read("debug.stage");
const nextSave = (c, s) =>
  s === 0 ? { cabinet: c - 1, stage: 1 } : c < 4 ? { cabinet: c, stage: 0 } : null;
const isSave = (v, want) => v && want && v.cabinet === want.cabinet && v.stage === want.stage;
const startStage = async (c, s) => {
  for (let tries = 0; tries < 60; tries++) {
    const k = await clock();
    if (k && k.cabinet === c - 1 && k.stage === s) return true;
    await tap("Space");
    await wait(350);
  }
  return false;
};

const pct = (sorted, q) => sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))];
const summarize = (frames) => {
  const dts = frames.map((f) => f[1]).sort((a, b) => a - b);
  const mean = dts.reduce((a, b) => a + b, 0) / dts.length;
  // Worst window: the 5 s stretch with the highest mean frame time.
  let worst = { mean: 0, at: 0, fps: 0 };
  for (let i = 0, j = 0; i < frames.length; i++) {
    while (frames[i][0] - frames[j][0] > WINDOW_MS) j++;
    if (frames[i][0] - frames[j][0] < WINDOW_MS * 0.9) continue;
    let sum = 0;
    for (let k = j; k <= i; k++) sum += frames[k][1];
    const m = sum / (i - j + 1);
    if (m > worst.mean) worst = { mean: m, at: frames[i][3], fps: 1000 / m };
  }
  return {
    frames: dts.length,
    mean_ms: mean,
    p50_ms: pct(dts, 0.5),
    p95_ms: pct(dts, 0.95),
    p99_ms: pct(dts, 0.99),
    max_ms: dts[dts.length - 1],
    over_20ms_pct: (100 * dts.filter((d) => d > 20).length) / dts.length,
    worst_window_mean_ms: worst.mean,
    worst_window_fps: worst.fps,
    worst_window_progress: worst.at,
  };
};

const results = [];
try {
  const loadStart = Date.now();
  await page.goto(`${base}?debug&invincible&autopilot&time_scale=${SCALE}`);
  await page.waitForSelector("#loading", { state: "detached", timeout: 60_000 });
  console.log(`Load: ${((Date.now() - loadStart) / 1000).toFixed(1)} s from navigation to the title screen (local server, cold cache)`);
  await page.click("#game");
  await wait(500);
  await tap("Space"); // NEW RUN
  await wait(500);

  for (let c = 1; c < cabinet; c++) {
    for (let s = 0; s < 2; s++) {
      if (!(await startStage(c, s))) throw new Error(`could not start Stage ${c}-${s + 1} to skip it`);
      for (let tries = 0; tries < 10 && !isSave(await read("save"), nextSave(c, s)); tries++) {
        await tap("KeyC");
        await wait(400);
      }
    }
  }

  for (let s = 0; s < 2; s++) {
    if (!(await startStage(cabinet, s))) throw new Error(`Stage ${cabinet}-${s + 1} did not start`);
    await page.keyboard.down("KeyZ");
    const cpuStart = await cpuSeconds();
    await page.evaluate(() => {
      window.__perf.frames = [];
      window.__perf.on = true;
    });
    const budget = ((MAX_STAGE_SECONDS * 3) / SCALE) * 1000 + 30_000;
    const deadline = Date.now() + budget;
    while (Date.now() < deadline) {
      const k = await clock();
      const save = await read("save");
      if (isSave(save, nextSave(cabinet, s)) || (k && k.progress >= 1)) break;
      await wait(500);
    }
    await page.keyboard.up("KeyZ");
    const cpuUsed = (await cpuSeconds()) - cpuStart;
    const frames = await page.evaluate(() => {
      window.__perf.on = false;
      return window.__perf.frames.filter((f) => f[2] >= 0);
    });
    // Drop the Stage-end screens: keep frames up to the last progress < 1.
    const play = frames.filter((f) => f[3] < 1);
    const name = `${cabinet}-${s + 1}`;
    if (play.length < 100) {
      failures.push(`Stage ${name}: only ${play.length} frames sampled`);
      continue;
    }
    const summary = summarize(play);
    // Counts the whole sampled span, including the few frames after the clear.
    results.push({
      stage: name,
      scale: SCALE,
      ...summary,
      cpu_ms_per_frame: (cpuUsed * 1000) / frames.length,
    });
    // Let the Stage-clear screens pass so the next Stage can start.
    await wait(600);
  }
} catch (e) {
  failures.push(`flow did not finish: ${e.message}`);
}

await browser.close();
await writeFile(`${out}/perf-${cabinet}.json`, JSON.stringify({ results, gameLog }, null, 2));
const f = (n) => n.toFixed(1);
for (const r of results) {
  console.log(
    `Stage ${r.stage} (x${r.scale}): ${r.frames} frames, mean ${f(r.mean_ms)} ms, ` +
      `p50 ${f(r.p50_ms)}, p95 ${f(r.p95_ms)}, p99 ${f(r.p99_ms)}, max ${f(r.max_ms)}, ` +
      `>20ms ${f(r.over_20ms_pct)}%, cpu ${f(r.cpu_ms_per_frame)} ms/frame, worst 5s ${f(r.worst_window_mean_ms)} ms ` +
      `(${f(r.worst_window_fps)} fps at ${(r.worst_window_progress * 100).toFixed(0)}%)`,
  );
}
if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}
