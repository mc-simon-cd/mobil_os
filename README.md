# Orion Core 📱

Welcome to **Orion Core**, a lightweight, modular, zero-dependency base mobile operating system engine built **100% in Rust**.

Orion Core provides the foundational OS layer: PID 1 process supervisor, Binder-style IPC directory, multi-layer software display compositor (`surfaceflinger`), input dispatcher, power manager, REST API gateway, and signed package manager (`opk`).

Higher-level **Mobile Shells** (Statusbar, Launcher, Settings, Control Center, or custom UI frameworks) sit cleanly **on top of Orion Core** by communicating through clean Unix Domain Socket IPC APIs.

---

## 🏛️ System Architecture

```
┌────────────────────────────────────────────────────────┐
│             CUSTOM MOBILE SHELL / APPLICATIONS         │
│     (launcher, settings, dialer, control center, etc.) │
└───────────────────────────┬────────────────────────────┘
                            │ Public Core IPC APIs
────────────────────────────┼────────────────────────────
                     ORION CORE ENGINE
────────────────────────────┼────────────────────────────
┌───────────────────────────▼────────────────────────────┐
│                    Mobile Shell UI                     │
│               (statusbar, notifications)               │
└───────────────────────────┬────────────────────────────┘
                            │ (libipc-rs)
┌───────────────────────────▼────────────────────────────┐
│                  Core System Services                  │
│   (surfaceflinger, inputflinger, powermanager, etc.)   │
└───────────────────────────┬────────────────────────────┘
                            │ (Unix Domain Sockets)
┌───────────────────────────▼────────────────────────────┐
│                servicemanager (Registry)               │
└───────────────────────────┬────────────────────────────┘
                            │ (Custom init.rc)
┌───────────────────────────▼────────────────────────────┐
│                  core/init (Rust PID 1)                │
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

---

## 📘 Specifications & Documentation

- 📄 **[ORION_CORE_SPECIFICATION.md](file:///home/can/Masaüstü/mobil_işletim_sistemi/ORION_CORE_SPECIFICATION.md)** — Orion Core System Engine Architecture
- 📄 **[ORION_SHELL_API_SPECIFICATION.md](file:///home/can/Masaüstü/mobil_işletim_sistemi/ORION_SHELL_API_SPECIFICATION.md)** — Shell & Surface IPC Protocol Guide
- 📄 **[RUST_MIGRATION_STATUS.md](file:///home/can/Masaüstü/mobil_işletim_sistemi/RUST_MIGRATION_STATUS.md)** — 100% Pure-Rust Component Migration Status
- 📄 **[KULLANIM_KILAVUZU.md](file:///home/can/Masaüstü/mobil_işletim_sistemi/KULLANIM_KILAVUZU.md)** — Türkçe Kullanım ve Geliştirici Kılavuzu

---

## 🛠️ Building & Running Orion Core

### 1. Compile Orion Core
Compiles all Rust workspace components statically for ARM64 (`aarch64-unknown-linux-gnu`):
```bash
./scripts/build.sh
```

### 2. Prepare Rootfs & Kernel
```bash
./scripts/download-kernel.sh   # Download ARM64 kernel + modules
./scripts/make-rootfs.sh       # Build cpio.gz initramfs and ext4 image
```

### 3. Run QEMU Emulation
```bash
./scripts/qemu-run.sh --headless
```

---

## 📄 License & Copyright

Copyright © 2026 mcsimon. All rights reserved. Licensed under the Apache License, Version 2.0.
