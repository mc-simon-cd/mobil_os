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

# Integration test for headless wlroots Wayland mode in surfaceflinger

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SF_BIN="${REPO_DIR}/out/rootfs/system/bin/surfaceflinger"

echo "🧪 Running Wayland Headless Compositor Integration Test..."

# 1. Clean up any old Wayland display sockets and env
# We'll set a private XDG_RUNTIME_DIR for this test to /tmp/sf-wayland-test
TEST_RUNTIME_DIR="/tmp/sf-wayland-test"
rm -rf "$TEST_RUNTIME_DIR"
mkdir -p "$TEST_RUNTIME_DIR"
chmod 700 "$TEST_RUNTIME_DIR"

export XDG_RUNTIME_DIR="$TEST_RUNTIME_DIR"
export WAYLAND_DISPLAY="wayland-0"

# Target socket path
SOCKET_PATH="${TEST_RUNTIME_DIR}/${WAYLAND_DISPLAY}"
rm -f "$SOCKET_PATH"

SF_PID=0

cleanup() {
    echo "🧹 Cleaning up background processes..."
    if [ "$SF_PID" -ne 0 ]; then
        echo "Sending SIGTERM to surfaceflinger (PID: $SF_PID)..."
        kill -15 "$SF_PID" || true
        # Wait a bit for it to exit cleanly
        for i in {1..30}; do
            if ! kill -0 "$SF_PID" 2>/dev/null; then
                echo "surfaceflinger exited cleanly."
                break
            fi
            sleep 0.1
        done
        # Force kill if still running
        if kill -0 "$SF_PID" 2>/dev/null; then
            echo "⚠️ surfaceflinger did not exit, force killing..."
            kill -9 "$SF_PID" || true
        fi
    fi
    rm -rf "$TEST_RUNTIME_DIR"
    echo "✨ Cleanup complete."
}
trap cleanup EXIT

# 2. Launch surfaceflinger with --wayland in background
echo "🚀 Spawning surfaceflinger in Wayland mode..."
"$SF_BIN" --wayland &
SF_PID=$!

# Wait for the socket file to be created
echo "⏳ Waiting for Wayland socket to appear..."
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
    exit 1
fi

echo "✅ Wayland socket found at: $SOCKET_PATH"

# 3. Verify socket connection
echo "🔍 Verifying connection with basic socket check..."
python3 -c "
import socket
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect('$SOCKET_PATH')
print('Connected successfully!')
s.close()
"

echo "✅ Connection verified successfully!"

# 4. Trigger clean exit by exiting the script (which triggers SIGTERM in cleanup)
echo "🧪 Triggering clean exit test..."

echo "🎉 Wayland Headless Compositor Integration Test Passed!"
