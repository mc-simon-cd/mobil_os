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

# Mobile OS - Master Build Script

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${WORKSPACE_DIR}/out"
ROOTFS_DIR="${OUT_DIR}/rootfs"

echo "============================================="
echo "🛠️   Mobile OS Compilation Pipeline starting"
echo "============================================="

# 1. Run Dependency Verification
bash "${WORKSPACE_DIR}/scripts/check-deps.sh" || echo "⚠️ [WARN] Missing some cross-compilation packages. Proceeding with available compilers..."

# 2. Establish Outputs Workspace Directories
echo "📂 [INFO] Prepping output folder hierarchy..."
mkdir -p "${ROOTFS_DIR}/bin"
mkdir -p "${ROOTFS_DIR}/sbin"
mkdir -p "${ROOTFS_DIR}/etc"
mkdir -p "${ROOTFS_DIR}/lib"
mkdir -p "${ROOTFS_DIR}/usr/bin"
mkdir -p "${ROOTFS_DIR}/usr/lib"
mkdir -p "${ROOTFS_DIR}/proc"
mkdir -p "${ROOTFS_DIR}/sys"
mkdir -p "${ROOTFS_DIR}/dev"
mkdir -p "${ROOTFS_DIR}/system/bin"
mkdir -p "${ROOTFS_DIR}/system/lib"

# 3. Create Default configuration overlays
echo "📝 [INFO] Generating basic rootfs config files..."

# Generate fstab
cat << 'EOF' > "${ROOTFS_DIR}/etc/fstab"
# <file system> <mount point>   <type>      <options>   <dump>  <pass>
proc            /proc           proc        defaults    0       0
sysfs           /sys            sysfs       defaults    0       0
devtmpfs        /dev            devtmpfs    defaults    0       0
tmpfs           /tmp            tmpfs       defaults    0       0
EOF

# Generate inittab (PID 1 boot behavior)
cat << 'EOF' > "${ROOTFS_DIR}/etc/inittab"
::sysinit:/etc/init.d/rcS
::respawn:/system/bin/servicemanager
::respawn:/system/bin/powermanager
::respawn:/system/bin/inputflinger
::respawn:/system/bin/apigateway
::respawn:/system/bin/surfaceflinger
::respawn:/system/bin/statusbar
::respawn:/system/bin/shell
::ctrlaltdel:/sbin/reboot
::shutdown:/bin/umount -a -r
EOF

# Generate OS release identification
cat << 'EOF' > "${ROOTFS_DIR}/etc/os-release"
NAME="Mobile OS"
VERSION="1.0-alpha"
ID=mobileos
PRETTY_NAME="Mobile OS 1.0 Alpha (QEMU-ARM64)"
EOF

# 4. Copy locale assets
echo "📂 [INFO] Copying modular localization resource files..."
mkdir -p "${ROOTFS_DIR}/system/usr/share/locale"
cp -r "${WORKSPACE_DIR}/rootfs/system/usr/share/locale/"* "${ROOTFS_DIR}/system/usr/share/locale/"

# 5. Copy and package application assets
echo "📂 [INFO] Copying apps directory..."
mkdir -p "${ROOTFS_DIR}/system/apps"
cp -r "${WORKSPACE_DIR}/rootfs/system/apps/"* "${ROOTFS_DIR}/system/apps/"
# Copy translations inside control_center/locale for static web serving
mkdir -p "${ROOTFS_DIR}/system/apps/control_center/locale"
cp -r "${WORKSPACE_DIR}/rootfs/system/usr/share/locale/"* "${ROOTFS_DIR}/system/apps/control_center/locale/"

# 6. Trigger Makefiles if present
if [ -f "${WORKSPACE_DIR}/Makefile" ]; then
    echo "🏗️  [INFO] Invoking Makefile systems..."
    make -C "${WORKSPACE_DIR}"
fi

echo "============================================="
echo "🎉  [SUCCESS] Mobile OS Build Complete!"
echo "    Rootfs prepared: ${ROOTFS_DIR}"
echo "============================================="
