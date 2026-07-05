use mcpkg_core::policy::{SandboxPolicy, Permission};

pub struct AppArmorGenerator;

impl AppArmorGenerator {
    pub fn generate_profile(package_id: &str, policy: &SandboxPolicy) -> String {
        let mut profile = format!("profile {} {{\n", package_id);

        // Filesystem permissions
        profile.push_str("  # Filesystem access\n");
        if policy.filesystem.home_read == Permission::Allow {
            profile.push_str(&format!("  owner /home/*/.local/share/{}/** r,\n", package_id));
        }
        if policy.filesystem.home_write == Permission::Allow {
            profile.push_str(&format!("  owner /home/*/.local/share/{}/** rw,\n", package_id));
            profile.push_str(&format!("  owner /home/*/.config/{}/** rw,\n", package_id));
        }
        if policy.filesystem.temp_access == Permission::Allow {
            profile.push_str(&format!("  /tmp/mcsimon-{}/** rw,\n", package_id));
        }
        if policy.filesystem.system_read == Permission::Deny {
            profile.push_str("  deny /etc/** r,\n");
            profile.push_str("  deny /usr/** rw,\n");
        }

        // Network permissions
        profile.push_str("\n  # Network access\n");
        if policy.network.internet == Permission::Allow {
            profile.push_str("  network inet stream,\n");
            profile.push_str("  network inet6 stream,\n");
        } else {
            profile.push_str("  deny network inet,\n");
            profile.push_str("  deny network inet6,\n");
        }

        profile.push_str("}\n");
        profile
    }

    pub fn load_profile(package_id: &str, profile_content: &str) -> std::io::Result<()> {
        use std::process::{Command, Stdio};
        use std::io::Write;

        // Check if apparmor_parser exists
        if let Err(_) = Command::new("apparmor_parser").arg("--version").output() {
            println!("  WARNING: apparmor_parser not found. Skipping profile load (Simulated).");
            return Ok(());
        }

        let mut child = Command::new("apparmor_parser")
            .arg("-r") // Replace existing profile
            .stdin(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to open stdin to apparmor_parser")
        })?;

        stdin.write_all(profile_content.as_bytes())?;
        drop(stdin);

        let status = child.wait()?;
        if !status.success() {
            println!("  WARNING: apparmor_parser failed (exit code: {:?}). This is expected if you are running as an unprivileged user. Skipping AppArmor enforcement.", status.code());
            return Ok(());
        }

        println!("  AppArmor profile loaded for {}", package_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcpkg_core::policy::{Meta, Filesystem, Network, Hardware, System};

    fn mock_policy() -> SandboxPolicy {
        SandboxPolicy {
            meta: Meta {
                policy_version: "1.0".to_string(),
                strict_mode: false,
            },
            filesystem: Filesystem {
                home_read: Permission::Allow,
                home_write: Permission::Allow,
                temp_access: Permission::Allow,
                documents_read: Permission::Deny,
                documents_write: Permission::Deny,
                downloads_read: Permission::Deny,
                system_read: Permission::Deny,
                arbitrary_path: Permission::Deny,
            },
            network: Network {
                internet: Permission::Deny,
                localhost: Permission::Deny,
                local_network: Permission::Deny,
            },
            hardware: Hardware {
                camera: Permission::Deny,
                microphone: Permission::Deny,
                gpu_access: Permission::Deny,
                usb_devices: Permission::Deny,
                bluetooth: Permission::Deny,
                location: Permission::Deny,
            },
            system: System {
                notifications: Permission::Deny,
                clipboard_read: Permission::Deny,
                clipboard_write: Permission::Deny,
                autostart: Permission::Deny,
                background_run: Permission::Deny,
                ipc: Permission::Deny,
                exec_subprocess: Permission::Deny,
            },
        }
    }

    #[test]
    fn test_apparmor_generator() {
        let policy = mock_policy();
        let profile = AppArmorGenerator::generate_profile("com.test.app", &policy);
        
        assert!(profile.contains("profile com.test.app"));
        assert!(profile.contains("owner /home/*/.local/share/com.test.app/** rw"));
        assert!(profile.contains("deny network inet"));
    }
}
