//! Frp 路径辅助函数：数据根目录、厂商根目录、隧道/状态文件、ID 校验
//!
//! 所有路径基于 `Storage::instance().base_dir()` 派生，集中在此处避免散落
//! 在子模块（provider/install/binary/process）造成路径漂移。

use std::path::PathBuf;

/// Frp 数据根目录（`<base_dir>/frp/`）
pub fn frp_data_dir() -> PathBuf {
    crate::storage::Storage::instance().base_dir().join("frp")
}

/// 厂商根目录（`<base_dir>/providers/`）
pub fn providers_root() -> PathBuf {
    crate::storage::Storage::instance()
        .base_dir()
        .join("providers")
}

/// 隧道配置文件路径（`<base_dir>/frp/tunnels.json`）
pub fn tunnels_path() -> PathBuf {
    frp_data_dir().join("tunnels.json")
}

/// 厂商启用状态文件（`<base_dir>/frp/providers.json`）
pub fn providers_state_path() -> PathBuf {
    frp_data_dir().join("providers.json")
}

/// 校验厂商 ID 合法性（kebab-case：小写字母 + 数字 + 连字符，
/// 不以连字符开头 / 结尾，最长 64 字符）
pub fn validate_provider_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("厂商 ID 不能为空".to_string());
    }
    if id.len() > 64 {
        return Err("厂商 ID 不能超过 64 字符".to_string());
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err("厂商 ID 仅允许小写字母、数字、连字符".to_string());
    }
    if id.starts_with('-') || id.ends_with('-') {
        return Err("厂商 ID 不能以连字符开头或结尾".to_string());
    }
    Ok(())
}

/// frpc 日志目录（`<base_dir>/frp/logs/`）
pub fn frp_logs_dir() -> PathBuf {
    frp_data_dir().join("logs")
}

/// frpc 运行时配置文件目录（`<base_dir>/frp/config/`）
pub fn frp_config_dir() -> PathBuf {
    frp_data_dir().join("config")
}

/// 确保目录存在
pub fn ensure_dir(path: &std::path::Path) -> Result<(), String> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("创建目录失败 {}: {}", path.display(), e))?;
    }
    Ok(())
}
