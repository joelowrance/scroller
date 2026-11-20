#!/usr/bin/env bash
# Screenshots Cabinet 1's debug enemy gallery in headless Chromium. Each run
# goes to a new folder, target/gallery/<timestamp>/.
#
#   scripts/gallery-web.sh                 build dist/, serve it, screenshot
#   SMOKE_SKIP_BUILD=1 scripts/gallery-web.sh   use the existing dist/
#
# Needs what scripts/smoke-web.sh needs.
set -euo pipefail
cd "$(dirname "$0")/.."

out=target/gallery/$(date +%Y%m%d-%H%M%S)
mkdir -p "$out"

if [ ! -d tests/web/node_modules ]; then
  npm ci --prefix tests/web --no-audit --no-fund
fi
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
node tests/web/gallery.mjs "$url" "$out"
