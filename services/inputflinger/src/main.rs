use std::convert::TryInto;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use libipc_rs::{ipc_connect, ipc_send_transaction, Parcel};

const SOCKET_PATH: &str = "/tmp/inputflinger.sock";

// Inputflinger Commands
const CMD_REGISTER_LISTENER: i32 = 1;
const CMD_SEND_INPUT_EVENT: i32 = 2;
const CMD_GET_LAST_EVENT: i32 = 3;

#[derive(Debug, Clone, Copy)]
struct InputEvent {
    event_type: i32,
    event_code: i32,
    event_value: i32,
}

struct InputState {
    last_event: Option<InputEvent>,
    listeners: Vec<String>,
}

fn register_with_servicemanager() {
    println!("[INFO] [INPUTFLINGER] Registering with servicemanager...");
    match ipc_connect("/tmp/servicemanager.sock") {
        Ok(mut stream) => {
            let mut data = Parcel::new();
            data.write_string("mobile.input");
            data.write_string(SOCKET_PATH);
            
            // CMD_REGISTER_SERVICE = 1
            match ipc_send_transaction(&mut stream, 1, &data) {
                Ok(mut reply) => {
                    if let Ok(status) = reply.read_i32() {
                        if status == 0 {
                            println!("[INFO] [INPUTFLINGER] Registered successfully in system directory.");
                        } else {
                            eprintln!("[WARN] [INPUTFLINGER] Registration failed with status: {}", status);
                        }
                    } else {
                        eprintln!("[WARN] [INPUTFLINGER] Failed to parse registration status");
                    }
                }
                Err(e) => {
                    eprintln!("[WARN] [INPUTFLINGER] IPC register transaction failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("[WARN] [INPUTFLINGER] Servicemanager not running. Proceeding standalone: {}", e);
        }
    }
}

fn main() {
    println!("[INFO] [INPUTFLINGER] Starting Input Flinger Daemon...");

    // Register with servicemanager
    register_with_servicemanager();

    // Clean up existing socket
    if Path::new(SOCKET_PATH).exists() {
        let _ = fs::remove_file(SOCKET_PATH);
    }

    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ERR] [INPUTFLINGER] Failed to bind to {}: {}", SOCKET_PATH, e);
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

    println!("[INFO] [INPUTFLINGER] Listening at {}", SOCKET_PATH);

    let state = Arc::new(Mutex::new(InputState {
        last_event: None,
        listeners: Vec::new(),
    }));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state_clone = Arc::clone(&state);
                if let Err(e) = handle_client(stream, state_clone) {
                    eprintln!("[WARN] [INPUTFLINGER] Error handling client: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[WARN] [INPUTFLINGER] Connection failed: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: UnixStream, state: Arc<Mutex<InputState>>) -> std::io::Result<()> {
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
        CMD_REGISTER_LISTENER => {
            match data.read_string() {
                Ok(listener_path) => {
                    let mut s = state.lock().unwrap();
                    if !s.listeners.contains(&listener_path) {
                        s.listeners.push(listener_path.clone());
                    }
                    reply.write_i32(0); // 0 = Success
                    println!("[INFO] [INPUTFLINGER] CMD_REGISTER_LISTENER -> registered '{}'", listener_path);
                }
                Err(e) => {
                    eprintln!("[ERR] [INPUTFLINGER] Malformed CMD_REGISTER_LISTENER request: {}", e);
                    reply.write_i32(-1); // Error
                }
            }
        }
        CMD_SEND_INPUT_EVENT => {
            let ev_type = data.read_i32();
            let ev_code = data.read_i32();
            let ev_value = data.read_i32();
            
            match (ev_type, ev_code, ev_value) {
                (Ok(t), Ok(c), Ok(v)) => {
                    let event = InputEvent {
                        event_type: t,
                        event_code: c,
                        event_value: v,
                    };
                    
                    println!(
                        "[INFO] [INPUTFLINGER] CMD_SEND_INPUT_EVENT -> Injected event (type={}, code={}, val={})",
                        t, c, v
                    );

                    let mut s = state.lock().unwrap();
                    s.last_event = Some(event);

                    // Dispatch to all listeners, removing dead ones
                    let mut dead_listeners = Vec::new();
                    for (i, listener_path) in s.listeners.iter().enumerate() {
                        match ipc_connect(listener_path) {
                            Ok(mut dest_stream) => {
                                let mut event_parcel = Parcel::new();
                                event_parcel.write_i32(t);
                                event_parcel.write_i32(c);
                                event_parcel.write_i32(v);
                                
                                // CMD_DISPATCH_EVENT = 1
                                if ipc_send_transaction(&mut dest_stream, 1, &event_parcel).is_err() {
                                    dead_listeners.push(i);
                                }
                            }
                            Err(_) => {
                                dead_listeners.push(i);
                            }
                        }
                    }

                    // Remove dead listeners
                    for i in dead_listeners.into_iter().rev() {
                        let removed = s.listeners.remove(i);
                        println!("[INFO] [INPUTFLINGER] Removed dead listener: '{}'", removed);
                    }

                    reply.write_i32(0); // 0 = Success
                }
                _ => {
                    eprintln!("[ERR] [INPUTFLINGER] Malformed CMD_SEND_INPUT_EVENT request");
                    reply.write_i32(-1);
                }
            }
        }
        CMD_GET_LAST_EVENT => {
            let s = state.lock().unwrap();
            if let Some(event) = s.last_event {
                reply.write_i32(0); // status Success
                reply.write_i32(event.event_type);
                reply.write_i32(event.event_code);
                reply.write_i32(event.event_value);
                println!(
                    "[INFO] [INPUTFLINGER] CMD_GET_LAST_EVENT -> Last event (type={}, code={}, val={})",
                    event.event_type, event.event_code, event.event_value
                );
            } else {
                reply.write_i32(-1); // No event yet
                println!("[INFO] [INPUTFLINGER] CMD_GET_LAST_EVENT -> No event stored");
            }
        }
        _ => {
            eprintln!("[WARN] [INPUTFLINGER] Unknown command transaction code: {}", code);
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
