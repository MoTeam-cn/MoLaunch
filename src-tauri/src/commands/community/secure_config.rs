//! CurseForge 配置命令
//!
//! 重构后 get/set 由 `get_config` / `apply_config` 统一处理。
//! 本文件保留为占位。
//!
//! 安全约束：`CfConfig` 仅派生 `Deserialize`，避免 `api_key` 被 `to_value` 误暴露到 IPC。

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfConfig {
    pub enabled: bool,
    pub api_key: String,
}
