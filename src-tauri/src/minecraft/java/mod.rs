//! Java 检测和管理模块

pub mod download;

mod detect;
mod search;
mod select;

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

// 重新导出公共 API，保持 `crate::minecraft::java::*` 路径稳定
pub use detect::{detect_java, detect_java_version};
pub use search::{search_java, search_java_with_paths};
pub use select::select_best_java;
