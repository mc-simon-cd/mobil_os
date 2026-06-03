#!/usr/bin/env bash
# Host Integration Test for App Runner (apprunner) and Multi-Language Runtimes

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

echo "🏗️  Compiling components for HOST (local x86_64)..."

# Build dependencies
make -C "${REPO_DIR}/libs/libipc" clean
make -C "${REPO_DIR}/libs/libipc" CC=gcc AR=ar

make -C "${REPO_DIR}/libs/libipc-rs" clean
make -C "${REPO_DIR}/libs/libipc-rs" CC=gcc AR=ar

# Build service managers & daemons
make -C "${REPO_DIR}/services/servicemanager" clean
make -C "${REPO_DIR}/services/servicemanager" CC=gcc AR=ar

make -C "${REPO_DIR}/services/powermanager" clean
make -C "${REPO_DIR}/services/powermanager" CC=gcc AR=ar

make -C "${REPO_DIR}/services/inputflinger" clean
make -C "${REPO_DIR}/services/inputflinger" CC=gcc AR=ar

make -C "${REPO_DIR}/services/apigateway" clean
make -C "${REPO_DIR}/services/apigateway" CC=gcc AR=ar

# Build the new apprunner
make -C "${REPO_DIR}/services/apprunner" clean
make -C "${REPO_DIR}/services/apprunner" CC=gcc AR=ar

# Re-run build.sh to package everything to out/rootfs/system/apps
echo "📂 Packaging rootfs..."
"${REPO_DIR}/scripts/build.sh"

echo "🧹 Cleaning up old sockets..."
rm -f /tmp/servicemanager.sock
rm -f /tmp/powermanager.sock
rm -f /tmp/inputflinger.sock

# PIDs to clean up
SM_PID=0
PM_PID=0
IN_PID=0
API_PID=0
WEB_PID=0

cleanup() {
    echo "🧹 Cleaning up background processes..."
    [ "$WEB_PID" -ne 0 ] && kill -9 "$WEB_PID" || true
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
sleep 0.8

echo "🧪 Running Multi-Language Runtime Integration Tests..."

# Test 1: Run Python app "sys_reporter" via apprunner
echo "👉 Test 1: Running Python App (sys_reporter)"
export API_PORT=8085
export LANG=en
PYTHON_OUT=$("${REPO_DIR}/out/rootfs/system/bin/apprunner" "${REPO_DIR}/out/rootfs/system/apps/sys_reporter")
echo "--- Python Output ---"
echo "$PYTHON_OUT"
echo "---------------------"

if [[ "$PYTHON_OUT" != *"System Report"* || "$PYTHON_OUT" != *"Battery"* || "$PYTHON_OUT" != *"85%"* ]]; then
    echo "❌ Test 1 Failed: Python output does not contain expected battery report!"
    exit 1
fi
echo "✅ Test 1 Passed (Python runtime executes successfully and gets translated keys)!"

# Test 2: Run Python app in Turkish locale to verify translation loading
echo "👉 Test 2: Running Python App in Turkish Locale (LANG=tr)"
export LANG=tr
PYTHON_OUT_TR=$("${REPO_DIR}/out/rootfs/system/bin/apprunner" "${REPO_DIR}/out/rootfs/system/apps/sys_reporter")
echo "--- Python TR Output ---"
echo "$PYTHON_OUT_TR"
echo "------------------------"

if [[ "$PYTHON_OUT_TR" != *"Pil: 85%"* || "$PYTHON_OUT_TR" != *"Güç Modu: Dengeli"* ]]; then
    echo "❌ Test 2 Failed: Turkish localization failed for Python app!"
    exit 1
fi
echo "✅ Test 2 Passed (Python app translates keys to Turkish dynamically)!"

# Test 3: Run JS app "sys_monitor" via apprunner in test-once mode
echo "👉 Test 3: Running JavaScript App (sys_monitor)"
export TEST_ONCE=true
export API_PORT=8085
JS_OUT=$("${REPO_DIR}/out/rootfs/system/bin/apprunner" "${REPO_DIR}/out/rootfs/system/apps/sys_monitor")
echo "--- JS Output ---"
echo "$JS_OUT"
echo "-----------------"

if [[ "$JS_OUT" != *"Battery: 85%"* || "$JS_OUT" != *"Mode: balanced"* ]]; then
    echo "❌ Test 3 Failed: JS output does not match expected pattern!"
    exit 1
fi
echo "✅ Test 3 Passed (JS runtime executes successfully via Node and reads API)!"

# Test 4: Run Web App server via apprunner and fetch static files
echo "👉 Test 4: Serving Web App (control_center) and verifying HTTP responses"
export APP_PORT=8090
"${REPO_DIR}/out/rootfs/system/bin/apprunner" "${REPO_DIR}/out/rootfs/system/apps/control_center" &
WEB_PID=$!
sleep 0.8

# Fetch index.html
HTML_RESP=$(curl -s http://localhost:8090/index.html)
if [[ "$HTML_RESP" != *"Mobile OS - Control Center"* || "$HTML_RESP" != *"app.js"* ]]; then
    echo "❌ Test 4 Failed: HTML Response from served Web App is missing or incorrect!"
    exit 1
fi

# Fetch style.css
CSS_RESP=$(curl -s http://localhost:8090/style.css)
if [[ "$CSS_RESP" != *"--bg-dark"* || "$CSS_RESP" != *"glass-card"* ]]; then
    echo "❌ Test 4 Failed: CSS Response from served Web App is missing or incorrect!"
    exit 1
fi

# Fetch app.js
JS_RESP=$(curl -s http://localhost:8090/app.js)
if [[ "$JS_RESP" != *"fetchSystemStatus"* || "$JS_RESP" != *"simulateKeyClick"* ]]; then
    echo "❌ Test 4 Failed: JS Response from served Web App is missing or incorrect!"
    exit 1
fi

# Fetch locale files served relative to app
LOCALE_RESP=$(curl -s http://localhost:8090/locale/tr.txt)
if [[ "$LOCALE_RESP" != *"power.battery=Pil"* ]]; then
    echo "❌ Test 4 Failed: Translations from served Web App locale folder are not reachable!"
    exit 1
fi

echo "✅ Test 4 Passed (Web App served correctly, static assets and locales are fully fetchable)!"

echo "🎉 All App Runner & Multi-Language Runtime Integration Tests Passed Successfully!"
