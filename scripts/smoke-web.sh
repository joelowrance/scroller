#!/usr/bin/env bash
# Browser smoke test: builds the web version, serves it, drives a short Run in
# headless Chromium and fails on any page error, console error or bad Save /
# High Score state, then plays both Stages of every Cabinet that has them
# unattended. Each run's screenshots go to a new folder,
# target/smoke/<timestamp>/, and target/smoke/latest points at the newest.
#
#   scripts/smoke-web.sh                 build dist/, serve it locally, test it
#   scripts/smoke-web.sh <base-url>      test an already-running site instead,
#                                        e.g. http://joelowrance.com/scroller/
#   SMOKE_SKIP_BUILD=1 scripts/smoke-web.sh   test the existing dist/ as is
#
# Needs Node.js and Playwright's Chromium (see the README).
set -euo pipefail
cd "$(dirname "$0")/.."

out=target/smoke/$(date +%Y%m%d-%H%M%S)
mkdir -p "$out"
ln -sfn "$(basename "$out")" target/smoke/latest

if [ ! -d tests/web/node_modules ]; then
  npm ci --prefix tests/web --no-audit --no-fund
fi

url=${1:-}
if [ -z "$url" ]; then
  if [ -z "${SMOKE_SKIP_BUILD:-}" ]; then
    ./scripts/build-web.sh
  fi
  port=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
  python3 -m http.server --bind 127.0.0.1 --directory dist "$port" >"$out/server.log" 2>&1 &
  server=$!
  trap 'kill "$server" 2>/dev/null || true' EXIT
  url="http://127.0.0.1:$port/"
  for _ in $(seq 50); do
    curl -sf -o /dev/null "$url" && break
    sleep 0.1
  done
fi

echo "Smoke testing $url"
node tests/web/smoke.mjs "$url" "$out"
# Each Cabinet with Stage files must clear both unattended (invincible Hero,
# sped up). Add a Cabinet here when its Stages land.
for cabinet in 1 2 3 4; do
  node tests/web/stages.mjs "$url" "$out" "$cabinet"
done
