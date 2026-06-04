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

# Mobile OS - Precompiled ARM64 Linux Kernel Downloader

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KERNEL_DIR="${WORKSPACE_DIR}/out/kernel"
KERNEL_IMAGE="${KERNEL_DIR}/Image"

# Debian Stable netboot installer kernel is a raw ARM64 uncompressed Image
DEBIAN_KERNEL_URL="http://ftp.debian.org/debian/dists/stable/main/installer-arm64/current/images/netboot/debian-installer/arm64/linux"

echo "============================================="
echo "📥 Downloading Precompiled ARM64 Linux Kernel"
echo "============================================="

mkdir -p "$KERNEL_DIR"

if command -v curl >/dev/null 2>&1; then
    echo "Using curl to download kernel..."
    curl -L -o "$KERNEL_IMAGE" "$DEBIAN_KERNEL_URL"
elif command -v wget >/dev/null 2>&1; then
    echo "Using wget to download kernel..."
    wget -O "$KERNEL_IMAGE" "$DEBIAN_KERNEL_URL"
else
    echo "❌ [ERROR] Neither curl nor wget is installed on the host!"
    echo "           Please install curl or wget first:"
    echo "           sudo apt-get install curl -y"
    exit 1
fi

echo "============================================="
echo "✅ [SUCCESS] Kernel downloaded successfully!"
echo "   Path: $KERNEL_IMAGE"
echo "============================================="
echo "🚀 You can now run QEMU emulator using:"
echo "   ./scripts/qemu-run.sh --headless"
echo "============================================="
