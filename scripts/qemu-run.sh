#!/usr/bin/env bash
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

# 3. Construct QEMU commands
QEMU_ARGS=(
    "-M" "$QEMU_MACHINE"
    "-cpu" "$QEMU_CPU"
    "-smp" "$QEMU_CORES"
    "-m" "$QEMU_RAM"
    "-netdev" "user,id=net0,hostfwd=tcp::8080-:8080"
    "-device" "virtio-net-device,netdev=net0"
)

if [ "$HEADLESS" = true ]; then
    QEMU_ARGS+=("-nographic")
    QEMU_ARGS+=("-append" "console=$SYSTEM_CONSOLE init=/system/bin/init root=/dev/vda rw")
else
    QEMU_ARGS+=("-device" "$QEMU_DISPLAY")
    QEMU_ARGS+=("-append" "console=$SYSTEM_CONSOLE init=/system/bin/init root=/dev/vda rw quiet logo.nologo")
    # For fully visual support, fall back dynamically if host X11 is not available
    if [ -z "${DISPLAY:-}" ]; then
        echo "⚠️  [WARN] No host DISPLAY variable detected. Forcing headless mode..."
        QEMU_ARGS+=("-nographic")
    fi
fi

# Placeholder for image checks (since kernel isn't built yet)
echo "🔍 [INFO] Scanning for kernel & rootfs binaries..."
KERNEL_IMAGE="${WORKSPACE_DIR}/out/kernel/Image"

if [ ! -f "$KERNEL_IMAGE" ]; then
    echo "⚠️  [WARN] Kernel image not compiled yet at: $KERNEL_IMAGE"
    echo "          Please build the OS modules and kernel before running."
    echo "          (Running in verification/dry-run configuration)"
    DRY_RUN=true
fi

# 4. Run or Dry-Run Execution
if [ "$DRY_RUN" = true ]; then
    echo "📝 [DRY RUN] Command draft:"
    echo "    qemu-system-aarch64 ${QEMU_ARGS[*]}"
    echo "============================================="
    echo "✅ [SUCCESS] Dry-run validation complete."
    exit 0
else
    echo "🚀 [BOOT] Booting target emulation..."
    exec qemu-system-aarch64 "${QEMU_ARGS[@]}" -kernel "$KERNEL_IMAGE"
fi
