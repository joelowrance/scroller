#!/usr/bin/env bash
# Frame-time check: plays both Stages of every Cabinet unattended in headless
# Chromium and prints frame-time percentiles for each (see tests/web/perf.mjs).
# Builds with the fps-log feature, so the game's own log lines are captured too.
# Results go to target/perf/<timestamp>/.
#
#   scripts/perf-web.sh                 all four Cabinets, small window (CPU cost)
#   scripts/perf-web.sh 2 4             only Cabinets 2 and 4
#   PERF_VIEWPORT=960x720 PERF_UNCAPPED= scripts/perf-web.sh
#                                       full-size window at the 60 Hz cap, so
#                                       software rendering is part of the cost
#   SMOKE_SKIP_BUILD=1 scripts/perf-web.sh   use the existing dist/ as is
#
# Software rendering on a shared machine makes absolute numbers pessimistic and
# noisy (check `uptime` first). Compare runs on the same machine, close in time.
set -euo pipefail
cd "$(dirname "$0")/.."

out=target/perf/$(date +%Y%m%d-%H%M%S)
mkdir -p "$out"
cabinets=${*:-1 2 3 4}
export PERF_VIEWPORT=${PERF_VIEWPORT:-320x240}
export PERF_UNCAPPED=${PERF_UNCAPPED-1}

if [ ! -d tests/web/node_modules ]; then
  npm ci --prefix tests/web --no-audit --no-fund
fi
if [ -z "${SMOKE_SKIP_BUILD:-}" ]; then
  CARGO_FLAGS="--features fps-log" ./scripts/build-web.sh
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

echo "Frame times at $url (window $PERF_VIEWPORT, uncapped: ${PERF_UNCAPPED:-no})"
echo "wasm: $(stat -c %s dist/scroller_bg.wasm) bytes, $(gzip -c dist/scroller_bg.wasm | wc -c) gzipped"
for c in $cabinets; do
  node tests/web/perf.mjs "$url" "$out" "$c" 8
done
