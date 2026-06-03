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

use std::fs;
use std::path::Path;
use std::process::Command;
use std::net::TcpListener;
use std::io::{Read, Write};

fn extract_json_value(json: &str, key: &str) -> Option<String> {
    let target = format!("\"{}\"", key);
    let key_idx = json.find(&target)?;
    let colon_idx = json[key_idx..].find(':')? + key_idx;
    let remaining = &json[colon_idx + 1..];
    let start_quote = remaining.find('"')?;
    let end_quote = remaining[start_quote + 1..].find('"')? + start_quote + 1;
    Some(remaining[start_quote + 1..end_quote].to_string())
}

fn run_web_server(app_dir: &str, entry_point: &str) {
    let port = std::env::var("APP_PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ERR] [APPRUNNER] Failed to bind Web server to {}: {}", addr, e);
            std::process::exit(1);
        }
    };
    println!("[INFO] [APPRUNNER] Web App Server running at http://localhost:{} (serving {})", port, app_dir);
    println!("[INFO] [APPRUNNER] Press Ctrl+C to stop.");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0u8; 1024];
            if let Ok(bytes_read) = stream.read(&mut buffer) {
                if bytes_read == 0 { continue; }
                let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
                let first_line = match request_str.lines().next() {
                    Some(line) => line,
                    None => continue,
                };
                let mut parts = first_line.split_whitespace();
                let _method = parts.next().unwrap_or("GET");
                let uri = parts.next().unwrap_or("/");

                // Clean up URI: remove query parameters or hash
                let clean_uri = uri.split('?').next().unwrap_or("/").split('#').next().unwrap_or("/");
                let file_path = if clean_uri == "/" {
                    Path::new(app_dir).join(entry_point)
                } else {
                    Path::new(app_dir).join(&clean_uri[1..])
                };

                if file_path.exists() && file_path.is_file() {
                    if let Ok(content) = fs::read(&file_path) {
                        let content_type = match file_path.extension().and_then(|s| s.to_str()) {
                            Some("html") => "text/html",
                            Some("css") => "text/css",
                            Some("js") => "application/javascript",
                            Some("png") => "image/png",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("svg") => "image/svg+xml",
                            _ => "text/plain",
                        };

                        let response = format!(
                            "HTTP/1.1 200 OK\r\n\
                             Content-Type: {}\r\n\
                             Content-Length: {}\r\n\
                             Access-Control-Allow-Origin: *\r\n\
                             Connection: close\r\n\r\n",
                            content_type, content.len()
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.write_all(&content);
                    } else {
                        let body = "500 Internal Server Error";
                        let response = format!(
                            "HTTP/1.1 500 Internal Server Error\r\n\
                             Content-Length: {}\r\n\
                             Connection: close\r\n\r\n{}",
                            body.len(), body
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                } else {
                    let body = "404 Not Found";
                    let response = format!(
                        "HTTP/1.1 404 Not Found\r\n\
                         Content-Length: {}\r\n\
                         Connection: close\r\n\r\n{}",
                        body.len(), body
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
                let _ = stream.flush();
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <app_directory_path>", args[0]);
        std::process::exit(1);
    }

    let app_dir = &args[1];
    let manifest_path = Path::new(app_dir).join("manifest.json");

    if !manifest_path.exists() {
        eprintln!("[ERR] [APPRUNNER] manifest.json not found in {}", app_dir);
        std::process::exit(1);
    }

    let manifest_content = match fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERR] [APPRUNNER] Failed to read manifest.json: {}", e);
            std::process::exit(1);
        }
    };

    let name = extract_json_value(&manifest_content, "name").unwrap_or_else(|| "Unknown App".to_string());
    let app_type = match extract_json_value(&manifest_content, "type") {
        Some(t) => t,
        None => {
            eprintln!("[ERR] [APPRUNNER] 'type' field missing in manifest.json");
            std::process::exit(1);
        }
    };
    let entry_point = match extract_json_value(&manifest_content, "entry_point") {
        Some(ep) => ep,
        None => {
            eprintln!("[ERR] [APPRUNNER] 'entry_point' field missing in manifest.json");
            std::process::exit(1);
        }
    };

    println!("[INFO] [APPRUNNER] Launching app '{}' (type: {}, entry: {})", name, app_type, entry_point);

    match app_type.as_str() {
        "native" => {
            let exe_path = Path::new(app_dir).join(&entry_point);
            let mut status = Command::new(&exe_path)
                .current_dir(app_dir)
                .status();
            
            // Fallback to "./entry_point" style if absolute/relative resolve fails
            if status.is_err() {
                status = Command::new(format!("./{}", entry_point))
                    .current_dir(app_dir)
                    .status();
            }

            match status {
                Ok(s) => {
                    if !s.success() {
                        eprintln!("[WARN] [APPRUNNER] Native app exited with failure: {:?}", s.code());
                        std::process::exit(s.code().unwrap_or(1));
                    }
                }
                Err(e) => {
                    eprintln!("[ERR] [APPRUNNER] Failed to execute native app: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "python" => {
            let status = Command::new("python3")
                .arg(&entry_point)
                .current_dir(app_dir)
                .status();

            match status {
                Ok(s) => {
                    if !s.success() {
                        eprintln!("[WARN] [APPRUNNER] Python app exited with failure: {:?}", s.code());
                        std::process::exit(s.code().unwrap_or(1));
                    }
                }
                Err(e) => {
                    eprintln!("[ERR] [APPRUNNER] Failed to execute python3: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "javascript" => {
            let status = Command::new("node")
                .arg(&entry_point)
                .current_dir(app_dir)
                .status();

            match status {
                Ok(s) => {
                    if !s.success() {
                        eprintln!("[WARN] [APPRUNNER] JavaScript app exited with failure: {:?}", s.code());
                        std::process::exit(s.code().unwrap_or(1));
                    }
                }
                Err(e) => {
                    eprintln!("[ERR] [APPRUNNER] Failed to execute node: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "web" => {
            run_web_server(app_dir, &entry_point);
        }
        _ => {
            eprintln!("[ERR] [APPRUNNER] Unknown application type: {}", app_type);
            std::process::exit(1);
        }
    }
}
