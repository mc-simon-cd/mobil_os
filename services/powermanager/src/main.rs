use std::convert::TryInto;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use libipc_rs::{ipc_connect, ipc_send_transaction, Parcel};

const SOCKET_PATH: &str = "/tmp/powermanager.sock";

// Commands
const CMD_GET_BATTERY_LEVEL: i32 = 1;
const CMD_SET_POWER_MODE: i32 = 2;
const CMD_GET_POWER_MODE: i32 = 3;

struct PowerState {
    power_mode: String,
}

fn register_with_servicemanager() {
    println!("[INFO] [POWERMANAGER] Registering with servicemanager...");
    match ipc_connect("/tmp/servicemanager.sock") {
        Ok(mut stream) => {
            let mut data = Parcel::new();
            data.write_string("mobile.powermanager");
            data.write_string("/tmp/powermanager.sock");
            
            // CMD_REGISTER_SERVICE = 1
            match ipc_send_transaction(&mut stream, 1, &data) {
                Ok(mut reply) => {
                    if let Ok(status) = reply.read_i32() {
                        if status == 0 {
                            println!("[INFO] [POWERMANAGER] Registered successfully in system directory.");
                        } else {
                            eprintln!("[WARN] [POWERMANAGER] Registration failed with status: {}", status);
                        }
                    } else {
                        eprintln!("[WARN] [POWERMANAGER] Failed to parse registration status");
                    }
                }
                Err(e) => {
                    eprintln!("[WARN] [POWERMANAGER] IPC register transaction failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("[WARN] [POWERMANAGER] Servicemanager not running. Proceeding standalone: {}", e);
        }
    }
}

fn main() {
    println!("[INFO] [POWERMANAGER] Starting Power Manager Daemon...");

    // Register with servicemanager
    register_with_servicemanager();

    // Clean up existing socket
    if Path::new(SOCKET_PATH).exists() {
        let _ = fs::remove_file(SOCKET_PATH);
    }

    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ERR] [POWERMANAGER] Failed to bind to {}: {}", SOCKET_PATH, e);
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

    println!("[INFO] [POWERMANAGER] Listening at {}", SOCKET_PATH);

    let state = Arc::new(Mutex::new(PowerState {
        power_mode: "balanced".to_string(),
    }));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state_clone = Arc::clone(&state);
                if let Err(e) = handle_client(stream, state_clone) {
                    eprintln!("[WARN] [POWERMANAGER] Error handling client: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[WARN] [POWERMANAGER] Connection failed: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: UnixStream, state: Arc<Mutex<PowerState>>) -> std::io::Result<()> {
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
        CMD_GET_BATTERY_LEVEL => {
            // Returns battery level integer (e.g., 85)
            reply.write_i32(85);
            println!("[INFO] [POWERMANAGER] CMD_GET_BATTERY_LEVEL -> 85%");
        }
        CMD_SET_POWER_MODE => {
            match data.read_string() {
                Ok(mode) => {
                    let mut s = state.lock().unwrap();
                    s.power_mode = mode.clone();
                    reply.write_i32(0); // 0 = Success
                    println!("[INFO] [POWERMANAGER] CMD_SET_POWER_MODE -> '{}' success", mode);
                }
                Err(e) => {
                    eprintln!("[ERR] [POWERMANAGER] Malformed CMD_SET_POWER_MODE request: {}", e);
                    reply.write_i32(-1); // Error
                }
            }
        }
        CMD_GET_POWER_MODE => {
            let s = state.lock().unwrap();
            reply.write_string(&s.power_mode);
            println!("[INFO] [POWERMANAGER] CMD_GET_POWER_MODE -> '{}'", s.power_mode);
        }
        _ => {
            eprintln!("[WARN] [POWERMANAGER] Unknown command transaction code: {}", code);
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
