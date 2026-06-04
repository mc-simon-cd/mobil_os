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

# Mobile OS - ARM64 Rootfs + Initramfs Builder
# Produces out/initramfs.cpio.gz with init engine + all system services.

set -euo pipefail

WORKSPACE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${WORKSPACE}/out"
ROOTFS_STAGE="${OUT_DIR}/rootfs_stage"
INITRAMFS="${OUT_DIR}/initramfs.cpio.gz"
CC="${CC:-aarch64-linux-gnu-gcc}"

echo "============================================="
echo "📦  Mobile OS — Rootfs + Initramfs Builder"
echo "============================================="
echo "   Workspace : $WORKSPACE"
echo "   Staging   : $ROOTFS_STAGE"
echo "   Output    : $INITRAMFS"
echo "   Compiler  : $CC"
echo "============================================="

# ── 1. Clean & create staging tree ─────────────────────────────────────────
echo "🗂️  [1/6] Creating directory tree..."
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
    "${ROOTFS_STAGE}/data" \
    "${ROOTFS_STAGE}/cache" \
    "${ROOTFS_STAGE}/var/log" \
    "${ROOTFS_STAGE}/etc/init.d" \
    "${ROOTFS_STAGE}/system/bin" \
    "${ROOTFS_STAGE}/system/lib" \
    "${ROOTFS_STAGE}/mnt"

# ── 2. Compile project init (PID 1) ────────────────────────────────────────
echo "🔨 [2/6] Compiling ARM64 init binary (static)..."
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
    -static
echo "   ✅ /sbin/init: $(du -sh "$INIT_BIN" | cut -f1)"

# /init — initramfs entry point (kernel looks for this first)
cp "$INIT_BIN" "${ROOTFS_STAGE}/init"
echo "   ✅ /init entry point set"

# ── 3. Copy system service binaries ────────────────────────────────────────
echo "📋 [3/6] Copying system service binaries..."
SRC_BIN="${OUT_DIR}/rootfs/system/bin"

SERVICES=(servicemanager powermanager apigateway inputflinger apprunner statusbar surfaceflinger)

for svc in "${SERVICES[@]}"; do
    if [ -f "${SRC_BIN}/${svc}" ]; then
        cp "${SRC_BIN}/${svc}" "${ROOTFS_STAGE}/system/bin/${svc}"
        echo "   ✅ ${svc} ($(du -sh "${ROOTFS_STAGE}/system/bin/${svc}" | cut -f1))"
    else
        echo "   ⚠️  ${svc} not found in ${SRC_BIN} — skipping"
    fi
done

# ── 3.5 Copy kernel modules ────────────────────────────────────────────────
echo "📦 [3.5/6] Copying kernel modules..."
mkdir -p "${ROOTFS_STAGE}/lib/modules"
if [ -f "${OUT_DIR}/e1000.ko" ]; then
    cp "${OUT_DIR}/e1000.ko" "${ROOTFS_STAGE}/lib/modules/e1000.ko"
    echo "   ✅ e1000.ko copied ($(du -sh "${ROOTFS_STAGE}/lib/modules/e1000.ko" | cut -f1))"
else
    echo "   ⚠️  out/e1000.ko not found!"
fi


# ── 4. Populate etc/ ────────────────────────────────────────────────────────
echo "📝 [4/6] Writing etc/ config files..."

cp "${INIT_SRC}/init.rc" "${ROOTFS_STAGE}/etc/init.rc"
echo "   ✅ init.rc copied ($(wc -l < "${ROOTFS_STAGE}/etc/init.rc") lines)"

cat > "${ROOTFS_STAGE}/etc/passwd" <<'PASSWDEOF'
root:x:0:0:root:/root:/bin/sh
system:x:1000:1000:system:/:/bin/false
PASSWDEOF

cat > "${ROOTFS_STAGE}/etc/group" <<'GROUPEOF'
root:x:0:
system:x:1000:
GROUPEOF

cat > "${ROOTFS_STAGE}/etc/fstab" <<'FSTABEOF'
proc      /proc     proc      defaults  0 0
sysfs     /sys      sysfs     defaults  0 0
devtmpfs  /dev      devtmpfs  defaults  0 0
tmpfs     /tmp      tmpfs     defaults  0 0
tmpfs     /run      tmpfs     defaults  0 0
FSTABEOF

cat > "${ROOTFS_STAGE}/etc/os-release" <<'OSEOF'
NAME="Mobile OS"
VERSION="1.0.0-alpha"
ID=mobileos
PRETTY_NAME="Mobile OS v1.0.0-alpha"
HOME_URL="https://github.com/mc-simon-cd/mobil_os"
OSEOF

# ── 5. Summary ──────────────────────────────────────────────────────────────
echo "📊 [5/6] Staging summary:"
echo "   /init           : $(du -sh "${ROOTFS_STAGE}/init" | cut -f1)"
echo "   /sbin/init      : $(du -sh "${ROOTFS_STAGE}/sbin/init" | cut -f1)"
echo "   /system/bin/*   : $(ls "${ROOTFS_STAGE}/system/bin/" | wc -l) binaries ($(ls "${ROOTFS_STAGE}/system/bin/"))"
echo "   /etc/init.rc    : $(wc -l < "${ROOTFS_STAGE}/etc/init.rc") lines"

# ── 6. Pack into cpio initramfs ─────────────────────────────────────────────
echo "📦 [6/6] Packing initramfs (cpio.gz)..."
( cd "$ROOTFS_STAGE" && find . | cpio -H newc -o 2>/dev/null | gzip > "$INITRAMFS" )

echo "============================================="
echo "✅ Initramfs ready: $INITRAMFS"
echo "   Size: $(du -sh "$INITRAMFS" | cut -f1)"
echo "============================================="
echo "🚀 Now run:  ./scripts/qemu-run.sh --headless"
echo "============================================="
