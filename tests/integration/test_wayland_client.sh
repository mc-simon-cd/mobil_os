#!/usr/bin/env bash
# Copyright 2026 mcsimon
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# Integration test for Wayland Client-Server communication (Milestone 18)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SF_BIN="${REPO_DIR}/out/rootfs/system/bin/surfaceflinger"
LAUNCHER_BIN="${REPO_DIR}/out/rootfs/system/bin/launcher"

echo "🧪 Running Wayland Client-Server Integration Test..."

# 1. Setup clean environment
TEST_RUNTIME_DIR="/tmp/sf-wayland-test-client"
rm -rf "$TEST_RUNTIME_DIR"
mkdir -p "$TEST_RUNTIME_DIR"
chmod 700 "$TEST_RUNTIME_DIR"

export XDG_RUNTIME_DIR="$TEST_RUNTIME_DIR"
export WAYLAND_DISPLAY="wayland-0"
SOCKET_PATH="${TEST_RUNTIME_DIR}/${WAYLAND_DISPLAY}"

SF_LOG="/tmp/sf_wayland_server.log"
LAUNCHER_LOG="/tmp/launcher_wayland_client.log"
rm -f "$SF_LOG" "$LAUNCHER_LOG"

SF_PID=0
LAUNCHER_PID=0

cleanup() {
    echo "🧹 Cleaning up background processes..."
    if [ "$LAUNCHER_PID" -ne 0 ] && kill -0 "$LAUNCHER_PID" 2>/dev/null; then
        kill -9 "$LAUNCHER_PID" || true
    fi
    if [ "$SF_PID" -ne 0 ]; then
        echo "Sending SIGTERM to surfaceflinger (PID: $SF_PID)..."
        kill -15 "$SF_PID" || true
        for i in {1..30}; do
            if ! kill -0 "$SF_PID" 2>/dev/null; then
                break
            fi
            sleep 0.1
        done
        if kill -0 "$SF_PID" 2>/dev/null; then
            kill -9 "$SF_PID" || true
        fi
    fi
    rm -rf "$TEST_RUNTIME_DIR"
    echo "✨ Cleanup complete."
}
trap cleanup EXIT

# 2. Launch surfaceflinger in background
echo "🚀 Spawning surfaceflinger in Wayland mode..."
"$SF_BIN" --wayland > "$SF_LOG" 2>&1 &
SF_PID=$!

# Wait for Wayland socket
echo "⏳ Waiting for Wayland socket..."
SOCKET_CREATED=false
for i in {1..50}; do
    if [ -S "$SOCKET_PATH" ]; then
        SOCKET_CREATED=true
        break
    fi
    sleep 0.1
done

if [ "$SOCKET_CREATED" = false ]; then
    echo "❌ Test Failed: Wayland socket was not created at $SOCKET_PATH"
    echo "=== surfaceflinger logs ==="
    cat "$SF_LOG"
    exit 1
fi

echo "✅ Wayland socket registered."

# 3. Launch launcher client with --wayland and --oneshot
echo "🚀 Spawning launcher in Wayland client mode..."
"$LAUNCHER_BIN" --wayland --oneshot > "$LAUNCHER_LOG" 2>&1 &
LAUNCHER_PID=$!

# Wait for launcher to exit
echo "⏳ Waiting for launcher to complete oneshot render..."
LAUNCHER_EXITED=false
for i in {1..50}; do
    if ! kill -0 "$LAUNCHER_PID" 2>/dev/null; then
        LAUNCHER_EXITED=true
        break
    fi
    sleep 0.1
done

if [ "$LAUNCHER_EXITED" = false ]; then
    echo "❌ Test Failed: Launcher client did not exit in a reasonable time."
    echo "=== launcher logs ==="
    cat "$LAUNCHER_LOG"
    exit 1
fi

# Get exit code of launcher
wait "$LAUNCHER_PID" && LAUNCHER_CODE=0 || LAUNCHER_CODE=$?
if [ "$LAUNCHER_CODE" -ne 0 ]; then
    echo "❌ Test Failed: Launcher client exited with non-zero code $LAUNCHER_CODE"
    echo "=== launcher logs ==="
    cat "$LAUNCHER_LOG"
    exit 1
fi

echo "✅ Launcher oneshot render completed successfully."

# 4. Verify surfaceflinger logged the XDG surface creation
echo "🔍 Verifying surfaceflinger logs for client connection..."
sleep 0.5 # Let log buffers flush

if grep -q "New XDG surface: toplevel" "$SF_LOG"; then
    echo "✅ Verified: surfaceflinger successfully created XDG surface for launcher client."
else
    echo "❌ Test Failed: 'New XDG surface: toplevel' was not found in surfaceflinger logs."
    echo "=== surfaceflinger logs ==="
    cat "$SF_LOG"
    echo "=== launcher logs ==="
    cat "$LAUNCHER_LOG"
    exit 1
fi

echo "🎉 Wayland Client-Server Integration Test Passed Successfully!"
