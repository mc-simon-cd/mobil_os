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

use libipc_rs::{Parcel, ipc_connect, ipc_send_transaction};
use std::ffi::CString;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Mutex;

const SERVICEMANAGER_SOCKET: &str = "/tmp/servicemanager.sock";
const SURFACEFLINGER_SOCKET: &str = "/tmp/surfaceflinger.sock";

const CMD_REGISTER_SERVICE: i32 = 1;

const CMD_ALLOC_SURFACE: i32 = 1;
const CMD_COMPOSITE: i32 = 2;
const CMD_DESTROY_SURFACE: i32 = 3;

extern "C" {
    fn wl_compositor_init() -> bool;
    fn wl_compositor_run();
    fn wl_compositor_cleanup();
    fn wl_compositor_inject_touch(ev_type: libc::c_int, ev_code: libc::c_int, ev_val: libc::c_int);
    fn wl_compositor_force_composite();
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Surface {
    id: i32,
    width: i32,
    height: i32,
    format: i32,
    active: bool,
}

struct Composer {
    surfaces: Vec<Surface>,
    next_surface_id: i32,
}

impl Composer {
    fn new() -> Self {
        println!("[INFO] [SURFACEFLINGER] Graphics Composer engine initialized.");
        Self {
            surfaces: Vec::new(),
            next_surface_id: 1,
        }
    }

    fn allocate_surface(&mut self, width: i32, height: i32, format: i32) -> i32 {
        let id = self.next_surface_id;
        self.next_surface_id += 1;

        self.surfaces.push(Surface {
            id,
            width,
            height,
            format,
            active: true,
        });

        println!(
            "[INFO] [SURFACEFLINGER] Allocated surface ID {} ({}x{}, Format: {})",
            id, width, height, format
        );
        id
    }

    fn destroy_surface(&mut self, id: i32) -> i32 {
        if let Some(idx) = self.surfaces.iter().position(|s| s.id == id) {
            println!("[INFO] [SURFACEFLINGER] Destroyed surface ID {}", id);
            self.surfaces.remove(idx);
            0
        } else {
            println!(
                "[WARN] [SURFACEFLINGER] Attempted to destroy non-existent surface ID {}",
                id
            );
            -1
        }
    }

    fn composite_layers(&self) -> i32 {
        if self.surfaces.is_empty() {
            return 0;
        }

        println!("[INFO] [SURFACEFLINGER] Starting software frame composition...");

        let master_w = 1080i32;
        let master_h = 2400i32;
        let mut master_rgb = vec![0u8; (master_w * master_h * 3) as usize];

        // Fill background with a deep dark slate color (15, 13, 22)
        for i in 0..((master_w * master_h) as usize) {
            master_rgb[i * 3 + 0] = 15;
            master_rgb[i * 3 + 1] = 13;
            master_rgb[i * 3 + 2] = 22;
        }

        // Sort layers by height descending so smaller overlays (statusbar) are rendered last on top
        let mut sorted_surfaces = self.surfaces.clone();
        sorted_surfaces.sort_by(|a, b| b.height.cmp(&a.height));

        for s in sorted_surfaces {
            if !s.active {
                continue;
            }

            let file_path = format!("out/surface_{}.ppm", s.id);
            let img = match load_ppm(&file_path) {
                Ok(im) => im,
                Err(_) => {
                    println!(
                        "  ⚠️  No visual buffer found for surface ID {} (looked for {})",
                        s.id, file_path
                    );
                    continue;
                }
            };

            // Determine starting y offset based on layer heights
            let mut y_start = 0;
            if s.height == 2200 {
                y_start = 100; // middle application window
            } else if s.height <= 100 {
                y_start = 0; // top statusbar panel overlay
            }

            println!(
                "  👉 Compositing surface layer ID {} [{}x{}] at y={}",
                s.id, s.width, s.height, y_start
            );

            // Copy source pixels to master canvas
            for src_y in 0..img.height {
                let dest_y = y_start + src_y;
                if dest_y >= master_h {
                    break;
                }

                for src_x in 0..img.width {
                    let dest_x = src_x;
                    if dest_x >= master_w {
                        break;
                    }

                    let src_idx = ((src_y * img.width + src_x) * 3) as usize;
                    let dest_idx = ((dest_y * master_w + dest_x) * 3) as usize;

                    master_rgb[dest_idx + 0] = img.rgb_data[src_idx + 0];
                    master_rgb[dest_idx + 1] = img.rgb_data[src_idx + 1];
                    master_rgb[dest_idx + 2] = img.rgb_data[src_idx + 2];
                }
            }
        }

        // Save final combined PPM image
        let _ = std::fs::create_dir_all("out");
        let mut out = match File::create("out/display_composited.ppm") {
            Ok(f) => f,
            Err(_) => {
                eprintln!("[ERR] [SURFACEFLINGER] Failed to write to out/display_composited.ppm");
                return -1;
            }
        };

        let header = format!("P6\n{} {}\n255\n", master_w, master_h);
        if out.write_all(header.as_bytes()).is_err() || out.write_all(&master_rgb).is_err() {
            eprintln!("[ERR] [SURFACEFLINGER] Failed to write to out/display_composited.ppm");
            return -1;
        }

        println!("[INFO] [SURFACEFLINGER] Saved combined composited frame to out/display_composited.ppm");
        0
    }
}

struct PPMImage {
    width: i32,
    height: i32,
    rgb_data: Vec<u8>,
}

fn load_ppm(path: &str) -> Result<PPMImage, std::io::Error> {
    let f = File::open(path)?;
    let mut r = BufReader::new(f);

    let mut header = String::new();
    r.read_line(&mut header)?;
    if header.trim() != "P6" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Not P6 format",
        ));
    }

    let mut dims = String::new();
    loop {
        dims.clear();
        r.read_line(&mut dims)?;
        if dims.trim().starts_with('#') {
            continue;
        }
        if !dims.trim().is_empty() {
            break;
        }
    }

    let parts: Vec<&str> = dims.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid dimensions",
        ));
    }
    let width: i32 = parts[0]
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Bad width"))?;
    let height: i32 = parts[1]
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Bad height"))?;

    let mut max_val_str = String::new();
    r.read_line(&mut max_val_str)?;

    let mut rgb_data = vec![0u8; (width * height * 3) as usize];
    r.read_exact(&mut rgb_data)?;

    Ok(PPMImage {
        width,
        height,
        rgb_data,
    })
}

static G_COMPOSER: Mutex<Option<Composer>> = Mutex::new(None);

fn register_with_servicemanager() {
    println!("[INFO] [SURFACEFLINGER] Registering with servicemanager...");

    let mut stream = match ipc_connect(SERVICEMANAGER_SOCKET) {
        Ok(s) => s,
        Err(_) => {
            println!("[WARN] [SURFACEFLINGER] Servicemanager not running. Proceeding standalone.");
            return;
        }
    };

    let mut data = Parcel::new();
    data.write_string("mobile.surfaceflinger");
    data.write_string(SURFACEFLINGER_SOCKET);

    match ipc_send_transaction(&mut stream, CMD_REGISTER_SERVICE, &data) {
        Ok(mut reply) => {
            reply.reset_read();
            let status = reply.read_i32().unwrap_or(-1);
            if status == 0 {
                println!("[INFO] [SURFACEFLINGER] Registered successfully in system directory.");
            } else {
                println!(
                    "[WARN] [SURFACEFLINGER] Registration failed with status: {}",
                    status
                );
            }
        }
        Err(e) => {
            println!(
                "[WARN] [SURFACEFLINGER] IPC register transaction failed: {}",
                e
            );
        }
    }
}

fn handle_client(mut stream: UnixStream) {
    let mut header_bytes = [0u8; 8];
    if stream.read_exact(&mut header_bytes).is_err() {
        return;
    }

    let code = i32::from_ne_bytes(header_bytes[0..4].try_into().unwrap());
    let data_size = u32::from_ne_bytes(header_bytes[4..8].try_into().unwrap());

    let mut payload = vec![0u8; data_size as usize];
    if data_size > 0 {
        if stream.read_exact(&mut payload).is_err() {
            return;
        }
    }

    let mut data = Parcel::from_vec(payload);
    let mut reply = Parcel::new();

    if code == CMD_ALLOC_SURFACE {
        if let (Ok(w), Ok(h), Ok(fmt)) = (data.read_i32(), data.read_i32(), data.read_i32()) {
            let mut guard = G_COMPOSER.lock().unwrap();
            let sid = guard.as_mut().unwrap().allocate_surface(w, h, fmt);
            reply.write_i32(sid);
        } else {
            eprintln!("[ERR] [SURFACEFLINGER] Malformed CMD_ALLOC_SURFACE request!");
            reply.write_i32(-1);
        }
    } else if code == CMD_COMPOSITE {
        let guard = G_COMPOSER.lock().unwrap();
        let status = guard.as_ref().unwrap().composite_layers();
        reply.write_i32(status);
    } else if code == CMD_DESTROY_SURFACE {
        if let Ok(sid) = data.read_i32() {
            let mut guard = G_COMPOSER.lock().unwrap();
            let status = guard.as_mut().unwrap().destroy_surface(sid);
            reply.write_i32(status);
        } else {
            eprintln!("[ERR] [SURFACEFLINGER] Malformed CMD_DESTROY_SURFACE request!");
            reply.write_i32(-1);
        }
    } else {
        eprintln!(
            "[WARN] [SURFACEFLINGER] Unknown CMD transaction code: {}",
            code
        );
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

fn run_wayland_ipc_listener() {
    let _ = std::fs::remove_file(SURFACEFLINGER_SOCKET);
    let listener = match UnixListener::bind(SURFACEFLINGER_SOCKET) {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "[ERR] [SURFACEFLINGER] Failed to bind Wayland IPC socket: {}",
                e
            );
            return;
        }
    };

    unsafe {
        let socket_c = CString::new(SURFACEFLINGER_SOCKET).unwrap();
        libc::chmod(socket_c.as_ptr(), 0o666);
    }

    println!(
        "[INFO] [SURFACEFLINGER] Wayland IPC touch socket: {}",
        SURFACEFLINGER_SOCKET
    );

    std::thread::spawn(move || {
        for stream_res in listener.incoming() {
            let mut stream = match stream_res {
                Ok(s) => s,
                Err(_) => continue,
            };

            let mut header_bytes = [0u8; 8];
            if stream.read_exact(&mut header_bytes).is_err() {
                continue;
            }

            let code = i32::from_ne_bytes(header_bytes[0..4].try_into().unwrap());
            let data_size = u32::from_ne_bytes(header_bytes[4..8].try_into().unwrap());

            let mut payload = vec![0u8; data_size as usize];
            if data_size > 0 {
                if stream.read_exact(&mut payload).is_err() {
                    continue;
                }
            }

            let mut data = Parcel::from_vec(payload);

            // CMD_COMPOSITE = 2 (Force composition)
            if code == 2 {
                unsafe {
                    wl_compositor_force_composite();
                }
            }
            // CMD_SEND_INPUT_EVENT = 4 (Wayland touch injection)
            if code == 4 {
                if let (Ok(ev_type), Ok(ev_code), Ok(ev_val)) = (data.read_i32(), data.read_i32(), data.read_i32()) {
                    unsafe {
                        wl_compositor_inject_touch(ev_type, ev_code, ev_val);
                    }
                }
            }

            let mut reply = Parcel::new();
            reply.write_i32(0);

            let mut reply_header_bytes = [0u8; 8];
            let reply_code = 0i32;
            let reply_size = reply.size() as u32;
            reply_header_bytes[0..4].copy_from_slice(&reply_code.to_ne_bytes());
            reply_header_bytes[4..8].copy_from_slice(&reply_size.to_ne_bytes());

            if stream.write_all(&reply_header_bytes).is_ok() && reply_size > 0 {
                let _ = stream.write_all(reply.data());
            }
        }
    });
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut use_wayland = false;
    for arg in &args {
        if arg == "--wayland" {
            use_wayland = true;
            break;
        }
    }

    if use_wayland {
        println!("[INFO] [SURFACEFLINGER] Starting Surfaceflinger in Wayland Compositor mode...");
        unsafe {
            if !wl_compositor_init() {
                eprintln!("[ERR] [SURFACEFLINGER] Failed to initialize Wayland compositor. Exiting.");
                std::process::exit(1);
            }
        }

        register_with_servicemanager();
        run_wayland_ipc_listener();

        unsafe {
            wl_compositor_run();
            wl_compositor_cleanup();
        }
        return;
    }

    println!("[INFO] [SURFACEFLINGER] Starting Surfaceflinger Graphics Daemon in legacy PPM mode...");

    {
        let mut guard = G_COMPOSER.lock().unwrap();
        *guard = Some(Composer::new());
    }

    register_with_servicemanager();

    let _ = std::fs::remove_file(SURFACEFLINGER_SOCKET);

    let listener = match UnixListener::bind(SURFACEFLINGER_SOCKET) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ERR] [SURFACEFLINGER] Failed to bind socket: {}", e);
            std::process::exit(1);
        }
    };

    unsafe {
        let socket_c = CString::new(SURFACEFLINGER_SOCKET).unwrap();
        libc::chmod(socket_c.as_ptr(), 0o666);
    }

    println!(
        "[INFO] [SURFACEFLINGER] Graphics Server listening at {}",
        SURFACEFLINGER_SOCKET
    );

    for stream_res in listener.incoming() {
        if let Ok(stream) = stream_res {
            handle_client(stream);
        }
    }
}
