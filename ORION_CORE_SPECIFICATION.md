# Orion Core — System Engine & Architecture Specification

**Orion Core** is the foundational, zero-dependency, 100% Pure-Rust base mobile operating system engine. It provides the low-level runtime, process supervisor, Binder-style IPC directory, software display compositor, input dispatcher, and package manager.

Higher-level Mobile Shells (Launcher, Statusbar, Settings, Control Center, custom UI frameworks) sit **on top of Orion Core** by communicating through clean Unix Domain Socket IPC APIs.

---

## 🏛️ System Layering Model

```
┌─────────────────────────────────────────────────────────────┐
│              CUSTOM MOBILE SHELL / APPLICATIONS             │
│        (Orion Shell, Custom Shells, Native/Web Apps)        │
└──────────────────────────────┬──────────────────────────────┘
                               │ Public Core IPC APIs
───────────────────────────────┼───────────────────────────────
                       ORION CORE ENGINE
───────────────────────────────┼───────────────────────────────
┌──────────────────────────────▼──────────────────────────────┐
│                      Core System Daemons                    │
│   • servicemanager  (IPC Directory)                         │
│   • surfaceflinger  (Window Compositor & Layer Blitter)     │
│   • inputflinger    (Event Dispatcher)                      │
│   • powermanager    (Battery & Power Profiles)              │
│   • apigateway      (REST HTTP Gateway)                     │
│   • apprunner       (App Sandbox & Lifecycle Manager)       │
├─────────────────────────────────────────────────────────────┤
│                    Core Base Libraries                      │
│   • libipc-rs       (Parcel IPC Wire Format)                │
│   • libgraphics     (2D Software Rasterizer)                │
│   • libi18n         (Localization Engine)                   │
├─────────────────────────────────────────────────────────────┤
│                    Core Boot Engine                         │
│   • core/init       (Rust PID 1 Process Supervisor)         │
├─────────────────────────────────────────────────────────────┤
│                    Package Manager                          │
│   • opk             (Orion Package Kit)                     │
└──────────────────────────────┬──────────────────────────────┘
                               │ VFS / System Calls
┌──────────────────────────────▼──────────────────────────────┐
│                        Linux Kernel                         │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔌 Connecting a Custom Shell to Orion Core

Any shell or user interface application connects to Orion Core using 3 standard steps:

### Step 1: Resolve Core Services
Connect to `/tmp/servicemanager.sock` and query required core socket paths:
```rust
let sf_path = resolve_service("mobile.surfaceflinger");
let pow_path = resolve_service("mobile.powermanager");
```

### Step 2: Request Graphics Surface Layer
Allocate a surface layer from `mobile.surfaceflinger`:
```rust
// Request 1080x2400 RGBA Surface Layer
let mut data = Parcel::new();
data.write_i32(1080); // Width
data.write_i32(2400); // Height
data.write_i32(1);    // RGBA Format
let mut reply = ipc_send_transaction(&mut sf_stream, CMD_ALLOC_SURFACE, &data)?;
let surface_id = reply.read_i32()?;
```

### Step 3: Draw & Trigger Composite
Draw visual elements onto the surface buffer using `libgraphics` and trigger compositing:
```rust
// Save rendered frame
canvas_save_ppm(&mut canvas, format!("out/surface_{}.ppm", surface_id));

// Trigger compositor tick
let data = Parcel::new();
let _ = ipc_send_transaction(&mut sf_stream, CMD_COMPOSITE, &data);
```

---

## 🛠️ Orion Core Component Summary

| Component | Path | Language | Sockets | Function |
| :--- | :--- | :--- | :--- | :--- |
| **PID 1 Init** | `core/init` | Rust (`crt-static`) | — | Mounts VFS, loads drivers, parses `init.rc`, supervises services. |
| **Service Directory** | `services/servicemanager` | Rust | `/tmp/servicemanager.sock` | Service lookup and socket path resolution. |
| **Compositor** | `services/surfaceflinger` | Rust | `/tmp/surfaceflinger.sock` | Multi-layer window compositing & frame blending. |
| **Input Dispatcher** | `services/inputflinger` | Rust | `/tmp/inputflinger.sock` | Touch & key event routing. |
| **Power Manager** | `services/powermanager` | Rust | `/tmp/powermanager.sock` | Battery level & power profile manager. |
| **API Gateway** | `services/apigateway` | Rust | `/tmp/apigateway.sock` (Port 8080) | REST HTTP to IPC gateway. |
| **App Runner** | `services/apprunner` | Rust | `/tmp/apprunner.sock` | App sandbox & process lifecycle supervisor. |
| **Package Kit** | `orion_package_kit` | Rust | — | Signed package manager & verifier. |
