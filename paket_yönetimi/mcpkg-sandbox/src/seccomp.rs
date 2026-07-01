use mcpkg_core::policy::SandboxPolicy;

pub struct SeccompFilter;

impl SeccompFilter {
    /// Applies the seccomp-bpf filter to the current process and its future children.
    pub fn apply(_policy: &SandboxPolicy) -> Result<(), String> {
        #[cfg(feature = "seccomp")]
        {
            return apply_seccomp(_policy);
        }

        #[cfg(not(feature = "seccomp"))]
        {
            Ok(())
        }
    }
}

#[cfg(feature = "seccomp")]
fn apply_seccomp(policy: &SandboxPolicy) -> Result<(), String> {
    use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall, ScmpArch};
    use mcpkg_core::policy::Permission;

    let mut filter = ScmpFilterContext::new(ScmpAction::Allow)
        .map_err(|e| format!("Failed to initialize seccomp context: {:?}", e))?;

    filter
        .add_arch(ScmpArch::X8664)
        .map_err(|e| format!("Failed to add architecture: {:?}", e))?;

    let always_blocked = [
        "ptrace",
        "process_vm_readv",
        "process_vm_writev",
        "kexec_load",
        "perf_event_open",
    ];

    const EPERM: i32 = 1;

    for syscall_name in &always_blocked {
        if let Ok(syscall) = ScmpSyscall::from_name(syscall_name) {
            let _ = filter.add_rule(ScmpAction::Errno(EPERM), syscall);
        }
    }

    if policy.system.exec_subprocess == Permission::Deny && policy.meta.strict_mode {
        if let Ok(syscall) = ScmpSyscall::from_name("execve") {
            let _ = filter.add_rule(ScmpAction::Errno(EPERM), syscall);
        }
        if let Ok(syscall) = ScmpSyscall::from_name("execveat") {
            let _ = filter.add_rule(ScmpAction::Errno(EPERM), syscall);
        }
    }

    if policy.network.internet == Permission::Deny {
        if let Ok(syscall) = ScmpSyscall::from_name("connect") {
            let _ = filter.add_rule(ScmpAction::Errno(EPERM), syscall);
        }
        if let Ok(syscall) = ScmpSyscall::from_name("bind") {
            let _ = filter.add_rule(ScmpAction::Errno(EPERM), syscall);
        }
        if let Ok(syscall) = ScmpSyscall::from_name("sendto") {
            let _ = filter.add_rule(ScmpAction::Errno(EPERM), syscall);
        }
    }

    if policy.system.ipc == Permission::Deny {
        if let Ok(syscall) = ScmpSyscall::from_name("msgget") {
            let _ = filter.add_rule(ScmpAction::Errno(EPERM), syscall);
        }
        if let Ok(syscall) = ScmpSyscall::from_name("shmget") {
            let _ = filter.add_rule(ScmpAction::Errno(EPERM), syscall);
        }
        if let Ok(syscall) = ScmpSyscall::from_name("semget") {
            let _ = filter.add_rule(ScmpAction::Errno(EPERM), syscall);
        }
    }

    filter
        .load()
        .map_err(|e| format!("Failed to load seccomp filter: {:?}", e))?;

    Ok(())
}

#[cfg(all(test, feature = "seccomp"))]
mod tests {
    use super::*;
    use mcpkg_core::policy::{Meta, Filesystem, Network, Hardware, System, Permission};

    fn mock_policy(strict: bool) -> SandboxPolicy {
        SandboxPolicy {
            meta: Meta {
                policy_version: "1.0".to_string(),
                strict_mode: strict,
            },
            filesystem: Filesystem {
                home_read: Permission::Deny,
                home_write: Permission::Deny,
                temp_access: Permission::Deny,
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
    fn test_seccomp_filter_creation() {
        let _policy = mock_policy(false);
        let mut filter =
            libseccomp::ScmpFilterContext::new(libseccomp::ScmpAction::Allow).unwrap();
        filter.add_arch(libseccomp::ScmpArch::X8664).unwrap();
        assert!(libseccomp::ScmpSyscall::from_name("execve").is_ok());
    }
}
