# Orion OS - Installation, Build & Verification Guide

This guide describes how to set up the development environment, compile the Orion OS, run the multi-language applications, and verify the builds using integration test suites.

---

## 📋 Development Prerequisites

Before building Orion OS, install the necessary host packages. The recommended way is the automated pipeline driven by [`deps/deps.yml`](deps/deps.yml):

```bash
# Verify environment
./scripts/update-deps.sh --check

# Install missing packages (apt on Debian/Ubuntu, pacman on Arch)
./scripts/update-deps.sh --install

# Full pipeline: install + cargo update + kernel + build + initramfs
./scripts/update-deps.sh --all
```

Equivalent Makefile targets:

```bash
make check-deps
make update-deps
```

### What `deps/deps.yml` manages

| Section | Purpose |
|:---|:---|
| `host.apt` / `host.pacman` | OS package lists (auto-detected) |
| `required_binaries` | Tools that must exist on `PATH` |
| `rust.targets` | `rustup target add aarch64-unknown-linux-gnu` |
| `rust.workspaces` | Crates receiving `cargo update` |
| `kernel` | Triggers `scripts/download-kernel.sh` |
| `build.scripts` | Post-update `build.sh` + `make-rootfs.sh` |

### Manual install (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install build-essential rustc cargo make \
  gcc-aarch64-linux-gnu g++-aarch64-linux-gnu \
  qemu-system-arm curl wget xz-utils python3 nodejs -y
rustup target add aarch64-unknown-linux-gnu
```

### Manual install (Arch Linux)

```bash
sudo pacman -S --needed base-devel rust cargo make \
  aarch64-linux-gnu-gcc qemu-system-aarch64 \
  curl wget xz python nodejs
rustup target add aarch64-unknown-linux-gnu
```

---

## 🏗️ Building Orion OS

You can build Orion OS either for the local host architecture (used for fast testing and continuous integration) or cross-compile it for the target `aarch64` mobile board.

### 1. Fast Host Compilation
To build all daemons and libraries for the local machine:
```bash
make clean
make CC=gcc CXX=g++
```

### 2. Target Cross-Compilation (`aarch64`)
To build and package the complete binary tree:
```bash
make clean
CC=aarch64-linux-gnu-gcc CXX=aarch64-linux-gnu-g++ ./scripts/build.sh
```

This builds all C/C++/Rust libraries and daemons and prepares the rootfs hierarchy inside `out/rootfs/`.

### 3. Initramfs Boot Image (QEMU)
```bash
./scripts/download-kernel.sh   # out/kernel/Image + out/e1000.ko
./scripts/make-rootfs.sh       # out/initramfs.cpio.gz
./scripts/qemu-run.sh --headless
```

The initramfs includes static `/init`, all `/system/bin` services, `/system/apps`, locale assets, and the `e1000` network module.

---

## 🧪 Verification & Testing

Integration test suites under `tests/integration/` verify system features on the host.

### 1. API Gateway Integration Tests
```bash
./tests/integration/test_api_host.sh
```

### 2. App Runner & Multi-Language Runtime Tests
```bash
./tests/integration/test_apprunner_host.sh
```

### 3. UI Shell Statusbar Notification Tests
```bash
./tests/integration/test_statusbar_host.sh
```

### 4. Display Preview API (Milestone 15)
```bash
./tests/integration/test_display_preview.sh
```

### 5. Rootfs ext4 Disk Image (Milestone 16)
```bash
./tests/integration/test_rootfs_disk.sh
```

**Disk boot in QEMU** (initramfs yerine virtio-blk):
```bash
./scripts/make-rootfs.sh          # out/rootfs.ext4 + initramfs.cpio.gz
./scripts/qemu-run.sh --disk --headless
```

---


## 📱 Running Runtimes & Control Center Dashboard

`apprunner` is **not** a standalone daemon — it requires an app directory with `manifest.json`:

```bash
apprunner <app_directory_path>
```

At QEMU boot, `init.rc` starts Control Center automatically:

```
service apprunner /system/bin/apprunner
    args /system/apps/control_center
    respawn
```

### Host-side manual launch

Start backend daemons, then run apps via `apprunner`:

```bash
make CC=gcc CXX=g++
./scripts/build.sh
rm -f /tmp/*.sock
./out/rootfs/system/bin/servicemanager &
./out/rootfs/system/bin/powermanager &
./out/rootfs/system/bin/inputflinger &
./out/rootfs/system/bin/surfaceflinger &
./out/rootfs/system/bin/statusbar &
PORT=8085 ./out/rootfs/system/bin/apigateway &
```

* **Python Reporter:**
  ```bash
  API_PORT=8085 LANG=tr ./out/rootfs/system/bin/apprunner ./out/rootfs/system/apps/sys_reporter
  ```
* **JavaScript Monitor:**
  ```bash
  API_PORT=8085 ./out/rootfs/system/bin/apprunner ./out/rootfs/system/apps/sys_monitor
  ```
* **Control Center Dashboard:**
  ```bash
  APP_PORT=8090 ./out/rootfs/system/bin/apprunner ./out/rootfs/system/apps/control_center
  ```
  Open `http://localhost:8090` in your browser.

---

## 🛠️ Repository Directory Structure

```
├── apps/               # Built-in system applications (launcher, settings, dialer)
├── board/              # QEMU board configs & defconfigs
├── core/               # Early boot systems (init, mount, properties, init.rc)
├── deps/               # deps.yml — dependency & library update manifest
├── libs/               # Shared system libraries (libipc, libipc-rs, libgraphics, libi18n)
├── out/                # Build output (rootfs, kernel, initramfs, e1000.ko)
├── rootfs/             # Root filesystem templates, apps, locale assets
├── scripts/            # build.sh, make-rootfs.sh, update-deps.sh, qemu-run.sh
├── services/           # System daemons (servicemanager, powermanager, apprunner, …)
├── ui/                 # UI shell (statusbar)
└── tests/              # Unit and integration test suites
```
