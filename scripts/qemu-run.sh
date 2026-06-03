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

# Mobile OS - QEMU AArch64 Emulation Launcher

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BOARD_MK="${WORKSPACE_DIR}/board/qemu-arm64/board.mk"

# 1. Import Board variables
if [ -f "$BOARD_MK" ]; then
    # Parse Makefile-style variables safely into shell variables
    BOARD_NAME=$(grep "BOARD_NAME" "$BOARD_MK" | cut -d'=' -f2 | xargs)
    QEMU_MACHINE=$(grep "QEMU_MACHINE" "$BOARD_MK" | cut -d'=' -f2 | xargs)
    QEMU_CPU=$(grep "QEMU_CPU" "$BOARD_MK" | cut -d'=' -f2 | xargs)
    QEMU_RAM=$(grep "QEMU_RAM" "$BOARD_MK" | cut -d'=' -f2 | xargs)
    QEMU_CORES=$(grep "QEMU_CORES" "$BOARD_MK" | cut -d'=' -f2 | xargs)
    QEMU_DISPLAY=$(grep "QEMU_DISPLAY" "$BOARD_MK" | cut -d'=' -f2 | xargs)
    SYSTEM_CONSOLE=$(grep "SYSTEM_CONSOLE" "$BOARD_MK" | cut -d'=' -f2 | xargs)
else
    echo "⚠️ [ERROR] Board definition not found at $BOARD_MK!"
    exit 1
fi

# 2. Setup launch parameters
DRY_RUN=false
HEADLESS=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --headless)
            HEADLESS=true
            shift
            ;;
        *)
            echo "❌ [ERROR] Unknown option: $1"
            echo "Usage: $0 [--dry-run] [--headless]"
            exit 1
            ;;
    esac
done

echo "============================================="
echo "💻  Launching: $BOARD_NAME"
echo "============================================="
echo "   Machine: $QEMU_MACHINE"
echo "   CPU:     $QEMU_CPU (Cores: $QEMU_CORES)"
echo "   RAM:     $QEMU_RAM"
echo "   Console: $SYSTEM_CONSOLE"
echo "============================================="

# 3. Port configuration
HOST_HTTP_PORT=9595   # host:9595  → guest:8080  (HTTP / web server)
echo "🌐 [NET] Port mapping: host:${HOST_HTTP_PORT} → guest:8080 (HTTP)"

# 4. Construct QEMU commands
QEMU_ARGS=(
    "-M" "$QEMU_MACHINE"
    "-cpu" "$QEMU_CPU"
    "-smp" "$QEMU_CORES"
    "-m" "$QEMU_RAM"
    "-netdev" "user,id=net0,hostfwd=tcp::${HOST_HTTP_PORT}-:8080"
    "-device" "virtio-net-device,netdev=net0"
)

if [ "$HEADLESS" = true ]; then
    QEMU_ARGS+=("-nographic")
    QEMU_ARGS+=("-serial" "mon:stdio")
else
    QEMU_ARGS+=("-device" "$QEMU_DISPLAY")
    # For fully visual support, fall back dynamically if host X11 is not available
    if [ -z "${DISPLAY:-}" ]; then
        echo "⚠️  [WARN] No host DISPLAY variable detected. Forcing headless mode..."
        QEMU_ARGS+=("-nographic")
        QEMU_ARGS+=("-serial" "mon:stdio")
    fi
fi

# Kernel console append args
KERNEL_CMDLINE="console=$SYSTEM_CONSOLE earlycon panic=5"

# 5. Scan for kernel & optional rootfs
echo "🔍 [INFO] Scanning for kernel & rootfs binaries..."
KERNEL_IMAGE="${WORKSPACE_DIR}/out/kernel/Image"
ROOTFS_IMAGE="${WORKSPACE_DIR}/out/rootfs.ext4"

if [ ! -f "$KERNEL_IMAGE" ]; then
    echo "⚠️  [WARN] Kernel image not found at: $KERNEL_IMAGE"
    echo "          Please run: ./scripts/download-kernel.sh"
    echo "          (Running in verification/dry-run configuration)"
    DRY_RUN=true
fi

# Attach rootfs only if it exists
if [ -f "$ROOTFS_IMAGE" ]; then
    echo "💾 [INFO] Rootfs found: $ROOTFS_IMAGE"
    QEMU_ARGS+=(
        "-drive" "file=${ROOTFS_IMAGE},format=raw,if=virtio"
    )
    KERNEL_CMDLINE+=" root=/dev/vda rw init=/system/bin/init"
else
    echo "⚠️  [INFO] No rootfs image found — booting kernel only (expect kernel panic, normal for first boot)"
fi

QEMU_ARGS+=("-append" "$KERNEL_CMDLINE")

# 6. Run or Dry-Run Execution
if [ "$DRY_RUN" = true ]; then
    echo "📝 [DRY RUN] Command draft:"
    echo "    qemu-system-aarch64 ${QEMU_ARGS[*]} -kernel $KERNEL_IMAGE"
    echo "============================================="
    echo "✅ [SUCCESS] Dry-run validation complete."
    exit 0
else
    echo "🚀 [BOOT] Booting target emulation..."
    echo "   (Press Ctrl+A then X to exit QEMU)"
    echo "============================================="
    exec qemu-system-aarch64 "${QEMU_ARGS[@]}" -kernel "$KERNEL_IMAGE"
fi
