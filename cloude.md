# Mobile OS - Developer & AI Assistant Guide (cloude.md)

Welcome! This guide outlines the system design principles, coding standards, and architectural conventions for developers and AI agents (such as Antigravity) building the Mobile OS.

---

## 🛠️ Tech Stack & Coding Standards

1. **System & Core (`core/`, `services/`, `libs/`)**:
   - Written in **C99/C11** for low-level performance, fast boot time, and predictability.
   - **Zero-Dependency Policy**: Standard library only. Any utility or data structure (e.g., dynamic arrays, hash maps) should be implemented in `libs/libsystem` or `libs/libipc` if needed.
   - **Memory Management**: Every allocation (`malloc`, `calloc`) must have a corresponding, well-defined lifecycle and cleanup (`free`). No memory leaks.
   - **File Descriptors**: All open files, sockets, and device drivers must be safely closed under error handling and shutdown sequences.

2. **UI & Window Manager (`ui/`, `apps/`)**:
   - Wayland compositor based on `wlroots` and Wayland protocols.
   - Core composition is written in C or **C++17/20** using modern OOP principles for surface management, rendering pipelines, and window hierarchies.
   - Avoid bloated UI toolkits. Custom widgets are built directly on our minimal widget library (`libs/libui`).

---

## 📡 IPC & Service Registry Conventions

All system services communicate using a unified Inter-Process Communication (IPC) framework (`libipc`) based on Unix Domain Sockets:

- **Service Registration**: On boot, every system service (e.g., `inputflinger`, `powermanager`) must connect to the `/system/bin/servicemanager` Unix Domain Socket and register itself with a unique service name (e.g., `mobile.power`, `mobile.input`).
- **Communication Protocol**: Data exchanged between clients and services must be serialized using our minimal binder-like packaging library `libs/libipc/parcel.h`.
- **Naming Convention**:
  - Services use namespace format: `mobile.<service_name>` (e.g., `mobile.surfaceflinger`).
  - Interface methods are defined as numeric command IDs (e.g., `CMD_POWER_SUSPEND = 1`).

---

## 📂 Repository Directory Layout

Maintain the exact folder layout as specified below. Do not create ad-hoc directories in the root.

```
mobile-os/
├── board/          # Hardware support (defconfigs, device trees, board.mk)
├── kernel/         # Linux kernel configs, modules, and patches
├── rootfs/         # Minimal root filesystem layout (etc, bin, lib)
├── core/           # Core PID 1 init process and linker
├── services/       # Base system daemons (servicemanager, power, input)
├── ui/             # Compositor and System Shell
├── apps/           # Native system applications (settings, dialer, launcher)
├── libs/           # Shared libraries (libipc, libui, libgraphics)
├── tools/          # Dev utilities (logcat, package manager, emulator configs)
├── tests/          # Unit and integration tests
├── docs/           # System documentation
└── scripts/        # Build, flash, and emulation scripts
```

---

## ✍️ Code Design & Safety Directives

- **Explicit Errors**: Never ignore function return values. Check every `read()`, `write()`, `socket()`, `ioctl()`, and custom API call.
- **Logging Rules**: Use our standard logging macros defined in `<system/logging.h>`:
  - `LOG_INFO(TAG, fmt, ...)`
  - `LOG_WARN(TAG, fmt, ...)`
  - `LOG_ERROR(TAG, fmt, ...)`
- **Thread Safety**: Core daemons must follow non-blocking asynchronous event loops (using `poll()` or `epoll()`) instead of heavily threaded models to keep system footprint low.
