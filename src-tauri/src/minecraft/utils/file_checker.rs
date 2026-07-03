//! File verification module - SHA1/MD5/SHA256/file size checking

use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;

/// File checker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChecker {
    pub actual_size: i64,
    pub min_size: i64,
    pub hash: Option<String>,
    pub can_use_exists_file: bool,
    pub is_json: bool,
}

impl Default for FileChecker {
    fn default() -> Self {
        Self {
            actual_size: -1,
            min_size: -1,
            hash: None,
            can_use_exists_file: true,
            is_json: false,
        }
    }
}

impl FileChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_actual_size(mut self, size: i64) -> Self {
        self.actual_size = size;
        self
    }

    pub fn with_min_size(mut self, size: i64) -> Self {
        self.min_size = size;
        self
    }

    pub fn with_hash(mut self, hash: Option<String>) -> Self {
        self.hash = hash;
        self
    }

    pub fn with_can_use_exists_file(mut self, can: bool) -> Self {
        self.can_use_exists_file = can;
        self
    }

    pub fn with_is_json(mut self, is_json: bool) -> Self {
        self.is_json = is_json;
        self
    }

    /// Check file. Returns None on success, error description on failure.
    pub fn check(&self, local_path: &str) -> Option<String> {
        let path = Path::new(local_path);

        if !path.exists() {
            return Some(format!("File not found: {}", local_path));
        }

        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(e) => return Some(format!("Cannot read metadata: {}", e)),
        };

        let file_size = metadata.len() as i64;

        if self.actual_size >= 0 && self.actual_size != file_size {
            return Some(format!("Size mismatch: expected {} B, got {} B", self.actual_size, file_size));
        }

        if self.min_size >= 0 && self.min_size > file_size {
            return Some(format!("Size too small: expected > {} B, got {} B", self.min_size, file_size));
        }

        if let Some(ref hash) = self.hash {
            if !hash.is_empty() {
                let computed_hash = if hash.len() < 35 {
                    compute_file_hash(local_path, HashMethod::Md5)
                } else if hash.len() == 64 {
                    compute_file_hash(local_path, HashMethod::Sha256)
                } else {
                    compute_file_hash(local_path, HashMethod::Sha1)
                };

                match computed_hash {
                    Ok(actual_hash) => {
                        if hash.to_lowercase() != actual_hash.to_lowercase() {
                            return Some(format!("Hash mismatch: expected {}, got {}", hash, actual_hash));
                        }
                    }
                    Err(e) => return Some(format!("Hash computation failed: {}", e)),
                }
            }
        }

        if self.is_json {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    if content.is_empty() {
                        return Some("File is empty".to_string());
                    }
                    if serde_json::from_str::<serde_json::Value>(&content).is_err() {
                        return Some("Not valid JSON".to_string());
                    }
                }
                Err(e) => return Some(format!("Read failed: {}", e)),
            }
        }

        None
    }

    pub fn is_valid(&self, local_path: &str) -> bool {
        self.check(local_path).is_none()
    }
}

pub enum HashMethod {
    Md5,
    Sha1,
    Sha256,
}

pub fn compute_file_hash(file_path: &str, method: HashMethod) -> anyhow::Result<String> {
    let mut file = std::fs::File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let hash_bytes = match method {
        HashMethod::Md5 => {
            let digest = md5::compute(&buffer);
            digest.to_vec()
        }
        HashMethod::Sha1 => {
            use sha1::Digest;
            let mut hasher = sha1::Sha1::new();
            hasher.update(&buffer);
            hasher.finalize().to_vec()
        }
        HashMethod::Sha256 => {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(&buffer);
            hasher.finalize().to_vec()
        }
    };

    Ok(hex::encode(hash_bytes))
}
