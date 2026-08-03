//! Java 运行时信息类型

use serde::{Deserialize, Serialize};

/// Java运行时信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaRuntime {
    pub executable: String,
    pub path_folder: String,
    pub is_user_import: bool,
    pub version: String,
    pub major_version: u32,
    pub is_jre: bool,
    pub is_64bit: bool,
}