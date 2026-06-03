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

# Mobile OS - Host Toolchain and Dependency Checker

set -euo pipefail

# Define lists of tools to verify
REQUIRED_BINARIES=(
    "make"
    "aarch64-linux-gnu-gcc"
    "aarch64-linux-gnu-g++"
    "qemu-system-aarch64"
)

echo "🔍 [INFO] Verifying development environment dependencies..."
FAILED=0

for binary in "${REQUIRED_BINARIES[@]}"; do
    if command -v "$binary" &> /dev/null; then
        echo "  ✅ Found: $binary -> $(which "$binary")"
    else
        echo "  ❌ Missing: $binary"
        FAILED=1
    fi
done

if [ "$FAILED" -eq 1 ]; then
    echo "⚠️ [ERROR] Missing critical packages. Please install them before proceeding:"
    echo "    sudo apt-get update"
    echo "    sudo apt-get install make gcc-aarch64-linux-gnu g++-aarch64-linux-gnu qemu-system-arm"
    exit 1
else
    echo "🎉 [SUCCESS] Environment is fully prepared for cross-compiling."
    exit 0
fi
