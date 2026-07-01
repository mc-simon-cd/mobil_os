# Orion OS - Developer & AI Assistant Guide (cloude.md)

Welcome! This guide outlines the system design principles, coding standards, and architectural conventions for developers and AI agents (such as Antigravity) building the Orion OS.

---

## 🛠️ Tech Stack & Coding Standards

1. **System & Core (`core/`, `services/`, `libs/`)**:
   - Written in **C99/C11** for low-level performance, fast boot time, and predictability.
   - **Zero-Dependency Policy**: Standard library only. Any utility or data structure (e.g., dynamic arrays, hash maps) should be implemented in `libs/libsystem` or `libs/libipc` if needed.
   - **Memory Management**: Every allocation (`malloc`, `calloc`) must have a corresponding, well-defined lifecycle and cleanup (`free`). No memory leaks.
   - **File Descriptors**: All open files, sockets, and device drivers must be safely closed under error handling and shutdown sequences.
   
2. **UI & Window Manager (`ui/`, `apps/`)**:
   - **Mevcut Durum (v1 Production):** Tamamen Wayland + wlroots yığınana geçilmiştir. `surfaceflinger` artık bir wlroots compositor'dır ve ekran çıkışını DRM/KMS (gerçek cihaz) veya Headless backend (QEMU) üzerinden yönetir.
   - **Geriye Uyumluluk (Fallback):** Wayland soketi (`WAYLAND_DISPLAY=wayland-0`) herhangi bir sebeple başlatılamazsa, sistem otomatik olarak eski PPM legacy fallback (`persist.graphics.backend=ppm`) moduna geçer. Yazılan tüm UI kodları bu çift yönlü mimariyi desteklemelidir.
   - **Bellek Yönetimi:** İstemciler (apps), `libgraphics` içindeki `canvas_init_external` apisi ile POSIX Shared Memory (`shm_open`, `mmap`) kullanarak tampon arabellekleri (wl_shm_pool) üzerinden `surfaceflinger` ile paylaşır. Doğrudan disk I/O yapılmamalıdır.
   - **Geliştirme Kuralı:** Custom widget'lar hala `libgraphics` primitiflerine (rect, rounded rect, gradient) dayanır. Ağır UI kütüphaneleri yasaktır. Yeni eklenecek pencereler `xdg_shell` veya `layer_shell` protokollerine uymalıdır.

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
├── deps/           # deps.yml — dependency & library update manifest
├── kernel/         # Linux kernel configs, modules, and patches
├── rootfs/         # Minimal root filesystem layout (etc, apps, locale)
├── core/           # Core PID 1 init process and init.rc
├── services/       # Base system daemons (servicemanager, power, input, apprunner)
├── ui/             # Compositor and System Shell
├── apps/           # Native system applications (settings, dialer, launcher)
├── libs/           # Shared libraries (libipc, libipc-rs, libgraphics, libi18n)
├── tools/          # Dev utilities (logcat, package manager, emulator configs)
├── tests/          # Unit and integration tests
├── docs/           # System documentation
└── scripts/        # build.sh, update-deps.sh, make-rootfs.sh, qemu-run.sh
```

---

## ✍️ Code Design & Safety Directives

- **Explicit Errors**: Never ignore function return values. Check every `read()`, `write()`, `socket()`, `ioctl()`, and custom API call.
- **Logging Rules**: Use our standard logging macros defined in `<system/logging.h>`:
  - `LOG_INFO(TAG, fmt, ...)`
  - `LOG_WARN(TAG, fmt, ...)`
  - `LOG_ERROR(TAG, fmt, ...)`
- **Thread Safety**: Core daemons must follow non-blocking asynchronous event loops (using `poll()` or `epoll()`) instead of heavily threaded models to keep system footprint low.

---

## 📦 Dependency Management

- **Single source of truth**: All host packages, Rust workspaces, kernel scripts, and build steps are declared in `deps/deps.yml`.
- **Automation**: Run `./scripts/update-deps.sh --all` to install packages, update Cargo lockfiles, download kernel/modules, and rebuild.
- **Init boot config**: Service startup order and arguments belong in `core/init/init.rc` — not hardcoded in shell scripts.
- **`apprunner`**: Always invoked with an app path (`args /system/apps/control_center` in init.rc). Never register it without arguments.
