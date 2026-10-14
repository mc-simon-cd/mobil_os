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

use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use libipc_rs::Parcel;

const SOCKET_PATH: &str = "/tmp/servicemanager.sock";

const CMD_REGISTER_SERVICE: i32 = 1;
const CMD_GET_SERVICE: i32 = 2;

struct Registry {
    services: HashMap<String, String>,
}

fn main() {
    println!("[INFO] [SERVICEMANAGER] Starting IPC Service Manager...");

    // Clean up existing socket
    if Path::new(SOCKET_PATH).exists() {
        let _ = fs::remove_file(SOCKET_PATH);
    }

    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ERR] [SERVICEMANAGER] Failed to bind to {}: {}", SOCKET_PATH, e);
            std::process::exit(1);
        }
    };

    // Set permission masks so system & ui users can connect dynamically
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(SOCKET_PATH) {
            let mut perms = meta.permissions();
            perms.set_mode(0o666);
            let _ = fs::set_permissions(SOCKET_PATH, perms);
        }
    }

    println!("[INFO] [SERVICEMANAGER] Listening for IPC service registrations at {}", SOCKET_PATH);

    let registry = Arc::new(Mutex::new(Registry {
        services: HashMap::new(),
    }));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let registry_clone = Arc::clone(&registry);
                if let Err(e) = handle_client(stream, registry_clone) {
                    eprintln!("[WARN] [SERVICEMANAGER] Error handling client: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[WARN] [SERVICEMANAGER] Connection failed: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: UnixStream, registry: Arc<Mutex<Registry>>) -> std::io::Result<()> {
    // 1. Read transaction header
    let mut header_bytes = [0u8; 8];
    stream.read_exact(&mut header_bytes)?;

    let code = i32::from_ne_bytes(header_bytes[0..4].try_into().unwrap());
    let data_size = u32::from_ne_bytes(header_bytes[4..8].try_into().unwrap());

    // 2. Read transaction payload if size > 0
    let mut data_bytes = vec![0u8; data_size as usize];
    if data_size > 0 {
        stream.read_exact(&mut data_bytes)?;
    }

    let mut data = Parcel::from_vec(data_bytes);
    let mut reply = Parcel::new();

    // 3. Process commands
    match code {
        CMD_REGISTER_SERVICE => {
            let name_res = data.read_string();
            let path_res = data.read_string();

            match (name_res, path_res) {
                (Ok(name), Ok(path)) => {
                    let mut r = registry.lock().unwrap();
                    r.services.insert(name.clone(), path.clone());
                    println!("[INFO] [SERVICEMANAGER] Registered: '{}' -> {}", name, path);
                    reply.write_i32(0); // 0 = Success
                }
                _ => {
                    eprintln!("[ERR] [SERVICEMANAGER] Malformed register request received!");
                    reply.write_i32(-1);
                }
            }
        }
        CMD_GET_SERVICE => {
            match data.read_string() {
                Ok(name) => {
                    let r = registry.lock().unwrap();
                    let path = r.services.get(&name).map(|s| s.as_str());
                    println!(
                        "[INFO] [SERVICEMANAGER] Resolved query: '{}' -> {}",
                        name,
                        path.unwrap_or("NULL")
                    );
                    reply.write_string(path.unwrap_or(""));
                }
                Err(_) => {
                    eprintln!("[ERR] [SERVICEMANAGER] Malformed get request received!");
                    reply.write_string("");
                }
            }
        }
        _ => {
            eprintln!("[WARN] [SERVICEMANAGER] Unknown command transaction code: {}", code);
            reply.write_i32(-1);
        }
    }

    // 4. Send response header and payload
    let reply_header_code = 0i32;
    let reply_data_size = reply.size() as u32;

    let mut reply_header_bytes = [0u8; 8];
    reply_header_bytes[0..4].copy_from_slice(&reply_header_code.to_ne_bytes());
    reply_header_bytes[4..8].copy_from_slice(&reply_data_size.to_ne_bytes());

    stream.write_all(&reply_header_bytes)?;
    if reply_data_size > 0 {
        stream.write_all(reply.data())?;
    }
    stream.flush()?;

    Ok(())
}
