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

**Status:** the framework is in place (screens, Run rules, Save, High Score
Table, Practice, pause, tuning). Every Cabinet is currently a stand-in that
scrolls and clears on a timer. The real Cabinets are next: see
[ROADMAP.md](ROADMAP.md) and the [issues](https://github.com/joelowrance/scroller/issues).
The settled design is in [docs/DESIGN.md](docs/DESIGN.md).

## Play it

- **In the browser:** http://joelowrance.com/scroller/ (rebuilt on every push to `main`)
- **On desktop:** `cargo run --release`

## Controls

| Action | Keyboard                  | Gamepad         |
|--------|---------------------------|-----------------|
| Move   | WASD or arrow keys        | D-pad or stick  |
| A      | Space (or Z)              | South (A/Cross) |
| B      | `;` (or X)                | East or West    |
| Start  | Enter or Escape (pause)   | Start           |

While the stand-in Cabinets are in place, developer keys are on: **K** loses a
life, **P** adds 1000 points, **C** clears the Stage.

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

## Where data is kept

The Save, High Score Table and Practice unlocks are stored as JSON files in
your data directory (`~/.local/share/scroller/` on Linux) on desktop, and in
`localStorage` in the browser.
