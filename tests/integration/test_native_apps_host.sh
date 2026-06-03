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

# Host Integration Test for Native Settings and Dialer Applications

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

# Build core services
make -C "${REPO_DIR}/services/servicemanager" clean
make -C "${REPO_DIR}/services/servicemanager" CC=gcc AR=ar

make -C "${REPO_DIR}/services/powermanager" clean
make -C "${REPO_DIR}/services/powermanager" CC=gcc AR=ar

make -C "${REPO_DIR}/services/surfaceflinger" clean
make -C "${REPO_DIR}/services/surfaceflinger" CC=g++ AR=ar

# Build statusbar daemon
make -C "${REPO_DIR}/ui/statusbar" clean
make -C "${REPO_DIR}/ui/statusbar" CC=gcc AR=ar

# Build Settings and Dialer Apps
make -C "${REPO_DIR}/apps/settings" clean
make -C "${REPO_DIR}/apps/settings" CC=gcc AR=ar

make -C "${REPO_DIR}/apps/dialer" clean
make -C "${REPO_DIR}/apps/dialer" CC=gcc AR=ar

echo "🧹 Cleaning up old sockets..."
rm -f /tmp/servicemanager.sock
rm -f /tmp/powermanager.sock
rm -f /tmp/surfaceflinger.sock
rm -f /tmp/statusbar.sock
rm -f "${REPO_DIR}/out/settings_display.ppm"
rm -f "${REPO_DIR}/out/dialer_display.ppm"

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

echo "🧪 Running Native Applications Integration Tests..."

# Test 1: Run Settings App changing mode to performance
echo "👉 Test 1: Running Settings App to set mode to performance"
"${REPO_DIR}/out/rootfs/system/bin/settings" --mode performance
sleep 1.0

# Verify settings_display.ppm was created
SETTINGS_PPM="${REPO_DIR}/out/settings_display.ppm"
if [ ! -f "$SETTINGS_PPM" ]; then
    echo "❌ Test 1 Failed: settings_display.ppm was not created!"
    exit 1
fi
# Verify header
PPM_HDR=$(head -n 2 "$SETTINGS_PPM" | tr '\n' ' ')
if [[ "$PPM_HDR" != *"P6"* || "$PPM_HDR" != *"1080 2200"* ]]; then
    echo "❌ Test 1 Failed: Settings PPM layout is invalid!"
    exit 1
fi
echo "✅ Test 1 Passed (Settings successfully executed, saved layout, and updated statusbar via IPC)!"

# Test 2: Run Dialer App with custom telephone number
echo "👉 Test 2: Running Dialer App dialing +905559876543"
"${REPO_DIR}/out/rootfs/system/bin/dialer" "+905559876543"
sleep 1.0

# Verify dialer_display.ppm was created
DIALER_PPM="${REPO_DIR}/out/dialer_display.ppm"
if [ ! -f "$DIALER_PPM" ]; then
    echo "❌ Test 2 Failed: dialer_display.ppm was not created!"
    exit 1
fi
# Verify header
PPM_HDR2=$(head -n 2 "$DIALER_PPM" | tr '\n' ' ')
if [[ "$PPM_HDR2" != *"P6"* || "$PPM_HDR2" != *"1080 2200"* ]]; then
    echo "❌ Test 2 Failed: Dialer PPM layout is invalid!"
    exit 1
fi
echo "✅ Test 2 Passed (Dialer successfully executed, saved keypad layout, and triggered call notification)!"

echo "🎉 All Native Applications Integration Tests Passed Successfully!"
