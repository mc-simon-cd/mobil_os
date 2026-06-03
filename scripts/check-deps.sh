#!/usr/bin/env bash
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
