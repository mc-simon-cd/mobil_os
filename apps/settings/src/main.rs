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
    canvas_clear, canvas_draw_line, canvas_draw_rect, canvas_draw_text,
    canvas_free, canvas_init, canvas_save_ppm, canvas_t,
};
use libipc_rs::{ipc_connect, ipc_send_transaction, Parcel};
use std::ffi::{CStr, CString};

const SERVICEMANAGER_SOCKET: &str = "/tmp/servicemanager.sock";

const CMD_GET_SERVICE: i32 = 2;

// Power Manager commands
const CMD_GET_BATTERY_LEVEL: i32 = 1;
const CMD_SET_POWER_MODE: i32 = 2;
const CMD_GET_POWER_MODE: i32 = 3;

// SurfaceFlinger commands
const CMD_ALLOC_SURFACE: i32 = 1;
const CMD_COMPOSITE: i32 = 2;

// Statusbar commands
const CMD_SHOW_NOTIFICATION: i32 = 1;

// Theme colors
const THEME_BG: u32          = 0x0E0B16FF;
const THEME_CARD_BG: u32     = 0x1A1429FF;
const THEME_CARD_BORDER: u32 = 0x33254FFF;
const THEME_CARD_DARK: u32   = 0x221A35FF;
const THEME_WHITE: u32       = 0xFFFFFFFF;
const THEME_MUTED: u32       = 0x9F9BA9FF;
const THEME_PURPLE: u32      = 0x8B5CFFFF;
const THEME_GREEN: u32       = 0x10B981FF;
const THEME_BLUE: u32        = 0x3B82F6FF;
const THEME_AMBER: u32       = 0xF59E0BFF;

// Screen resolution
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

fn get_battery_level(pm_path: &str) -> i32 {
    if pm_path.is_empty() { return 100; }
    if let Ok(mut stream) = ipc_connect(pm_path) {
        let data = Parcel::new();
        if let Ok(mut reply) = ipc_send_transaction(&mut stream, CMD_GET_BATTERY_LEVEL, &data) {
            reply.reset_read();
            if let Ok(v) = reply.read_i32() { return v; }
        }
    }
    100
}

fn get_power_mode(pm_path: &str) -> String {
    if pm_path.is_empty() { return "balanced".to_string(); }
    if let Ok(mut stream) = ipc_connect(pm_path) {
        let data = Parcel::new();
        if let Ok(mut reply) = ipc_send_transaction(&mut stream, CMD_GET_POWER_MODE, &data) {
            reply.reset_read();
            if let Ok(m) = reply.read_string() { return m; }
        }
    }
    "balanced".to_string()
}

fn set_power_mode(pm_path: &str, mode: &str) -> bool {
    if pm_path.is_empty() { return false; }
    if let Ok(mut stream) = ipc_connect(pm_path) {
        let mut data = Parcel::new();
        data.write_string(mode);
        if let Ok(mut reply) = ipc_send_transaction(&mut stream, CMD_SET_POWER_MODE, &data) {
            reply.reset_read();
            if let Ok(status) = reply.read_i32() {
                return status == 0;
            }
        }
    }
    false
}

fn send_notification(sb_path: &str, msg: &str) {
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

fn draw_settings_layout(
    c: *mut canvas_t,
    battery_level: i32,
    current_mode: &str,
    lang_code: &str,
) {
    // Background
    unsafe { canvas_clear(c, THEME_BG); }

    // --- Header ---
    let title = get_translation("settings.title");
    let title_c = CString::new(format!("Settings - {}", title)).unwrap_or_else(|_| CString::new("Settings").unwrap());
    unsafe {
        canvas_draw_text(c, 80,  100, CString::new("*").unwrap().as_ptr(), THEME_PURPLE);
        canvas_draw_text(c, 130, 100, title_c.as_ptr(), THEME_WHITE);
        canvas_draw_line(c, 50, 160, 1030, 160, THEME_CARD_DARK);
    }

    // --- Card 1: Power & Battery Info ---
    unsafe {
        canvas_draw_rect(c, 50, 200, 980, 500, THEME_CARD_BG, true);
        canvas_draw_rect(c, 50, 200, 980, 500, THEME_CARD_BORDER, false);
        let hdr_c = CString::new("Power & Battery Info").unwrap();
        canvas_draw_text(c, 80, 250, hdr_c.as_ptr(), THEME_GREEN);
    }

    let bat_lbl = get_translation("power.battery");
    let bat_text = format!("{}: {}%", bat_lbl, battery_level);
    let bat_text_c = CString::new(bat_text).unwrap();
    unsafe { canvas_draw_text(c, 100, 340, bat_text_c.as_ptr(), THEME_WHITE); }

    // Battery bar
    let fill_w = (battery_level * 400) / 100;
    unsafe {
        canvas_draw_rect(c, 100, 390, 400, 30, THEME_CARD_BORDER, true);
        if fill_w > 0 {
            let bar_col = if battery_level < 20 { 0xEF4444FF } else if battery_level < 50 { THEME_AMBER } else { THEME_GREEN };
            canvas_draw_rect(c, 100, 390, fill_w, 30, bar_col, true);
        }
    }

    let mode_lbl = get_translation("power.mode");
    let mode_key = match current_mode {
        "performance" => "power.mode.performance",
        "powersave"   => "power.mode.powersave",
        _             => "power.mode.balanced",
    };
    let mode_val = get_translation(mode_key);
    let mode_text = format!("{}: {} ({})", mode_lbl, mode_val, current_mode);
    let mode_text_c = CString::new(mode_text).unwrap();
    unsafe { canvas_draw_text(c, 100, 480, mode_text_c.as_ptr(), THEME_WHITE); }

    // --- Card 2: Power Profile Selection ---
    unsafe {
        canvas_draw_rect(c, 50, 750, 980, 600, THEME_CARD_BG, true);
        canvas_draw_rect(c, 50, 750, 980, 600, THEME_CARD_BORDER, false);
        let hdr_c = CString::new("Select Power Profile").unwrap();
        canvas_draw_text(c, 80, 800, hdr_c.as_ptr(), THEME_BLUE);
    }

    let profiles = [
        ("powersave",    870, "1. Power Saver",          THEME_GREEN),
        ("balanced",     980, "2. Balanced (Default)",   THEME_BLUE),
        ("performance", 1090, "3. High Performance",     THEME_AMBER),
    ];

    for (mode, y, label, col) in &profiles {
        unsafe {
            canvas_draw_rect(c, 100, *y,      880, 80, THEME_CARD_DARK,   true);
            canvas_draw_rect(c, 100, *y,      880, 80, THEME_CARD_BORDER, false);
            let lbl_c = CString::new(*label).unwrap();
            canvas_draw_text(c, 150, y + 50, lbl_c.as_ptr(), *col);
        }
        // Highlight active
        if *mode == current_mode {
            unsafe {
                canvas_draw_rect(c, 95, y - 5, 890, 90, *col, false);
            }
        }
    }

    // --- Card 3: System Info ---
    unsafe {
        canvas_draw_rect(c, 50, 1400, 980, 450, THEME_CARD_BG, true);
        canvas_draw_rect(c, 50, 1400, 980, 450, THEME_CARD_BORDER, false);
        let hdr_c = CString::new("System Info").unwrap();
        canvas_draw_text(c, 80, 1450, hdr_c.as_ptr(), THEME_AMBER);
    }

    let os_c   = CString::new("OS: Orion OS v1.0 Alpha (ARM64)").unwrap();
    let lang_s = format!("Active Language: {}", lang_code);
    let lang_c = CString::new(lang_s).unwrap();
    let kern_c = CString::new("Kernel: Linux 6.12-QEMU").unwrap();

    unsafe {
        canvas_draw_text(c, 100, 1530, os_c.as_ptr(),   THEME_WHITE);
        canvas_draw_text(c, 100, 1610, lang_c.as_ptr(), THEME_WHITE);
        canvas_draw_text(c, 100, 1690, kern_c.as_ptr(), THEME_MUTED);
    }
}

fn main() {
    println!("\n=============================================");
    println!("⚙️   Booting Orion Settings Application");
    println!("=============================================\n");

    let args: Vec<String> = std::env::args().collect();
    let mut set_mode: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--mode" && i + 1 < args.len() {
            set_mode = Some(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    // 1. Initialize i18n
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
    println!("[INFO] [SETTINGS] Localization initialized: '{}'", lang_code);

    // 2. Resolve services
    println!("[INFO] [SETTINGS] Resolving system services...");
    let pm_path = resolve_service_path("mobile.powermanager");
    let sf_path = resolve_service_path("mobile.surfaceflinger");
    let sb_path = resolve_service_path("mobile.statusbar");

    // 3. Apply mode change if requested
    if let Some(ref mode) = set_mode {
        println!("[INFO] [SETTINGS] Applying power mode: {}", mode);
        if set_power_mode(&pm_path, mode) {
            println!("[INFO] [SETTINGS] Power mode changed successfully.");
            send_notification(&sb_path, &format!("Mode Set: {}", mode));
        } else {
            eprintln!("[ERR]  [SETTINGS] Failed to set power mode via IPC.");
        }
    }

    // 4. Query current state
    let battery_level  = get_battery_level(&pm_path);
    let current_mode   = get_power_mode(&pm_path);
    println!("[INFO] [SETTINGS] Battery: {}%, Mode: {}", battery_level, current_mode);

    // 5. Initialize canvas
    let mut canvas = canvas_t { pixels: std::ptr::null_mut(), width: 0, height: 0 };
    unsafe {
        if canvas_init(&mut canvas, SCREEN_W, SCREEN_H) != 0 {
            std::process::exit(1);
        }
    }

    // 6. Allocate surface with surfaceflinger
    let surface_id = allocate_surface(&sf_path);
    if surface_id >= 0 {
        println!("[INFO] [SETTINGS] Allocated graphics surface ID {} on surfaceflinger.", surface_id);
    }

    // 7. Draw the settings UI
    draw_settings_layout(&mut canvas, battery_level, &current_mode, &lang_code);

    // 8. Output frame
    let _ = std::fs::create_dir_all("out");
    let out_path = if surface_id >= 0 {
        format!("out/surface_{}.ppm", surface_id)
    } else {
        "out/settings_display.ppm".to_string()
    };
    let out_path_c  = CString::new(out_path.clone()).unwrap();
    let back_path_c = CString::new("out/settings_display.ppm").unwrap();
    unsafe {
        canvas_save_ppm(&mut canvas, out_path_c.as_ptr());
        canvas_save_ppm(&mut canvas, back_path_c.as_ptr());
    }
    trigger_composite(&sf_path);
    println!("[INFO] [SETTINGS] Frame rendering complete: {}", out_path);

    // 9. Cleanup
    unsafe {
        canvas_free(&mut canvas);
        libi18n::i18n_free();
    }

    println!("[INFO] [SETTINGS] Settings application complete.");
}
