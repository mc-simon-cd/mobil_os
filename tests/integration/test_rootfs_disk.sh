#!/usr/bin/env bash
# Integration smoke test for out/rootfs.ext4 (Milestone 16)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DISK="$ROOT/out/rootfs.ext4"
STAGE="$ROOT/out/rootfs_stage"

echo "🏗️  Building rootfs artifacts..."
"$ROOT/scripts/make-rootfs.sh" >/dev/null

if [[ ! -f "$DISK" ]]; then
    echo "❌ rootfs.ext4 not found — install e2fsprogs (mkfs.ext4)"
    exit 1
fi

echo "👉 Verify ext4 filesystem type"
file "$DISK" | grep -qi 'ext4'

echo "👉 Verify volume label"
if command -v dumpe2fs >/dev/null 2>&1; then
    dumpe2fs -h "$DISK" 2>/dev/null | grep -q 'orion-root'
elif command -v debugfs >/dev/null 2>&1; then
    debugfs -R 'show_super_stats' "$DISK" 2>/dev/null | grep -q 'orion-root'
else
    echo "   (skip label check — dumpe2fs/debugfs unavailable)"
fi

echo "👉 Verify /init on disk image"
if command -v debugfs >/dev/null 2>&1; then
    debugfs -R 'stat /init' "$DISK" 2>/dev/null | grep -q 'Inode:'
else
    echo "   (skip inode check — debugfs unavailable)"
fi

echo "👉 QEMU dry-run (--disk)"
DRY_OUT="$("$ROOT/scripts/qemu-run.sh" --disk --dry-run 2>&1)"
echo "$DRY_OUT" | grep -q 'disk-initramfs.cpio.gz'
echo "$DRY_OUT" | grep -q 'rdinit=/init'

echo "🎉 Rootfs ext4 disk image smoke test passed."
