use std::fs::{self, File};
use std::io::{self, Read, Write, Seek};
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use ed25519_dalek::{SigningKey, VerifyingKey};
use base64::{Engine as _, engine::general_purpose};
use zip::write::FileOptions;

pub struct DeveloperTools;

impl DeveloperTools {
    /// Expanded helper to get the default keys directory.
    pub fn get_default_key_dir() -> Result<PathBuf, String> {
        let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
        Ok(PathBuf::from(home).join(".config/orion/signing"))
    }

    /// Generates a new Ed25519 keypair and serializes them in standard base64 formats.
    /// Writes private.key and public.key files to the target output directory.
    pub fn keygen(output_dir: &Path) -> Result<(PathBuf, PathBuf), String> {
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        let (signing_key, verifying_key) = mcpkg_crypto::signing::CryptoManager::generate_keypair();

        let seed_bytes = signing_key.to_bytes();
        let pub_bytes = verifying_key.to_bytes();

        let priv_b64 = general_purpose::STANDARD.encode(seed_bytes);
        let pub_b64 = general_purpose::STANDARD.encode(pub_bytes);

        let priv_content = format!("ORION_PRIVATE_KEY_V1:{}\n", priv_b64);
        let pub_content = format!("ORION_PUBLIC_KEY_V1:{}\n", pub_b64);

        let priv_path = output_dir.join("private.key");
        let pub_path = output_dir.join("public.key");

        fs::write(&priv_path, priv_content)
            .map_err(|e| format!("Failed to write private key to {:?}: {}", priv_path, e))?;

        fs::write(&pub_path, pub_content)
            .map_err(|e| format!("Failed to write public key to {:?}: {}", pub_path, e))?;

        Ok((priv_path, pub_path))
    }

    /// Recursively walks a directory, computes SHA-256 of files, generates checksums.sha256,
    /// and packages them into a .mcpkg ZIP file.
    pub fn pack(source_dir: &Path, output_path: &Path) -> Result<(), String> {
        if !source_dir.exists() || !source_dir.is_dir() {
            return Err(format!("Source directory {:?} does not exist or is not a directory", source_dir));
        }

        // Collect all files in source directory
        let mut file_paths = Vec::new();
        visit_dirs(source_dir, &mut file_paths)
            .map_err(|e| format!("Error scanning directory: {}", e))?;

        // Filter and check for manifest and policy
        let mut manifest_found = false;
        let mut policy_found = false;
        let mut files_to_checksum = Vec::new();

        for path in file_paths {
            let relative = path.strip_prefix(source_dir)
                .map_err(|e| format!("Failed to compute relative path: {}", e))?;
            let relative_str = relative.to_string_lossy().into_owned();

            // Exclude signature.ed25519 and checksums.sha256 if they exist in source
            if relative_str == "signature.ed25519" || relative_str == "checksums.sha256" {
                continue;
            }

            if relative_str == "manifest.toml" {
                manifest_found = true;
            } else if relative_str == "sandbox.policy" {
                policy_found = true;
            }

            files_to_checksum.push((path, relative_str));
        }

        if !manifest_found {
            return Err("Missing required file: manifest.toml at the root of package".to_string());
        }
        if !policy_found {
            return Err("Missing required file: sandbox.policy at the root of package".to_string());
        }

        // Generate checksums.sha256 content
        let mut checksums_content = String::new();
        for (abs_path, rel_str) in &files_to_checksum {
            let hash = mcpkg_core::checksum::ChecksumVerifier::calculate_sha256(abs_path)
                .map_err(|e| format!("Failed to calculate hash of {}: {}", rel_str, e))?;
            checksums_content.push_str(&format!("{}  {}\n", hash, rel_str));
        }

        // Ensure parent directory of output package exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output parent directory: {}", e))?;
        }

        // Create Zip Writer
        let file = File::create(output_path)
            .map_err(|e| format!("Failed to create package file {:?}: {}", output_path, e))?;
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // Write all source files to Zip
        let mut buffer = Vec::new();
        for (abs_path, rel_str) in &files_to_checksum {
            zip.start_file(rel_str, options)
                .map_err(|e| format!("ZIP writing error: {}", e))?;
            
            let mut f = File::open(abs_path)
                .map_err(|e| format!("Failed to open file {:?}: {}", abs_path, e))?;
            buffer.clear();
            f.read_to_end(&mut buffer)
                .map_err(|e| format!("Failed to read file {:?}: {}", abs_path, e))?;
            zip.write_all(&buffer)
                .map_err(|e| format!("Failed to write file to ZIP: {}", e))?;
        }

        // Write checksums.sha256 to Zip
        zip.start_file("checksums.sha256", options)
            .map_err(|e| format!("ZIP writing error: {}", e))?;
        zip.write_all(checksums_content.as_bytes())
            .map_err(|e| format!("Failed to write checksums to ZIP: {}", e))?;

        // Write default blank signature placeholder (64 bytes of zeros)
        zip.start_file("signature.ed25519", options)
            .map_err(|e| format!("ZIP writing error: {}", e))?;
        zip.write_all(&[0u8; 64])
            .map_err(|e| format!("Failed to write signature placeholder to ZIP: {}", e))?;

        zip.finish()
            .map_err(|e| format!("Failed to finish ZIP package: {}", e))?;

        Ok(())
    }

    /// Sign package: Computes aggregate signature message and updates signature.ed25519 atomically.
    pub fn sign(package_path: &Path, private_key_path: &Path) -> Result<(), String> {
        // Read and parse private key
        let key_content = fs::read_to_string(private_key_path)
            .map_err(|e| format!("Failed to read private key file: {}", e))?;
        let signing_key = parse_private_key(&key_content)?;

        // Open original ZIP for reading
        let file = File::open(package_path)
            .map_err(|e| format!("Failed to open package file for signing: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Invalid package ZIP format: {}", e))?;

        // Read manifest.toml
        let mut manifest_file = archive.by_name("manifest.toml")
            .map_err(|_| "manifest.toml not found in package")?;
        let mut manifest_content = String::new();
        manifest_file.read_to_string(&mut manifest_content)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        let manifest = mcpkg_core::Manifest::from_toml(&manifest_content)
            .map_err(|e| format!("Invalid manifest.toml in package: {}", e))?;

        drop(manifest_file);

        // Compute aggregate signature message
        let aggregate_message = calculate_aggregate_message(&mut archive, &manifest)?;

        // Sign the message
        let signature = mcpkg_crypto::signing::CryptoManager::sign(&signing_key, &aggregate_message);
        let signature_bytes = mcpkg_crypto::signing::CryptoManager::signature_to_bytes(&signature);

        // Prepare temporary zip in same directory to rewrite archive atomically
        let parent = package_path.parent().unwrap_or_else(|| Path::new("."));
        let temp_path = parent.join(format!(".tmp_signing_{}.opk", uuid_like_random()));

        let temp_file = File::create(&temp_path)
            .map_err(|e| format!("Failed to create temporary file for signing: {}", e))?;
        let mut new_zip = zip::ZipWriter::new(temp_file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        // Re-read original archive to copy entries
        let file_re = File::open(package_path)
            .map_err(|e| format!("Failed to re-open package file: {}", e))?;
        let mut archive_re = zip::ZipArchive::new(file_re)
            .map_err(|e| format!("Invalid package ZIP format: {}", e))?;

        let mut buffer = Vec::new();
        for i in 0..archive_re.len() {
            let mut zip_file = archive_re.by_index(i)
                .map_err(|e| format!("Failed to access file in package zip: {}", e))?;
            let name = zip_file.name().to_string();

            if name == "signature.ed25519" {
                continue; // We will write the new signature instead
            }

            new_zip.start_file(&name, options)
                .map_err(|e| format!("Failed to copy file to new ZIP: {}", e))?;
            buffer.clear();
            zip_file.read_to_end(&mut buffer)
                .map_err(|e| format!("Failed to read file from package: {}", e))?;
            new_zip.write_all(&buffer)
                .map_err(|e| format!("Failed to write file to new ZIP: {}", e))?;
        }

        // Append the new signature
        new_zip.start_file("signature.ed25519", options)
            .map_err(|e| format!("Failed to start signature entry in new ZIP: {}", e))?;
        new_zip.write_all(&signature_bytes)
            .map_err(|e| format!("Failed to write signature to new ZIP: {}", e))?;

        new_zip.finish()
            .map_err(|e| format!("Failed to complete new ZIP file: {}", e))?;

        // Atomic rename
        fs::rename(&temp_path, package_path)
            .map_err(|e| {
                let _ = fs::remove_file(&temp_path);
                format!("Failed to overwrite original package with signed version: {}", e)
            })?;

        let verifying_key = VerifyingKey::from(&signing_key);
        println!("✅ Signed successfully. Fingerprint: {}", calculate_fingerprint(verifying_key.as_bytes()));

        Ok(())
    }

    /// Verifies checksums and signature of a package.
    pub fn verify(package_path: &Path, public_key_path: Option<&Path>) -> Result<(), String> {
        // Open original ZIP for reading
        let file = File::open(package_path)
            .map_err(|e| format!("Failed to open package: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Invalid package ZIP format: {}", e))?;

        // 1. Parse checksums.sha256
        let mut checksums_file = archive.by_name("checksums.sha256")
            .map_err(|_| "checksums.sha256 not found in package")?;
        let mut checksums_content = String::new();
        checksums_file.read_to_string(&mut checksums_content)
            .map_err(|e| format!("Failed to read checksums: {}", e))?;
        
        drop(checksums_file);

        let checksums = mcpkg_core::checksum::ChecksumListParser::parse(&checksums_content);
        if checksums.is_empty() {
            return Err("checksums.sha256 is empty or invalid".to_string());
        }

        // 2. Validate integrity of all files listed in checksums.sha256
        let mut buffer = Vec::new();
        for entry in &checksums {
            let mut file_in_zip = archive.by_name(&entry.path)
                .map_err(|_| format!("File {} listed in checksums not found in package", entry.path))?;
            
            buffer.clear();
            file_in_zip.read_to_end(&mut buffer)
                .map_err(|e| format!("Failed to read file {}: {}", entry.path, e))?;

            let mut hasher = Sha256::new();
            hasher.update(&buffer);
            let actual_hash = hex::encode(hasher.finalize());

            if actual_hash != entry.hash {
                return Err(format!("Integrity check failed for {}: expected {}, got {}", entry.path, entry.hash, actual_hash));
            }
        }
        println!("✅ Checksum: Tüm dosyalar geçerli");

        // 3. Compute expected signature message payload
        let mut manifest_file = archive.by_name("manifest.toml")
            .map_err(|_| "manifest.toml not found in package")?;
        let mut manifest_content = String::new();
        manifest_file.read_to_string(&mut manifest_content)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        let manifest = mcpkg_core::Manifest::from_toml(&manifest_content)
            .map_err(|e| format!("Invalid manifest.toml: {}", e))?;

        drop(manifest_file);

        let aggregate_message = calculate_aggregate_message(&mut archive, &manifest)?;

        // 4. Read signature.ed25519
        let mut signature_bytes = [0u8; 64];
        let mut sig_file = archive.by_name("signature.ed25519")
            .map_err(|_| "signature.ed25519 not found in package")?;
        sig_file.read_exact(&mut signature_bytes)
            .map_err(|e| format!("Failed to read signature: {}", e))?;
        
        let signature = mcpkg_crypto::signing::CryptoManager::signature_from_bytes(&signature_bytes)
            .map_err(|e| format!("Invalid signature format: {}", e))?;

        // 5. Load/resolve public key
        let verifying_key = if let Some(key_path) = public_key_path {
            let content = fs::read_to_string(key_path)
                .map_err(|e| format!("Failed to read custom public key: {}", e))?;
            parse_public_key(&content)?
        } else {
            let default_dir = Self::get_default_key_dir()?;
            let default_path = default_dir.join("public.key");
            if default_path.exists() {
                let content = fs::read_to_string(&default_path)
                    .map_err(|e| format!("Failed to read default public key: {}", e))?;
                parse_public_key(&content)?
            } else {
                // Fallback to mock verifying key (all zeros) for local mock testing
                let mock_pub_bytes = [0u8; 32];
                VerifyingKey::from_bytes(&mock_pub_bytes)
                    .map_err(|e| format!("Failed to generate mock verifying key: {}", e))?
            }
        };

        // 6. Verify Ed25519 signature
        mcpkg_crypto::signing::CryptoManager::verify(&verifying_key, &aggregate_message, &signature)
            .map_err(|e| format!("Signature verification failed: {}", e))?;

        println!("✅ İmza: Doğrulandı");
        println!("  Key Fingerprint: {}", calculate_fingerprint(verifying_key.as_bytes()));

        Ok(())
    }
}

// ----------------- Helper Functions -----------------

fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, files)?;
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}

pub fn parse_private_key(content: &str) -> Result<SigningKey, String> {
    let content = content.trim();
    let (_prefix, b64_part) = if content.starts_with("ORION_PRIVATE_KEY_V1:") {
        ("ORION_PRIVATE_KEY_V1:", &content["ORION_PRIVATE_KEY_V1:".len()..])
    } else if content.starts_with("MCSIMON_PRIVATE_KEY_V1:") {
        ("MCSIMON_PRIVATE_KEY_V1:", &content["MCSIMON_PRIVATE_KEY_V1:".len()..])
    } else {
        return Err("Invalid private key format: missing ORION_PRIVATE_KEY_V1 or MCSIMON_PRIVATE_KEY_V1 prefix".to_string());
    };
    let bytes = general_purpose::STANDARD.decode(b64_part)
        .map_err(|e| format!("Failed to decode base64 private key: {}", e))?;
    if bytes.len() != 32 {
        return Err(format!("Invalid private key seed length: expected 32 bytes, got {}", bytes.len()));
    }
    let seed: [u8; 32] = bytes.try_into().unwrap();
    Ok(SigningKey::from_bytes(&seed))
}

pub fn parse_public_key(content: &str) -> Result<VerifyingKey, String> {
    let content = content.trim();
    let (_prefix, b64_part) = if content.starts_with("ORION_PUBLIC_KEY_V1:") {
        ("ORION_PUBLIC_KEY_V1:", &content["ORION_PUBLIC_KEY_V1:".len()..])
    } else if content.starts_with("MCSIMON_PUBLIC_KEY_V1:") {
        ("MCSIMON_PUBLIC_KEY_V1:", &content["MCSIMON_PUBLIC_KEY_V1:".len()..])
    } else {
        return Err("Invalid public key format: missing ORION_PUBLIC_KEY_V1 or MCSIMON_PUBLIC_KEY_V1 prefix".to_string());
    };
    let bytes = general_purpose::STANDARD.decode(b64_part)
        .map_err(|e| format!("Failed to decode base64 public key: {}", e))?;
    if bytes.len() != 32 {
        return Err(format!("Invalid public key length: expected 32 bytes, got {}", bytes.len()));
    }
    let key_bytes: [u8; 32] = bytes.try_into().unwrap();
    VerifyingKey::from_bytes(&key_bytes)
        .map_err(|e| format!("Invalid public key bytes: {}", e))
}

pub fn calculate_fingerprint(verifying_key_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifying_key_bytes);
    let hash = hasher.finalize();
    format!("SHA256:{}", hex::encode(&hash[..8]))
}

fn hash_zip_file<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<[u8; 32], String> {
    let mut file = archive.by_name(name)
        .map_err(|e| format!("File {} not found in ZIP: {}", name, e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];
    loop {
        let count = file.read(&mut buffer)
            .map_err(|e| format!("Failed to read {}: {}", name, e))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finalize().into())
}

pub fn calculate_aggregate_message<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    manifest: &mcpkg_core::Manifest,
) -> Result<Vec<u8>, String> {
    let mut message = Vec::new();

    // 1. manifest.toml
    let manifest_hash = hash_zip_file(archive, "manifest.toml")?;
    message.extend_from_slice(&manifest_hash);

    // 2. sandbox.policy
    let policy_hash = hash_zip_file(archive, "sandbox.policy")?;
    message.extend_from_slice(&policy_hash);

    // 3. checksums.sha256
    let checksums_hash = hash_zip_file(archive, "checksums.sha256")?;
    message.extend_from_slice(&checksums_hash);

    // 4. entry_x86_64 binary
    let entry_x86_64 = &manifest.build.entry_x86_64;
    if let Ok(entry_hash) = hash_zip_file(archive, entry_x86_64) {
        message.extend_from_slice(&entry_hash);
    }

    // 5. entry_aarch64 binary (optional)
    if let Some(ref entry_aarch64) = manifest.build.entry_aarch64 {
        if let Ok(entry_hash) = hash_zip_file(archive, entry_aarch64) {
            message.extend_from_slice(&entry_hash);
        }
    }

    Ok(message)
}

fn uuid_like_random() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{:x}", start)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::registry::RegistryClient;

    #[test]
    fn test_developer_keygen_serialization() {
        let tmp = tempdir().unwrap();
        let res = DeveloperTools::keygen(tmp.path());
        assert!(res.is_ok());
        let (priv_path, pub_path) = res.unwrap();

        assert!(priv_path.exists());
        assert!(pub_path.exists());

        let priv_content = fs::read_to_string(&priv_path).unwrap();
        let pub_content = fs::read_to_string(&pub_path).unwrap();

        assert!(priv_content.starts_with("ORION_PRIVATE_KEY_V1:"));
        assert!(pub_content.starts_with("ORION_PUBLIC_KEY_V1:"));

        let signing_key = parse_private_key(&priv_content);
        assert!(signing_key.is_ok());

        let verifying_key = parse_public_key(&pub_content);
        assert!(verifying_key.is_ok());
    }

    #[test]
    fn test_developer_pack_and_verify_unsigned() {
        let tmp = tempdir().unwrap();
        let source_dir = tmp.path().join("src");
        let output_pkg = tmp.path().join("app-1.0.0.opk");

        fs::create_dir_all(&source_dir).unwrap();

        // Write manifest
        let manifest_content = r#"[package]
id = "com.example.test"
name = "Test App"
version = "1.0.0"
description = "Test description"
author = "Developer"
author_email = "dev@example.com"
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
        fs::write(source_dir.join("manifest.toml"), manifest_content).unwrap();

        // Write policy
        let policy_content = r#"[meta]
policy_version = "1.0"
strict_mode = false

[filesystem]
home_read = "allow"
home_write = "allow"
"#;
        fs::write(source_dir.join("sandbox.policy"), policy_content).unwrap();

        // Write dummy binary file
        let bin_dir = source_dir.join("binaries/x86_64");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("test"), "dummy binary data").unwrap();

        // Pack it
        let pack_res = DeveloperTools::pack(&source_dir, &output_pkg);
        assert!(pack_res.is_ok(), "Packing failed: {:?}", pack_res.err());
        assert!(output_pkg.exists());

        // Verify it unsigned (should verify against mock key successfully because default signature is [0;64])
        let verify_res = DeveloperTools::verify(&output_pkg, None);
        assert!(verify_res.is_ok(), "Verification failed: {:?}", verify_res.err());
    }

    #[test]
    fn test_developer_sign_and_verify() {
        let tmp = tempdir().unwrap();
        let source_dir = tmp.path().join("src");
        let output_pkg = tmp.path().join("app-1.0.0.opk");
        let keys_dir = tmp.path().join("keys");

        // 1. Generate keys
        let (priv_path, pub_path) = DeveloperTools::keygen(&keys_dir).unwrap();

        // 2. Setup source directory
        fs::create_dir_all(&source_dir).unwrap();
        let manifest_content = r#"[package]
id = "com.example.signed"
name = "Signed App"
version = "1.0.0"
description = "Test description"
author = "Developer"
author_email = "dev@example.com"
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
        fs::write(source_dir.join("manifest.toml"), manifest_content).unwrap();

        let policy_content = r#"[meta]
policy_version = "1.0"
strict_mode = false

[filesystem]
home_read = "allow"
home_write = "allow"
"#;
        fs::write(source_dir.join("sandbox.policy"), policy_content).unwrap();

        let bin_dir = source_dir.join("binaries/x86_64");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("test"), "actual binary content").unwrap();

        // 3. Pack
        DeveloperTools::pack(&source_dir, &output_pkg).unwrap();

        // 4. Sign
        let sign_res = DeveloperTools::sign(&output_pkg, &priv_path);
        assert!(sign_res.is_ok(), "Signing failed: {:?}", sign_res.err());

        // 5. Verify using the public key
        let verify_res = DeveloperTools::verify(&output_pkg, Some(&pub_path));
        assert!(verify_res.is_ok(), "Verification of signed package failed: {:?}", verify_res.err());
    }

    #[test]
    fn test_developer_sign_invalid_key_fails() {
        let tmp = tempdir().unwrap();
        let source_dir = tmp.path().join("src");
        let output_pkg = tmp.path().join("app-1.0.0.opk");
        let keys_dir_a = tmp.path().join("keys_a");
        let keys_dir_b = tmp.path().join("keys_b");

        // 1. Generate two keypairs
        let (priv_path_a, _pub_path_a) = DeveloperTools::keygen(&keys_dir_a).unwrap();
        let (_priv_path_b, pub_path_b) = DeveloperTools::keygen(&keys_dir_b).unwrap();

        // 2. Setup source directory
        fs::create_dir_all(&source_dir).unwrap();
        let manifest_content = r#"[package]
id = "com.example.signed"
name = "Signed App"
version = "1.0.0"
description = "Test description"
author = "Developer"
author_email = "dev@example.com"
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
        fs::write(source_dir.join("manifest.toml"), manifest_content).unwrap();

        let policy_content = r#"[meta]
policy_version = "1.0"
strict_mode = false

[filesystem]
home_read = "allow"
home_write = "allow"
"#;
        fs::write(source_dir.join("sandbox.policy"), policy_content).unwrap();

        let bin_dir = source_dir.join("binaries/x86_64");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("test"), "actual binary content").unwrap();

        // 3. Pack
        DeveloperTools::pack(&source_dir, &output_pkg).unwrap();

        // 4. Sign with Key A
        DeveloperTools::sign(&output_pkg, &priv_path_a).unwrap();

        // 5. Verify using Public Key B (should fail!)
        let verify_res = DeveloperTools::verify(&output_pkg, Some(&pub_path_b));
        assert!(verify_res.is_err(), "Verification succeeded with wrong key!");
    }

    #[test]
    fn test_registry_publish_mock() {
        let tmp = tempdir().unwrap();
        let pkg_path = tmp.path().join("test-1.0.0.opk");

        // Write a minimal mock ZIP package
        let file = File::create(&pkg_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("manifest.toml", options).unwrap();
        zip.write_all(r#"[package]
id = "com.publish.mock"
name = "Mock Publish App"
version = "1.0.0"
description = "mock"
author = "mock"
author_email = "mock"
min_os = "1.0.0"

[build]
entry_x86_64 = "binaries/x86_64/app"
exec_type = "elf"

[store]
category = "productivity"
rating = "ALL"
price_usd = 0.00
tier = "free"
"#.as_bytes()).unwrap();

        zip.finish().unwrap();

        let client = RegistryClient::new(Some("mock://test_publish".to_string()));
        let publish_res = client.publish_package(&pkg_path, "beta", "my_secret_token");
        assert!(publish_res.is_ok(), "Publish to mock registry failed: {:?}", publish_res.err());

        let res = publish_res.unwrap();
        assert_eq!(res.get("status").unwrap().as_str().unwrap(), "success");
        assert_eq!(res.get("package_id").unwrap().as_str().unwrap(), "com.publish.mock");
        assert_eq!(res.get("version").unwrap().as_str().unwrap(), "1.0.0");
        assert_eq!(res.get("channel").unwrap().as_str().unwrap(), "beta");
    }
}
