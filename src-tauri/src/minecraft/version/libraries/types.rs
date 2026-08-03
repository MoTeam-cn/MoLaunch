//! 库条目类型与 Maven 路径转换

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