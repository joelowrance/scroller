# Scroller

An 80s-arcade mashup built with Rust and [Bevy](https://bevyengine.org/). One
Hero plays through four original Cabinets, each in a classic arcade genre:

1. Side-scrolling shooter
2. Vertical shooter
3. Run-and-gun platformer
4. Vertical climber

Each Cabinet has two Stages. One score and one set of lives carry through
the whole Run, and losing the last life ends it for good. The game saves
after every cleared Stage, so you can stop between Stages and pick up later.
The domain language is defined in [GLOSSARY.md](GLOSSARY.md).

**Status:** the game is playable end to end in the browser and on desktop.
Cabinet 1 (STATIC TIDE, the side-scrolling shooter) and the Cabinets after
it each have Stages, enemies, music, sound effects and a Cabinet Transition
into the next. Some Cabinets are still being finished and playtested; see
[ROADMAP.md](ROADMAP.md) and the [issues](https://github.com/joelowrance/scroller/issues)
for what is left. The settled design is in [docs/DESIGN.md](docs/DESIGN.md).

## Play it

- **In the browser:** http://joelowrance.com/scroller/ (rebuilt on every push to `main`)
- **On desktop:** `cargo run --release`

## How to play

Press A on the title screen to start a NEW RUN (or CONTINUE a saved one).
You play one Hero through four Cabinets, two Stages each. Shoot, dodge and
grab Power-ups; each Cabinet plays differently, from a side-scrolling shooter
to a vertical climber. You have a few lives for the whole Run, and extra
lives come with high scores. Lose the last one and the Run is over: enter
your initials if your score makes the High Score Table. Clearing a Stage
saves the Run, so you can quit between Stages and pick up later with
CONTINUE. Beating a Cabinet unlocks it in Practice, where you can replay it.
The version and short git hash are shown small on the title screen.

## Controls

| Action | Keyboard                  | Gamepad         |
|--------|---------------------------|-----------------|
| Move   | WASD or arrow keys        | D-pad or stick  |
| A      | Space (or Z)              | South (A/Cross) |
| B      | `;` (or X)                | East or West    |
| Start  | Enter or Escape (pause)   | Start           |

**M** mutes the sound. The setting is kept with the high scores. In the browser,
sound starts with the first key press or click.

### Developer keys

Off in release builds and on the live site. They are on in dev builds
(`cargo run`) and, in the browser, when the URL has `?debug`
(for example `http://joelowrance.com/scroller/?debug`): **K** loses a life,
**P** adds 1000 points, **C** clears the Stage, **F2** shows the sprite
sheet. With `?debug` the URL can also add `invincible`, `autopilot`,
`gallery` and `time_scale=N`. `debug_keys: true` in `assets/tuning.ron`
turns them on for a desktop release build too. Debug mode also writes
`scroller.debug.stage` to `localStorage`, which the browser Stage test reads.

## Tuning

Every gameplay number lives in [`assets/tuning.ron`](assets/tuning.ron).
On desktop the file is re-read while the game runs, so you can edit and see
the change without restarting. The web build uses the committed values.

## Building

On Linux, Bevy needs these system packages:

```sh
sudo apt install libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev pkg-config
```

```sh
cargo run              # desktop, dev build
cargo test             # unit tests for the Run rules, Save, scores and input
./scripts/build-web.sh # browser build into dist/
```

The web build needs `rustup target add wasm32-unknown-unknown` and a
`wasm-bindgen-cli` matching the `wasm-bindgen` version in `Cargo.lock`.

## Browser smoke test

`scripts/smoke-web.sh` builds the web version, serves it locally and drives a
short Run in headless Chromium. It plays the live-site behaviour first (no `?debug`: the developer keys
must do nothing), then a short Run in debug mode. It fails on any page error,
console error or warning, or wrong Save / High Score Table state in `localStorage`. Each run saves a
screenshot of every screen in a new `target/smoke/<timestamp>/` folder;
`target/smoke/latest` links to the newest.

One-time setup (needs Node.js 20 or newer):

```sh
npm ci --prefix tests/web
npx --prefix tests/web playwright install --with-deps chromium  # --with-deps needs sudo; drop it if the libraries are present
```

Then:

```sh
./scripts/smoke-web.sh                                  # build, serve and test
SMOKE_SKIP_BUILD=1 ./scripts/smoke-web.sh               # test the existing dist/
./scripts/smoke-web.sh http://joelowrance.com/scroller/ # test a deployed site
./scripts/perf-web.sh [cabinet...]                      # frame times per Stage (fps-log build)
```

CI runs it too, as the `smoke` job, on the web build from the `check` job. Its
screenshots are attached to the run as the `smoke-screenshots` artifact.

## Where data is kept

The Save, High Score Table and Practice unlocks are stored as JSON files in
your data directory (`~/.local/share/scroller/` on Linux) on desktop, and in
`localStorage` in the browser.
