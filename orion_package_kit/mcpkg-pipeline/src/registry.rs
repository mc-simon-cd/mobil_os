use std::io::Read;
use std::path::{Path, PathBuf};

pub struct RegistryClient {
    pub base_url: String,
}

impl RegistryClient {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "https://registry.orion.app/v1".to_string()),
        }
    }

    pub fn get_package(&self, id: &str) -> Result<serde_json::Value, String> {
        if self.base_url.starts_with("mock://") {
            let temp_dir = tempfile::tempdir()
                .map_err(|e| format!("Failed to create temp dir for mock hash generation: {}", e))?;
            let mock_pkg_path = temp_dir.path().join("mock.opk");
            self.generate_mock_package_at(&format!("mock://download/{}", id), &mock_pkg_path)?;
            let expected_sha256 = mcpkg_core::checksum::ChecksumVerifier::calculate_sha256(&mock_pkg_path)
                .map_err(|e| format!("Failed to calculate hash of mock package: {}", e))?;

            let val = serde_json::json!({
                "id": id,
                "name": format!("Mock Application ({})", id),
                "version": "1.0.0",
                "description": "A mocked package for offline unit testing.",
                "latest": {
                    "version": "1.0.0",
                    "download_url": format!("mock://download/{}", id),
                    "sha256": expected_sha256,
                    "size_bytes": std::fs::metadata(&mock_pkg_path).map(|m| m.len()).unwrap_or(0)
                }
            });
            return Ok(val);
        }

        let url = format!("{}/packages/{}", self.base_url, id);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        let response = client.get(&url)
            .send()
            .map_err(|e| format!("Failed to send GET request to {}: {}", url, e))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(format!("Package not found: {}", id));
        }

        if !response.status().is_success() {
            return Err(format!("Registry API returned error: {}", response.status()));
        }

        let json_val = response.json::<serde_json::Value>()
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        Ok(json_val)
    }

    pub fn download_package(&self, url: &str, target: &Path) -> Result<(), String> {
        let mut retries = 3;
        let mut delay = std::time::Duration::from_secs(1);
        loop {
            match self.download_package_impl(url, target) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if retries == 0 {
                        return Err(format!("Download failed after multiple attempts: {}", e));
                    }
                    println!("Download attempt failed: {}. Retrying in {:?}...", e, delay);
                    std::thread::sleep(delay);
                    retries -= 1;
                    delay *= 2;
                }
            }
        }
    }

    fn download_package_impl(&self, url: &str, target: &Path) -> Result<(), String> {
        if url.starts_with("mock://") {
            return self.generate_mock_package_at(url, target);
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 minutes timeout
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        let mut response = client.get(url)
            .send()
            .map_err(|e| format!("Failed to send GET request: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server returned error status: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut dest_file = std::fs::File::create(target)
            .map_err(|e| format!("Failed to create download file: {}", e))?;

        let mut downloaded: u64 = 0;
        let mut buffer = [0u8; 8192];
        loop {
            let bytes_read = response.read(&mut buffer)
                .map_err(|e| format!("Error while reading chunk: {}", e))?;
            if bytes_read == 0 {
                break;
            }
            use std::io::Write;
            dest_file.write_all(&buffer[..bytes_read])
                .map_err(|e| format!("Failed to write chunk to file: {}", e))?;
            downloaded += bytes_read as u64;

            if total_size > 0 {
                let percent = (downloaded as f64 / total_size as f64 * 100.0) as u32;
                let bar_width = 30;
                let progress = (percent as f64 / 100.0 * bar_width as f64) as usize;
                let bar: String = std::iter::repeat("=").take(progress)
                    .chain(std::iter::repeat(" ").take(bar_width - progress))
                    .collect();
                print!("\r[{}] {}% ({}/{})", bar, percent, downloaded, total_size);
                let _ = std::io::stdout().flush();
            } else {
                print!("\rDownloaded: {} bytes", downloaded);
                let _ = std::io::stdout().flush();
            }
        }
        println!();
        Ok(())
    }

    fn generate_mock_package_at(&self, url: &str, target: &Path) -> Result<(), String> {
        use zip::write::FileOptions;
        use std::io::Write;
        use sha2::Digest;

        let file = std::fs::File::create(target).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        let app_id = url.strip_prefix("mock://download/").unwrap_or("com.mock.app");

        // 1. manifest.toml
        zip.start_file("manifest.toml", options).map_err(|e| e.to_string())?;
        let manifest_toml = format!(
            "[package]\nid = \"{}\"\nname = \"Mock App\"\nversion = \"1.0.0\"\ndescription = \"Test desc\"\nauthor = \"Tester\"\nauthor_email = \"t@e.com\"\nmin_os = \"1.0.0\"\n\n[build]\nentry_x86_64 = \"binaries/x86_64/test\"\nexec_type = \"elf\"\n\n[store]\ncategory = \"utility\"\nrating = \"ALL\"\nprice_usd = 0.00\ntier = \"free\"\n",
            app_id
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
        let binary_content = "#!/bin/bash\necho 'hello from mock registry package'\n";
        zip.write_all(binary_content.as_bytes()).map_err(|e| e.to_string())?;

        // 4. Calculate checksums
        let mut hasher = sha2::Sha256::new();
        hasher.update(manifest_toml.as_bytes());
        let manifest_hash = format!("{:x}", hasher.finalize());

        let mut hasher = sha2::Sha256::new();
        hasher.update(policy_toml.as_bytes());
        let policy_hash = format!("{:x}", hasher.finalize());

        let mut hasher = sha2::Sha256::new();
        hasher.update(binary_content.as_bytes());
        let binary_hash = format!("{:x}", hasher.finalize());

        let checksums_content = format!(
            "{}  manifest.toml\n{}  sandbox.policy\n{}  binaries/x86_64/test\n",
            manifest_hash, policy_hash, binary_hash
        );

        // 5. Write checksums.sha256
        zip.start_file("checksums.sha256", options).map_err(|e| e.to_string())?;
        zip.write_all(checksums_content.as_bytes()).map_err(|e| e.to_string())?;

        // 6. signature.ed25519
        zip.start_file("signature.ed25519", options).map_err(|e| e.to_string())?;
        zip.write_all(&[0u8; 64]).map_err(|e| e.to_string())?;

        zip.finish().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn publish_package(&self, package_path: &Path, channel: &str, api_key: &str) -> Result<serde_json::Value, String> {
        if self.base_url.starts_with("mock://") {
            let mock_registry_dir = PathBuf::from("/tmp/orion_mock_registry");
            std::fs::create_dir_all(&mock_registry_dir)
                .map_err(|e| format!("Failed to create mock registry directory: {}", e))?;
            
            let filename = package_path.file_name()
                .ok_or_else(|| "Invalid package path filename".to_string())?;
            let dest_path = mock_registry_dir.join(filename);
            std::fs::copy(package_path, &dest_path)
                .map_err(|e| format!("Failed to copy package to mock registry: {}", e))?;

            // Extract manifest to construct the response
            let file = std::fs::File::open(package_path)
                .map_err(|e| format!("Failed to open mock package: {}", e))?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| format!("Failed to open mock package ZIP: {}", e))?;
            let mut manifest_file = archive.by_name("manifest.toml")
                .map_err(|_| "manifest.toml not found in mock package")?;
            let mut manifest_content = String::new();
            manifest_file.read_to_string(&mut manifest_content)
                .map_err(|e| format!("Failed to read manifest: {}", e))?;
            let manifest = mcpkg_core::Manifest::from_toml(&manifest_content)
                .map_err(|e| format!("Invalid manifest.toml in mock package: {}", e))?;

            let res_json = serde_json::json!({
                "status": "success",
                "message": "Package uploaded successfully to mock registry",
                "package_id": manifest.package.id,
                "version": manifest.package.version,
                "channel": channel,
                "mock_file_path": dest_path.to_string_lossy()
            });

            return Ok(res_json);
        }

        let url = format!("{}/packages/upload", self.base_url);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        let form = reqwest::blocking::multipart::Form::new()
            .text("channel", channel.to_string())
            .file("package", package_path)
            .map_err(|e| format!("Failed to attach package file: {}", e))?;

        let response = client.post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .map_err(|e| format!("Failed to send POST request to registry: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Registry API returned error: {} - {}", response.status(), response.text().unwrap_or_default()));
        }

        let json_val = response.json::<serde_json::Value>()
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        Ok(json_val)
    }
}

pub struct CacheManager {
    pub cache_root: PathBuf,
}

impl CacheManager {
    pub fn new(cache_root: PathBuf) -> Self {
        Self { cache_root }
    }

    pub fn get_default_cache_root() -> Result<PathBuf, String> {
        let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".cache/orion/packages"))
    }

    pub fn get_cache_path(&self, id: &str, version: &str) -> PathBuf {
        self.cache_root.join(id).join(format!("{}.opk", version))
    }

    pub fn is_cached(&self, id: &str, version: &str) -> bool {
        self.get_cache_path(id, version).exists()
    }

    pub fn resolve_and_download(
        &self,
        client: &RegistryClient,
        id: &str,
        version: Option<&str>,
    ) -> Result<PathBuf, String> {
        let pkg_info = client.get_package(id)?;

        let version_str = match version {
            Some(v) => v.to_string(),
            None => {
                pkg_info.get("latest")
                    .and_then(|l| l.get("version"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Failed to resolve latest version from registry".to_string())?
                    .to_string()
            }
        };

        let cached_path = self.get_cache_path(id, &version_str);
        if cached_path.exists() {
            println!("Package {} v{} is already cached at {:?}", id, version_str, cached_path);
            return Ok(cached_path);
        }

        let latest = pkg_info.get("latest")
            .ok_or_else(|| "Missing 'latest' field in package metadata".to_string())?;
        
        let download_url = latest.get("download_url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| "Missing 'download_url' in package metadata".to_string())?;

        let expected_sha256 = latest.get("sha256")
            .and_then(|s| s.as_str())
            .ok_or_else(|| "Missing 'sha256' in package metadata".to_string())?;

        let temp_dir = tempfile::tempdir()
            .map_err(|e| format!("Failed to create temporary download directory: {}", e))?;
        let temp_file_path = temp_dir.path().join(format!("{}-{}.opk", id, version_str));

        println!("Downloading {} v{} from {}...", id, version_str, download_url);
        client.download_package(download_url, &temp_file_path)?;

        println!("Verifying file integrity...");
        let actual_sha256 = mcpkg_core::checksum::ChecksumVerifier::calculate_sha256(&temp_file_path)
            .map_err(|e| format!("Failed to calculate hash of downloaded package: {}", e))?;

        if actual_sha256 != expected_sha256 {
            return Err(format!(
                "Downloaded package integrity check failed: expected {}, got {}",
                expected_sha256, actual_sha256
            ));
        }
        println!("Integrity check passed.");

        if let Some(parent) = cached_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create cache directories: {}", e))?;
        }
        std::fs::copy(&temp_file_path, &cached_path)
            .map_err(|e| format!("Failed to cache package: {}", e))?;

        Ok(cached_path)
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;

    #[test]
    fn test_registry_client_mock_metadata() {
        let client = RegistryClient::new(Some("mock://test_api".to_string()));
        let pkg_info = client.get_package("com.example.notepad");
        assert!(pkg_info.is_ok());
        let info = pkg_info.unwrap();
        assert_eq!(info.get("id").unwrap().as_str().unwrap(), "com.example.notepad");
        assert_eq!(info.get("version").unwrap().as_str().unwrap(), "1.0.0");
        
        let latest = info.get("latest").unwrap();
        assert_eq!(latest.get("version").unwrap().as_str().unwrap(), "1.0.0");
        assert_eq!(latest.get("download_url").unwrap().as_str().unwrap(), "mock://download/com.example.notepad");
        assert!(latest.get("sha256").is_some());
    }

    #[test]
    fn test_cache_manager_resolve_and_download_mock() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_root = temp_dir.path().join("cache");
        let cache_manager = CacheManager::new(cache_root);
        let client = RegistryClient::new(Some("mock://test_api".to_string()));

        // Initial download
        let res = cache_manager.resolve_and_download(&client, "com.mock.test", None);
        assert!(res.is_ok());
        let cached_path = res.unwrap();
        assert!(cached_path.exists());
        assert!(cached_path.to_string_lossy().contains("com.mock.test"));
        assert!(cached_path.to_string_lossy().ends_with("1.0.0.opk"));

        // Second resolve (should hit cache)
        let res2 = cache_manager.resolve_and_download(&client, "com.mock.test", Some("1.0.0"));
        assert!(res2.is_ok());
        let cached_path2 = res2.unwrap();
        assert_eq!(cached_path, cached_path2);

        // Verify it is a valid zip package
        let file = std::fs::File::open(&cached_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        assert!(archive.by_name("manifest.toml").is_ok());
        assert!(archive.by_name("sandbox.policy").is_ok());
        assert!(archive.by_name("binaries/x86_64/test").is_ok());
    }
}
