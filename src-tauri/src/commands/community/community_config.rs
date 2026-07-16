//! 社区资源配置命令
//!
//! 重构后 get/set 由 `get_config` / `apply_config` 统一处理。
//! 本文件保留为占位。

use serde::{Deserialize, Serialize};

/// 社区资源配置（前端 ↔ 后端 传输结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityConfig {
    pub source: u8,
    pub filename_format: u8,
    pub mod_local_name_style: u8,
    pub ignore_quilt: bool,
}
