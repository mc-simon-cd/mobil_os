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

# Orion OS — Automated dependency & library update pipeline
# Reads deps/deps.yml and installs/updates host tools, Rust crates, kernel assets.

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPS_FILE="${WORKSPACE_DIR}/deps/deps.yml"
# shellcheck source=lib/deps-yaml.sh
source "${WORKSPACE_DIR}/scripts/lib/deps-yaml.sh"

DO_CHECK=false
DO_INSTALL=false
DO_CARGO=false
DO_KERNEL=false
DO_BUILD=false
USE_SUDO=true
APT_UPDATE=true

usage() {
    cat <<'EOF'
Usage: ./scripts/update-deps.sh [OPTIONS]

Automates host package install, Rust target/workspace updates, kernel download,
and optional rebuild using deps/deps.yml.

Options:
  --check         Verify required binaries/packages only (no changes)
  --install       Install missing host packages (apt/pacman) and Rust targets
  --cargo         Run cargo update in configured Rust workspaces
  --kernel        Download ARM64 kernel + e1000 module
  --build         Run build.sh and make-rootfs.sh after updates
  --all           Equivalent to --install --cargo --kernel --build
  --no-sudo       Skip apt install (check/cargo/kernel/build still run)
  --no-apt-update Skip apt-get update before install
  -h, --help      Show this help

Examples:
  ./scripts/update-deps.sh --check
  ./scripts/update-deps.sh --install
  ./scripts/update-deps.sh --all
  ./scripts/update-deps.sh --cargo --kernel
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --check)      DO_CHECK=true; shift ;;
        --install)    DO_INSTALL=true; shift ;;
        --cargo)      DO_CARGO=true; shift ;;
        --kernel)     DO_KERNEL=true; shift ;;
        --build)      DO_BUILD=true; shift ;;
        --all)
            DO_INSTALL=true
            DO_CARGO=true
            DO_KERNEL=true
            DO_BUILD=true
            shift
            ;;
        --no-sudo)    USE_SUDO=false; shift ;;
        --no-apt-update) APT_UPDATE=false; shift ;;
        -h|--help)    usage; exit 0 ;;
        *)
            echo "❌ Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

if ! $DO_CHECK && ! $DO_INSTALL && ! $DO_CARGO && ! $DO_KERNEL && ! $DO_BUILD; then
    DO_CHECK=true
    DO_INSTALL=true
    DO_CARGO=true
    DO_KERNEL=true
fi

if [[ ! -f "$DEPS_FILE" ]]; then
    echo "❌ [ERROR] Dependency manifest not found: $DEPS_FILE"
    exit 1
fi

apt_pkg_installed() {
    dpkg -s "$1" >/dev/null 2>&1
}

pacman_pkg_installed() {
    pacman -Qi "$1" >/dev/null 2>&1
}

host_pkg_manager() {
    local configured
    configured="$(deps_yaml_nested_value "$DEPS_FILE" "host" "manager")"
    detect_pkg_manager "${configured:-auto}"
}

step_check_host_packages() {
    local manager
    manager="$(host_pkg_manager)"
    if [[ "$manager" == "none" ]]; then
        echo "ℹ️  [CHECK] No supported package manager; using binary checks only."
        return 0
    fi

    echo "🔍 [CHECK] Host packages (${manager})..."
    local failed=0
    local pkg

    while IFS= read -r pkg; do
        [[ -z "$pkg" ]] && continue
        local installed=0
        if [[ "$manager" == "apt" ]]; then
            apt_pkg_installed "$pkg" && installed=1
        else
            pacman_pkg_installed "$pkg" && installed=1
        fi
        if [[ "$installed" -eq 1 ]]; then
            echo "  ✅ $pkg"
        else
            echo "  ⚠️  Not installed: $pkg"
            failed=1
        fi
    done < <(deps_yaml_deep_list "$DEPS_FILE" "host" "$manager" "packages")

    # Package names are advisory; missing packages are warnings if binaries exist.
    if [[ "$failed" -ne 0 ]]; then
        echo "ℹ️  [CHECK] Some ${manager} packages missing (binary check is authoritative)."
        return 0
    fi
    return 0
}

step_install_host_packages() {
    local manager
    manager="$(host_pkg_manager)"
    if [[ "$manager" == "none" ]]; then
        echo "⚠️  [WARN] No supported package manager detected."
        return 1
    fi

    local to_install=()
    local pkg
    while IFS= read -r pkg; do
        [[ -z "$pkg" ]] && continue
        local missing=1
        if [[ "$manager" == "apt" ]]; then
            apt_pkg_installed "$pkg" || missing=0
        else
            pacman_pkg_installed "$pkg" || missing=0
        fi
        if [[ "$missing" -eq 0 ]]; then
            to_install+=("$pkg")
        fi
    done < <(deps_yaml_deep_list "$DEPS_FILE" "host" "$manager" "packages")

    if [[ ${#to_install[@]} -eq 0 ]]; then
        echo "✅ [${manager^^}] All configured packages are already installed."
        return 0
    fi

    echo "📦 [${manager^^}] Installing: ${to_install[*]}"
    if ! $USE_SUDO; then
        echo "⚠️  [WARN] --no-sudo set; skipping package install."
        return 1
    fi

    if [[ "$manager" == "apt" ]]; then
        if $APT_UPDATE; then
            sudo apt-get update -y
        fi
        sudo apt-get install -y "${to_install[@]}"
    else
        if $APT_UPDATE; then
            sudo pacman -Sy --noconfirm
        fi
        sudo pacman -S --needed --noconfirm "${to_install[@]}"
    fi
}

step_check_binaries() {
    echo "🔍 [CHECK] Required binaries..."
    local failed=0
    local binary

    while IFS= read -r binary; do
        [[ -z "$binary" ]] && continue
        if command -v "$binary" >/dev/null 2>&1; then
            echo "  ✅ $binary -> $(command -v "$binary")"
        else
            echo "  ❌ Missing: $binary"
            failed=1
        fi
    done < <(deps_yaml_list "$DEPS_FILE" "required_binaries")

    echo "🔍 [CHECK] Optional binaries..."
    while IFS= read -r binary; do
        [[ -z "$binary" ]] && continue
        if command -v "$binary" >/dev/null 2>&1; then
            echo "  ✅ $binary"
        else
            echo "  ⚠️  Optional missing: $binary"
        fi
    done < <(deps_yaml_list "$DEPS_FILE" "optional_binaries")

    return "$failed"
}

step_install_rust_targets() {
    if ! command -v rustup >/dev/null 2>&1; then
        echo "⚠️  [RUST] rustup not found; skip target install."
        echo "         Install rustup or ensure aarch64 target is available."
        return 0
    fi

    local target
    while IFS= read -r target; do
        [[ -z "$target" ]] && continue
        echo "🦀 [RUST] Adding target: $target"
        rustup target add "$target"
    done < <(deps_yaml_nested_list "$DEPS_FILE" "rust" "targets")
}

step_update_cargo_workspaces() {
    if ! command -v cargo >/dev/null 2>&1; then
        echo "❌ [CARGO] cargo not found on PATH."
        return 1
    fi

    local workspace
    while IFS= read -r workspace; do
        [[ -z "$workspace" ]] && continue
        local crate_dir="${WORKSPACE_DIR}/${workspace}"
        if [[ ! -f "${crate_dir}/Cargo.toml" ]]; then
            echo "⚠️  [CARGO] Skipping missing workspace: ${workspace}"
            continue
        fi
        echo "🦀 [CARGO] Updating ${workspace}..."
        ( cd "$crate_dir" && cargo update )
        echo "  ✅ ${workspace}"
    done < <(deps_yaml_nested_list "$DEPS_FILE" "rust" "workspaces")
}

step_download_kernel() {
    local enabled script
    enabled="$(deps_yaml_nested_value "$DEPS_FILE" "kernel" "enabled")"
    script="$(deps_yaml_nested_value "$DEPS_FILE" "kernel" "script")"

    if ! deps_yaml_bool "$enabled"; then
        echo "ℹ️  [KERNEL] Download disabled in deps.yml"
        return 0
    fi

    script="${script:-scripts/download-kernel.sh}"
    local script_path="${WORKSPACE_DIR}/${script}"
    if [[ ! -f "$script_path" ]]; then
        echo "❌ [KERNEL] Script not found: $script_path"
        return 1
    fi

    echo "📥 [KERNEL] Running ${script}..."
    bash "$script_path"
}

step_build_project() {
    local build_script rootfs_script cc cxx
    build_script="$(deps_yaml_deep_value "$DEPS_FILE" "build" "scripts" "build")"
    rootfs_script="$(deps_yaml_deep_value "$DEPS_FILE" "build" "scripts" "rootfs")"
    cc="$(deps_yaml_nested_value "$DEPS_FILE" "build" "cc")"
    cxx="$(deps_yaml_nested_value "$DEPS_FILE" "build" "cxx")"

    build_script="${build_script:-scripts/build.sh}"
    rootfs_script="${rootfs_script:-scripts/make-rootfs.sh}"

    local build_path="${WORKSPACE_DIR}/${build_script}"
    local rootfs_path="${WORKSPACE_DIR}/${rootfs_script}"

    echo "🏗️  [BUILD] Running ${build_script}..."
    CC="${cc:-aarch64-linux-gnu-gcc}" CXX="${cxx:-aarch64-linux-gnu-g++}" bash "$build_path"

    if [[ -x "$rootfs_path" || -f "$rootfs_path" ]]; then
        echo "📦 [BUILD] Running ${rootfs_script}..."
        bash "$rootfs_path"
    fi
}

echo "============================================="
echo "🔄  Orion OS — Dependency Update Pipeline"
echo "============================================="
echo "   Manifest: $DEPS_FILE"
echo "============================================="

CHECK_FAILED=0
if $DO_CHECK; then
    step_check_binaries || CHECK_FAILED=1
    step_check_host_packages || true
    if [[ "$CHECK_FAILED" -ne 0 ]]; then
        echo "⚠️  [CHECK] Some dependencies are missing."
    else
        echo "✅ [CHECK] All required dependencies are present."
    fi
fi

if $DO_INSTALL; then
    step_install_host_packages || true
    step_install_rust_targets || true
fi

if $DO_CARGO; then
    step_update_cargo_workspaces
fi

if $DO_KERNEL; then
    step_download_kernel
fi

if $DO_BUILD; then
    step_build_project
fi

echo "============================================="
echo "✅ Dependency update pipeline finished."
echo "============================================="

if $DO_CHECK && [[ "$CHECK_FAILED" -ne 0 ]]; then
    echo "💡 Run: ./scripts/update-deps.sh --install"
    exit 1
fi
