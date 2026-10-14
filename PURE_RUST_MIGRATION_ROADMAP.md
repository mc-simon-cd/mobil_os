# Orion OS — Pure-Rust OS Stack Migration Roadmap

This document outlines the step-by-step roadmap for migrating Orion OS from a hybrid C/C++/Wayland system to a **100% Pure-Rust OS Stack**.

---

## 🎯 Primary Migration Goals

1. **Eliminate C Wayland Dependencies**: Remove all legacy C Wayland client wrappers (`wl_client.c`, `xdg-shell-protocol.c`) and host `pkg-config` Wayland probes from `statusbar` and `settings`.
2. **Port Remaining Native Apps to Rust**:
   - `apps/dialer` -> Pure Rust binary
   - `apps/launcher` -> Pure Rust binary
3. **Simplify Toolchain & Build Pipeline**: Eliminate `libwlroots-dev` and `wayland-protocols` build requirements. Achieve single-command static cross-compilation (`crt-static`) for ARM64 (`aarch64-unknown-linux-gnu`).

---

## 🗺️ Migration Phases

```
┌────────────────────────────────────────────────────────┐
│ Phase 1: Strip C Wayland Wrappers (statusbar, settings)│
└───────────────────────────┬────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────┐
│ Phase 2: Port `apps/dialer` to Pure Rust               │
└───────────────────────────┬────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────┐
│ Phase 3: Port `apps/launcher` to Pure Rust             │
└───────────────────────────┬────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────┐
│ Phase 4: Clean Toolchain & Dependencies (`deps.yml`)   │
└───────────────────────────┬────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────┐
│ Phase 5: Full ARM64 & QEMU System Verification         │
└────────────────────────────────────────────────────────┘
```

---

## Phase 1: Strip C Wayland Wrappers

**Target Components**: `ui/statusbar`, `apps/settings`

### Tasks:
- [ ] Remove `src/wl_client.c` from `ui/statusbar` and `apps/settings`.
- [ ] Simplify `build.rs` in `ui/statusbar` and `apps/settings` to remove `cc::Build` and `wayland-client` pkg-config probing.
- [ ] Remove `extern "C"` Wayland function calls (`wl_client_init`, `wl_client_commit`, etc.) from `src/main.rs`.
- [ ] Update `Makefile` to remove `wayland-scanner` header/source generation rules.

---

## Phase 2: Port `apps/dialer` to Pure Rust

**Target Component**: `apps/settings/../dialer` (`apps/dialer`)

### Functionality:
- Renders 1080x2200 phone keypad layout (digits 0–9, `*`, `#`, call button).
- Resolves `mobile.surfaceflinger` and `mobile.statusbar`.
- Sends `Calling: <number>` notifications to `statusbar` via Binder IPC.
- Renders keypad UI using `libgraphics`.

### Tasks:
- [ ] Create `apps/dialer/Cargo.toml` declaring dependencies (`libc`, `libipc-rs`, `libgraphics`, `libi18n`).
- [ ] Create `apps/dialer/src/main.rs`.
- [ ] Update `apps/dialer/Makefile` for Cargo cross-compilation.
- [ ] Add `apps/dialer` to `deps/deps.yml` under `rust.workspaces`.
- [ ] Remove old C implementation `apps/dialer/src/main.c`.

---

## Phase 3: Port `apps/launcher` to Pure Rust

**Target Component**: `apps/launcher`

### Functionality:
- Renders Orion OS mobile launcher home screen (grid icons for Settings, Dialer, Package Manager).
- Interacts with `mobile.surfaceflinger` to manage wallpaper & launcher surface layers.
- Interacts with `mobile.apprunner` to launch selected system applications.

### Tasks:
- [ ] Create `apps/launcher/Cargo.toml`.
- [ ] Create `apps/launcher/src/main.rs`.
- [ ] Update `apps/launcher/Makefile` for Cargo cross-compilation.
- [ ] Add `apps/launcher` to `deps/deps.yml` under `rust.workspaces`.
- [ ] Remove old C implementation `apps/launcher/src/main.c`.

---

## Phase 4: Clean Toolchain & Dependencies

**Target Files**: `deps/deps.yml`, `scripts/update-deps.sh`, `scripts/build.sh`

### Tasks:
- [ ] Remove `libwlroots-dev` and `wayland-protocols` from host apt/pacman dependency lists in `deps/deps.yml`.
- [ ] Verify that building all workspaces requires only standard `cargo` and `aarch64-linux-gnu-gcc`.

---

## Phase 5: Full QEMU System Verification

### Tasks:
- [ ] Execute `./tests/integration/test_native_apps_host.sh` on host.
- [ ] Execute `make clean && ./scripts/build.sh`.
- [ ] Execute `./scripts/make-rootfs.sh`.
- [ ] Run `./scripts/qemu-run.sh --headless` and verify that all daemons and UI apps boot without errors.
- [ ] Update [RUST_MIGRATION_STATUS.md](file:///home/can/Masaüstü/mobil_işletim_sistemi/RUST_MIGRATION_STATUS.md) to mark system 100% Rust migrated.
