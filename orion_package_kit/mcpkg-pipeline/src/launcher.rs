use mcpkg_core::{Manifest, SandboxPolicy};
use mcpkg_sandbox::{NamespaceManager, AppArmorGenerator, SeccompFilter};
use std::path::PathBuf;
use std::process::Command;

pub struct Launcher;

impl Launcher {
    pub fn launch(app_id: &str, apps_root: &PathBuf) -> Result<(), String> {
        let app_dir = apps_root.join(app_id);
        if !app_dir.exists() {
            return Err(format!("App {} not found at {:?}", app_id, app_dir));
        }

        // 1. Load manifest and policy
        let manifest_path = app_dir.join("manifest.toml");
        let manifest_content = std::fs::read_to_string(manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        let manifest = Manifest::from_toml(&manifest_content)
            .map_err(|e| format!("Invalid manifest: {}", e))?;

        let policy_path = app_dir.join("sandbox.policy");
        let policy_content = std::fs::read_to_string(policy_path)
            .map_err(|e| format!("Failed to read policy: {}", e))?;
        let policy = SandboxPolicy::from_toml(&policy_content)
            .map_err(|e| format!("Invalid policy: {}", e))?;

        println!("Launching {}...", manifest.package.name);

        // 2. Setup Sandbox
        println!("  Setting up sandbox...");
        
        // a. Generate and Load AppArmor Profile
        let profile_content = AppArmorGenerator::generate_profile(app_id, &policy);
        AppArmorGenerator::load_profile(app_id, &profile_content)
            .map_err(|e| format!("Failed to load AppArmor profile: {}", e))?;

        // 3. Select Binary according to Host Architecture
        let host_arch = std::env::consts::ARCH;
        let entry_rel = if host_arch == "aarch64" {
            manifest.build.entry_aarch64.as_deref().unwrap_or(&manifest.build.entry_x86_64)
        } else {
            &manifest.build.entry_x86_64
        };
        let binary_path = app_dir.join(entry_rel);
        println!("  Target architecture: {} (Binary: {:?})", host_arch, binary_path);

        use std::os::unix::process::CommandExt;
        let mut command = Command::new(&binary_path);

        let uid = nix::unistd::getuid();
        let gid = nix::unistd::getgid();
        let policy_clone = policy.clone();
        let app_id_owned = app_id.to_string();

        unsafe {
            command.pre_exec(move || {
                // b. Enter Namespaces (Mount, PID, Net, User)
                match NamespaceManager::enter_namespaces() {
                    Ok(_) => {
                        // Setup user UID/GID mapping in rootless user namespace
                        if let Err(e) = NamespaceManager::setup_user_mappings(uid.as_raw(), gid.as_raw()) {
                            if policy_clone.meta.strict_mode {
                                return Err(e);
                            }
                        }
                        // Setup advanced mount isolation (private mounts, private tmp)
                        if let Err(e) = NamespaceManager::setup_mount_isolation(&app_id_owned, &policy_clone) {
                            if policy_clone.meta.strict_mode {
                                return Err(e);
                            }
                        }
                    }
                    Err(e) => {
                        if policy_clone.meta.strict_mode {
                            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to enter namespaces: {}", e)));
                        }
                    }
                }

                // c. Apply Seccomp BPF filter
                if let Err(e) = SeccompFilter::apply(&policy_clone) {
                    if policy_clone.meta.strict_mode {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
                    }
                }

                Ok(())
            });
        }

        let mut child = command.spawn()
            .map_err(|e| format!("Failed to execute binary: {}", e))?;

        // d. Setup cgroups v2 limits and attach spawned process
        let limits = mcpkg_sandbox::ResourceLimits {
            cpu_percent: manifest.resources.as_ref().and_then(|r| r.cpu_percent),
            memory_bytes: manifest.resources.as_ref().and_then(|r| r.memory_mb).map(|mb| mb * 1024 * 1024),
        };
        if let Err(e) = mcpkg_sandbox::CgroupManager::setup_cgroup(app_id, &limits, child.id()) {
            println!("  ℹ️ cgroups v2 setup notice: {}", e);
        }

        let status = child.wait().map_err(|e| format!("Process execution failed: {}", e))?;
        let _ = mcpkg_sandbox::CgroupManager::remove_cgroup(app_id);
        println!("App exited with status: {}", status);

        Ok(())
    }
}
