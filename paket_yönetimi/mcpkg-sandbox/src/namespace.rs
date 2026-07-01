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
}
