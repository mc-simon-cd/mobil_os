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
