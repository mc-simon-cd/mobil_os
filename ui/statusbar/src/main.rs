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
    canvas_clear, canvas_draw_circle, canvas_draw_line, canvas_draw_rect,
    canvas_draw_rounded_rect, canvas_draw_text, canvas_free, canvas_init,
    canvas_save_ppm, canvas_t,
};
use libipc_rs::{ipc_connect, ipc_send_transaction, Parcel};
use std::ffi::{CStr, CString};
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicBool, Ordering};

const SERVICEMANAGER_SOCKET: &str = "/tmp/servicemanager.sock";
const STATUSBAR_SOCKET: &str = "/tmp/statusbar.sock";

const CMD_REGISTER_SERVICE: i32 = 1;
const CMD_GET_SERVICE: i32 = 2;

const CMD_SHOW_NOTIFICATION: i32 = 1;

const THEME_CARD_BORDER: u32 = 0x33254FFF;
const THEME_TEXT_PRIMARY: u32 = 0xFFFFFFFF;
const THEME_TEXT_SECONDARY: u32 = 0x9F9BA9FF;
const THEME_ACCENT_GREEN: u32 = 0x10B981FF;
const THEME_ACCENT_BLUE: u32 = 0x3B82F6FF;
const THEME_ACCENT_AMBER: u32 = 0xF59E0BFF;

static G_RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn on_signal(_sig: libc::c_int) {
    G_RUNNING.store(false, Ordering::SeqCst);
}

fn get_translation(key: &str) -> String {
    let key_c = CString::new(key).unwrap();
    unsafe {
        let ptr = libi18n::i18n_get(key_c.as_ptr());
        if !ptr.is_null() {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        } else {
            key.to_string()
        }
    }
}

fn resolve_service_path(service_name: &str) -> String {
    let mut out_path = String::new();
    if let Ok(mut stream) = ipc_connect(SERVICEMANAGER_SOCKET) {
        let mut data = Parcel::new();
        data.write_string(service_name);

        if let Ok(mut reply) = ipc_send_transaction(&mut stream, CMD_GET_SERVICE, &data) {
            reply.reset_read();
            if let Ok(path) = reply.read_string() {
                out_path = path;
            }
        }
    }
    out_path
}

fn allocate_statusbar_surface(surface_flinger_path: &str, surface_id: &mut i32) {
    if surface_flinger_path.is_empty() {
        return;
    }

    if let Ok(mut stream) = ipc_connect(surface_flinger_path) {
        let mut data = Parcel::new();
        data.write_i32(1080); // Width
        data.write_i32(100);  // Height
        data.write_i32(1);    // Format (RGBA)

        if let Ok(mut reply) = ipc_send_transaction(&mut stream, 1, &data) {
            reply.reset_read();
            if let Ok(sid) = reply.read_i32() {
                if sid >= 0 {
                    *surface_id = sid;
                    println!(
                        "[INFO] [STATUSBAR] Allocated graphics surface ID {} on surfaceflinger.",
                        sid
                    );
                }
            }
        }
    }
}

fn trigger_composite_tick(surface_flinger_path: &str) {
    if surface_flinger_path.is_empty() {
        return;
    }

    if let Ok(mut stream) = ipc_connect(surface_flinger_path) {
        let data = Parcel::new();
        let _ = ipc_send_transaction(&mut stream, 2, &data);
    }
}

fn query_power_metrics(power_manager_path: &str, battery: &mut i32, mode: &mut String) {
    *battery = 100;
    *mode = "balanced".to_string();

    if power_manager_path.is_empty() {
        return;
    }

    // 1. Query Battery Level
    if let Ok(mut stream) = ipc_connect(power_manager_path) {
        let data = Parcel::new();
        if let Ok(mut reply) = ipc_send_transaction(&mut stream, 1, &data) {
            reply.reset_read();
            if let Ok(bat) = reply.read_i32() {
                *battery = bat;
            }
        }
    }

    // 2. Query Power Mode
    if let Ok(mut stream) = ipc_connect(power_manager_path) {
        let data = Parcel::new();
        if let Ok(mut reply) = ipc_send_transaction(&mut stream, 3, &data) {
            reply.reset_read();
            if let Ok(m) = reply.read_string() {
                *mode = m;
            }
        }
    }
}

fn draw_signal_bars(c: *mut canvas_t, x: i32, y: i32, level: i32) {
    let mut lvl = level;
    if lvl < 0 {
        lvl = 0;
    }
    if lvl > 4 {
        lvl = 4;
    }
    for i in 0..4 {
        let bar_h = 8 + i * 5;
        let col = if i < lvl {
            THEME_TEXT_PRIMARY
        } else {
            0xFFFFFF22
        };
        unsafe {
            canvas_draw_rect(c, x + i * 8, y + (20 - bar_h), 5, bar_h, col, true);
        }
    }
}

fn draw_wifi_glyph(c: *mut canvas_t, cx: i32, cy: i32) {
    unsafe {
        canvas_draw_circle(c, cx, cy + 8, 3, THEME_TEXT_PRIMARY, true);
        canvas_draw_circle(c, cx, cy + 2, 8, THEME_TEXT_SECONDARY, false);
        canvas_draw_circle(c, cx, cy - 2, 14, THEME_TEXT_SECONDARY, false);
    }
}

fn draw_battery_gauge(c: *mut canvas_t, x: i32, y: i32, level: i32) {
    let mut lvl = level;
    if lvl < 0 {
        lvl = 0;
    }
    if lvl > 100 {
        lvl = 100;
    }

    unsafe {
        canvas_draw_rounded_rect(c, x, y, 36, 18, 4, THEME_CARD_BORDER, false);
        canvas_draw_rect(c, x + 36, y + 5, 3, 8, THEME_CARD_BORDER, true);
    }

    let col = if lvl < 20 {
        0xEF4444FF
    } else if lvl < 50 {
        THEME_ACCENT_AMBER
    } else {
        THEME_ACCENT_GREEN
    };

    let fill_w = (lvl * 30) / 100;
    if fill_w > 0 {
        unsafe {
            canvas_draw_rounded_rect(c, x + 3, y + 3, fill_w, 12, 3, col, true);
        }
    }
}

fn draw_and_update(
    c: *mut canvas_t,
    surface_id: i32,
    surface_flinger_path: &str,
    power_manager_path: &str,
    notification: &str,
) {
    // 1. Fetch current time
    let mut time_str = "00:00".to_string();
    unsafe {
        let mut rawtime: libc::time_t = std::mem::zeroed();
        libc::time(&mut rawtime);
        let timeinfo = libc::localtime(&rawtime);
        if !timeinfo.is_null() {
            let mut buf = vec![0u8; 32];
            let fmt_c = CString::new("%H:%M").unwrap();
            let size = libc::strftime(
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.len(),
                fmt_c.as_ptr(),
                timeinfo,
            );
            if size > 0 {
                time_str = String::from_utf8_lossy(&buf[0..size]).into_owned();
            }
        }
    }

    // 2. Query power status
    let mut battery_level = 100;
    let mut power_mode = "balanced".to_string();
    query_power_metrics(power_manager_path, &mut battery_level, &mut power_mode);

    // 3. Clear canvas (Slate dark color 0x141221FF)
    unsafe {
        canvas_clear(c, 0x141221FF);
        canvas_draw_line(c, 0, 99, 1080, 99, 0x332E4AFF);
    }

    // 4. Draw system metrics labels
    let bat_lbl = get_translation("power.battery");
    let mode_lbl = get_translation("power.mode");

    let mode_key = match power_mode.as_str() {
        "performance" => "power.mode.performance",
        "powersave" => "power.mode.powersave",
        _ => "power.mode.balanced",
    };
    let mode_val_lbl = get_translation(mode_key);

    draw_battery_gauge(c, 20, 38, battery_level);

    let battery_text = format!("{} {}%", bat_lbl, battery_level);
    let battery_text_c = CString::new(battery_text).unwrap();
    unsafe {
        canvas_draw_text(c, 64, 44, battery_text_c.as_ptr(), THEME_ACCENT_GREEN);
    }

    let mode_text = format!("{}: {}", mode_lbl, mode_val_lbl);
    let mode_text_c = CString::new(mode_text).unwrap();
    unsafe {
        canvas_draw_text(c, 200, 44, mode_text_c.as_ptr(), THEME_ACCENT_BLUE);
    }

    let time_str_c = CString::new(time_str).unwrap();
    unsafe {
        canvas_draw_text(c, 500, 44, time_str_c.as_ptr(), THEME_TEXT_PRIMARY);
    }

    draw_wifi_glyph(c, 900, 50);
    draw_signal_bars(c, 930, 30, 4);

    if !notification.is_empty() {
        let notif_text = format!("! {}", notification);
        let notif_text_c = CString::new(notif_text).unwrap();
        unsafe {
            canvas_draw_rounded_rect(c, 680, 28, 380, 44, 8, 0xF59E0B33, true);
            canvas_draw_text(c, 700, 44, notif_text_c.as_ptr(), THEME_ACCENT_AMBER);
        }
    }

    // 5. Save PPM and Composite
    let _ = std::fs::create_dir_all("out");
    let mut out_path = "out/statusbar_display.ppm".to_string();
    if surface_id >= 0 {
        out_path = format!("out/surface_{}.ppm", surface_id);
    }

    let out_path_c = CString::new(out_path).unwrap();
    unsafe {
        canvas_save_ppm(c, out_path_c.as_ptr());
        let backup_c = CString::new("out/statusbar_display.ppm").unwrap();
        canvas_save_ppm(c, backup_c.as_ptr());
    }

    trigger_composite_tick(surface_flinger_path);
}

fn register_statusbar_service() {
    println!("[INFO] [STATUSBAR] Registering mobile.statusbar with servicemanager...");
    if let Ok(mut stream) = ipc_connect(SERVICEMANAGER_SOCKET) {
        let mut data = Parcel::new();
        data.write_string("mobile.statusbar");
        data.write_string(STATUSBAR_SOCKET);

        let _ = ipc_send_transaction(&mut stream, CMD_REGISTER_SERVICE, &data);
    }
}

fn main() {
    println!("\n=============================================");
    println!("🚥   Booting Mobile Shell Statusbar Daemon");
    println!("=============================================\n");

    // 1. Initialize Localization Engine
    let lang = std::env::var("LANG").unwrap_or_else(|_| "en".to_string());
    let mut lang_code = "en".to_string();
    if lang.len() >= 2 {
        lang_code = lang[0..2].to_string();
    }

    let path1 = CString::new("/system/usr/share/locale").unwrap();
    let path2 = CString::new("./rootfs/system/usr/share/locale").unwrap();
    let path3 = CString::new("../../rootfs/system/usr/share/locale").unwrap();
    let lang_c = CString::new(lang_code.clone()).unwrap();

    unsafe {
        if libi18n::i18n_init(path1.as_ptr(), lang_c.as_ptr()) != 0 {
            if libi18n::i18n_init(path2.as_ptr(), lang_c.as_ptr()) != 0 {
                libi18n::i18n_init(path3.as_ptr(), lang_c.as_ptr());
            }
        }
    }
    println!("[INFO] [STATUSBAR] Localization initialized: '{}'", lang_code);

    // 2. Initialize Canvas Buffer (1080 x 100)
    let mut canvas = canvas_t {
        pixels: std::ptr::null_mut(),
        width: 0,
        height: 0,
    };

    unsafe {
        if canvas_init(&mut canvas, 1080, 100) != 0 {
            std::process::exit(1);
        }
    }

    println!("[INFO] [STATUSBAR] Resolving system services...");
    let mut power_manager_path = resolve_service_path("mobile.powermanager");
    let mut surface_flinger_path = resolve_service_path("mobile.surfaceflinger");
    let mut surface_id = -1;
    allocate_statusbar_surface(&surface_flinger_path, &mut surface_id);

    // 3. Establish Unix Socket Listener
    let _ = std::fs::remove_file(STATUSBAR_SOCKET);
    let listener = match UnixListener::bind(STATUSBAR_SOCKET) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind statusbar socket: {}", e);
            std::process::exit(1);
        }
    };

    let server_fd = listener.as_raw_fd();
    unsafe {
        let socket_c = CString::new(STATUSBAR_SOCKET).unwrap();
        libc::chmod(socket_c.as_ptr(), 0o666);
    }

    register_statusbar_service();

    unsafe {
        libc::signal(libc::SIGINT, on_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, on_signal as *const () as libc::sighandler_t);
    }

    let mut current_notification = "System Booting...".to_string();
    println!(
        "[INFO] [STATUSBAR] Event listener listening at {}",
        STATUSBAR_SOCKET
    );

    draw_and_update(
        &mut canvas,
        surface_id,
        &surface_flinger_path,
        &power_manager_path,
        &current_notification,
    );

    while G_RUNNING.load(Ordering::SeqCst) {
        unsafe {
            let mut readfds: libc::fd_set = std::mem::zeroed();
            libc::FD_ZERO(&mut readfds);
            libc::FD_SET(server_fd, &mut readfds);

            let mut timeout = libc::timeval {
                tv_sec: 1,
                tv_usec: 0,
            };

            let activity = libc::select(
                server_fd + 1,
                &mut readfds,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut timeout,
            );

            if activity < 0 {
                let err = *libc::__errno_location();
                if err == libc::EINTR {
                    continue;
                }
                eprintln!("select error: {}", err);
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

                        if code == CMD_SHOW_NOTIFICATION {
                            let mut msg = String::new();
                            if let Ok(m) = data.read_string() {
                                msg = m;
                            }
                            println!("[INFO] [STATUSBAR] Received shell notification: {}", msg);
                            current_notification = msg;
                            reply.write_i32(0);
                        } else {
                            reply.write_i32(-1);
                        }

                        let mut reply_header_bytes = [0u8; 8];
                        let reply_code = 0i32;
                        let reply_size = reply.size() as u32;
                        reply_header_bytes[0..4].copy_from_slice(&reply_code.to_ne_bytes());
                        reply_header_bytes[4..8].copy_from_slice(&reply_size.to_ne_bytes());

                        if stream.write_all(&reply_header_bytes).is_ok() && reply_size > 0 {
                            let _ = stream.write_all(reply.data());
                        }
                    }
                }
            }
        }

        if power_manager_path.is_empty() {
            power_manager_path = resolve_service_path("mobile.powermanager");
        }
        if surface_flinger_path.is_empty() {
            surface_flinger_path = resolve_service_path("mobile.surfaceflinger");
            allocate_statusbar_surface(&surface_flinger_path, &mut surface_id);
        }

        draw_and_update(
            &mut canvas,
            surface_id,
            &surface_flinger_path,
            &power_manager_path,
            &current_notification,
        );
    }

    let _ = std::fs::remove_file(STATUSBAR_SOCKET);
    unsafe {
        canvas_free(&mut canvas);
        libi18n::i18n_free();
    }
}
