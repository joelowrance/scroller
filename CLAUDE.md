# Scroller

An 80s-arcade mashup in Rust and Bevy 0.20, played in the browser and on desktop.

- **Roadmap:** read [ROADMAP.md](ROADMAP.md) before picking up work. It sets the ticket order, the definition of done, and how to make decisions alone.
- **Design:** the settled decisions are in [docs/DESIGN.md](docs/DESIGN.md). Each Cabinet's design is in `docs/cabinets/`.
- **Language:** use the terms in [GLOSSARY.md](GLOSSARY.md) in code, docs and commits. For example, the human is the player and the character is the Hero.
- **Bevy 0.20 notes:**
  - Events are `Message`s (`MessageReader` / `MessageWriter`).
  - Run conditions have no `.and`/`.or`, so chain `.run_if` calls or write a condition function.
  - Screen entities use `DespawnOnExit(Screen::…)`.
- **Web build:** `wasm-bindgen-cli` has to match the `wasm-bindgen` version in `Cargo.lock` exactly.
- **This machine:** no display or controller, so check visual work with the headless browser screenshots.
