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

# Orion OS - Precompiled ARM64 Linux Kernel + e1000 Module Downloader

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KERNEL_DIR="${WORKSPACE_DIR}/out/kernel"
KERNEL_IMAGE="${KERNEL_DIR}/Image"
E1000_MODULE="${WORKSPACE_DIR}/out/e1000.ko"
VIRTIO_BLK_MODULE="${WORKSPACE_DIR}/out/virtio_blk.ko"
EXT4_MODULE="${WORKSPACE_DIR}/out/ext4.ko"
JBD2_MODULE="${WORKSPACE_DIR}/out/jbd2.ko"
MODULE_CACHE="${WORKSPACE_DIR}/out/.module_cache"

DEBIAN_KERNEL_URL="http://ftp.debian.org/debian/dists/stable/main/installer-arm64/current/images/netboot/debian-installer/arm64/linux"
DEBIAN_NIC_POOL="http://ftp.debian.org/debian/pool/main/l/linux-signed-arm64"

download_file() {
    local url="$1"
    local dest="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$dest" "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$dest" "$url"
    else
        echo "❌ [ERROR] Neither curl nor wget is installed on the host!"
        echo "           Please install curl or wget first:"
        echo "           sudo apt-get install curl -y"
        exit 1
    fi
}

extract_e1000_module() {
    if [ ! -f "$KERNEL_IMAGE" ]; then
        echo "⚠️  [WARN] Kernel image missing; skipping e1000 module extraction."
        return 0
    fi

    if ! command -v strings >/dev/null 2>&1; then
        echo "⚠️  [WARN] 'strings' not found; cannot match kernel module version."
        return 0
    fi

    local kernel_line kver_full kver_deb udeb_name udeb_url module_xz
    kernel_line="$(strings "$KERNEL_IMAGE" | grep -m1 '^Linux version ' || true)"
    if [ -z "$kernel_line" ]; then
        echo "⚠️  [WARN] Could not parse kernel version from Image."
        return 0
    fi

    kver_full="$(sed -n 's/Linux version \([^ ]*\).*/\1/p' <<< "$kernel_line")"
    kver_deb="$(sed -n 's/.*# SMP Debian \([^ ]*\).*/\1/p' <<< "$kernel_line")"
    if [ -z "$kver_full" ] || [ -z "$kver_deb" ]; then
        echo "⚠️  [WARN] Could not derive Debian module package version."
        echo "          Kernel line: $kernel_line"
        return 0
    fi

    udeb_name="nic-modules-${kver_full}-di_${kver_deb}_arm64.udeb"
    udeb_url="${DEBIAN_NIC_POOL}/${udeb_name}"
    module_xz="lib/modules/${kver_full}/kernel/drivers/net/ethernet/intel/e1000/e1000.ko.xz"

    echo "📦 [INFO] Fetching network modules: ${udeb_name}"
    mkdir -p "$MODULE_CACHE"
    local udeb_path="${MODULE_CACHE}/${udeb_name}"

    if [ ! -f "$udeb_path" ]; then
        if ! download_file "$udeb_url" "$udeb_path"; then
            echo "⚠️  [WARN] Failed to download ${udeb_name}"
            echo "          Network driver module will not be bundled."
            return 0
        fi
    fi

    local extract_dir="${MODULE_CACHE}/nic-modules"
    rm -rf "$extract_dir"
    mkdir -p "$extract_dir"
    ( cd "$extract_dir" && ar x "$udeb_path" && tar xf data.tar.* )

    local src_module="${extract_dir}/${module_xz}"
    if [ ! -f "$src_module" ]; then
        src_module="$(find "$extract_dir" -path '*/e1000/e1000.ko.xz' | head -1 || true)"
    fi

    if [ -z "$src_module" ] || [ ! -f "$src_module" ]; then
        echo "⚠️  [WARN] e1000.ko.xz not found inside ${udeb_name}"
        return 0
    fi

    if ! command -v xz >/dev/null 2>&1; then
        echo "⚠️  [WARN] 'xz' not found; cannot decompress e1000.ko.xz"
        echo "          Install: sudo apt-get install xz-utils -y"
        return 0
    fi

    xz -dc "$src_module" > "$E1000_MODULE"
    echo "✅ [SUCCESS] e1000 network module ready: $E1000_MODULE"
    echo "   Size: $(du -sh "$E1000_MODULE" | cut -f1)"
}

extract_virtio_blk_module() {
    if [ ! -f "$KERNEL_IMAGE" ]; then
        echo "⚠️  [WARN] Kernel image missing; skipping virtio_blk module extraction."
        return 0
    fi

    local kernel_line kver_full kver_deb udeb_name udeb_url module_xz
    kernel_line="$(strings "$KERNEL_IMAGE" | grep -m1 '^Linux version ' || true)"
    if [ -z "$kernel_line" ]; then
        return 0
    fi

    kver_full="$(sed -n 's/Linux version \([^ ]*\).*/\1/p' <<< "$kernel_line")"
    kver_deb="$(sed -n 's/.*# SMP Debian \([^ ]*\).*/\1/p' <<< "$kernel_line")"
    if [ -z "$kver_full" ] || [ -z "$kver_deb" ]; then
        return 0
    fi

    udeb_name="scsi-modules-${kver_full}-di_${kver_deb}_arm64.udeb"
    udeb_url="${DEBIAN_NIC_POOL}/${udeb_name}"
    module_xz="lib/modules/${kver_full}/kernel/drivers/block/virtio_blk.ko.xz"

    echo "📦 [INFO] Fetching block modules: ${udeb_name}"
    mkdir -p "$MODULE_CACHE"
    local udeb_path="${MODULE_CACHE}/${udeb_name}"

    if [ ! -f "$udeb_path" ]; then
        if ! download_file "$udeb_url" "$udeb_path"; then
            echo "⚠️  [WARN] Failed to download ${udeb_name}"
            return 0
        fi
    fi

    local extract_dir="${MODULE_CACHE}/scsi-modules"
    rm -rf "$extract_dir"
    mkdir -p "$extract_dir"
    ( cd "$extract_dir" && ar x "$udeb_path" && tar xf data.tar.* )

    local src_module="${extract_dir}/${module_xz}"
    if [ ! -f "$src_module" ]; then
        src_module="$(find "$extract_dir" -path '*/virtio_blk.ko.xz' | head -1 || true)"
    fi

    if [ -z "$src_module" ] || [ ! -f "$src_module" ]; then
        echo "⚠️  [WARN] virtio_blk.ko.xz not found inside ${udeb_name}"
        return 0
    fi

    if ! command -v xz >/dev/null 2>&1; then
        echo "⚠️  [WARN] 'xz' not found; cannot decompress virtio_blk.ko.xz"
        return 0
    fi

    xz -dc "$src_module" > "$VIRTIO_BLK_MODULE"
    echo "✅ [SUCCESS] virtio_blk module ready: $VIRTIO_BLK_MODULE"
    echo "   Size: $(du -sh "$VIRTIO_BLK_MODULE" | cut -f1)"
}

extract_ext4_modules() {
    if [ ! -f "$KERNEL_IMAGE" ]; then
        return 0
    fi

    local kernel_line kver_full kver_deb udeb_name udeb_url
    kernel_line="$(strings "$KERNEL_IMAGE" | grep -m1 '^Linux version ' || true)"
    [ -n "$kernel_line" ] || return 0

    kver_full="$(sed -n 's/Linux version \([^ ]*\).*/\1/p' <<< "$kernel_line")"
    kver_deb="$(sed -n 's/.*# SMP Debian \([^ ]*\).*/\1/p' <<< "$kernel_line")"
    [ -n "$kver_full" ] && [ -n "$kver_deb" ] || return 0

    udeb_name="ext4-modules-${kver_full}-di_${kver_deb}_arm64.udeb"
    udeb_url="${DEBIAN_NIC_POOL}/${udeb_name}"

    echo "📦 [INFO] Fetching ext4 modules: ${udeb_name}"
    mkdir -p "$MODULE_CACHE"
    local udeb_path="${MODULE_CACHE}/${udeb_name}"

    if [ ! -f "$udeb_path" ]; then
        if ! download_file "$udeb_url" "$udeb_path"; then
            echo "⚠️  [WARN] Failed to download ${udeb_name}"
            return 0
        fi
    fi

    local extract_dir="${MODULE_CACHE}/ext4-modules"
    rm -rf "$extract_dir"
    mkdir -p "$extract_dir"
    ( cd "$extract_dir" && ar x "$udeb_path" && tar xf data.tar.* )

    local jbd2_xz ext4_xz
    jbd2_xz="$(find "$extract_dir" -path '*/jbd2/jbd2.ko.xz' | head -1 || true)"
    ext4_xz="$(find "$extract_dir" -path '*/ext4/ext4.ko.xz' | head -1 || true)"

    if [ -z "$jbd2_xz" ] || [ -z "$ext4_xz" ]; then
        echo "⚠️  [WARN] ext4/jbd2 modules not found in ${udeb_name}"
        return 0
    fi

    xz -dc "$jbd2_xz" > "$JBD2_MODULE"
    xz -dc "$ext4_xz" > "$EXT4_MODULE"
    echo "✅ [SUCCESS] ext4 modules ready: $JBD2_MODULE, $EXT4_MODULE"
}

echo "============================================="
echo "📥 Downloading Precompiled ARM64 Linux Kernel"
echo "============================================="

mkdir -p "$KERNEL_DIR"

if [ -f "$KERNEL_IMAGE" ]; then
    echo "ℹ️  [INFO] Kernel already present: $KERNEL_IMAGE"
else
    echo "Using $(command -v curl >/dev/null && echo curl || echo wget) to download kernel..."
    download_file "$DEBIAN_KERNEL_URL" "$KERNEL_IMAGE"
    echo "✅ [SUCCESS] Kernel downloaded: $KERNEL_IMAGE"
    echo "   Size: $(du -sh "$KERNEL_IMAGE" | cut -f1)"
fi

echo "============================================="
echo "📦 Extracting e1000 network driver module"
echo "============================================="
extract_e1000_module

echo "============================================="
echo "📦 Extracting virtio_blk block driver module"
echo "============================================="
extract_virtio_blk_module

echo "============================================="
echo "📦 Extracting ext4 filesystem modules"
echo "============================================="
extract_ext4_modules

echo "============================================="
echo "✅ Kernel setup complete!"
echo "   Kernel     : $KERNEL_IMAGE"
echo "   e1000      : $E1000_MODULE"
echo "   virtio_blk : $VIRTIO_BLK_MODULE"
echo "   ext4/jbd2  : $EXT4_MODULE"
echo "============================================="
echo "🚀 Next steps:"
echo "   ./scripts/make-rootfs.sh"
echo "   ./scripts/qemu-run.sh --headless"
echo "============================================="
