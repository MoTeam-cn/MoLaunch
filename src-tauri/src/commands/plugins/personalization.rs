//! 个性化配置读写（%APPDATA%/.MolaLaunch/personalization.json）
//!
//! - `read_personalization`：读取个性化配置
//! - `write_personalization`：写入个性化配置
//!
//! 设计目的：将个性化配置（插件启用状态 / 主页模式 / 自定义布局配置）
//! 存储到 AppData 而非游戏目录，确保不同 game_dir 的启动器实例加载同一份配置。
//!
//! 存储路径：
//! - Windows: `%APPDATA%/.MolaLaunch/personalization.json`
//! - 其他平台: `~/.config/MolaLaunch/personalization.json`
//!
//! 文件格式：JSON（直接透传前端传来的 `serde_json::Value`，全量覆盖写入）。

use crate::log_info;
use std::path::PathBuf;

/// 读取个性化配置
///
/// 文件不存在时返回空 JSON 对象 `{}`，调用方按 Partial 语义合并默认值。
#[tauri::command]
pub async fn read_personalization() -> Result<serde_json::Value, String> {
    let path = personalization_path()?;

    if !path.exists() {
        return Ok(serde_json::json!({}));
    }

    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    // 解析失败时返回空对象而非错误（容错：损坏的 JSON 不阻塞启动）
    let value: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| {
            crate::log_warn!("个性化配置 JSON 损坏，已忽略: {}", e);
            serde_json::json!({})
        });

    Ok(value)
}

/// 写入个性化配置
///
/// 全量覆盖写入。调用方传入完整的 `PersonalizationData` 结构。
#[tauri::command]
pub async fn write_personalization(data: serde_json::Value) -> Result<(), String> {
    let path = personalization_path()?;

    // 美化输出（4 空格缩进），便于用户手动查看 / 编辑
    let content = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    std::fs::write(&path, content).map_err(|e| e.to_string())?;

    log_info!("个性化配置已写入: {}", path.display());

    Ok(())
}

/// 解析个性化配置文件路径
///
/// 自动创建父目录。
fn personalization_path() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "APPDATA environment variable not set".to_string())?;
        let dir = PathBuf::from(appdata).join(".MolaLaunch");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir.join("personalization.json"))
    }

    #[cfg(not(windows))]
    {
        let home = std::env::var("HOME")
            .map_err(|_| "HOME environment variable not set".to_string())?;
        let dir = PathBuf::from(home).join(".config").join("MolaLaunch");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir.join("personalization.json"))
    }
}
