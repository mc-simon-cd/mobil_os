# Mobile OS - Installation, Build & Verification Guide

This guide describes how to set up the development environment, compile the Mobile OS, run the multi-language applications, and verify the builds using integration test suites.

---

## 📋 Development Prerequisites

Before building Mobile OS, install the necessary host packages:

### 1. Build Tools & Compilers
* **C/C++ Compilers**: `gcc` and `g++` (for local host builds)
* **Rust Toolchain**: `rustc` and `cargo` (minimum edition 2021)
* **Make Utility**: GNU `make`

On Ubuntu/Debian:
```bash
sudo apt update
sudo apt install build-essential rustc cargo make -y
```

### 2. Target Cross-Compilation Toolchain
To compile for the target `aarch64` (ARM64 QEMU emulator board):
```bash
sudo apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu -y
```
Also add the target architecture in Rust:
```bash
rustup target add aarch64-unknown-linux-gnu
```

### 3. Emulation & Runtimes
* **QEMU Emulator**: `qemu-system-aarch64` (to boot the system image)
* **Python**: `python3` (for running the Python `sys_reporter` app)
* **Node.js**: `node` (for running the JavaScript `sys_monitor` app)
* **Web Fetch Tools**: `curl` (for testing API endpoints)

---

## 🏗️ Building Mobile OS

You can build Mobile OS either for the local host architecture (used for fast testing and continuous integration) or cross-compile it for the target `aarch64` mobile board.

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
This builds all C/C++/Rust libraries and daemons, prepares the rootfs hierarchy inside `out/rootfs/`, and configures automatic daemon boot settings inside `out/rootfs/etc/inittab`.

---

## 🧪 Verification & Testing

Three integration test suites are available under `tests/integration/` to verify system features on the host.

### 1. API Gateway Integration Tests
Verifies REST endpoints, power manager states, and CORS headers:
```bash
./tests/integration/test_api_host.sh
```

### 2. App Runner & Multi-Language Runtime Tests
Verifies manifest-based execution of Python, JS (Node.js), and Web apps:
```bash
./tests/integration/test_apprunner_host.sh
```

### 3. UI Shell Statusbar Notification Tests
Verifies local time drawing, query to power daemon, and notification listener IPC:
```bash
./tests/integration/test_statusbar_host.sh
```

---

## 📱 Running Runtimes & Control Center Dashboard

To test the multi-language capabilities and Web Control Center dashboard locally on the host:

### 1. Start System Daemons in Background
Compile for host and run the backend nervous system:
```bash
# Compile host binaries
make CC=gcc CXX=g++
./scripts/build.sh

# Run daemons (clean up old sockets first)
rm -f /tmp/*.sock
./out/rootfs/system/bin/servicemanager &
./out/rootfs/system/bin/powermanager &
./out/rootfs/system/bin/inputflinger &
./out/rootfs/system/bin/surfaceflinger &
./out/rootfs/system/bin/statusbar &
PORT=8085 ./out/rootfs/system/bin/apigateway &
```

### 2. Run Applications via App Runner
Launch the modular showcase apps:
* **Python Reporter** (outputs localized battery stats):
  ```bash
  API_PORT=8085 LANG=tr ./out/rootfs/system/bin/apprunner ./out/rootfs/system/apps/sys_reporter
  ```
* **JavaScript Monitor** (polls status periodically):
  ```bash
  API_PORT=8085 ./out/rootfs/system/bin/apprunner ./out/rootfs/system/apps/sys_monitor
  ```
* **Control Center Dashboard** (runs web server to host dashboard):
  ```bash
  APP_PORT=8090 ./out/rootfs/system/bin/apprunner ./out/rootfs/system/apps/control_center
  ```
  Open `http://localhost:8090` in your host browser to view the premium glassmorphism Control Center. You can click on buttons to change power modes, inject key events, and read logs in real time.

---

## 🛠️ Repository Directory Structure

```
├── apps/               # Built-in system applications (launcher)
├── board/              # QEMU board configs & defconfigs
├── core/               # Early boot systems (init, mount, properties)
├── libs/               # Shared system libraries:
│   ├── libipc/         # C-based Parcel/Binder IPC socket library
│   ├── libipc-rs/      # Wire-compatible Rust IPC serialization library
│   ├── libgraphics/    # 2D drawing primitives & canvas engine
│   └── libi18n/        # Multi-language localization C library
├── out/                # Created during build (rootfs, targets)
├── rootfs/             # Root filesystem config templates & locale assets
├── scripts/            # Compilation pipelines and dev scripts
├── services/           # System daemons (servicemanager, powermanager, etc.)
├── ui/                 # UI Shell components (statusbar & notifications)
└── tests/              # Integration and unit test suites
```
