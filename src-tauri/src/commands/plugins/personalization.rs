//! 个性化配置读写（%APPDATA%/.Molaunch/personalization.json）
//!
//! 将个性化配置（插件启用状态 / 主页模式 / 自定义布局配置）存储到 AppData
//! 而非游戏目录，确保不同 game_dir 的启动器实例加载同一份配置。
//! JSON 格式，直接透传前端 `serde_json::Value`，全量覆盖写入。

use crate::error_util::log_err;
use crate::log_info;
use std::path::PathBuf;

/// 读取个性化配置
///
/// 文件不存在时返回空 JSON 对象 `{}`，调用方按 Partial 语义合并默认值。
pub async fn read_personalization() -> Result<serde_json::Value, String> {
    let path = personalization_path()?;

    if !path.exists() {
        return Ok(serde_json::json!({}));
    }

    let content =
        std::fs::read_to_string(&path).map_err(log_err("Failed to read personalization config"))?;

    // 解析失败时返回空对象而非错误（容错：损坏的 JSON 不阻塞启动）
    let value: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|e| {
        crate::log_warn!("个性化配置 JSON 损坏，已忽略: {}", e);
        serde_json::json!({})
    });

    Ok(value)
}

/// 写入个性化配置
///
/// 全量覆盖写入。调用方传入完整的 `PersonalizationData` 结构。
pub async fn write_personalization(data: serde_json::Value) -> Result<(), String> {
    let path = personalization_path()?;

    // 美化输出（4 空格缩进），便于用户手动查看 / 编辑
    let content = serde_json::to_string_pretty(&data)
        .map_err(log_err("Failed to serialize personalization config"))?;

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(log_err("Failed to create personalization directory"))?;
    }

    std::fs::write(&path, content).map_err(log_err("Failed to write personalization config"))?;

    log_info!("个性化配置已写入: {}", path.display());

    Ok(())
}

/// 解析个性化配置文件路径
///
/// 复用 `crate::storage::appdata::appdata_root`，与 online/auth/certs/providers 等
/// 全局共享资源保持一致的目录约定（`%APPDATA%/.Molaunch/`）。自动创建父目录。
fn personalization_path() -> Result<PathBuf, String> {
    let dir = crate::storage::appdata::appdata_root()?;
    std::fs::create_dir_all(&dir).map_err(log_err("Failed to create personalization directory"))?;
    Ok(dir.join("personalization.json"))
}
