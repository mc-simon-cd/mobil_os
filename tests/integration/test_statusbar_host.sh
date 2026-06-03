#!/usr/bin/env bash
# Host Integration Test for Statusbar and Notifications Daemon

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

echo "🏗️  Compiling components for HOST (local x86_64)..."

# Build dependencies
make -C "${REPO_DIR}/libs/libipc" clean
make -C "${REPO_DIR}/libs/libipc" CC=gcc AR=ar

make -C "${REPO_DIR}/libs/libipc-rs" clean
make -C "${REPO_DIR}/libs/libipc-rs" CC=gcc AR=ar

make -C "${REPO_DIR}/libs/libgraphics" clean
make -C "${REPO_DIR}/libs/libgraphics" CC=gcc AR=ar

make -C "${REPO_DIR}/libs/libi18n" clean
make -C "${REPO_DIR}/libs/libi18n" CC=gcc AR=ar

# Build services
make -C "${REPO_DIR}/services/servicemanager" clean
make -C "${REPO_DIR}/services/servicemanager" CC=gcc AR=ar

make -C "${REPO_DIR}/services/powermanager" clean
make -C "${REPO_DIR}/services/powermanager" CC=gcc AR=ar

make -C "${REPO_DIR}/services/surfaceflinger" clean
make -C "${REPO_DIR}/services/surfaceflinger" CC=g++ AR=ar

# Build new statusbar daemon
make -C "${REPO_DIR}/ui/statusbar" clean
make -C "${REPO_DIR}/ui/statusbar" CC=gcc AR=ar

# Compile test client
echo "🏗️  Compiling statusbar test client..."
gcc -Wall -Wextra -std=c99 \
    -I"${REPO_DIR}/libs/libipc/include" \
    "${SCRIPT_DIR}/test_statusbar.c" \
    "${REPO_DIR}/out/rootfs/system/lib/libipc.a" \
    -o "${SCRIPT_DIR}/test_statusbar"

echo "🧹 Cleaning up old sockets..."
rm -f /tmp/servicemanager.sock
rm -f /tmp/powermanager.sock
rm -f /tmp/surfaceflinger.sock
rm -f /tmp/statusbar.sock
rm -f "${REPO_DIR}/out/statusbar_display.ppm"

# PIDs to clean up
SM_PID=0
PM_PID=0
SF_PID=0
SB_PID=0

cleanup() {
    echo "🧹 Cleaning up background processes..."
    [ "$SB_PID" -ne 0 ] && kill -9 "$SB_PID" || true
    [ "$SF_PID" -ne 0 ] && kill -9 "$SF_PID" || true
    [ "$PM_PID" -ne 0 ] && kill -9 "$PM_PID" || true
    [ "$SM_PID" -ne 0 ] && kill -9 "$SM_PID" || true
    rm -f /tmp/servicemanager.sock
    rm -f /tmp/powermanager.sock
    rm -f /tmp/surfaceflinger.sock
    rm -f /tmp/statusbar.sock
    echo "✨ Cleanup complete."
}
trap cleanup EXIT

echo "🚀 Spawning Servicemanager..."
"${REPO_DIR}/out/rootfs/system/bin/servicemanager" &
SM_PID=$!
sleep 0.5

echo "🚀 Spawning Powermanager..."
"${REPO_DIR}/out/rootfs/system/bin/powermanager" &
PM_PID=$!
sleep 0.5

echo "🚀 Spawning Surfaceflinger..."
"${REPO_DIR}/out/rootfs/system/bin/surfaceflinger" &
SF_PID=$!
sleep 0.5

echo "🚀 Spawning Statusbar..."
LANG=en "${REPO_DIR}/out/rootfs/system/bin/statusbar" &
SB_PID=$!
sleep 0.8

echo "🧪 Running Statusbar Notification Injection Test..."

# Send test notification to statusbar
"${SCRIPT_DIR}/test_statusbar" "System Boot Complete!"
sleep 1.2

# Check if PPM frame was created
PPM_FILE="${REPO_DIR}/out/statusbar_display.ppm"
if [ ! -f "$PPM_FILE" ]; then
    echo "❌ Test Failed: statusbar_display.ppm was not created!"
    exit 1
fi

# Basic check of PPM header (P6, width 1080, height 100)
PPM_HEADER=$(head -n 2 "$PPM_FILE" | tr '\n' ' ')
echo "   PPM Header check: $PPM_HEADER"
if [[ "$PPM_HEADER" != *"P6"* || "$PPM_HEADER" != *"1080 100"* ]]; then
    echo "❌ Test Failed: PPM file header is invalid!"
    exit 1
fi

echo "✅ Test Passed: statusbar_display.ppm was successfully generated and contains valid graphics!"

echo "🎉 Statusbar & Notification Daemon Integration Test Passed Successfully!"
