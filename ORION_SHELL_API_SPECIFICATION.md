# Orion OS — Pure-Rust Shell & Surface API Specification

This document details the architectural specification for the Orion OS system shell, graphics compositing pipeline, and application API bindings written entirely in Rust. It replaces legacy C Wayland dependencies with a lightweight, IPC-driven window and surface management model.

---

## 1. System Architecture Overview

Orion OS uses a modular IPC architecture over Unix Domain Sockets (`/tmp/*.sock`). All system daemons, compositors, and shell applications communicate through Binder-style parcel transactions handled by the `libipc-rs` library.

```
┌─────────────────────────────────────────────────────────┐
│     User Applications (Launcher, Settings, Dialer)      │
└──────────────┬──────────────────────────┬───────────────┘
               │ Service Lookup           │ Surface & Draw Calls
               ▼                          ▼
┌─────────────────────────────┐ ┌─────────────────────────┐
│       servicemanager        │ │     surfaceflinger      │
│  (Central IPC Directory)    │ │   (Rust Compositor)     │
└──────────────┬──────────────┘ └───────────┬─────────────┘
               │ Socket Path                │ Composite Frame
               ▼                            ▼
┌─────────────────────────────┐ ┌─────────────────────────┐
│         statusbar           │ │      libgraphics        │
│   (System UI Overlay)       │ │ (2D Software Renderer)  │
└─────────────────────────────┘ └─────────────────────────┘
```

---

## 2. Core Services Directory

| Service Name | Socket Path | Description |
| :--- | :--- | :--- |
| **`servicemanager`** | `/tmp/servicemanager.sock` | Central registry for service lookups and socket path resolution. |
| **`mobile.surfaceflinger`** | `/tmp/surfaceflinger.sock` | Window composer daemon responsible for surface layer allocation and frame compositing. |
| **`mobile.powermanager`** | `/tmp/powermanager.sock` | System power manager daemon (battery status, power profiles). |
| **`mobile.statusbar`** | `/tmp/statusbar.sock` | System statusbar overlay daemon (clock, battery status, notification injection). |
| **`mobile.inputflinger`** | `/tmp/inputflinger.sock` | Input event dispatch engine (touch, gesture, and keyboard events). |

---

## 3. Surface & Graphics IPC Protocol

Applications interact with `mobile.surfaceflinger` using `Parcel` transactions over `libipc-rs`.

### 3.1. Allocating a Graphics Surface Layer (`CMD_ALLOC_SURFACE = 1`)

**Request Payload:**
```
i32: width   (e.g., 1080)
i32: height  (e.g., 2200)
i32: format  (1 = RGBA / XRGB8888)
```

**Reply Payload:**
```
i32: surface_id (>= 0 on success, < 0 on error)
```

Once allocated, the application owns a surface layer buffer saved to `out/surface_<surface_id>.ppm`.

### 3.2. Triggering Composition (`CMD_COMPOSITE = 2`)

**Request Payload:**
```
(empty parcel)
```

**Description:**
Instructs `surfaceflinger` to blend all active surface layers (Statusbar layer + Active app layer) into the master composited frame buffer `out/display_composited.ppm`.

---

## 4. Statusbar & Notification API Protocol

Applications send system notifications to `mobile.statusbar` via transaction code `CMD_SHOW_NOTIFICATION = 1`.

**Request Payload:**
```
String: message (e.g., "Mode Set: performance" or "Calling: +905559876543")
```

**Reply Payload:**
```
i32: status (0 on success)
```

---

## 5. Power Manager API Protocol

Applications query or update power settings via `mobile.powermanager`.

| Code | Name | Request Payload | Reply Payload | Description |
| :--- | :--- | :--- | :--- | :--- |
| `1` | `CMD_GET_BATTERY_LEVEL` | *(empty)* | `i32: battery_level` | Returns battery percentage (0–100). |
| `2` | `CMD_SET_POWER_MODE` | `String: mode` | `i32: status` (0 = success) | Sets power mode (`powersave`, `balanced`, `performance`). |
| `3` | `CMD_GET_POWER_MODE` | *(empty)* | `String: mode` | Returns active power mode string. |

---

## 6. How a Rust Application Connects to the Shell

Here is the complete workflow and Rust code pattern for building a native Orion OS application and connecting it to the shell and compositor:

### 6.1. Workflow Summary

1. **Service Resolution**: Connect to `/tmp/servicemanager.sock` and resolve `mobile.surfaceflinger`, `mobile.powermanager`, and `mobile.statusbar`.
2. **Surface Allocation**: Request a surface ID from `mobile.surfaceflinger`.
3. **Canvas Initialization**: Initialize a `canvas_t` pixel buffer using `libgraphics`.
4. **Drawing**: Draw visual elements using `canvas_draw_rect`, `canvas_draw_text`, `canvas_draw_line`, etc.
5. **Frame Submission**: Save canvas buffer to `out/surface_<surface_id>.ppm` and call `CMD_COMPOSITE` on `mobile.surfaceflinger`.

### 6.2. Rust Code Integration Example

```rust
use libipc_rs::{ipc_connect, ipc_send_transaction, Parcel};
use libgraphics::{canvas_init, canvas_clear, canvas_draw_text, canvas_save_ppm, canvas_free, canvas_t};
use std::ffi::CString;

const SERVICEMANAGER_SOCKET: &str = "/tmp/servicemanager.sock";

fn resolve_service(name: &str) -> String {
    if let Ok(mut stream) = ipc_connect(SERVICEMANAGER_SOCKET) {
        let mut data = Parcel::new();
        data.write_string(name);
        if let Ok(mut reply) = ipc_send_transaction(&mut stream, 2, &data) { // CMD_GET_SERVICE = 2
            reply.reset_read();
            if let Ok(path) = reply.read_string() {
                return path;
            }
        }
    }
    String::new()
}

fn main() {
    // 1. Resolve SurfaceFlinger
    let sf_path = resolve_service("mobile.surfaceflinger");
    
    // 2. Allocate Surface (1080 x 2200)
    let mut surface_id = -1;
    if let Ok(mut stream) = ipc_connect(&sf_path) {
        let mut data = Parcel::new();
        data.write_i32(1080);
        data.write_i32(2200);
        data.write_i32(1);
        if let Ok(mut reply) = ipc_send_transaction(&mut stream, 1, &data) { // CMD_ALLOC_SURFACE = 1
            reply.reset_read();
            if let Ok(sid) = reply.read_i32() {
                surface_id = sid;
            }
        }
    }

    // 3. Initialize Graphics Canvas
    let mut canvas = canvas_t { pixels: std::ptr::null_mut(), width: 0, height: 0 };
    unsafe { canvas_init(&mut canvas, 1080, 2200); }

    // 4. Render UI
    unsafe {
        canvas_clear(&mut canvas, 0x141221FF); // Slate Dark
        let text = CString::new("Hello Orion OS Shell!").unwrap();
        canvas_draw_text(&mut canvas, 100, 200, text.as_ptr(), 0xFFFFFFFF);
    }

    // 5. Save PPM Frame & Trigger Composition
    let out_file = format!("out/surface_{}.ppm", surface_id);
    let out_c = CString::new(out_file).unwrap();
    unsafe { canvas_save_ppm(&mut canvas, out_c.as_ptr()); }

    // Trigger Composite
    if let Ok(mut stream) = ipc_connect(&sf_path) {
        let data = Parcel::new();
        let _ = ipc_send_transaction(&mut stream, 2, &data); // CMD_COMPOSITE = 2
    }

    // Cleanup
    unsafe { canvas_free(&mut canvas); }
}
```

---

## 7. Next Steps for Full Pure-Rust Integration

To complete the elimination of legacy C Wayland dependencies:
- Remove legacy `wl_client.c`, `xdg-shell-protocol.c`, and Wayland `build.rs` probes from `ui/statusbar`, `apps/settings`, `apps/dialer`, and `apps/launcher`.
- Simplify all makefiles to build pure Cargo binaries.
