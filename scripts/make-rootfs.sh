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

# Mobile OS - Minimal ARM64 Rootfs Builder
# Builds out/rootfs.ext4 from the project's own init binary + rootfs overlay.

set -euo pipefail

WORKSPACE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${WORKSPACE}/out"
ROOTFS_STAGE="${OUT_DIR}/rootfs_stage"   # staging directory
ROOTFS_IMG="${OUT_DIR}/rootfs.ext4"
ROOTFS_SIZE_MB=64                        # image size in MB
CC="${CC:-aarch64-linux-gnu-gcc}"

echo "============================================="
echo "📦  Mobile OS — Rootfs Builder"
echo "============================================="
echo "   Workspace : $WORKSPACE"
echo "   Staging   : $ROOTFS_STAGE"
echo "   Output    : $ROOTFS_IMG (${ROOTFS_SIZE_MB} MB)"
echo "   Compiler  : $CC"
echo "============================================="

# ── 1. Clean & create staging tree ─────────────────────────────────────────
echo "🗂️  [1/5] Creating directory tree..."
rm -rf "$ROOTFS_STAGE"
mkdir -p \
    "${ROOTFS_STAGE}/sbin" \
    "${ROOTFS_STAGE}/bin" \
    "${ROOTFS_STAGE}/lib" \
    "${ROOTFS_STAGE}/lib64" \
    "${ROOTFS_STAGE}/usr/bin" \
    "${ROOTFS_STAGE}/usr/lib" \
    "${ROOTFS_STAGE}/proc" \
    "${ROOTFS_STAGE}/sys" \
    "${ROOTFS_STAGE}/dev" \
    "${ROOTFS_STAGE}/tmp" \
    "${ROOTFS_STAGE}/run" \
    "${ROOTFS_STAGE}/var/log" \
    "${ROOTFS_STAGE}/etc/init.d" \
    "${ROOTFS_STAGE}/system/bin" \
    "${ROOTFS_STAGE}/system/lib" \
    "${ROOTFS_STAGE}/mnt"

# ── 2. Compile project init (PID 1) ────────────────────────────────────────
echo "🔨 [2/5] Compiling ARM64 init binary..."
INIT_SRC="${WORKSPACE}/core/init"
INIT_BIN="${ROOTFS_STAGE}/sbin/init"

$CC \
    -Wall -Wextra -std=c99 -O2 \
    -I"${INIT_SRC}/include" \
    "${INIT_SRC}/src/main.c" \
    "${INIT_SRC}/src/mount.c" \
    "${INIT_SRC}/src/properties.c" \
    "${INIT_SRC}/src/init.c" \
    -o "$INIT_BIN" \
    -static   # statically linked — no dynamic linker needed
echo "   ✅ init binary: $INIT_BIN ($(du -sh "$INIT_BIN" | cut -f1))"

# ── 3. Populate etc/ ────────────────────────────────────────────────────────
echo "📝 [3/5] Writing etc/ config files..."

# Copy project's init.rc
cp "${INIT_SRC}/init.rc" "${ROOTFS_STAGE}/etc/init.rc"

# /etc/passwd — minimal users
cat > "${ROOTFS_STAGE}/etc/passwd" <<'EOF'
root:x:0:0:root:/root:/bin/sh
system:x:1000:1000:system:/:/bin/false
EOF

# /etc/group
cat > "${ROOTFS_STAGE}/etc/group" <<'EOF'
root:x:0:
system:x:1000:
EOF

# /etc/fstab
cat > "${ROOTFS_STAGE}/etc/fstab" <<'EOF'
proc      /proc     proc    defaults  0 0
sysfs     /sys      sysfs   defaults  0 0
devtmpfs  /dev      devtmpfs defaults 0 0
tmpfs     /tmp      tmpfs   defaults  0 0
tmpfs     /run      tmpfs   defaults  0 0
EOF

# /etc/init.d/rcS — early userspace setup
cat > "${ROOTFS_STAGE}/etc/init.d/rcS" <<'EOF'
#!/bin/sh
mount -a 2>/dev/null || true
mount -t devtmpfs none /dev 2>/dev/null || true
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts 2>/dev/null || true
echo "Mobile OS v1.0.0-alpha booting..."
echo "  Architecture : $(uname -m)"
echo "  Kernel       : $(uname -r)"
EOF
chmod +x "${ROOTFS_STAGE}/etc/init.d/rcS"

# ── 4. Basic /dev nodes ─────────────────────────────────────────────────────
echo "🔌 [4/5] Creating device nodes (may need sudo)..."
# Use mknod if root, otherwise skip (kernel devtmpfs will populate /dev)
if [ "$(id -u)" -eq 0 ]; then
    mknod -m 666 "${ROOTFS_STAGE}/dev/null"    c 1 3
    mknod -m 666 "${ROOTFS_STAGE}/dev/zero"    c 1 5
    mknod -m 600 "${ROOTFS_STAGE}/dev/console" c 5 1
    mknod -m 666 "${ROOTFS_STAGE}/dev/tty"     c 5 0
    mknod -m 666 "${ROOTFS_STAGE}/dev/ttyAMA0" c 204 64
    echo "   ✅ Device nodes created."
else
    echo "   ⚠️  Not root — skipping static device nodes (kernel devtmpfs will handle /dev at boot)."
fi

# ── 5. Pack into ext4 image ─────────────────────────────────────────────────
echo "💾 [5/5] Creating ext4 image (${ROOTFS_SIZE_MB} MB)..."
dd if=/dev/zero of="$ROOTFS_IMG" bs=1M count="$ROOTFS_SIZE_MB" status=none
mkfs.ext4 -q -L "mobileos_root" "$ROOTFS_IMG"

# Mount, copy, unmount
MOUNT_TMP=$(mktemp -d)
if [ "$(id -u)" -eq 0 ]; then
    mount -o loop "$ROOTFS_IMG" "$MOUNT_TMP"
    cp -a "${ROOTFS_STAGE}/." "$MOUNT_TMP/"
    umount "$MOUNT_TMP"
    echo "   ✅ Rootfs packed with loop mount."
else
    # Fallback: use debugfs (no root needed)
    echo "   ℹ️  Using debugfs (no root) to populate image..."
    find "$ROOTFS_STAGE" -type f | while read -r f; do
        rel="${f#${ROOTFS_STAGE}/}"
        dir=$(dirname "$rel")
        debugfs -w "$ROOTFS_IMG" -R "mkdir $dir" 2>/dev/null || true
        debugfs -w "$ROOTFS_IMG" -R "write $f $rel" 2>/dev/null || true
    done
    echo "   ✅ Rootfs packed via debugfs."
fi
rmdir "$MOUNT_TMP" 2>/dev/null || true

echo "============================================="
echo "✅ Rootfs image ready: $ROOTFS_IMG"
echo "   Size: $(du -sh "$ROOTFS_IMG" | cut -f1)"
echo "============================================="
echo "🚀 Now run:  ./scripts/qemu-run.sh --headless"
echo "============================================="
