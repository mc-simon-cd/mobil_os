use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct ResourceLimits {
    pub cpu_percent: Option<f64>,
    pub memory_bytes: Option<u64>,
}

pub struct CgroupManager;

impl CgroupManager {
    /// Returns default root path for cgroups v2 hierarchy.
    pub fn get_default_cgroup_root() -> PathBuf {
        if let Ok(val) = std::env::var("ORION_CGROUP_ROOT") {
            return PathBuf::from(val);
        }
        PathBuf::from("/sys/fs/cgroup/orion/apps")
    }

    /// Sets up cgroup directory for the package, writes cpu/memory limits and attaches pid.
    pub fn setup_cgroup(package_id: &str, limits: &ResourceLimits, pid: u32) -> io::Result<PathBuf> {
        let base = Self::get_default_cgroup_root();
        let cgroup_dir = base.join(package_id);

        fs::create_dir_all(&cgroup_dir)?;

        // CPU limit: "max 100000" or "{quota} 100000"
        let cpu_max = match limits.cpu_percent {
            Some(pct) => format!("{} 100000\n", (pct.clamp(1.0, 100.0) * 1000.0) as u64),
            None => "max 100000\n".to_string(),
        };
        let _ = fs::write(cgroup_dir.join("cpu.max"), cpu_max);

        // Memory limit in bytes
        let mem_max = match limits.memory_bytes {
            Some(b) => format!("{}\n", b),
            None => "max\n".to_string(),
        };
        let _ = fs::write(cgroup_dir.join("memory.max"), mem_max);

        // Swap off: 0
        let _ = fs::write(cgroup_dir.join("memory.swap.max"), "0\n");

        // Attach pid to cgroup.procs
        let _ = fs::write(cgroup_dir.join("cgroup.procs"), format!("{}\n", pid));

        Ok(cgroup_dir)
    }

    /// Removes cgroup directory for a given package_id.
    pub fn remove_cgroup(package_id: &str) -> io::Result<()> {
        let base = Self::get_default_cgroup_root();
        let cgroup_dir = base.join(package_id);
        if cgroup_dir.exists() {
            let _ = fs::remove_dir_all(&cgroup_dir);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cgroup_setup_and_cleanup() {
        let tmp = tempdir().unwrap();
        unsafe {
            std::env::set_var("ORION_CGROUP_ROOT", tmp.path());
        }

        let limits = ResourceLimits {
            cpu_percent: Some(50.0),
            memory_bytes: Some(512 * 1024 * 1024),
        };

        let cgroup_dir = CgroupManager::setup_cgroup("com.test.cgroupapp", &limits, 12345).unwrap();
        assert!(cgroup_dir.exists());

        let cpu = fs::read_to_string(cgroup_dir.join("cpu.max")).unwrap();
        assert_eq!(cpu.trim(), "50000 100000");

        let mem = fs::read_to_string(cgroup_dir.join("memory.max")).unwrap();
        assert_eq!(mem.trim(), "536870912");

        let procs = fs::read_to_string(cgroup_dir.join("cgroup.procs")).unwrap();
        assert_eq!(procs.trim(), "12345");

        CgroupManager::remove_cgroup("com.test.cgroupapp").unwrap();
        assert!(!cgroup_dir.exists());
    }
}
