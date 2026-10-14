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
    canvas_draw_text, canvas_free, canvas_init, canvas_save_ppm, canvas_t,
};
use libipc_rs::{ipc_connect, ipc_send_transaction, Parcel};
use std::ffi::{CStr, CString};

const SERVICEMANAGER_SOCKET: &str = "/tmp/servicemanager.sock";

const CMD_GET_SERVICE: i32 = 2;
const CMD_ALLOC_SURFACE: i32 = 1;
const CMD_COMPOSITE: i32 = 2;
const CMD_SHOW_NOTIFICATION: i32 = 1;

const SCREEN_W: i32 = 1080;
const SCREEN_H: i32 = 2200;

fn resolve_service_path(service_name: &str) -> String {
    if let Ok(mut stream) = ipc_connect(SERVICEMANAGER_SOCKET) {
        let mut data = Parcel::new();
        data.write_string(service_name);
        if let Ok(mut reply) = ipc_send_transaction(&mut stream, CMD_GET_SERVICE, &data) {
            reply.reset_read();
            if let Ok(path) = reply.read_string() {
                return path;
            }
        }
    }
    String::new()
}

fn send_statusbar_notification(sb_path: &str, msg: &str) {
    if sb_path.is_empty() { return; }
    if let Ok(mut stream) = ipc_connect(sb_path) {
        let mut data = Parcel::new();
        data.write_string(msg);
        let _ = ipc_send_transaction(&mut stream, CMD_SHOW_NOTIFICATION, &data);
    }
}

fn allocate_surface(sf_path: &str) -> i32 {
    if sf_path.is_empty() { return -1; }
    if let Ok(mut stream) = ipc_connect(sf_path) {
        let mut data = Parcel::new();
        data.write_i32(SCREEN_W);
        data.write_i32(SCREEN_H);
        data.write_i32(1); // Format RGBA
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

fn draw_dialer_interface(c: *mut canvas_t, dial_number: &str) {
    unsafe {
        canvas_clear(c, 0x0C0E14FF); // Dark navy
    }

    // Top heading
    let title = get_translation("dialer.title");
    let title_c = CString::new(format!("Dialer - {}", title)).unwrap_or_else(|_| CString::new("Dialer").unwrap());
    unsafe {
        canvas_draw_text(c, 80, 100, CString::new("*").unwrap().as_ptr(), 0x10B981FF);
        canvas_draw_text(c, 130, 100, title_c.as_ptr(), 0xFFFFFFFF);
        canvas_draw_line(c, 50, 160, 1030, 160, 0x221E2CFF);
    }

    // Dial display box
    let num_c = CString::new(dial_number).unwrap();
    unsafe {
        canvas_draw_rect(c, 80, 220, 920, 180, 0x171B26FF, true);
        canvas_draw_rect(c, 80, 220, 920, 180, 0x2C3345FF, false);
        canvas_draw_text(c, 120, 320, num_c.as_ptr(), 0xFFFFFFFF);
    }

    // Keypad grid
    let keys = [
        "1", "2", "3",
        "4", "5", "6",
        "7", "8", "9",
        "*", "0", "#",
    ];

    let start_x = 180;
    let start_y = 500;
    let spacing_x = 280;
    let spacing_y = 280;

    for (i, key) in keys.iter().enumerate() {
        let row = (i / 3) as i32;
        let col = (i % 3) as i32;
        let cx = start_x + col * spacing_x;
        let cy = start_y + row * spacing_y;

        let key_c = CString::new(*key).unwrap();
        unsafe {
            canvas_draw_circle(c, cx, cy, 90, 0x1A202CFF, true);
            canvas_draw_circle(c, cx, cy, 90, 0x2D3748FF, false);
            canvas_draw_text(c, cx - 10, cy + 10, key_c.as_ptr(), 0xFFFFFFFF);
        }
    }

    // CALL button
    let call_c = CString::new("CALL").unwrap();
    unsafe {
        canvas_draw_rect(c, 180, 1650, 720, 120, 0x10B981FF, true);
        canvas_draw_text(c, 470, 1720, call_c.as_ptr(), 0xFFFFFFFF);
    }

    // Calling notification string
    let calling_str = format!("Calling {}...", dial_number);
    let calling_c = CString::new(calling_str).unwrap();
    unsafe {
        canvas_draw_text(c, 360, 1850, calling_c.as_ptr(), 0x10B981FF);
    }
}

fn main() {
    println!("[INFO] [DIALER] Starting Dialer Application...");

    let args: Vec<String> = std::env::args().collect();
    let mut dial_number = "555-0199".to_string();
    if args.len() > 1 && !args[1].starts_with("--") {
        dial_number = args[1].clone();
    }

    // 1. Resolve services
    let sf_path = resolve_service_path("mobile.surfaceflinger");
    let sb_path = resolve_service_path("mobile.statusbar");

    // 2. Allocate surface layer
    let surface_id = allocate_surface(&sf_path);
    if surface_id >= 0 {
        println!("[INFO] [DIALER] Allocated graphics surface ID {} on surfaceflinger.", surface_id);
    }

    println!("[INFO] [DIALER] Outgoing number configured: {}", dial_number);
    send_statusbar_notification(&sb_path, &format!("Calling: {}", dial_number));

    // 3. Initialize translation engine
    let lang = std::env::var("LANG").unwrap_or_else(|_| "en".to_string());
    let lang_code = if lang.len() >= 2 { lang[0..2].to_string() } else { "en".to_string() };

    let paths = [
        "/system/usr/share/locale",
        "./rootfs/system/usr/share/locale",
        "../../rootfs/system/usr/share/locale",
    ];
    for p in &paths {
        let path_c = CString::new(*p).unwrap();
        let lang_c = CString::new(lang_code.as_str()).unwrap();
        let ok = unsafe { libi18n::i18n_init(path_c.as_ptr(), lang_c.as_ptr()) };
        if ok == 0 { break; }
    }

    // 4. Initialize Canvas Buffer (1080 x 2200)
    let mut canvas = canvas_t { pixels: std::ptr::null_mut(), width: 0, height: 0 };
    unsafe {
        if canvas_init(&mut canvas, SCREEN_W, SCREEN_H) != 0 {
            std::process::exit(1);
        }
    }

    // 5. Draw UI Layout
    draw_dialer_interface(&mut canvas, &dial_number);

    // 6. Save image & trigger compositor
    let _ = std::fs::create_dir_all("out");
    let out_path = if surface_id >= 0 {
        format!("out/surface_{}.ppm", surface_id)
    } else {
        "out/dialer_display.ppm".to_string()
    };

    let out_path_c  = CString::new(out_path.clone()).unwrap();
    let back_path_c = CString::new("out/dialer_display.ppm").unwrap();
    unsafe {
        canvas_save_ppm(&mut canvas, out_path_c.as_ptr());
        canvas_save_ppm(&mut canvas, back_path_c.as_ptr());
    }
    trigger_composite(&sf_path);
    println!("[INFO] [DIALER] Frame rendering complete: {}", out_path);

    // 7. Cleanup
    unsafe {
        canvas_free(&mut canvas);
        libi18n::i18n_free();
    }
}
