//! Libraries dependency resolution module

mod download;
mod filter;
mod parse;

// Re-export 公共 API（与原 libraries.rs 完全向后兼容）
pub use download::build_download_urls;
pub use filter::find_missing_libs;
pub use parse::{check_rules, is_native_matching_arch, parse_libraries};

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Library entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibEntry {
    pub original_name: Option<String>,
    pub local_path: String,
    pub size: i64,
    pub is_natives: bool,
    pub sha1: Option<String>,
    pub url: Option<String>,
}

impl LibEntry {
    pub fn name(&self) -> String {
        if let Some(ref original_name) = self.original_name {
            let parts: Vec<&str> = original_name.split(':').collect();
            if parts.len() >= 2 {
                return format!("{}:{}", parts[0], parts[1]);
            }
            original_name.clone()
        } else {
            String::new()
        }
    }
}

/// Maven coordinate to local path
pub fn maven_to_path(name: &str, game_dir: &Path) -> String {
    crate::minecraft::utils::maven::maven_to_local_path(name, game_dir)
        .to_string_lossy()
        .to_string()
}
