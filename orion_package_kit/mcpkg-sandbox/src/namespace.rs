use nix::sched::{unshare, CloneFlags};
use std::process::Command;
use std::io;

pub struct NamespaceManager;

impl NamespaceManager {
    /// Unshares the current process into new namespaces.
    pub fn enter_namespaces() -> nix::Result<()> {
        unshare(
            CloneFlags::CLONE_NEWNS   // Mount
            | CloneFlags::CLONE_NEWPID  // PID
            | CloneFlags::CLONE_NEWNET  // Network
            | CloneFlags::CLONE_NEWUSER // User
        )
    }

    /// Sets up a mount namespace by making all mounts private.
    pub fn setup_mount_namespace() -> io::Result<()> {
        // Equivalent to mount --make-rprivate /
        let status = Command::new("mount")
            .args(&["--make-rprivate", "/"])
            .status()?;
        
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to make mounts private"));
        }
        Ok(())
    }

    /// Sets up UID and GID mappings for rootless user namespace.
    pub fn setup_user_mappings(outer_uid: u32, outer_gid: u32) -> io::Result<()> {
        use std::fs;
        // Write "deny" to /proc/self/setgroups before writing to gid_map
        let _ = fs::write("/proc/self/setgroups", "deny\n");
        fs::write("/proc/self/uid_map", format!("0 {} 1\n", outer_uid))?;
        fs::write("/proc/self/gid_map", format!("0 {} 1\n", outer_gid))?;
        Ok(())
    }

    /// Sets up mount isolation based on sandbox policy:
    /// - Makes mounts private (MS_REC | MS_PRIVATE)
    /// - Creates and mounts isolated application private /tmp directory
    /// - If filesystem.system_read is Allowed, /etc is mounted as read-only
    pub fn setup_mount_isolation(package_id: &str, policy: &mcpkg_core::SandboxPolicy) -> io::Result<()> {
        // 1. Make all existing mounts private
        Self::setup_mount_namespace()?;

        // 2. Private tmp mount
        let tmp_dir = format!("/tmp/orion-{}", package_id);
        let _ = std::fs::create_dir_all(&tmp_dir);
        let _ = nix::mount::mount(
            Some(tmp_dir.as_str()),
            "/tmp",
            None::<&str>,
            nix::mount::MsFlags::MS_BIND,
            None::<&str>,
        );

        // 3. Read-only system mounts if allowed
        if policy.filesystem.system_read == mcpkg_core::policy::Permission::Allow {
            let _ = nix::mount::mount(
                Some("/etc"),
                "/etc",
                None::<&str>,
                nix::mount::MsFlags::MS_BIND | nix::mount::MsFlags::MS_RDONLY,
                None::<&str>,
            );
        }

        Ok(())
    }
}
