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

# Host Integration Test for API Gateway

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

echo "🏗️  Compiling components for HOST (x86_64/local)..."

# Build libipc for host
make -C "${REPO_DIR}/libs/libipc" clean
make -C "${REPO_DIR}/libs/libipc" CC=gcc AR=ar

# Build libipc-rs for host
make -C "${REPO_DIR}/libs/libipc-rs" clean
make -C "${REPO_DIR}/libs/libipc-rs" CC=gcc AR=ar

# Build servicemanager for host
make -C "${REPO_DIR}/services/servicemanager" clean
make -C "${REPO_DIR}/services/servicemanager" CC=gcc AR=ar

# Build powermanager for host
make -C "${REPO_DIR}/services/powermanager" clean
make -C "${REPO_DIR}/services/powermanager" CC=gcc AR=ar

# Build apigateway for host
make -C "${REPO_DIR}/services/apigateway" clean
make -C "${REPO_DIR}/services/apigateway" CC=gcc AR=ar

# Build inputflinger for host
make -C "${REPO_DIR}/services/inputflinger" clean
make -C "${REPO_DIR}/services/inputflinger" CC=gcc AR=ar

echo "🧹 Cleaning up old sockets..."
rm -f /tmp/servicemanager.sock
rm -f /tmp/powermanager.sock
rm -f /tmp/inputflinger.sock

# PIDs to clean up
SM_PID=0
PM_PID=0
IN_PID=0
API_PID=0

cleanup() {
    echo "🧹 Cleaning up background daemons..."
    [ "$API_PID" -ne 0 ] && kill -9 "$API_PID" || true
    [ "$IN_PID" -ne 0 ] && kill -9 "$IN_PID" || true
    [ "$PM_PID" -ne 0 ] && kill -9 "$PM_PID" || true
    [ "$SM_PID" -ne 0 ] && kill -9 "$SM_PID" || true
    rm -f /tmp/servicemanager.sock
    rm -f /tmp/powermanager.sock
    rm -f /tmp/inputflinger.sock
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

echo "🚀 Spawning Inputflinger..."
"${REPO_DIR}/out/rootfs/system/bin/inputflinger" &
IN_PID=$!
sleep 0.5

echo "🚀 Spawning API Gateway on port 8085..."
PORT=8085 "${REPO_DIR}/out/rootfs/system/bin/apigateway" &
API_PID=$!
sleep 0.5

echo "🧪 Running HTTP API Tests..."

# Test 1: GET /api/status
echo "👉 Test 1: GET /api/status"
STATUS_RESP=$(curl -s http://localhost:8085/api/status)
echo "   Response: $STATUS_RESP"
if [[ "$STATUS_RESP" != *'"status":"ok"'* ]]; then
    echo "❌ Test 1 Failed!"
    exit 1
fi
echo "✅ Test 1 Passed!"

# Test 2: GET /api/power (Initial)
echo "👉 Test 2: GET /api/power (Initial)"
POWER_RESP=$(curl -s http://localhost:8085/api/power)
echo "   Response: $POWER_RESP"
if [[ "$POWER_RESP" != *'"battery_level":85'* || "$POWER_RESP" != *'"power_mode":"balanced"'* ]]; then
    echo "❌ Test 2 Failed!"
    exit 1
fi
echo "✅ Test 2 Passed!"

# Test 3: POST /api/power/mode?mode=performance
echo "👉 Test 3: POST /api/power/mode?mode=performance"
MODE_RESP=$(curl -s -X POST "http://localhost:8085/api/power/mode?mode=performance")
echo "   Response: $MODE_RESP"
if [[ "$MODE_RESP" != *'"status":"success"'* || "$MODE_RESP" != *'"power_mode":"performance"'* ]]; then
    echo "❌ Test 3 Failed!"
    exit 1
fi
echo "✅ Test 3 Passed!"

# Test 4: GET /api/power (Verifying updated mode)
echo "👉 Test 4: GET /api/power (Verifying updated mode)"
POWER_RESP2=$(curl -s http://localhost:8085/api/power)
echo "   Response: $POWER_RESP2"
if [[ "$POWER_RESP2" != *'"battery_level":85'* || "$POWER_RESP2" != *'"power_mode":"performance"'* ]]; then
    echo "❌ Test 4 Failed!"
    exit 1
fi
echo "✅ Test 4 Passed!"

# Test 5: OPTIONS CORS check
echo "👉 Test 5: OPTIONS CORS preflight check"
OPTIONS_RESP=$(curl -s -D - -o /dev/null -X OPTIONS http://localhost:8085/api/power)
echo "$OPTIONS_RESP"
if [[ "$OPTIONS_RESP" != *"Access-Control-Allow-Origin: *"* ]]; then
    echo "❌ Test 5 Failed! CORS headers missing."
    exit 1
fi
echo "✅ Test 5 Passed!"

# Test 6: POST /api/input/inject
echo "👉 Test 6: POST /api/input/inject (KEY_POWER press)"
INJECT_RESP=$(curl -s -X POST "http://localhost:8085/api/input/inject?type=1&code=116&value=1")
echo "   Response: $INJECT_RESP"
if [[ "$INJECT_RESP" != *'"status":"success"'* || "$INJECT_RESP" != *'"type":1'* || "$INJECT_RESP" != *'"code":116'* || "$INJECT_RESP" != *'"value":1'* ]]; then
    echo "❌ Test 6 Failed!"
    exit 1
fi
echo "✅ Test 6 Passed!"

# Test 7: GET /api/input/last
echo "👉 Test 7: GET /api/input/last"
LAST_RESP=$(curl -s http://localhost:8085/api/input/last)
echo "   Response: $LAST_RESP"
if [[ "$LAST_RESP" != *'"status":"success"'* || "$LAST_RESP" != *'"type":1'* || "$LAST_RESP" != *'"code":116'* || "$LAST_RESP" != *'"value":1'* ]]; then
    echo "❌ Test 7 Failed!"
    exit 1
fi
echo "✅ Test 7 Passed!"

echo "🎉 All API Gateway Integration Tests Passed Successfully!"
