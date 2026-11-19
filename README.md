# Scroller

An 80s-arcade-style game built with Rust and [Bevy](https://bevyengine.org/):
mostly side-scrolling, with some upward-scrolling sections, mixing on-foot
platforming stages with ship stages.

## Build and run

```sh
cargo run            # dev build
cargo run --release  # optimized build
```

On Linux, Bevy needs these system packages:

```sh
sudo apt install libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev pkg-config
```
