#!/usr/bin/env bash
# Builds the browser version into dist/. Needs the wasm32-unknown-unknown
# target and a wasm-bindgen-cli matching the wasm-bindgen version in Cargo.lock.
set -euo pipefail
cd "$(dirname "$0")/.."

# Extra cargo flags can be passed through, e.g. CARGO_FLAGS="--features bevy/debug".
cargo build --profile web --target wasm32-unknown-unknown ${CARGO_FLAGS:-}
rm -rf dist
wasm-bindgen --no-typescript --remove-name-section --remove-producers-section --target web --out-dir dist --out-name scroller \
  target/wasm32-unknown-unknown/web/scroller.wasm
cp web/index.html dist/
cp -r assets dist/
echo "Built dist/ — serve it with: python3 -m http.server -d dist 8080"
