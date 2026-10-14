# Orion OS — Project Progress Dashboard

This dashboard tracks the progress of the **Orion Core Engine** and the **Shell & Applications Layer**.

---

## 🏛️ Architectural Layering Boundary

```
┌─────────────────────────────────────────────────────────────┐
│                 SHELL & APPLICATIONS LAYER                  │
│       (Orion Shell, Statusbar, Settings, Dialer, Web UI)    │
└──────────────────────────────┬──────────────────────────────┘
                               │ Public Core IPC APIs
───────────────────────────────┼───────────────────────────────
                       ORION CORE ENGINE
───────────────────────────────┼───────────────────────────────
┌──────────────────────────────▼──────────────────────────────┐
│                    Core System Daemons                      │
│   • servicemanager  (IPC Service Directory)                 │
│   • surfaceflinger  (Multi-Layer Software Compositor)       │
│   • inputflinger    (Input Dispatcher)                      │
│   • powermanager    (Battery & Power State Manager)         │
│   • apigateway      (REST HTTP Gateway)                     │
│   • apprunner       (App Sandbox Supervisor)                │
├─────────────────────────────────────────────────────────────┤
│                    Core Base Libraries                      │
│   • libipc-rs       (Wire-Compatible Parcel & Binder IPC)   │
│   • libgraphics     (2D Software Rasterizer)                │
│   • libi18n         (Localization Engine)                   │
├─────────────────────────────────────────────────────────────┤
│                    Core Boot Engine                         │
│   • core/init       (Rust Static PID 1 Supervisor)          │
├─────────────────────────────────────────────────────────────┤
│                    Package Manager                          │
│   • opk             (Orion Package Kit Manager)             │
└─────────────────────────────────────────────────────────────┘
```

---

## 🟢 1. Orion Core Engine (Base OS Layer) — 100% Complete

All core daemons, boot supervisors, IPC protocols, and libraries are **100% Pure Rust**, static ARM64 cross-compiled (`crt-static`), and verified in QEMU.

| Module | Location | Language | Status | Core Description |
|:---|:---|:---:|:---:|:---|
| **Init Engine** | `core/init/` | Rust | 🟢 *Complete* | PID 1 process supervisor (`init.rc` parser, VFS mount, respawn) |
| **Service Directory**| `services/servicemanager` | Rust | 🟢 *Complete* | Central IPC service registry socket server |
| **IPC Framework**| `libs/libipc-rs/` | Rust | 🟢 *Complete* | Parcel binary serialization & socket transaction engine |
| **Graphics Engine** | `libs/libgraphics/` | Rust | 🟢 *Complete* | 2D software drawing engine & font rasterizer |
| **Localization Engine** | `libs/libi18n/` | Rust | 🟢 *Complete* | Multi-language translation engine |
| **Compositor Engine** | `services/surfaceflinger/` | Rust | 🟢 *Complete* | Multi-layer surface compositor & layer blitter |
| **Power Manager**| `services/powermanager/` | Rust | 🟢 *Complete* | Power state & battery management daemon |
| **Input Dispatcher**| `services/inputflinger/` | Rust | 🟢 *Complete* | Touch & key input listener & routing engine |
| **API Gateway**| `services/apigateway/` | Rust | 🟢 *Complete* | REST HTTP API to IPC converter |
| **App Sandbox Supervisor**| `services/apprunner/` | Rust | 🟢 *Complete* | Process lifecycle & namespace supervisor |
| **Package Kit**| `orion_package_kit/` | Rust | 🟢 *Complete* | Ed25519-signed application package manager |

---

## 📱 2. Shell & Applications Layer (Pluggable Layer)

These applications run **on top of Orion Core** as clients using core IPC sockets (`/tmp/*.sock`).

| Application | Location | Type | Status | Description |
|:---|:---|:---:|:---:|:---|
| **Statusbar Overlay** | `ui/statusbar/` | Shell UI | 🟢 *Complete* | Top status panel (clock, battery, notifications) |
| **Launcher Shell** | `apps/launcher/` | Shell UI | 🟢 *Complete* | Mobile home screen (app grid, dock, swipe lock) |
| **Settings App** | `apps/settings/` | Shell App | 🟢 *Complete* | Power mode & system settings UI |
| **Dialer App** | `apps/dialer/` | Shell App | 🟢 *Complete* | Keypad phone dialing UI |
| **Control Center** | `rootfs/system/apps/control_center` | Web UI | 🟢 *Complete* | Live web dashboard & remote device monitor |

---

## 📋 3. Gelecek Geliştirme Roadmap Listesi (Kullanıcı Tarafı Özellikleri)

Core motoru temiz tutularak, kabuk ve uygulama katmanında ileride yapılabilecek modüler eklemeler:

- [ ] **Shell Etkileşim Derinleştirmesi**: `dialer` uygulamasında canlı numara girme ve `settings` uygulamasında alt sekme geçişleri.
- [ ] **Canlı Tampon Çıktısı (Direct Framebuffer / DRM Driver)**: `surfaceflinger` çıktısını `/dev/fb0` cihazına doğrudan aktarma.
- [ ] **OPK GUI Yöneticisi**: `launcher` içerisinden `.opk` paketlerinin arayüzle yüklenip kaldırılması.
