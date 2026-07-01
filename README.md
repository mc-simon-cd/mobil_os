# Orion OS 📱

Welcome to **Orion OS**, a lightweight, modular, and completely independent open-source mobile operating system. Engineered from scratch, it layers custom user-space utilities, a fast Wayland compositor, and custom system applications over a minimal, performance-tuned Linux kernel.

Our mission is to create a fully controllable mobile experience built around modularity, memory efficiency, and modern Unix design principles.

---

## 🏛️ System Architecture

Orion OS separates system layers clean and strictly:

```
┌────────────────────────────────────────────────────────┐
│                   System Applications                  │
│     (launcher, settings, dialer, messaging, browser)   │
└───────────────────────────┬────────────────────────────┘
                            │ (libui / libgraphics)
┌───────────────────────────▼────────────────────────────┐
│                    Mobile Shell UI                     │
│        (statusbar, quicksettings, notifications)       │
└───────────────────────────┬────────────────────────────┘
                            │ (libipc)
┌───────────────────────────▼────────────────────────────┐
│                    System Services                     │
│   (surfaceflinger, inputflinger, powermanager, etc.)   │
└───────────────────────────┬────────────────────────────┘
                            │ (Unix Domain Sockets)
┌───────────────────────────▼────────────────────────────┐
│                servicemanager (Registry)               │
└───────────────────────────┬────────────────────────────┘
                            │ (Custom init.rc)
┌───────────────────────────▼────────────────────────────┐
│                    core/init (PID 1)                   │
└───────────────────────────┬────────────────────────────┘
                            │ (VFS Mounts)
┌───────────────────────────▼────────────────────────────┐
│                      Linux Kernel                      │
└───────────────────────────┬────────────────────────────┘
                            │ (ARM64 / QEMU / PinePhone)
┌───────────────────────────▼────────────────────────────┐
│                        Hardware                        │
└────────────────────────────────────────────────────────┘
```

- **Boot / core/init (PID 1)**: A zero-dependency boot daemon written in C. It mounts virtual filesystems, loads the `e1000` network module, parses `init.rc` (with `class core/main` service ordering and `args` support), and supervises system daemons with automatic respawn.
- **servicemanager**: The centralized registry for Inter-Process Communication (IPC). Every service connects here to register or resolve message endpoints.
- **surfaceflinger**: Software display compositor (PPM layer stack). Client apps write `out/surface_<id>.ppm`; the daemon composites them into `out/display_composited.ppm`. Wayland/wlroots is a planned future migration, not used in the current tree.
- **libipc**: Our clean message serialization and socket-based transport framework, facilitating secure, real-time message passing between apps and services.

---

## 📂 Detailed Folder Structure

The project code is organized logically:

* **`board/`**: Hardware definitions, default device configurations (`defconfig`), device trees, and hardware-specific boot parameters (e.g. Raspberry Pi 4, PinePhone, QEMU ARM64).
* **`kernel/`**: Custom kernel configs, external drivers/modules, and critical patches (e.g., Binder features, memory optimization, power-suspend).
* **`rootfs/`**: Skeleton layout of the virtual system, including mounting rules (`fstab`), system groups/passwords, and config files.
* **`core/`**: Critical system runtime programs including our custom C `init` daemon and linker.
* **`services/`**: Standard background system daemons (input dispatch, power profiles, IPC registration).
* **`ui/`**: Core window management compositor protocols and the visual system shell (statusbar, notifications, dock).
* **`apps/`**: System applications that come pre-bundled with the OS (Dialer, Messaging, File Manager).
* **`deps/`**: Central dependency manifest (`deps.yml`) consumed by automated update scripts.
* **`paket_yönetimi/`**: `opk` — Ed25519-signed application package manager (install, verify, sandbox launch).
* **`libs/`**: Internal libraries providing UI components (`libui`), serialization (`libipc`), and drawing tools (`libgraphics`).
* **`tools/`**: Host developer scripts, log listeners (`logcat`), packaging mechanisms, and emulator launcher.
* **`tests/`**: Unit test suits and boot integration checklists.
* **`scripts/`**: Automation tools for compilation, dependency updates, kernel download, and QEMU launch.

---

## 🛠️ Building & Emulating

You can easily build and test the operating system inside our virtual ARM64 sandbox. For a comprehensive, step-by-step setup guide with verification testing instructions, please refer to the detailed [install.md](install.md) installation guide.

### Prerequisites (Automated)

Install and verify all host dependencies from the central manifest:

```bash
./scripts/update-deps.sh --check    # verify only
./scripts/update-deps.sh --install  # install missing apt/pacman packages
./scripts/update-deps.sh --all      # install + cargo update + kernel + build
```

Package lists live in [`deps/deps.yml`](deps/deps.yml). Manual install (Debian/Ubuntu):

```bash
sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu qemu-system-arm make curl xz-utils
```

### 1. Compile Everything
Compiles the core libs, services, shell, apps, and generates the bootable rootfs:
```bash
./scripts/build.sh
```

### 2. Prepare Initramfs & Kernel
```bash
./scripts/download-kernel.sh   # ARM64 kernel + e1000.ko module
./scripts/make-rootfs.sh       # cpio.gz initramfs
```

### 3. Run the Emulator
Launches QEMU running our custom kernel and user-space environment:
```bash
./scripts/qemu-run.sh --headless
```

---

## 🧪 Testing & Verification Suites

We have built rigorous unit and end-to-end integration test suites under `tests/` to guarantee communication and process stability before flashing:

### 1. IPC Serialization Unit Tests
Validates dynamic parcel buffer expansions and binary data sequence integrity:
```bash
cd tests/unit
gcc -Wall -Wextra -std=c99 -I../../libs/libipc/include -O2 test_ipc.c ../../out/rootfs/system/lib/libipc.a -o test_ipc
./test_ipc
```

### 2. Service Manager Integration Tests
Forks `servicemanager` in the background and issues transaction registrations/queries over sockets:
```bash
cd tests/integration
gcc -Wall -Wextra -std=c99 -I../../libs/libipc/include -O2 test_registry.c ../../out/rootfs/system/lib/libipc.a -o test_registry
./test_registry
```

### 3. Graphics Compositor Integration Tests
Verifies C++ graphics surface lists, allocations, and layer composition loops:
```bash
cd tests/integration
gcc -Wall -Wextra -std=c99 -I../../libs/libipc/include -O2 test_graphics.c ../../out/rootfs/system/lib/libipc.a -o test_graphics
./test_graphics
```

### 4. Complete OS Bootstrap Integration Tests
Orchestrates the entire boot loop sequence (spawning `servicemanager` & `surfaceflinger` in the background, launching the visual C client `launcher` desktop grid, and cleanly tearing down background jobs):
```bash
cd tests/integration
gcc -Wall -Wextra -std=c99 -O2 test_launcher.c -o test_launcher
./test_launcher
```

---

## 🗺️ Roadmap & Contributing

If you wish to contribute to Orion OS:
1. Refer to [cloude.md](cloude.md) for strict memory layout and coding guidelines.
2. Check [progress.md](progress.md) to inspect active sprint goals and unimplemented modules.
3. For the Wayland compositor roadmap, see [docs/wayland-migration.md](docs/wayland-migration.md).
4. Open a Pull Request conforming to standards defined in `CONTRIBUTING.md`.
## 📄 License & Copyright

Copyright © 2026 mcsimon. All rights reserved.

This project belongs to **mcsimon** and is licensed under the Apache License, Version 2.0. For the complete licensing terms, see the [LICENSE](LICENSE) file.

