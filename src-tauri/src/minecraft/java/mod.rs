//! Java检测和管理模块
//!
//! 子模块组织：
//! - `download`: Java 自动下载（Mojang Runtime 索引）
//! - `detect`: 单个 Java 检测与版本号解析
//! - `search`: 系统级 Java 搜索（环境变量 / 磁盘 / 用户目录等）
//! - `select`: 基于版本区间的基础选择（MC 版本感知选择见 `java_selector`）

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
