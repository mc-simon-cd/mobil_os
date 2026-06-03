# Board Support Package - QEMU ARM64 Configuration (board.mk)

BOARD_NAME := QEMU ARM64 Emulator
BOARD_ARCH := aarch64
BOARD_CPU  := cortex-a72

# Emulation parameters
QEMU_MACHINE := virt
QEMU_CPU     := cortex-a72
QEMU_RAM     := 2048M
QEMU_CORES   := 4

# Graphics & Screen Configuration
QEMU_DISPLAY  := virtio-gpu-pci
QEMU_KEYBOARD := virtio-keyboard-pci
QEMU_TABLET   := virtio-tablet-pci

# System Configurations
KERNEL_DEFCONFIG := qemu_arm64_defconfig
SYSTEM_CONSOLE   := ttyAMA0
