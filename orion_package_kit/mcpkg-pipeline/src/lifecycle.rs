use mcpkg_core::{Manifest, SandboxPolicy};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: String,
    pub install_path: PathBuf,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfoSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub author_email: String,
    pub min_os: String,
    pub max_os: Option<String>,
    pub entry_point: String,
    pub exec_type: String,
    pub category: String,
    pub price_usd: f64,
    pub tier: String,
    pub is_installed: bool,
    pub install_path: Option<PathBuf>,
    pub permissions: Vec<String>,
}

pub struct LifecycleManager;

impl LifecycleManager {
    /// Determines the default root directory for installed applications.
    pub fn get_default_apps_root() -> PathBuf {
        if let Ok(val) = std::env::var("ORION_APPS_ROOT") {
            return PathBuf::from(val);
        }
        
        let opt_apps = PathBuf::from("/opt/orion/apps");
        if opt_apps.exists() {
            if let Ok(meta) = fs::metadata(&opt_apps) {
                if !meta.permissions().readonly() {
                    return opt_apps;
                }
            }
        }
        
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".local/share/orion/apps");
        }
        
        PathBuf::from("/tmp/orion/apps")
    }

    /// Returns the standard desktop entry directory.
    pub fn get_default_desktop_dir() -> Result<PathBuf, String> {
        if let Ok(val) = std::env::var("ORION_DESKTOP_DIR") {
            return Ok(PathBuf::from(val));
        }
        let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
        Ok(PathBuf::from(home).join(".local/share/applications"))
    }

    /// Returns the standard application icons directory.
    pub fn get_default_icons_dir() -> Result<PathBuf, String> {
        let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
        Ok(PathBuf::from(home).join(".local/share/icons/orion"))
    }

    /// Lists all currently installed applications in apps_root.
    pub fn list_installed(apps_root: &Path) -> Result<Vec<InstalledApp>, String> {
        if !apps_root.exists() {
            return Ok(Vec::new());
        }

        let mut apps = Vec::new();

        let entries = fs::read_dir(apps_root)
            .map_err(|e| format!("Failed to read apps directory {:?}: {}", apps_root, e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("manifest.toml");
                if manifest_path.exists() {
                    if let Ok(content) = fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = Manifest::from_toml(&content) {
                            let size_bytes = calculate_dir_size(&path);
                            apps.push(InstalledApp {
                                id: manifest.package.id,
                                name: manifest.package.name,
                                version: manifest.package.version,
                                description: manifest.package.description,
                                author: manifest.package.author,
                                category: manifest.store.category,
                                install_path: path,
                                size_bytes,
                            });
                        }
                    }
                }
            }
        }

        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(apps)
    }

    /// Inspects an application by reading from an installed folder or a local .opk package.
    pub fn get_info(path_or_id: &str, apps_root: &Path) -> Result<AppInfoSummary, String> {
        let path = Path::new(path_or_id);

        if path.exists() && path.is_file() {
            // Read from local .opk archive
            let file = fs::File::open(path)
                .map_err(|e| format!("Failed to open package {:?}: {}", path, e))?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| format!("Invalid ZIP archive: {}", e))?;

            let mut manifest_content = String::new();
            {
                use std::io::Read;
                let mut f = archive.by_name("manifest.toml")
                    .map_err(|_| "manifest.toml not found in package".to_string())?;
                f.read_to_string(&mut manifest_content)
                    .map_err(|e| format!("Failed to read manifest.toml: {}", e))?;
            }
            let manifest = Manifest::from_toml(&manifest_content)
                .map_err(|e| format!("Invalid manifest: {}", e))?;

            let mut policy_content = String::new();
            if let Ok(mut f) = archive.by_name("sandbox.policy") {
                use std::io::Read;
                let _ = f.read_to_string(&mut policy_content);
            }
            let policy = SandboxPolicy::from_toml(&policy_content).ok();

            let permissions = extract_permission_strings(policy.as_ref());

            return Ok(AppInfoSummary {
                id: manifest.package.id,
                name: manifest.package.name,
                version: manifest.package.version,
                description: manifest.package.description,
                author: manifest.package.author,
                author_email: manifest.package.author_email,
                min_os: manifest.package.min_os,
                max_os: manifest.package.max_os,
                entry_point: manifest.build.entry_x86_64,
                exec_type: manifest.build.exec_type,
                category: manifest.store.category,
                price_usd: manifest.store.price_usd,
                tier: manifest.store.tier,
                is_installed: false,
                install_path: None,
                permissions,
            });
        }

        // Try to find in installed apps
        let app_dir = apps_root.join(path_or_id);
        if app_dir.exists() && app_dir.is_dir() {
            let manifest_path = app_dir.join("manifest.toml");
            let manifest_content = fs::read_to_string(&manifest_path)
                .map_err(|e| format!("Failed to read manifest for {}: {}", path_or_id, e))?;
            let manifest = Manifest::from_toml(&manifest_content)
                .map_err(|e| format!("Invalid manifest: {}", e))?;

            let policy_path = app_dir.join("sandbox.policy");
            let policy_content = fs::read_to_string(&policy_path).unwrap_or_default();
            let policy = SandboxPolicy::from_toml(&policy_content).ok();

            let permissions = extract_permission_strings(policy.as_ref());

            return Ok(AppInfoSummary {
                id: manifest.package.id,
                name: manifest.package.name,
                version: manifest.package.version,
                description: manifest.package.description,
                author: manifest.package.author,
                author_email: manifest.package.author_email,
                min_os: manifest.package.min_os,
                max_os: manifest.package.max_os,
                entry_point: manifest.build.entry_x86_64,
                exec_type: manifest.build.exec_type,
                category: manifest.store.category,
                price_usd: manifest.store.price_usd,
                tier: manifest.store.tier,
                is_installed: true,
                install_path: Some(app_dir),
                permissions,
            });
        }

        Err(format!("Package '{}' not found locally or among installed apps", path_or_id))
    }

    /// Uninstalls an installed application cleanly.
    pub fn uninstall(
        app_id: &str,
        apps_root: &Path,
        desktop_dir: Option<&Path>,
        icons_dir: Option<&Path>,
        purge: bool,
    ) -> Result<(), String> {
        let app_dir = apps_root.join(app_id);
        if !app_dir.exists() {
            return Err(format!("Application '{}' is not installed at {:?}", app_id, apps_root));
        }

        println!("Uninstalling {}...", app_id);

        // 1. Terminate running processes belonging to this app
        terminate_app_processes(&app_dir);

        // 2. Remove AppArmor Profile
        unload_apparmor_profile(app_id);

        // 3. Remove Desktop Entry
        let d_dir = match desktop_dir {
            Some(d) => d.to_path_buf(),
            None => Self::get_default_desktop_dir().unwrap_or_else(|_| PathBuf::from("/tmp")),
        };
        let desktop_file = d_dir.join(format!("{}.desktop", app_id));
        if desktop_file.exists() {
            if let Err(e) = fs::remove_file(&desktop_file) {
                println!("  ⚠️ Warning: Could not remove desktop entry: {}", e);
            } else {
                println!("  Removed desktop entry: {:?}", desktop_file);
            }
        }

        // 4. Remove Icon File
        let i_dir = match icons_dir {
            Some(i) => i.to_path_buf(),
            None => Self::get_default_icons_dir().unwrap_or_else(|_| PathBuf::from("/tmp")),
        };
        let icon_file = i_dir.join(format!("{}.png", app_id));
        if icon_file.exists() {
            if let Err(e) = fs::remove_file(&icon_file) {
                println!("  ⚠️ Warning: Could not remove icon file: {}", e);
            } else {
                println!("  Removed icon file: {:?}", icon_file);
            }
        }

        // 5. Remove App Directory from apps_root
        fs::remove_dir_all(&app_dir)
            .map_err(|e| format!("Failed to remove app directory {:?}: {}", app_dir, e))?;
        println!("  Removed app directory: {:?}", app_dir);

        // 6. Optional purge of user data
        if purge {
            if let Ok(home) = std::env::var("HOME") {
                let user_data = PathBuf::from(&home).join(format!(".local/share/{}", app_id));
                let user_config = PathBuf::from(&home).join(format!(".config/{}", app_id));

                if user_data.exists() {
                    let _ = fs::remove_dir_all(&user_data);
                    println!("  Purged user data: {:?}", user_data);
                }
                if user_config.exists() {
                    let _ = fs::remove_dir_all(&user_config);
                    println!("  Purged user config: {:?}", user_config);
                }
            }
        } else {
            println!("  ℹ️ User data preserved in ~/.local/share/{} (use --purge to delete)", app_id);
        }

        // 6.5. Remove cgroups v2 directory
        let _ = mcpkg_sandbox::CgroupManager::remove_cgroup(app_id);

        // 7. Write audit log
        let audit_db = crate::bekci::AuditLogger::get_default_db_path();
        let audit_entry = crate::bekci::AuditEntry {
            id: None,
            timestamp: crate::bekci::get_current_timestamp_iso(),
            action: "uninstall".to_string(),
            package_id: app_id.to_string(),
            version: String::new(),
            trust_level: crate::bekci::TrustLevel::Unsigned,
            risk_level: crate::bekci::RiskLevel::Low,
            risk_score: 0,
            user_choice: "approved".to_string(),
            warnings: if purge { vec!["Kullanıcı verileri silindi (--purge)".to_string()] } else { Vec::new() },
            checksum_ok: true,
            sig_ok: true,
            key_fp: None,
        };
        let _ = crate::bekci::AuditLogger::log_event(&audit_db, &audit_entry);

        println!("✅ Successfully uninstalled {}", app_id);
        Ok(())
    }
}

// ---------------- Helper Functions ----------------

fn calculate_dir_size(dir: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = fs::metadata(&path) {
                    total += meta.len();
                }
            } else if path.is_dir() {
                total += calculate_dir_size(&path);
            }
        }
    }
    total
}

fn extract_permission_strings(policy: Option<&SandboxPolicy>) -> Vec<String> {
    use mcpkg_core::policy::Permission;
    let mut perms = Vec::new();
    if let Some(p) = policy {
        if p.network.internet == Permission::Allow {
            perms.push("Network: Full Internet Access".to_string());
        }
        if p.filesystem.home_read == Permission::Allow || p.filesystem.home_write == Permission::Allow {
            perms.push("Filesystem: App Data Directory Access".to_string());
        }
        if p.filesystem.documents_read == Permission::Allow || p.filesystem.documents_read == Permission::Ask {
            perms.push("Filesystem: Documents Directory Access".to_string());
        }
        if p.hardware.camera == Permission::Allow || p.hardware.camera == Permission::Ask {
            perms.push("Hardware: Camera Access".to_string());
        }
        if p.hardware.microphone == Permission::Allow || p.hardware.microphone == Permission::Ask {
            perms.push("Hardware: Microphone Access".to_string());
        }
        if p.hardware.location == Permission::Allow || p.hardware.location == Permission::Ask {
            perms.push("Hardware: Geolocation Access".to_string());
        }
        if p.system.notifications == Permission::Allow {
            perms.push("System: Desktop Notifications".to_string());
        }
    }
    perms
}

fn terminate_app_processes(app_dir: &Path) {
    let app_dir_canonical = match app_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => app_dir.to_path_buf(),
    };
    let app_dir_str = app_dir_canonical.to_string_lossy();

    if let Ok(proc_entries) = fs::read_dir("/proc") {
        for entry in proc_entries.flatten() {
            let name = entry.file_name();
            if let Ok(pid) = name.to_string_lossy().parse::<i32>() {
                let exe_link = PathBuf::from(format!("/proc/{}/exe", pid));
                if let Ok(target) = fs::read_link(&exe_link) {
                    if target.to_string_lossy().starts_with(app_dir_str.as_ref()) {
                        println!("  Terminating running app process (PID: {})...", pid);
                        let _ = nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), nix::sys::signal::Signal::SIGTERM);
                    }
                }
            }
        }
    }
}

fn unload_apparmor_profile(app_id: &str) {
    let _ = std::process::Command::new("apparmor_parser")
        .args(["-R", &format!("/etc/apparmor.d/orion/{}", app_id)])
        .output();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_list_installed_empty() {
        let tmp = tempdir().unwrap();
        let apps = LifecycleManager::list_installed(tmp.path()).unwrap();
        assert!(apps.is_empty());
    }

    #[test]
    fn test_list_and_uninstall_app() {
        let tmp = tempdir().unwrap();
        let apps_root = tmp.path().join("apps");
        let desktop_dir = tmp.path().join("desktop");
        let icons_dir = tmp.path().join("icons");

        fs::create_dir_all(&apps_root).unwrap();
        fs::create_dir_all(&desktop_dir).unwrap();
        fs::create_dir_all(&icons_dir).unwrap();

        // Create mock installed app
        let app_dir = apps_root.join("com.test.app");
        fs::create_dir_all(&app_dir).unwrap();

        let manifest_content = r#"[package]
id = "com.test.app"
name = "Test App"
version = "1.0.0"
description = "Test Description"
author = "Tester"
author_email = "test@example.com"
min_os = "1.0.0"

[build]
entry_x86_64 = "binaries/x86_64/test"
exec_type = "elf"

[store]
category = "productivity"
rating = "ALL"
price_usd = 0.00
tier = "free"
"#;
        fs::write(app_dir.join("manifest.toml"), manifest_content).unwrap();
        fs::write(app_dir.join("dummy_bin"), "binary").unwrap();

        // Create mock desktop and icon files
        let desktop_file = desktop_dir.join("com.test.app.desktop");
        fs::write(&desktop_file, "[Desktop Entry]\nName=Test App").unwrap();

        let icon_file = icons_dir.join("com.test.app.png");
        fs::write(&icon_file, "fake_png_data").unwrap();

        // 1. List installed apps
        let apps = LifecycleManager::list_installed(&apps_root).unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].id, "com.test.app");
        assert_eq!(apps[0].name, "Test App");

        // 2. Get info
        let info = LifecycleManager::get_info("com.test.app", &apps_root).unwrap();
        assert_eq!(info.id, "com.test.app");
        assert!(info.is_installed);

        // 3. Uninstall app
        let res = LifecycleManager::uninstall(
            "com.test.app",
            &apps_root,
            Some(&desktop_dir),
            Some(&icons_dir),
            false,
        );
        assert!(res.is_ok());

        assert!(!app_dir.exists());
        assert!(!desktop_file.exists());
        assert!(!icon_file.exists());

        // Verify list is now empty
        let apps_after = LifecycleManager::list_installed(&apps_root).unwrap();
        assert!(apps_after.is_empty());
    }
}
