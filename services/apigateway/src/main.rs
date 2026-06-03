// Copyright 2026 mcsimon
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use libipc_rs::{ipc_connect, ipc_send_transaction, Parcel};

const SERVICEMANAGER_SOCKET: &str = "/tmp/servicemanager.sock";

// Service Manager command codes
const CMD_GET_SERVICE: i32 = 2;

// Powermanager command codes
const CMD_GET_BATTERY_LEVEL: i32 = 1;
const CMD_SET_POWER_MODE: i32 = 2;
const CMD_GET_POWER_MODE: i32 = 3;

// Surfaceflinger command codes
const CMD_COMPOSITE: i32 = 2;

fn resolve_service(service_name: &str) -> std::io::Result<String> {
    let mut sm_stream = ipc_connect(SERVICEMANAGER_SOCKET)?;
    let mut data = Parcel::new();
    data.write_string(service_name);
    
    let mut reply = ipc_send_transaction(&mut sm_stream, CMD_GET_SERVICE, &data)?;
    match reply.read_string() {
        Ok(path) => {
            if path.is_empty() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Service {} resolved to empty path", service_name),
                ))
            } else {
                Ok(path)
            }
        }
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse resolved path: {}", e),
        )),
    }
}

fn handle_get_status() -> String {
    r#"{"status":"ok","service":"apigateway","version":"1.0"}"#.to_string()
}

fn handle_get_power() -> Result<String, String> {
    let path = resolve_service("mobile.powermanager")
        .map_err(|e| format!("Failed to resolve mobile.powermanager: {}", e))?;
    
    let mut pm_stream = ipc_connect(&path)
        .map_err(|e| format!("Failed to connect to powermanager at {}: {}", path, e))?;
    
    // Get battery level
    let data_battery = Parcel::new();
    let mut reply_battery = ipc_send_transaction(&mut pm_stream, CMD_GET_BATTERY_LEVEL, &data_battery)
        .map_err(|e| format!("Battery transaction failed: {}", e))?;
    let battery_level = reply_battery.read_i32()
        .map_err(|e| format!("Failed to read battery level: {}", e))?;
        
    // Reconnect to powermanager because socket is closed immediately on client completion
    let mut pm_stream_2 = ipc_connect(&path)
        .map_err(|e| format!("Failed to reconnect to powermanager at {}: {}", path, e))?;
        
    // Get power mode
    let data_mode = Parcel::new();
    let mut reply_mode = ipc_send_transaction(&mut pm_stream_2, CMD_GET_POWER_MODE, &data_mode)
        .map_err(|e| format!("Power mode transaction failed: {}", e))?;
    let power_mode = reply_mode.read_string()
        .map_err(|e| format!("Failed to read power mode: {}", e))?;
        
    Ok(format!(
        r#"{{"battery_level":{},"power_mode":"{}"}}"#,
        battery_level, power_mode
    ))
}

fn handle_post_power_mode(mode: &str) -> Result<String, String> {
    if mode.is_empty() {
        return Err("Mode parameter is empty".to_string());
    }
    
    let path = resolve_service("mobile.powermanager")
        .map_err(|e| format!("Failed to resolve mobile.powermanager: {}", e))?;
        
    let mut pm_stream = ipc_connect(&path)
        .map_err(|e| format!("Failed to connect to powermanager at {}: {}", path, e))?;
        
    let mut data = Parcel::new();
    data.write_string(mode);
    
    let mut reply = ipc_send_transaction(&mut pm_stream, CMD_SET_POWER_MODE, &data)
        .map_err(|e| format!("Set power mode transaction failed: {}", e))?;
        
    let status = reply.read_i32()
        .map_err(|e| format!("Failed to read set status: {}", e))?;
        
    if status == 0 {
        Ok(format!(r#"{{"status":"success","power_mode":"{}"}}"#, mode))
    } else {
        Err(format!("Power manager returned error code: {}", status))
    }
}

fn handle_post_graphics_composite() -> Result<String, String> {
    let path = resolve_service("mobile.surfaceflinger")
        .map_err(|e| format!("Failed to resolve mobile.surfaceflinger: {}", e))?;
        
    let mut sf_stream = ipc_connect(&path)
        .map_err(|e| format!("Failed to connect to surfaceflinger at {}: {}", path, e))?;
        
    let data = Parcel::new();
    let mut reply = ipc_send_transaction(&mut sf_stream, CMD_COMPOSITE, &data)
        .map_err(|e| format!("Composite transaction failed: {}", e))?;
        
    let status = reply.read_i32()
        .map_err(|e| format!("Failed to read composite status: {}", e))?;
        
    if status == 0 {
        Ok(r#"{"status":"success","composited":true}"#.to_string())
    } else {
        Err(format!("Surfaceflinger returned error code: {}", status))
    }
}

fn handle_post_input_inject(ev_type: i32, ev_code: i32, ev_value: i32) -> Result<String, String> {
    let path = resolve_service("mobile.input")
        .map_err(|e| format!("Failed to resolve mobile.input: {}", e))?;
        
    let mut input_stream = ipc_connect(&path)
        .map_err(|e| format!("Failed to connect to inputflinger at {}: {}", path, e))?;
        
    let mut data = Parcel::new();
    data.write_i32(ev_type);
    data.write_i32(ev_code);
    data.write_i32(ev_value);
    
    let mut reply = ipc_send_transaction(&mut input_stream, 2, &data)
        .map_err(|e| format!("Input injection transaction failed: {}", e))?;
        
    let status = reply.read_i32()
        .map_err(|e| format!("Failed to read injection status: {}", e))?;
        
    if status == 0 {
        Ok(format!(
            r#"{{"status":"success","type":{},"code":{},"value":{}}}"#,
            ev_type, ev_code, ev_value
        ))
    } else {
        Err(format!("Inputflinger returned error code: {}", status))
    }
}

fn handle_get_input_last() -> Result<String, String> {
    let path = resolve_service("mobile.input")
        .map_err(|e| format!("Failed to resolve mobile.input: {}", e))?;
        
    let mut input_stream = ipc_connect(&path)
        .map_err(|e| format!("Failed to connect to inputflinger at {}: {}", path, e))?;
        
    let data = Parcel::new();
    // CMD_GET_LAST_EVENT = 3
    let mut reply = ipc_send_transaction(&mut input_stream, 3, &data)
        .map_err(|e| format!("Get last event transaction failed: {}", e))?;
        
    let status = reply.read_i32()
        .map_err(|e| format!("Failed to read status: {}", e))?;
        
    if status == 0 {
        let t = reply.read_i32().map_err(|e| format!("Failed to read type: {}", e))?;
        let c = reply.read_i32().map_err(|e| format!("Failed to read code: {}", e))?;
        let v = reply.read_i32().map_err(|e| format!("Failed to read value: {}", e))?;
        
        Ok(format!(
            r#"{{"status":"success","type":{},"code":{},"value":{}}}"#,
            t, c, v
        ))
    } else {
        Err("No event recorded".to_string())
    }
}

fn get_query_param<'a>(query: &'a str, name: &str) -> Option<&'a str> {
    for pair in query.split('&') {
        let mut parts = pair.split('=');
        if let Some(k) = parts.next() {
            if k == name {
                return parts.next();
            }
        }
    }
    None
}

fn send_response(mut stream: &TcpStream, status_code: u16, status_text: &str, json_body: &str) {
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         Connection: close\r\n\r\n\
         {}",
        status_code, status_text, json_body.len(), json_body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0u8; 2048];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    
    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let first_line = match request_str.lines().next() {
        Some(line) => line,
        None => return,
    };
    
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let full_uri = parts.next().unwrap_or("/");
    
    if method == "OPTIONS" {
        send_response(&stream, 200, "OK", "");
        return;
    }
    
    let mut uri_parts = full_uri.split('?');
    let path = uri_parts.next().unwrap_or("/");
    let query_string = uri_parts.next().unwrap_or("");
    
    println!("[INFO] [APIGATEWAY] Request: {} {}", method, path);
    
    match (method, path) {
        ("GET", "/api/status") => {
            send_response(&stream, 200, "OK", &handle_get_status());
        }
        ("GET", "/api/power") => {
            match handle_get_power() {
                Ok(json) => send_response(&stream, 200, "OK", &json),
                Err(err) => {
                    let err_json = format!(r#"{{"error":"Service Unavailable","message":"{}"}}"#, err);
                    send_response(&stream, 503, "Service Unavailable", &err_json);
                }
            }
        }
        ("GET", "/api/input/last") => {
            match handle_get_input_last() {
                Ok(json) => send_response(&stream, 200, "OK", &json),
                Err(err) => {
                    let err_json = format!(r#"{{"error":"Service Unavailable","message":"{}"}}"#, err);
                    send_response(&stream, 503, "Service Unavailable", &err_json);
                }
            }
        }
        ("POST", "/api/power/mode") => {
            let mode = get_query_param(query_string, "mode").unwrap_or("");
            match handle_post_power_mode(mode) {
                Ok(json) => send_response(&stream, 200, "OK", &json),
                Err(err) => {
                    let err_json = format!(r#"{{"error":"Bad Request","message":"{}"}}"#, err);
                    send_response(&stream, 400, "Bad Request", &err_json);
                }
            }
        }
        ("POST", "/api/graphics/composite") => {
            match handle_post_graphics_composite() {
                Ok(json) => send_response(&stream, 200, "OK", &json),
                Err(err) => {
                    let err_json = format!(r#"{{"error":"Internal Server Error","message":"{}"}}"#, err);
                    send_response(&stream, 500, "Internal Server Error", &err_json);
                }
            }
        }
        ("POST", "/api/input/inject") => {
            let ev_type_str = get_query_param(query_string, "type").unwrap_or("0");
            let ev_code_str = get_query_param(query_string, "code").unwrap_or("0");
            let ev_value_str = get_query_param(query_string, "value").unwrap_or("0");
            
            let ev_type = ev_type_str.parse::<i32>().unwrap_or(0);
            let ev_code = ev_code_str.parse::<i32>().unwrap_or(0);
            let ev_value = ev_value_str.parse::<i32>().unwrap_or(0);
            
            match handle_post_input_inject(ev_type, ev_code, ev_value) {
                Ok(json) => send_response(&stream, 200, "OK", &json),
                Err(err) => {
                    let err_json = format!(r#"{{"error":"Bad Request","message":"{}"}}"#, err);
                    send_response(&stream, 400, "Bad Request", &err_json);
                }
            }
        }
        _ => {
            let err_json = format!(r#"{{"error":"Not Found","path":"{}"}}"#, path);
            send_response(&stream, 404, "Not Found", &err_json);
        }
    }
}

fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listen_addr = format!("0.0.0.0:{}", port);
    println!("[INFO] [APIGATEWAY] Starting API Gateway on http://{}...", listen_addr);
    
    let listener = match TcpListener::bind(&listen_addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ERR] [APIGATEWAY] Failed to bind to {}: {}", listen_addr, e);
            std::process::exit(1);
        }
    };
    
    println!("[INFO] [APIGATEWAY] Listening for incoming HTTP requests on port {}...", port);
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("[WARN] [APIGATEWAY] Connection failed: {}", e);
            }
        }
    }
}
