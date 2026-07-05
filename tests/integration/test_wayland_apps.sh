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

# Integration test for all Wayland Clients (Milestone 18-19 Phase 3)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

SF_BIN="${REPO_DIR}/out/rootfs/system/bin/surfaceflinger"
STATUSBAR_BIN="${REPO_DIR}/out/rootfs/system/bin/statusbar"
SETTINGS_BIN="${REPO_DIR}/out/rootfs/system/bin/settings"
DIALER_BIN="${REPO_DIR}/out/rootfs/system/bin/dialer"

echo "🧪 Compiling Wayland-supported components for HOST..."

# Compile everything for host architecture
make -C "${REPO_DIR}/libs/libipc" clean
make -C "${REPO_DIR}/libs/libipc" CC=gcc AR=ar

make -C "${REPO_DIR}/libs/libgraphics" clean
make -C "${REPO_DIR}/libs/libgraphics" CC=gcc AR=ar

make -C "${REPO_DIR}/libs/libi18n" clean
make -C "${REPO_DIR}/libs/libi18n" CC=gcc AR=ar

make -C "${REPO_DIR}/services/servicemanager" clean
make -C "${REPO_DIR}/services/servicemanager" CC=gcc AR=ar

make -C "${REPO_DIR}/services/powermanager" clean
make -C "${REPO_DIR}/services/powermanager" CC=gcc AR=ar

make -C "${REPO_DIR}/services/surfaceflinger" clean
make -C "${REPO_DIR}/services/surfaceflinger" CC=g++ AR=ar

make -C "${REPO_DIR}/ui/statusbar" clean
make -C "${REPO_DIR}/ui/statusbar" CC=gcc AR=ar

make -C "${REPO_DIR}/apps/settings" clean
make -C "${REPO_DIR}/apps/settings" CC=gcc AR=ar

make -C "${REPO_DIR}/apps/dialer" clean
make -C "${REPO_DIR}/apps/dialer" CC=gcc AR=ar

echo "🚀 Setup clean Wayland environment..."
TEST_RUNTIME_DIR="/tmp/sf-wayland-test-all-apps"
rm -rf "$TEST_RUNTIME_DIR"
mkdir -p "$TEST_RUNTIME_DIR"
chmod 700 "$TEST_RUNTIME_DIR"

export XDG_RUNTIME_DIR="$TEST_RUNTIME_DIR"
export WAYLAND_DISPLAY="wayland-0"
SOCKET_PATH="${TEST_RUNTIME_DIR}/${WAYLAND_DISPLAY}"

SF_LOG="/tmp/sf_wayland_all_apps_server.log"
STATUSBAR_LOG="/tmp/statusbar_wayland_client.log"
SETTINGS_LOG="/tmp/settings_wayland_client.log"
DIALER_LOG="/tmp/dialer_wayland_client.log"

rm -f "$SF_LOG" "$STATUSBAR_LOG" "$SETTINGS_LOG" "$DIALER_LOG" "${REPO_DIR}/out/display_composited.ppm"

SF_PID=0
SB_PID=0
SM_PID=0
PM_PID=0

cleanup() {
    echo "🧹 Cleaning up background processes..."
    [ "$SB_PID" -ne 0 ] && kill -9 "$SB_PID" 2>/dev/null || true
    [ "$SF_PID" -ne 0 ] && kill -15 "$SF_PID" 2>/dev/null || true
    [ "$PM_PID" -ne 0 ] && kill -9 "$PM_PID" 2>/dev/null || true
    [ "$SM_PID" -ne 0 ] && kill -9 "$SM_PID" 2>/dev/null || true
    rm -rf "$TEST_RUNTIME_DIR"
    echo "✨ Cleanup complete."
}
trap cleanup EXIT

# Start Servicemanager and Powermanager since statusbar/settings/dialer query them
echo "🚀 Spawning Servicemanager..."
"${REPO_DIR}/out/rootfs/system/bin/servicemanager" >/dev/null 2>&1 &
SM_PID=$!
sleep 0.2

echo "🚀 Spawning Powermanager..."
"${REPO_DIR}/out/rootfs/system/bin/powermanager" >/dev/null 2>&1 &
PM_PID=$!
sleep 0.2

# Launch surfaceflinger in Wayland mode in background
echo "🚀 Spawning surfaceflinger in Wayland mode..."
(cd "${REPO_DIR}" && "$SF_BIN" --wayland > "$SF_LOG" 2>&1) &
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

# Launch statusbar in Wayland mode in background
echo "🚀 Spawning statusbar daemon in Wayland client mode..."
LANG=en "$STATUSBAR_BIN" --wayland > "$STATUSBAR_LOG" 2>&1 &
SB_PID=$!
sleep 0.5

# Launch settings in Wayland mode (oneshot)
echo "🚀 Spawning settings in Wayland client mode..."
LANG=en "$SETTINGS_BIN" --wayland --mode performance > "$SETTINGS_LOG" 2>&1
sleep 0.2

# Launch dialer in Wayland mode (oneshot)
echo "🚀 Spawning dialer in Wayland client mode..."
LANG=en "$DIALER_BIN" --wayland "+905559876543" > "$DIALER_LOG" 2>&1
sleep 0.2

# Verify surfaceflinger logged connection events
echo "🔍 Verifying surfaceflinger logs for connection events..."
sleep 0.5

SF_LOG_CONTENT=$(cat "$SF_LOG")
echo "=== surfaceflinger logs ==="
echo "$SF_LOG_CONTENT"

TLEVEL_COUNT=$(echo "$SF_LOG_CONTENT" | grep -c "New XDG surface: toplevel" || true)
echo "   Found $TLEVEL_COUNT 'New XDG surface: toplevel' occurrences in surfaceflinger logs."

# We expect at least 3 occurrences (statusbar, settings, dialer)
if [ "$TLEVEL_COUNT" -ge 3 ]; then
    echo "✅ Success: All client apps successfully established Wayland connections!"
else
    echo "❌ Test Failed: Some client apps failed to connect via Wayland (expected at least 3 connections)."
    echo "=== statusbar logs ==="
    cat "$STATUSBAR_LOG"
    echo "=== settings logs ==="
    cat "$SETTINGS_LOG"
    echo "=== dialer logs ==="
    cat "$DIALER_LOG"
    exit 1
fi

# Verify in-memory Wayland surface compositing output
echo "🔍 Verifying in-memory Wayland compositing output..."
COMPOSITED_PPM="${REPO_DIR}/out/display_composited.ppm"

if [ ! -f "$COMPOSITED_PPM" ]; then
    echo "❌ Test Failed: display_composited.ppm was not created in Wayland mode!"
    exit 1
fi

PPM_HEADER=$(head -n 2 "$COMPOSITED_PPM" | tr '\n' ' ')
echo "   Composited PPM Header check: $PPM_HEADER"
if [[ "$PPM_HEADER" != *"P6"* || "$PPM_HEADER" != *"1080 2400"* ]]; then
    echo "❌ Test Failed: Composited PPM file header is invalid!"
    exit 1
fi
echo "✅ Success: In-memory Wayland surface composition output verified!"

echo "🎉 All Wayland App client connections verified successfully!"
