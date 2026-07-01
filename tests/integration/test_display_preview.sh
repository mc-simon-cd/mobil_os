#!/usr/bin/env bash
# Integration smoke test for GET /api/display/* (Milestone 15)
# Builds host-native apigateway (same host triple as rustc).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PORT=18080
HOST_TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"
APG_SRC="$ROOT/services/apigateway"
APG_BIN="$APG_SRC/target/${HOST_TRIPLE}/release/apigateway"

echo "🏗️  Building host-native apigateway (${HOST_TRIPLE})..."
make -C "$ROOT/libs/libipc-rs" clean >/dev/null 2>&1 || true
make -C "$ROOT/libs/libipc-rs" CC=gcc AR=ar >/dev/null
(
  cd "$APG_SRC"
  export CARGO_TARGET_DIR="$APG_SRC/target"
  cargo build --release --target "$HOST_TRIPLE"
)

if [[ ! -x "$APG_BIN" ]]; then
  echo "❌ apigateway binary not found: $APG_BIN"
  exit 1
fi

mkdir -p "$ROOT/out"
printf 'P6\n2 2\n255\n' > "$ROOT/out/display_composited.ppm"
printf '\xff\0\0\0\xff\0\0\0\xff' >> "$ROOT/out/display_composited.ppm"

cd "$ROOT"
PORT=$PORT "$APG_BIN" &
PID=$!
sleep 1

cleanup() { kill "$PID" 2>/dev/null || true; }
trap cleanup EXIT

echo "👉 GET /api/display/info"
BODY=$(curl -sf "http://127.0.0.1:${PORT}/api/display/info")
echo "   $BODY"
echo "$BODY" | grep -q '"available":true'
echo "$BODY" | grep -q '"width":2'

echo "👉 GET /api/display/frame"
FRAME_MAGIC=$(curl -sf "http://127.0.0.1:${PORT}/api/display/frame" | head -c 2)
[[ "$FRAME_MAGIC" == "P6" ]]

echo "🎉 Display preview API smoke test passed."
