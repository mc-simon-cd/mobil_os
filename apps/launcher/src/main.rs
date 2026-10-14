/*
 * Copyright 2026 mcsimon
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use libgraphics::{
    canvas_clear, canvas_draw_circle,
    canvas_draw_rounded_rect, canvas_draw_text, canvas_free, canvas_init,
    canvas_save_ppm, canvas_t,
};
use libipc_rs::{ipc_connect, ipc_send_transaction, Parcel};
use std::ffi::CString;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicBool, Ordering};

const SERVICEMANAGER_SOCKET: &str = "/tmp/servicemanager.sock";
const INPUTFLINGER_SOCKET: &str = "/tmp/inputflinger.sock";
const LAUNCHER_SOCKET: &str = "/tmp/launcher.sock";

const CMD_GET_SERVICE: i32 = 2;
const CMD_ALLOC_SURFACE: i32 = 1;
const CMD_COMPOSITE: i32 = 2;
const CMD_REGISTER_LISTENER: i32 = 1;
const CMD_DISPATCH_EVENT: i32 = 1;

const EV_KEY: i32 = 1;
const EV_ABS: i32 = 3;
const ABS_X: i32 = 0;
const ABS_Y: i32 = 1;
const BTN_TOUCH: i32 = 330;

const CANVAS_W: i32 = 1080;
const CANVAS_H: i32 = 2400;

// Colors
const THEME_BG_DARK: u32     = 0x0A0614FF;
const THEME_CARD_BG: u32     = 0x1A1429FF;
const THEME_CARD_BORDER: u32 = 0x33254FFF;
const THEME_TEXT_PRIMARY: u32 = 0xFFFFFFFF;
const THEME_TEXT_SECONDARY: u32 = 0x9F9BA9FF;
const THEME_PURPLE: u32      = 0x8B5CFFFF;
const THEME_GREEN: u32       = 0x10B981FF;
const THEME_BLUE: u32        = 0x3B82F6FF;
const THEME_AMBER: u32       = 0xF59E0BFF;

static G_RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn on_signal(_sig: libc::c_int) {
    G_RUNNING.store(false, Ordering::SeqCst);
}

struct AppIcon {
    name: &'static str,
    package: &'static str,
    color: u32,
}

const APPS: [AppIcon; 4] = [
    AppIcon { name: "Settings",  package: "mobile.settings",  color: THEME_PURPLE },
    AppIcon { name: "Dialer",    package: "mobile.dialer",    color: THEME_GREEN },
    AppIcon { name: "Messaging", package: "mobile.messaging", color: THEME_BLUE },
    AppIcon { name: "Browser",   package: "mobile.browser",   color: THEME_AMBER },
];

const DOCK_ITEMS: [AppIcon; 3] = [
    AppIcon { name: "Dialer",    package: "mobile.dialer",    color: THEME_GREEN },
    AppIcon { name: "Messaging", package: "mobile.messaging", color: THEME_BLUE },
    AppIcon { name: "Browser",   package: "mobile.browser",   color: THEME_AMBER },
];

fn resolve_service_path(service_name: &str) -> String {
    for _ in 0..10 {
        if let Ok(mut stream) = ipc_connect(SERVICEMANAGER_SOCKET) {
            let mut data = Parcel::new();
            data.write_string(service_name);
            if let Ok(mut reply) = ipc_send_transaction(&mut stream, CMD_GET_SERVICE, &data) {
                reply.reset_read();
                if let Ok(path) = reply.read_string() {
                    if !path.is_empty() {
                        return path;
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    String::new()
}

fn allocate_surface(sf_path: &str) -> i32 {
    if sf_path.is_empty() { return -1; }
    if let Ok(mut stream) = ipc_connect(sf_path) {
        let mut data = Parcel::new();
        data.write_i32(CANVAS_W);
        data.write_i32(CANVAS_H);
        data.write_i32(1);
        if let Ok(mut reply) = ipc_send_transaction(&mut stream, CMD_ALLOC_SURFACE, &data) {
            reply.reset_read();
            if let Ok(sid) = reply.read_i32() {
                return sid;
            }
        }
    }
    -1
}

fn trigger_composite(sf_path: &str) {
    if sf_path.is_empty() { return; }
    if let Ok(mut stream) = ipc_connect(sf_path) {
        let data = Parcel::new();
        let _ = ipc_send_transaction(&mut stream, CMD_COMPOSITE, &data);
    }
}

fn save_frame(c: *mut canvas_t, surface_id: i32, sf_path: &str) {
    let _ = std::fs::create_dir_all("out");
    let out_path = if surface_id >= 0 {
        format!("out/surface_{}.ppm", surface_id)
    } else {
        "out/launcher_display.ppm".to_string()
    };
    let out_c  = CString::new(out_path).unwrap();
    let back_c = CString::new("out/launcher_display.ppm").unwrap();
    unsafe {
        canvas_save_ppm(c, out_c.as_ptr());
        canvas_save_ppm(c, back_c.as_ptr());
    }
    trigger_composite(sf_path);
}

fn render_home(c: *mut canvas_t) {
    unsafe {
        canvas_clear(c, THEME_BG_DARK);
        let title_c = CString::new("Orion OS").unwrap();
        let sub_c   = CString::new("Home").unwrap();
        canvas_draw_text(c, 80, 360, title_c.as_ptr(), THEME_TEXT_PRIMARY);
        canvas_draw_text(c, 80, 400, sub_c.as_ptr(), THEME_TEXT_SECONDARY);
    }

    // Render Grid
    let grid_pad_x = 60;
    let grid_start_y = 520;
    let icon_box = 120;
    let usable = CANVAS_W - grid_pad_x * 2;
    let cell_w = usable / 4;

    for (i, app) in APPS.iter().enumerate() {
        let col = (i % 4) as i32;
        let cx = grid_pad_x + col * cell_w + cell_w / 2;
        let cy = grid_start_y + icon_box / 2;
        let x = cx - icon_box / 2;
        let y = cy - icon_box / 2;

        let name_c = CString::new(app.name).unwrap();
        unsafe {
            canvas_draw_rounded_rect(c, x, y, icon_box, icon_box, 28, app.color, true);
            canvas_draw_text(c, cx - 30, cy + icon_box / 2 + 20, name_c.as_ptr(), THEME_TEXT_PRIMARY);
        }
    }

    // Render Dock
    let dock_margin_x = 48;
    let dock_h = 140;
    let dock_bottom_pad = 80;
    let dock_y = CANVAS_H - dock_bottom_pad - dock_h;
    let dock_w = CANVAS_W - dock_margin_x * 2;
    let slot_w = dock_w / 3;

    unsafe {
        canvas_draw_rounded_rect(c, dock_margin_x, dock_y, dock_w, dock_h, 24, THEME_CARD_BG, true);
        canvas_draw_rounded_rect(c, dock_margin_x, dock_y, dock_w, dock_h, 24, THEME_CARD_BORDER, false);
    }

    for (i, app) in DOCK_ITEMS.iter().enumerate() {
        let cx = dock_margin_x + slot_w * (i as i32) + slot_w / 2;
        let cy = dock_y + dock_h / 2;
        let name_c = CString::new(app.name).unwrap();

        unsafe {
            canvas_draw_circle(c, cx, cy, 44, app.color, true);
            canvas_draw_text(c, cx - 24, cy + 50, name_c.as_ptr(), THEME_TEXT_SECONDARY);
        }
    }
}

fn launch_package(package: &str) {
    let bin = match package {
        "mobile.settings" => "settings",
        "mobile.dialer"   => "dialer",
        _ => {
            println!("[WARN] [LAUNCHER] No binary mapped for package: {}", package);
            return;
        }
    };

    println!("[INFO] [LAUNCHER] Spawning system app: '{}'", bin);
    let path_sys = format!("/system/bin/{}", bin);
    let path_dev = format!("../../out/rootfs/system/bin/{}", bin);

    let _ = std::process::Command::new(&path_sys)
        .spawn()
        .or_else(|_| std::process::Command::new(&path_dev).spawn());
}

fn handle_tap(x: i32, y: i32) {
    // Grid hit test
    let grid_pad_x = 60;
    let grid_start_y = 520;
    let icon_box = 120;
    let usable = CANVAS_W - grid_pad_x * 2;
    let cell_w = usable / 4;

    for (i, app) in APPS.iter().enumerate() {
        let col = (i % 4) as i32;
        let cx = grid_pad_x + col * cell_w + cell_w / 2;
        let cy = grid_start_y + icon_box / 2;
        let half = icon_box / 2 + 16;
        if x >= cx - half && x <= cx + half && y >= cy - half && y <= cy + half + 24 {
            launch_package(app.package);
            return;
        }
    }

    // Dock hit test
    let dock_margin_x = 48;
    let dock_h = 140;
    let dock_bottom_pad = 80;
    let dock_y = CANVAS_H - dock_bottom_pad - dock_h;
    let dock_w = CANVAS_W - dock_margin_x * 2;
    let slot_w = dock_w / 3;

    if y >= dock_y && y <= dock_y + dock_h + 30 {
        for (i, app) in DOCK_ITEMS.iter().enumerate() {
            let cx = dock_margin_x + slot_w * (i as i32) + slot_w / 2;
            let cy = dock_y + dock_h / 2;
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= 64 * 64 {
                launch_package(app.package);
                return;
            }
        }
    }
}

fn register_input_listener() -> bool {
    let mut if_path = resolve_service_path("mobile.input");
    if if_path.is_empty() {
        if_path = INPUTFLINGER_SOCKET.to_string();
    }

    if let Ok(mut stream) = ipc_connect(&if_path) {
        let mut data = Parcel::new();
        data.write_string(LAUNCHER_SOCKET);
        if let Ok(mut reply) = ipc_send_transaction(&mut stream, CMD_REGISTER_LISTENER, &data) {
            reply.reset_read();
            if let Ok(status) = reply.read_i32() {
                return status == 0;
            }
        }
    }
    false
}

fn setup_listener_socket() -> Option<UnixListener> {
    let _ = std::fs::remove_file(LAUNCHER_SOCKET);
    match UnixListener::bind(LAUNCHER_SOCKET) {
        Ok(l) => {
            let socket_c = CString::new(LAUNCHER_SOCKET).unwrap();
            unsafe { libc::chmod(socket_c.as_ptr(), 0o666); }
            Some(l)
        }
        Err(e) => {
            eprintln!("Failed to bind launcher socket: {}", e);
            None
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut oneshot = false;
    for arg in &args {
        if arg == "--oneshot" {
            oneshot = true;
        }
    }

    println!("\n=============================================");
    println!("📱   Booting Mobile Shell Launcher Screen");
    println!("=============================================\n");

    // 1. Resolve SurfaceFlinger
    let sf_path = resolve_service_path("mobile.surfaceflinger");
    let surface_id = allocate_surface(&sf_path);
    if surface_id >= 0 {
        println!("[INFO] [LAUNCHER] Surface ID: {}", surface_id);
    }

    // 2. Initialize Canvas Buffer (1080 x 2400)
    let mut canvas = canvas_t { pixels: std::ptr::null_mut(), width: 0, height: 0 };
    unsafe {
        if canvas_init(&mut canvas, CANVAS_W, CANVAS_H) != 0 {
            std::process::exit(1);
        }
    }

    if oneshot {
        render_home(&mut canvas);
        save_frame(&mut canvas, surface_id, &sf_path);
        println!("[INFO] [LAUNCHER] Oneshot frame complete.");
        unsafe { canvas_free(&mut canvas); }
        return;
    }

    // Interactive Loop Mode
    unsafe {
        libc::signal(libc::SIGINT, on_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, on_signal as *const () as libc::sighandler_t);
    }

    let _ = register_input_listener();
    let listener = match setup_listener_socket() {
        Some(l) => l,
        None => {
            render_home(&mut canvas);
            save_frame(&mut canvas, surface_id, &sf_path);
            unsafe { canvas_free(&mut canvas); }
            return;
        }
    };

    let server_fd = listener.as_raw_fd();
    println!("[INFO] [LAUNCHER] Interactive mode — listening at {}", LAUNCHER_SOCKET);

    render_home(&mut canvas);
    save_frame(&mut canvas, surface_id, &sf_path);

    let mut touch_x = 0i32;
    let mut touch_y = 0i32;

    while G_RUNNING.load(Ordering::SeqCst) {
        unsafe {
            let mut readfds: libc::fd_set = std::mem::zeroed();
            libc::FD_ZERO(&mut readfds);
            libc::FD_SET(server_fd, &mut readfds);

            let mut timeout = libc::timeval { tv_sec: 1, tv_usec: 0 };
            let activity = libc::select(
                server_fd + 1,
                &mut readfds,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut timeout,
            );

            if activity < 0 {
                let err = *libc::__errno_location();
                if err == libc::EINTR { continue; }
                break;
            }

            if activity > 0 && libc::FD_ISSET(server_fd, &mut readfds) {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut header_bytes = [0u8; 8];
                    if stream.read_exact(&mut header_bytes).is_ok() {
                        let code = i32::from_ne_bytes(header_bytes[0..4].try_into().unwrap());
                        let data_size = u32::from_ne_bytes(header_bytes[4..8].try_into().unwrap());

                        let mut payload = vec![0u8; data_size as usize];
                        if data_size > 0 {
                            let _ = stream.read_exact(&mut payload);
                        }

                        let mut data = Parcel::from_vec(payload);
                        let mut reply = Parcel::new();

                        if code == CMD_DISPATCH_EVENT {
                            let ev_type = data.read_i32().unwrap_or(0);
                            let ev_code = data.read_i32().unwrap_or(0);
                            let ev_val  = data.read_i32().unwrap_or(0);

                            if ev_type == EV_ABS && ev_code == ABS_X { touch_x = ev_val; }
                            if ev_type == EV_ABS && ev_code == ABS_Y { touch_y = ev_val; }
                            if ev_type == EV_KEY && ev_code == BTN_TOUCH && ev_val == 0 {
                                handle_tap(touch_x, touch_y);
                                render_home(&mut canvas);
                                save_frame(&mut canvas, surface_id, &sf_path);
                            }

                            reply.write_i32(0);
                        } else {
                            reply.write_i32(-1);
                        }

                        let mut reply_header = [0u8; 8];
                        let r_code = 0i32;
                        let r_size = reply.size() as u32;
                        reply_header[0..4].copy_from_slice(&r_code.to_ne_bytes());
                        reply_header[4..8].copy_from_slice(&r_size.to_ne_bytes());
                        if stream.write_all(&reply_header).is_ok() && r_size > 0 {
                            let _ = stream.write_all(reply.data());
                        }
                    }
                }
            }
        }
    }

    let _ = std::fs::remove_file(LAUNCHER_SOCKET);
    unsafe { canvas_free(&mut canvas); }
    println!("[INFO] [LAUNCHER] Launcher exit complete.");
}
