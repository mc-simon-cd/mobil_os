use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub struct ChecksumVerifier;

impl ChecksumVerifier {
    pub fn calculate_sha256<P: AsRef<Path>>(path: P) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];

        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    pub fn verify_checksum<P: AsRef<Path>>(path: P, expected_hash: &str) -> io::Result<bool> {
        let actual_hash = Self::calculate_sha256(path)?;
        Ok(actual_hash == expected_hash)
    }
}

#[derive(Debug)]
pub struct ChecksumEntry {
    pub hash: String,
    pub path: String,
}

pub struct ChecksumListParser;

impl ChecksumListParser {
    pub fn parse(content: &str) -> Vec<ChecksumEntry> {
        content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, "  ").collect();
                if parts.len() == 2 {
                    Some(ChecksumEntry {
                        hash: parts[0].to_string(),
                        path: parts[1].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_checksum_list_parser() {
        let content = r#"
# Checksums list
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  manifest.toml
ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb  sandbox.policy
"#;
        let entries = ChecksumListParser::parse(content);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "manifest.toml");
        assert_eq!(entries[1].path, "sandbox.policy");
    }

    #[test]
    fn test_calculate_sha256_and_verify() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("test.txt");
        fs::write(&file_path, "Hello Orion!").unwrap();

        let hash = ChecksumVerifier::calculate_sha256(&file_path).unwrap();
        assert_eq!(hash.len(), 64);

        assert!(ChecksumVerifier::verify_checksum(&file_path, &hash).unwrap());
        assert!(!ChecksumVerifier::verify_checksum(&file_path, "0000000000000000000000000000000000000000000000000000000000000000").unwrap());
    }
}

