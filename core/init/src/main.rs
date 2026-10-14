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
use std::ffi::{c_void, CString};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::os::raw::{c_char, c_long};
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

// Socket / network structures and constants
const SIOCGIFFLAGS: u64 = 0x8913;
const SIOCSIFFLAGS: u64 = 0x8914;
const SIOCSIFADDR: u64 = 0x8916;
const IFF_UP: i16 = 0x1;
const IFF_RUNNING: i16 = 0x40;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct sockaddr_in {
    sin_family: u16,
    sin_port: u16,
    sin_addr: in_addr,
    sin_zero: [u8; 8],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct in_addr {
    s_addr: u32,
}

#[repr(C)]
struct Ifreq {
    ifr_name: [c_char; 16],
    ifr_flags: i16,
    padding: [u8; 22],
}

#[repr(C)]
struct IfreqAddr {
    ifr_name: [c_char; 16],
    ifr_addr: sockaddr_in,
    padding: [u8; 8],
}

extern "C" {
    fn syscall(number: c_long, ...) -> c_long;
}

// Service definition representing daemons to launch/manage
struct Service {
    name: String,
    path: String,
    args: Vec<String>,
    pid: i32,
    respawn: bool,
    active: bool,
    class: String, // "core", "main", or "default"
}

static G_SERVICES: Mutex<Vec<Service>> = Mutex::new(Vec::new());
static G_PROPERTIES: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

unsafe fn setup_network_interface(name: &str, ip_addr: Option<&str>) {
    let sock = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0);
    if sock < 0 {
        eprintln!(
            "[ERR] [INIT] Failed to create socket for net config on {}: {}",
            name,
            std::io::Error::last_os_error()
        );
        return;
    }

    let mut ifr: Ifreq = std::mem::zeroed();
    for (i, byte) in name.bytes().enumerate() {
        if i >= 15 {
            break;
        }
        ifr.ifr_name[i] = byte as c_char;
    }

    // Get current flags
    if libc::ioctl(sock, SIOCGIFFLAGS as _, &mut ifr) < 0 {
        eprintln!(
            "[ERR] [INIT] Failed to get flags for {}: {}",
            name,
            std::io::Error::last_os_error()
        );
        libc::close(sock);
        return;
    }

    // Set UP and RUNNING flags
    ifr.ifr_flags |= IFF_UP | IFF_RUNNING;
    if libc::ioctl(sock, SIOCSIFFLAGS as _, &ifr) < 0 {
        eprintln!(
            "[ERR] [INIT] Failed to set flags (UP) for {}: {}",
            name,
            std::io::Error::last_os_error()
        );
        libc::close(sock);
        return;
    }
    println!("[INFO] [INIT] Interface {} brought UP", name);

    // Set IP address if provided
    if let Some(ip) = ip_addr {
        let mut ifr_addr: IfreqAddr = std::mem::zeroed();
        ifr_addr.ifr_name = ifr.ifr_name;
        ifr_addr.ifr_addr.sin_family = libc::AF_INET as u16;

        if let Ok(addr) = ip.parse::<std::net::Ipv4Addr>() {
            ifr_addr.ifr_addr.sin_addr.s_addr = u32::from_ne_bytes(addr.octets());
        }

        if libc::ioctl(sock, SIOCSIFADDR as _, &ifr_addr) < 0 {
            eprintln!(
                "[ERR] [INIT] Failed to set IP address {} for {}: {}",
                ip,
                name,
                std::io::Error::last_os_error()
            );
        } else {
            println!("[INFO] [INIT] Interface {} IP address set to {}", name, ip);
        }
    }

    libc::close(sock);
}

unsafe fn load_kernel_module(path: &str) -> i32 {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            println!("[WARN] [INIT] Could not open kernel module {}: {}", path, e);
            return -1;
        }
    };

    let mut buffer = Vec::new();
    if file.read_to_end(&mut buffer).is_err() {
        println!("[ERR] [INIT] Failed to read module {}", path);
        return -1;
    }

    let empty_opts = CString::new("").unwrap();

    #[cfg(target_arch = "aarch64")]
    const SYS_INIT_MODULE: c_long = 105;
    #[cfg(target_arch = "x86_64")]
    const SYS_INIT_MODULE: c_long = 175;

    let ret = syscall(
        SYS_INIT_MODULE,
        buffer.as_ptr() as *const c_void,
        buffer.len() as libc::size_t,
        empty_opts.as_ptr(),
    );

    if ret != 0 {
        println!(
            "[ERR] [INIT] init_module failed for {} (ret={}): {}",
            path,
            ret,
            std::io::Error::last_os_error()
        );
        return -1;
    }

    println!("[INFO] [INIT] Kernel module {} loaded successfully!", path);
    0
}

fn configure_network() {
    unsafe {
        setup_network_interface("lo", Some("127.0.0.1"));
        load_kernel_module("/lib/modules/e1000.ko");
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name != "." && name != ".." && name != "lo" {
                println!("[INFO] [INIT] Configuring detected ethernet interface: {}", name);
                unsafe {
                    setup_network_interface(&name, Some("10.0.2.15"));
                }
            }
        }
    } else {
        println!("[WARN] [INIT] Could not open /sys/class/net to scan interfaces");
    }
}

unsafe fn mount_essential_filesystems() -> Result<(), std::io::Error> {
    println!("[INFO] [INIT] Initializing virtual filesystems...");

    std::fs::create_dir_all("/proc")?;
    std::fs::create_dir_all("/sys")?;
    std::fs::create_dir_all("/dev")?;
    std::fs::create_dir_all("/tmp")?;

    mount_fs("proc", "/proc", "proc")?;
    mount_fs("sysfs", "/sys", "sysfs")?;
    mount_fs("devtmpfs", "/dev", "devtmpfs")?;
    mount_fs("tmpfs", "/tmp", "tmpfs")?;

    Ok(())
}

unsafe fn mount_fs(source: &str, target: &str, fstype: &str) -> Result<(), std::io::Error> {
    let src_c = CString::new(source).unwrap();
    let tgt_c = CString::new(target).unwrap();
    let typ_c = CString::new(fstype).unwrap();

    let ret = libc::mount(
        src_c.as_ptr(),
        tgt_c.as_ptr(),
        typ_c.as_ptr(),
        0,
        std::ptr::null(),
    );

    if ret != 0 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() != Some(libc::EBUSY) {
            return Err(err);
        }
    }
    println!("[INFO] [INIT] Mounted {} successfully.", target);
    Ok(())
}

fn property_init() {
    let mut map = HashMap::new();
    map.insert("ro.product.name".to_string(), "Orion OS".to_string());
    map.insert("ro.product.device".to_string(), "qemu-arm64".to_string());
    map.insert("ro.build.version".to_string(), "1.0-alpha".to_string());
    map.insert("sys.boot_completed".to_string(), "0".to_string());

    let mut guard = G_PROPERTIES.lock().unwrap();
    *guard = Some(map);
    println!("[INFO] [INIT] System property database initialized.");
}

fn property_set(key: &str, value: &str) -> i32 {
    let mut guard = G_PROPERTIES.lock().unwrap();
    if let Some(ref mut map) = *guard {
        if let Some(_) = map.get(key) {
            if key.starts_with("ro.") {
                eprintln!("[WARN] [INIT] Attempted write to read-only property '{}' rejected.", key);
                return -1;
            }
        }
        map.insert(key.to_string(), value.to_string());
        0
    } else {
        -1
    }
}

fn run_mkdir(path_str: &str, mode_str: Option<&str>) {
    let mode = if let Some(m_str) = mode_str {
        u32::from_str_radix(m_str, 8).unwrap_or(0o755)
    } else {
        0o755
    };

    let path = Path::new(path_str);
    if let Err(e) = std::fs::create_dir_all(path) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            println!("[WARN] [INIT] Failed to create directory {}: {}", path_str, e);
            return;
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(mode);
        let _ = std::fs::set_permissions(path, perms);
    }
    println!("[INFO] [INIT] Created system directory: {} (mode {:o})", path_str, mode);
}

fn run_write(path_str: &str, value_str: &str) {
    if let Err(e) = std::fs::write(path_str, format!("{}\n", value_str)) {
        println!("[WARN] [INIT] Failed to write value to {}: {}", path_str, e);
    } else {
        println!("[INFO] [INIT] Wrote value to {}", path_str);
    }
}

fn parse_init_rc(filename: &str) -> i32 {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            println!("[WARN] [INIT] Could not open boot config script {}: {}", filename, e);
            return -1;
        }
    };

    println!("[INFO] [INIT] Parsing system configuration: {}", filename);
    let reader = BufReader::new(file);
    let mut current_section = String::new();
    let mut current_svc: Option<Service> = None;

    for line_res in reader.lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with("on ") {
            if let Some(svc) = current_svc.take() {
                G_SERVICES.lock().unwrap().push(svc);
            }
            current_section = trimmed[3..].trim().to_string();
            println!("[INFO] [INIT] Processing boot event block: 'on {}'", current_section);
            continue;
        }

        if trimmed.starts_with("service ") {
            if let Some(svc) = current_svc.take() {
                G_SERVICES.lock().unwrap().push(svc);
            }
            let parts: Vec<&str> = trimmed[8..].split_whitespace().collect();
            if parts.len() < 2 {
                println!("[ERR] [INIT] Malformed service declaration: {}", trimmed);
                continue;
            }
            let name = parts[0].to_string();
            let path = parts[1].to_string();

            current_svc = Some(Service {
                name,
                path,
                args: Vec::new(),
                pid: -1,
                respawn: false,
                active: false,
                class: "default".to_string(),
            });
            continue;
        }

        // Handle commands inside triggers
        if !current_section.is_empty() && current_svc.is_none() {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if !parts.is_empty() {
                let cmd = parts[0];
                if cmd == "mkdir" && parts.len() >= 2 {
                    let path = parts[1];
                    let mode = parts.get(2).copied();
                    run_mkdir(path, mode);
                } else if cmd == "write" && parts.len() >= 3 {
                    let path = parts[1];
                    let value = parts[2];
                    run_write(path, value);
                }
            }
            continue;
        }

        // Handle service attributes (class, args, respawn)
        if let Some(ref mut svc) = current_svc {
            if trimmed.starts_with("class ") {
                svc.class = trimmed[6..].trim().to_string();
            } else if trimmed.starts_with("args ") {
                let arg = trimmed[5..].trim().to_string();
                svc.args.push(arg);
            } else if trimmed == "respawn" {
                svc.respawn = true;
            }
        }
    }

    if let Some(svc) = current_svc {
        G_SERVICES.lock().unwrap().push(svc);
    }

    0
}

fn spawn_service(svc: &mut Service) -> Result<(), std::io::Error> {
    if svc.pid != -1 {
        return Ok(());
    }

    let mut cmd = Command::new(&svc.path);
    cmd.args(&svc.args);

    let child = cmd.spawn()?;
    svc.pid = child.id() as i32;
    svc.active = true;

    println!("[INFO] [INIT] Spawned service daemon '{}' [PID: {}]", svc.name, svc.pid);
    Ok(())
}

fn launch_service_class(target_class: &str) -> i32 {
    let mut guard = G_SERVICES.lock().unwrap();
    for svc in guard.iter_mut() {
        if svc.class != target_class {
            continue;
        }
        if let Err(e) = spawn_service(svc) {
            println!("[ERR] [INIT] Failed to spawn service {}: {}", svc.name, e);
            return -1;
        }
        std::thread::sleep(std::time::Duration::from_millis(400));
    }
    0
}

fn launch_services() -> i32 {
    println!("[INFO] [INIT] Launching core services...");
    if launch_service_class("core") != 0 {
        return -1;
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    println!("[INFO] [INIT] Launching main services...");
    if launch_service_class("main") != 0 {
        return -1;
    }

    println!("[INFO] [INIT] Launching unclassified services...");
    let mut guard = G_SERVICES.lock().unwrap();
    for svc in guard.iter_mut() {
        if svc.class == "core" || svc.class == "main" {
            continue;
        }
        if let Err(e) = spawn_service(svc) {
            println!("[ERR] [INIT] Failed to spawn default service {}: {}", svc.name, e);
            return -1;
        }
    }

    0
}

fn handle_child_exit(pid: libc::pid_t, code: i32) {
    let mut guard = G_SERVICES.lock().unwrap();
    if let Some(svc) = guard.iter_mut().find(|s| s.pid == pid && s.active) {
        println!(
            "[WARN] [INIT] Service daemon '{}' [PID: {}] exited with status {}",
            svc.name, pid, code
        );
        svc.active = false;
        svc.pid = -1;
        if svc.respawn {
            let name = svc.name.clone();
            respawn_service_async(name);
        }
    } else {
        println!("[INFO] [INIT] Untracked child process [PID: {}] terminated.", pid);
    }
}

fn respawn_service_async(name: String) {
    std::thread::spawn(move || {
        println!("[WARN] [INIT] Respawning dead service daemon '{}' in 2 seconds...", name);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let mut guard = G_SERVICES.lock().unwrap();
        if let Some(svc) = guard.iter_mut().find(|s| s.name == name) {
            if let Err(e) = spawn_service(svc) {
                eprintln!("[ERR] [INIT] Failed to respawn {}: {}", name, e);
            }
        }
    });
}

fn main() {
    // Enforce running as PID 1
    if std::process::id() != 1 {
        eprintln!(
            "⚠️ [ERR] Init daemon must run as PID 1! Current PID: {}",
            std::process::id()
        );
        std::process::exit(1);
    }

    println!("\n=============================================");
    println!("📱   Booting Orion OS Init Engine (PID 1)");
    println!("=============================================\n");

    // Initialize physical virtual filesystems
    unsafe {
        if let Err(e) = mount_essential_filesystems() {
            println!("[ERR] [INIT] Fatal error during system filesystem mount phases: {}", e);
        }
    }

    // Configure loopback and ethernet interfaces
    configure_network();

    // Initialize state database
    property_init();

    // Detect Wayland boot mode from kernel command line
    if let Ok(cmdline) = std::fs::read_to_string("/proc/cmdline") {
        if cmdline.contains("wayland=1") {
            std::env::set_var("WAYLAND_ENABLED", "1");
            property_set("sys.graphics.backend", "wayland");
            println!("[INFO] [INIT] Kernel cmdline: wayland=1 detected. Enabling Wayland compositor mode.");
        } else {
            std::env::set_var("WAYLAND_ENABLED", "0");
            property_set("sys.graphics.backend", "ppm");
            println!("[INFO] [INIT] Kernel cmdline: No wayland=1. Using legacy PPM compositor mode.");
        }
    } else {
        std::env::set_var("WAYLAND_ENABLED", "0");
        property_set("sys.graphics.backend", "ppm");
        println!("[WARN] [INIT] Could not read /proc/cmdline. Defaulting to legacy PPM mode.");
    }

    // Parse standard boot configuration script
    if parse_init_rc("/etc/init.rc") != 0 {
        if parse_init_rc("init.rc") != 0 {
            println!("[WARN] [INIT] Could not read local init.rc configuration!");
        }
    }

    // Spawn registered background daemons
    launch_services();
    property_set("sys.boot_completed", "1");
    println!("[INFO] [INIT] Boot sequence completed. Entering main supervisor loop.");

    // Supervision event loop
    loop {
        let mut status = 0i32;
        let pid = unsafe { libc::waitpid(-1, &mut status, 0) };
        if pid > 0 {
            let exited = libc::WIFEXITED(status);
            let signaled = libc::WIFSIGNALED(status);
            let code = if exited {
                libc::WEXITSTATUS(status)
            } else if signaled {
                -libc::WTERMSIG(status)
            } else {
                0
            };

            handle_child_exit(pid, code);
        } else {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::ECHILD) {
                std::thread::sleep(std::time::Duration::from_secs(1));
            } else if err.raw_os_error() != Some(libc::EINTR) {
                eprintln!("[ERR] [INIT] waitpid failed: {}", err);
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }
}
