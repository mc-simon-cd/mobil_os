pub mod launcher;
pub use launcher::Launcher;
pub mod registry;
pub mod developer;
use mcpkg_core::Manifest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::io;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::time::Duration;
use std::os::unix::process::CommandExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallState {
    Idle,
    VerifyingChecksums,
    EvaluatingGatekeeper,
    AwaitingUserConfirmation,
    PreparingContainer,
    RunningPreInstall,
    PlacingFiles,
    RunningPostInstall,
    WritingAuditLog,
    Completed,
    Failed { stage: String, error: String },
}

pub struct PipelineManager {
    pub package_path: PathBuf,
    pub state: InstallState,
    pub apps_root: PathBuf,
    pub desktop_dir: Option<PathBuf>,
}

impl PipelineManager {
    pub fn new(package_path: PathBuf) -> Self {
        Self {
            package_path,
            state: InstallState::Idle,
            apps_root: PathBuf::from("/tmp/orion/apps"),
            desktop_dir: None,
        }
    }

    fn get_desktop_dir(&self) -> Result<PathBuf, String> {
        if let Some(ref dir) = self.desktop_dir {
            return Ok(dir.clone());
        }
        let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".local/share/applications"))
    }

    pub fn run_install(&mut self) -> Result<(), String> {
        let app_id_ref = Arc::new(Mutex::new(None));
        let has_backup_ref = Arc::new(Mutex::new(false));
        let created_desktop_ref = Arc::new(Mutex::new(false));

        let res = self.run_install_impl(
            Arc::clone(&app_id_ref),
            Arc::clone(&has_backup_ref),
            Arc::clone(&created_desktop_ref),
        );

        if let Err(e) = res {
            // Rollback!
            let app_id = app_id_ref.lock().unwrap().clone();
            let has_backup = *has_backup_ref.lock().unwrap();
            let created_desktop = *created_desktop_ref.lock().unwrap();

            if let Some(id) = app_id {
                let target_dir = self.apps_root.join(&id);
                let backup_dir = self.apps_root.join(format!("{}.backup", id));

                // 1. Delete new target directory if it exists
                if target_dir.exists() {
                    let _ = std::fs::remove_dir_all(&target_dir);
                }

                // 2. If a backup was made, restore it
                if has_backup && backup_dir.exists() {
                    let _ = std::fs::rename(&backup_dir, &target_dir);
                }

                // 3. Delete desktop entry if created
                if created_desktop {
                    if let Ok(desktop_dir) = self.get_desktop_dir() {
                        let desktop_path = desktop_dir.join(format!("{}.desktop", id));
                        if desktop_path.exists() {
                            let _ = std::fs::remove_file(&desktop_path);
                        }
                    }
                }
            }

            self.state = InstallState::Failed {
                stage: match self.state {
                    InstallState::Idle => "Idle".to_string(),
                    InstallState::VerifyingChecksums => "VerifyingChecksums".to_string(),
                    InstallState::EvaluatingGatekeeper => "EvaluatingGatekeeper".to_string(),
                    InstallState::AwaitingUserConfirmation => "AwaitingUserConfirmation".to_string(),
                    InstallState::PreparingContainer => "PreparingContainer".to_string(),
                    InstallState::RunningPreInstall => "RunningPreInstall".to_string(),
                    InstallState::PlacingFiles => "PlacingFiles".to_string(),
                    InstallState::RunningPostInstall => "RunningPostInstall".to_string(),
                    InstallState::WritingAuditLog => "WritingAuditLog".to_string(),
                    InstallState::Completed => "Completed".to_string(),
                    InstallState::Failed { ref stage, .. } => stage.clone(),
                },
                error: e.clone(),
            };

            return Err(e);
        }

        Ok(())
    }

    fn run_install_impl(
        &mut self,
        app_id_ref: Arc<Mutex<Option<String>>>,
        has_backup_ref: Arc<Mutex<bool>>,
        created_desktop_ref: Arc<Mutex<bool>>,
    ) -> Result<(), String> {
        // 1. Open ZIP archive
        let file = std::fs::File::open(&self.package_path)
            .map_err(|e| format!("Failed to open package: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Invalid ZIP archive: {}", e))?;

        let manifest: Manifest;
        let policy: mcpkg_core::SandboxPolicy;
        let checksums: Vec<mcpkg_core::checksum::ChecksumEntry>;

        // Scope manifest and checksum reading
        {
            // 2. Read manifest.toml
            {
                let mut manifest_file = archive.by_name("manifest.toml")
                    .map_err(|_| "manifest.toml not found in package")?;
                let mut manifest_content = String::new();
                use std::io::Read;
                manifest_file.read_to_string(&mut manifest_content)
                    .map_err(|e| format!("Failed to read manifest: {}", e))?;
                manifest = mcpkg_core::Manifest::from_toml(&manifest_content)
                    .map_err(|e| format!("Invalid manifest.toml: {}", e))?;
            }

            // Read sandbox.policy
            {
                let mut policy_file = archive.by_name("sandbox.policy")
                    .map_err(|_| "sandbox.policy not found in package")?;
                let mut policy_content = String::new();
                use std::io::Read;
                policy_file.read_to_string(&mut policy_content)
                    .map_err(|e| format!("Failed to read policy: {}", e))?;
                policy = mcpkg_core::SandboxPolicy::from_toml(&policy_content)
                    .map_err(|e| format!("Invalid sandbox.policy: {}", e))?;
            }

            // 3. Read checksums.sha256
            {
                let mut checksums_file = archive.by_name("checksums.sha256")
                    .map_err(|_| "checksums.sha256 not found in package")?;
                let mut content = String::new();
                use std::io::Read;
                checksums_file.read_to_string(&mut content)
                    .map_err(|e| format!("Failed to read checksums: {}", e))?;
                checksums = mcpkg_core::checksum::ChecksumListParser::parse(&content);
            }
        }

        let app_id = manifest.package.id.clone();
        *app_id_ref.lock().unwrap() = Some(app_id.clone());

        println!("Package: {} v{}", manifest.package.id, manifest.package.version);

        // 4. Verify Checksums
        self.state = InstallState::VerifyingChecksums;
        println!("Stage: Verifying Checksums...");

        // 5. Gatekeeper Evaluation (Mocked Key for now)
        self.state = InstallState::EvaluatingGatekeeper;
        println!("Stage: Evaluating Gatekeeper...");

        // a. Read signature.ed25519
        let mut signature_bytes = [0u8; 64];
        {
            let mut sig_file = archive.by_name("signature.ed25519")
                .map_err(|_| "signature.ed25519 not found in package")?;
            use std::io::Read;
            sig_file.read_exact(&mut signature_bytes)
                .map_err(|e| format!("Failed to read signature: {}", e))?;
        }
        let signature = mcpkg_crypto::signing::CryptoManager::signature_from_bytes(&signature_bytes)
            .map_err(|e| format!("Invalid signature format: {}", e))?;

        // b. Calculate signed message (aggregate hash) using robust developer tool logic
        let message = crate::developer::calculate_aggregate_message(&mut archive, &manifest)
            .map_err(|e| format!("Failed to calculate aggregate message for signature verification: {}", e))?;

        // c. Resolve verifying key
        let default_dir = crate::developer::DeveloperTools::get_default_key_dir()?;
        let default_path = default_dir.join("public.key");
        let verifying_key = if default_path.exists() {
            let content = std::fs::read_to_string(&default_path)
                .map_err(|e| format!("Failed to read default public key: {}", e))?;
            crate::developer::parse_public_key(&content)?
        } else {
            let mock_public_key_bytes = [0u8; 32];
            ed25519_dalek::VerifyingKey::from_bytes(&mock_public_key_bytes)
                .map_err(|e| format!("Invalid public key: {}", e))?
        };

        // d. Verification
        match mcpkg_crypto::signing::CryptoManager::verify(&verifying_key, &message, &signature) {
            Ok(_) => {
                println!("  Signature Verified (TRUSTED)");
                println!("  Key Fingerprint: {}", crate::developer::calculate_fingerprint(verifying_key.as_bytes()));
            }
            Err(_) => println!("  WARNING: Signature Verification FAILED (UNTRUSTED)"),
        }

        // 6. User Confirmation
        self.state = InstallState::AwaitingUserConfirmation;
        println!("Stage: Awaiting User Confirmation...");

        // 7. Prepare Container & Staging
        self.state = InstallState::PreparingContainer;
        println!("Stage: Preparing Container & Staging...");
        let staging_dir = tempfile::tempdir()
            .map_err(|e| format!("Failed to create staging dir: {}", e))?;

        // 8. Extract Files to Staging
        self.state = InstallState::PlacingFiles;
        println!("Stage: Extracting Files to Staging...");
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to access file in zip: {}", e))?;
            let outpath = match file.enclosed_name() {
                Some(path) => staging_dir.path().join(path),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                std::fs::create_dir_all(&outpath)
                    .map_err(|e| format!("Failed to create dir: {}", e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(&p)
                            .map_err(|e| format!("Failed to create dir: {}", e))?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)
                    .map_err(|e| format!("Failed to create file: {}", e))?;
                io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to extract file: {}", e))?;

                // Set executable permission for binaries or script files
                let path_str = outpath.to_string_lossy();
                if path_str.contains("/binaries/") || path_str.ends_with("pre-install.sh") || path_str.ends_with("post-install.sh") {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = outfile.metadata()
                        .map_err(|e| format!("Failed to get metadata for {}: {}", outpath.display(), e))?
                        .permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&outpath, perms)
                        .map_err(|e| format!("Failed to set permissions for {}: {}", outpath.display(), e))?;
                }
            }
        }

        // 8.5 Verify Hashes in Staging
        println!("Stage: Validating File Integrity...");
        for entry in checksums {
            let staged_file_path = staging_dir.path().join(&entry.path);
            if !staged_file_path.exists() {
                return Err(format!("File {} listed in checksums not found in package", entry.path));
            }
            let actual_hash = mcpkg_core::checksum::ChecksumVerifier::calculate_sha256(&staged_file_path)
                .map_err(|e| format!("Failed to hash file {}: {}", entry.path, e))?;
            
            if actual_hash != entry.hash {
                return Err(format!("Integrity check failed for {}: expected {}, got {}", entry.path, entry.hash, actual_hash));
            }
            println!("  Verified: {}", entry.path);
        }

        // 9. Running Pre-Install Script (if present)
        let pre_install_path = staging_dir.path().join("pre-install.sh");
        if pre_install_path.exists() {
            self.state = InstallState::RunningPreInstall;
            println!("Stage: Running pre-install.sh...");
            run_script_in_sandbox(&pre_install_path, staging_dir.path(), &policy)?;
        }

        // 10. Atomic Move
        self.state = InstallState::PlacingFiles;
        let target_dir = self.apps_root.join(&app_id);
        let backup_dir = self.apps_root.join(format!("{}.backup", app_id));

        if let Some(parent) = target_dir.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create apps parent dir: {}", e))?;
        }

        let has_backup = if target_dir.exists() {
            if backup_dir.exists() {
                std::fs::remove_dir_all(&backup_dir)
                    .map_err(|e| format!("Failed to remove old backup: {}", e))?;
            }
            std::fs::rename(&target_dir, &backup_dir)
                .map_err(|e| format!("Failed to rename target to backup: {}", e))?;
            true
        } else {
            false
        };
        *has_backup_ref.lock().unwrap() = has_backup;

        let staging_path = staging_dir.into_path();
        if let Err(e) = std::fs::rename(&staging_path, &target_dir) {
            let _ = std::fs::remove_dir_all(&staging_path);
            return Err(format!("Failed to rename staging to target (atomic move): {}", e));
        }

        // 11. Running Post-Install Script (if present)
        let post_install_path = target_dir.join("post-install.sh");
        if post_install_path.exists() {
            self.state = InstallState::RunningPostInstall;
            println!("Stage: Running post-install.sh...");
            run_script_in_sandbox(&post_install_path, &target_dir, &policy)?;
        }

        // 12. Writing Audit Log & Desktop Entry
        self.state = InstallState::WritingAuditLog;
        println!("Stage: Writing Audit Log & Desktop Entry...");

        // Generate `.desktop` entry file
        let desktop_dir = self.get_desktop_dir()?;
        std::fs::create_dir_all(&desktop_dir)
            .map_err(|e| format!("Failed to create desktop entry directory: {}", e))?;

        let desktop_path = desktop_dir.join(format!("{}.desktop", app_id));
        let desktop_content = format!(
            "[Desktop Entry]\nName={}\nExec=/opt/orion/launcher %u {}\nIcon=orion-{}\nType=Application\nCategories=Utility;\n",
            manifest.package.name, app_id, app_id
        );

        std::fs::write(&desktop_path, desktop_content)
            .map_err(|e| format!("Failed to write desktop entry: {}", e))?;
        *created_desktop_ref.lock().unwrap() = true;

        // If successful, remove the backup
        if has_backup && backup_dir.exists() {
            std::fs::remove_dir_all(&backup_dir)
                .map_err(|e| format!("Failed to clean up backup: {}", e))?;
        }

        // 13. Complete
        self.state = InstallState::Completed;
        println!("Installation Completed. App placed at: {:?}", target_dir);

        Ok(())
    }
}

fn wait_timeout(child: std::process::Child, timeout: Duration) -> Result<std::process::ExitStatus, String> {
    let child = Arc::new(Mutex::new(child));
    let (tx, rx) = mpsc::channel();

    let child_clone = Arc::clone(&child);
    std::thread::spawn(move || {
        let res = {
            let mut guard = child_clone.lock().unwrap();
            guard.wait()
        };
        let _ = tx.send(res);
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(status)) => Ok(status),
        Ok(Err(e)) => Err(format!("Process wait error: {}", e)),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            let mut guard = child.lock().unwrap();
            let _ = guard.kill();
            let _ = guard.wait();
            Err("Process timed out (exceeded 60s limit)".to_string())
        }
        Err(e) => Err(format!("Channel error: {}", e)),
    }
}

fn run_script_in_sandbox(
    script_path: &std::path::Path,
    working_dir: &std::path::Path,
    policy: &mcpkg_core::SandboxPolicy,
) -> Result<(), String> {
    let mut command = std::process::Command::new(script_path);
    command.current_dir(working_dir);

    let uid = nix::unistd::getuid();
    let gid = nix::unistd::getgid();
    let policy_clone = policy.clone();

    unsafe {
        command.pre_exec(move || {
            // Enter Namespaces (Mount, Net, User - skip PID to allow fork in script)
            let flags = nix::sched::CloneFlags::CLONE_NEWNS 
                      | nix::sched::CloneFlags::CLONE_NEWNET 
                      | nix::sched::CloneFlags::CLONE_NEWUSER;
            match nix::sched::unshare(flags) {
                Ok(_) => {
                    let _ = mcpkg_sandbox::NamespaceManager::setup_user_mappings(uid.as_raw(), gid.as_raw());
                    let _ = mcpkg_sandbox::NamespaceManager::setup_mount_namespace();
                }
                Err(_) => {
                    if policy_clone.meta.strict_mode {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to enter namespaces"));
                    }
                }
            }

            // Apply Seccomp
            if let Err(e) = mcpkg_sandbox::SeccompFilter::apply(&policy_clone) {
                if policy_clone.meta.strict_mode {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
                }
            }

            Ok(())
        });
    }

    let child = command.spawn()
        .map_err(|e| format!("Failed to spawn script: {}", e))?;

    let status = wait_timeout(child, std::time::Duration::from_secs(60))
        .map_err(|e| format!("Script execution failed or timed out: {}", e))?;

    if !status.success() {
        return Err(format!("Script exited with non-zero status: {:?}", status.code()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zip::write::FileOptions;
    use std::io::Write;
    use sha2::Digest;

    // Helper to generate mock package for testing
    fn create_test_package(
        pkg_path: &std::path::Path,
        app_id: &str,
        name: &str,
        pre_script: Option<&str>,
        post_script: Option<&str>,
        fail_sha: bool,
    ) -> Result<(), String> {
        let file = std::fs::File::create(pkg_path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // 1. manifest.toml
        zip.start_file("manifest.toml", options).map_err(|e| e.to_string())?;
        let manifest_toml = format!(
            "[package]\nid = \"{}\"\nname = \"{}\"\nversion = \"1.0.0\"\ndescription = \"Test desc\"\nauthor = \"Tester\"\nauthor_email = \"t@e.com\"\nmin_os = \"1.0.0\"\n\n[build]\nentry_x86_64 = \"binaries/x86_64/test\"\nexec_type = \"elf\"\n\n[store]\ncategory = \"utility\"\nrating = \"ALL\"\nprice_usd = 0.00\ntier = \"free\"\n",
            app_id, name
        );
        zip.write_all(manifest_toml.as_bytes()).map_err(|e| e.to_string())?;

        // 2. sandbox.policy
        zip.start_file("sandbox.policy", options).map_err(|e| e.to_string())?;
        let policy_toml = r#"[meta]
policy_version = "1.0"
strict_mode = false

[filesystem]
home_read = "allow"
home_write = "allow"
temp_access = "allow"
documents_read = "deny"
documents_write = "deny"
downloads_read = "deny"
system_read = "deny"
arbitrary_path = "deny"

[network]
internet = "deny"
localhost = "deny"
local_network = "deny"

[hardware]
camera = "deny"
microphone = "deny"
gpu_access = "deny"
usb_devices = "deny"
bluetooth = "deny"
location = "deny"

[system]
notifications = "deny"
clipboard_read = "deny"
clipboard_write = "deny"
autostart = "deny"
background_run = "deny"
ipc = "deny"
exec_subprocess = "deny"
"#;
        zip.write_all(policy_toml.as_bytes()).map_err(|e| e.to_string())?;

        // 3. binaries/x86_64/test
        zip.start_file("binaries/x86_64/test", options).map_err(|e| e.to_string())?;
        let binary_content = "#!/bin/bash\necho 'hello'\n";
        zip.write_all(binary_content.as_bytes()).map_err(|e| e.to_string())?;

        // 4. pre-install.sh
        if let Some(script) = pre_script {
            zip.start_file("pre-install.sh", options).map_err(|e| e.to_string())?;
            zip.write_all(script.as_bytes()).map_err(|e| e.to_string())?;
        }

        // 5. post-install.sh
        if let Some(script) = post_script {
            zip.start_file("post-install.sh", options).map_err(|e| e.to_string())?;
            zip.write_all(script.as_bytes()).map_err(|e| e.to_string())?;
        }

        // 6. Calculate checksums
        let mut hasher = sha2::Sha256::new();
        hasher.update(manifest_toml.as_bytes());
        let manifest_hash = format!("{:x}", hasher.finalize());

        let mut hasher = sha2::Sha256::new();
        hasher.update(policy_toml.as_bytes());
        let policy_hash = format!("{:x}", hasher.finalize());

        let mut hasher = sha2::Sha256::new();
        let binary_hash_val = if fail_sha { "badhash" } else { binary_content };
        hasher.update(binary_hash_val.as_bytes());
        let binary_hash = format!("{:x}", hasher.finalize());

        let mut checksums_content = format!(
            "{}  manifest.toml\n{}  sandbox.policy\n{}  binaries/x86_64/test\n",
            manifest_hash, policy_hash, binary_hash
        );

        if let Some(script) = pre_script {
            let mut hasher = sha2::Sha256::new();
            hasher.update(script.as_bytes());
            checksums_content.push_str(&format!("{}  pre-install.sh\n", format!("{:x}", hasher.finalize())));
        }

        if let Some(script) = post_script {
            let mut hasher = sha2::Sha256::new();
            hasher.update(script.as_bytes());
            checksums_content.push_str(&format!("{}  post-install.sh\n", format!("{:x}", hasher.finalize())));
        }

        // 7. Write checksums.sha256
        zip.start_file("checksums.sha256", options).map_err(|e| e.to_string())?;
        zip.write_all(checksums_content.as_bytes()).map_err(|e| e.to_string())?;

        // 8. signature.ed25519
        zip.start_file("signature.ed25519", options).map_err(|e| e.to_string())?;
        zip.write_all(&[0u8; 64]).map_err(|e| e.to_string())?;

        zip.finish().map_err(|e| e.to_string())?;
        Ok(())
    }

    #[test]
    fn test_successful_install_no_scripts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pkg_path = temp_dir.path().join("test_app.opk");
        let mock_home = temp_dir.path().join("mock_home");
        let mock_apps = temp_dir.path().join("mock_apps");

        create_test_package(&pkg_path, "com.test.noscripts", "No Scripts App", None, None, false).unwrap();

        let mut pipeline = PipelineManager::new(pkg_path);
        pipeline.apps_root = mock_apps.clone();
        pipeline.desktop_dir = Some(mock_home.join(".local/share/applications"));

        let res = pipeline.run_install();
        assert!(res.is_ok(), "Installation failed: {:?}", res.err());

        // Verify target path exists
        let target_dir = mock_apps.join("com.test.noscripts");
        assert!(target_dir.exists());
        assert!(target_dir.join("manifest.toml").exists());
        assert!(target_dir.join("sandbox.policy").exists());

        // Verify desktop entry exists
        let desktop_path = mock_home.join(".local/share/applications/com.test.noscripts.desktop");
        assert!(desktop_path.exists());
        let desktop_content = std::fs::read_to_string(&desktop_path).unwrap();
        assert!(desktop_content.contains("Name=No Scripts App"));
        assert!(desktop_content.contains("Exec=/opt/orion/launcher %u com.test.noscripts"));
    }

    #[test]
    fn test_successful_install_with_scripts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pkg_path = temp_dir.path().join("test_app.opk");
        let mock_home = temp_dir.path().join("mock_home");
        let mock_apps = temp_dir.path().join("mock_apps");

        let pre_script = "#!/bin/bash\necho 'pre-install running'\ntouch pre_done.txt\n";
        let post_script = "#!/bin/bash\necho 'post-install running'\ntouch post_done.txt\n";

        create_test_package(&pkg_path, "com.test.scripts", "Scripts App", Some(pre_script), Some(post_script), false).unwrap();

        let mut pipeline = PipelineManager::new(pkg_path);
        pipeline.apps_root = mock_apps.clone();
        pipeline.desktop_dir = Some(mock_home.join(".local/share/applications"));

        let res = pipeline.run_install();
        assert!(res.is_ok(), "Installation failed: {:?}", res.err());

        let target_dir = mock_apps.join("com.test.scripts");
        assert!(target_dir.exists());

        // Verify pre-install script created file and it was moved to target directory
        assert!(target_dir.join("pre_done.txt").exists());

        // Verify post-install script created file inside target directory
        assert!(target_dir.join("post_done.txt").exists());
    }

    #[test]
    fn test_checksum_verification_failure() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pkg_path = temp_dir.path().join("test_app_fail.opk");
        let mock_home = temp_dir.path().join("mock_home");
        let mock_apps = temp_dir.path().join("mock_apps");

        create_test_package(&pkg_path, "com.test.checksumfail", "Checksum Fail App", None, None, true).unwrap();

        let mut pipeline = PipelineManager::new(pkg_path);
        pipeline.apps_root = mock_apps.clone();
        pipeline.desktop_dir = Some(mock_home.join(".local/share/applications"));

        let res = pipeline.run_install();
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(err.contains("Integrity check failed"));

        // Verify no files are placed in apps
        let target_dir = mock_apps.join("com.test.checksumfail");
        assert!(!target_dir.exists());

        // Verify no desktop entry
        let desktop_path = mock_home.join(".local/share/applications/com.test.checksumfail.desktop");
        assert!(!desktop_path.exists());
    }

    #[test]
    fn test_pre_install_script_failure_rollback() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pkg_path = temp_dir.path().join("test_app_pre_fail.opk");
        let mock_home = temp_dir.path().join("mock_home");
        let mock_apps = temp_dir.path().join("mock_apps");

        let pre_script = "#!/bin/bash\nexit 1\n";

        create_test_package(&pkg_path, "com.test.prefail", "Pre Fail App", Some(pre_script), None, false).unwrap();

        let mut pipeline = PipelineManager::new(pkg_path);
        pipeline.apps_root = mock_apps.clone();
        pipeline.desktop_dir = Some(mock_home.join(".local/share/applications"));

        let res = pipeline.run_install();
        assert!(res.is_err());

        // Verify nothing installed
        let target_dir = mock_apps.join("com.test.prefail");
        assert!(!target_dir.exists());
    }

    #[test]
    fn test_post_install_script_failure_rollback_and_restore() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mock_home = temp_dir.path().join("mock_home");
        let mock_apps = temp_dir.path().join("mock_apps");

        // 1. Install an initial version successfully
        let pkg_v1 = temp_dir.path().join("app_v1.opk");
        create_test_package(&pkg_v1, "com.test.upgrade", "Upgrade App V1", None, None, false).unwrap();

        let mut pipeline = PipelineManager::new(pkg_v1);
        pipeline.apps_root = mock_apps.clone();
        pipeline.desktop_dir = Some(mock_home.join(".local/share/applications"));
        pipeline.run_install().unwrap();

        let target_dir = mock_apps.join("com.test.upgrade");
        assert!(target_dir.exists());
        assert!(target_dir.join("manifest.toml").exists());
        let manifest_content = std::fs::read_to_string(target_dir.join("manifest.toml")).unwrap();
        assert!(manifest_content.contains("Upgrade App V1"));

        // 2. Try upgrading to a version where post-install fails
        let pkg_v2 = temp_dir.path().join("app_v2.opk");
        let post_script = "#!/bin/bash\nexit 5\n"; // exits with non-zero
        create_test_package(&pkg_v2, "com.test.upgrade", "Upgrade App V2", None, Some(post_script), false).unwrap();

        let mut pipeline2 = PipelineManager::new(pkg_v2);
        pipeline2.apps_root = mock_apps.clone();
        pipeline2.desktop_dir = Some(mock_home.join(".local/share/applications"));
        let res = pipeline2.run_install();
        assert!(res.is_err());

        // 3. Verify target_dir was restored back to V1!
        assert!(target_dir.exists());
        let manifest_content_after = std::fs::read_to_string(target_dir.join("manifest.toml")).unwrap();
        assert!(manifest_content_after.contains("Upgrade App V1")); // restored!
        assert!(!manifest_content_after.contains("Upgrade App V2"));
    }
}
