//! Frp 路径辅助函数：数据根目录、厂商根目录、隧道/状态文件、ID 校验
//!
//! 路径分两类：
//! - **便携式**（基于 `Storage::base_dir()`）：frp/tunnels.json、frp/providers.json、frp/logs、
//!   frp/config — 当前启动器实例的运行时数据
//! - **全局共享**（基于 `storage::appdata`）：providers/ 外部 frpc 厂商二进制、frp_auth/ 认证 token —
//!   设备级数据，跨启动器实例共享（认证 token 与厂商账号绑定，便携版换目录/更新不丢）

use std::path::PathBuf;

/// Frp 数据根目录（便携式 `<base_dir>/frp/`）
///
/// 存放当前启动器实例的运行时数据：tunnels.json、providers.json、logs/、config/。
pub fn frp_data_dir() -> PathBuf {
    crate::storage::Storage::instance().base_dir().join("frp")
}

/// 厂商根目录（全局共享 `%APPDATA%/.Molaunch/providers/`）
///
/// 外部 frpc 厂商二进制是设备级资源，跨启动器实例共享，避免每实例重复下载。
/// 旧路径 `<exe_dir>/.Molaunch/providers/` 由 `Storage::init` 启动时自动迁移。
pub fn providers_root() -> PathBuf {
    crate::storage::appdata::ensure_appdata_subdir("providers").unwrap_or_else(|e| {
        crate::log_error!("Failed to create providers directory in AppData: {}", e);
        // 降级回便携式目录（极少发生：APPDATA 环境变量缺失）
        crate::storage::Storage::instance().base_dir().join("providers")
    })
}

/// 隧道配置文件路径（`<base_dir>/frp/tunnels.json`）
pub fn tunnels_path() -> PathBuf {
    frp_data_dir().join("tunnels.json")
}

/// 厂商启用状态文件（`<base_dir>/frp/providers.json`）
pub fn providers_state_path() -> PathBuf {
    frp_data_dir().join("providers.json")
}

/// 厂商认证 token 文件路径（全局共享 `%APPDATA%/.Molaunch/frp_auth/{provider_id}.json`）
///
/// 整份 token 记录经 SDK DES 加密后写入。与 frpc 厂商二进制（providers/）一致，
/// 属设备级共享数据（跨启动器实例共享，便携版换目录/更新不丢认证），
/// 非便携式实例数据（tunnels.json 等随实例走）。
/// 环境变量缺失时降级回便携式目录（极少发生）。
pub fn auth_file_path(provider_id: &str) -> PathBuf {
    crate::storage::appdata::ensure_appdata_subdir("frp_auth")
        .map(|dir| dir.join(format!("{}.json", provider_id)))
        .unwrap_or_else(|e| {
            crate::log_error!(
                "Failed to create frp_auth directory in AppData: {}, fallback to portable",
                e
            );
            frp_data_dir().join("auth").join(format!("{}.json", provider_id))
        })
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
