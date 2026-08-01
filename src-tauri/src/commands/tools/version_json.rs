//! 版本 JSON 读写
//! - `read`：读取 `{game_dir}/versions/{version_id}/{version_id}.json`
//! - `save`：先校验内容为合法 JSON，校验通过后写回文件
//!
//! 路径安全：version_id 不允许含 ".." 或路径分隔符，防穿越。

use std::path::Path;

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::state::resolve_game_dir;
use crate::state::AppState;

use super::types::{
    VersionJsonReadParams, VersionJsonReadResult, VersionJsonSaveParams, VersionJsonSaveResult,
};

/// 校验 version_id 不含路径穿越字符
fn validate_version_id(version_id: &str) -> Result<(), String> {
    if version_id.is_empty() {
        return Err("version_id 不能为空".to_string());
    }
    if version_id.contains("..") {
        return Err("version_id 不允许包含 \"..\"".to_string());
    }
    if version_id.contains('/') || version_id.contains('\\') {
        return Err("version_id 不允许包含路径分隔符".to_string());
    }
    Ok(())
}

/// 读取版本 JSON 文件，文件不存在返回 Err
pub async fn read(
    state: &AppState,
    params: VersionJsonReadParams,
) -> Result<serde_json::Value, String> {
    validate_version_id(&params.version_id)?;

    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    let json_path = game_dir
        .join("versions")
        .join(&params.version_id)
        .join(format!("{}.json", params.version_id));

    log_info!("[VersionJson] 读取: {}", json_path.display());

    if !json_path.exists() {
        log_warn!("[VersionJson] 文件不存在: {}", json_path.display());
        return Err(format!("版本 JSON 文件不存在: {}", json_path.display()));
    }

    let json_path_clone = json_path.clone();
    let (content, path_str) =
        tokio::task::spawn_blocking(move || -> Result<(String, String), String> {
            let content = std::fs::read_to_string(&json_path_clone)
                .map_err(|e| format!("读取文件失败: {}", e))?;
            let path_str = json_path_clone.to_str().unwrap_or("").to_string();
            Ok((content, path_str))
        })
        .await
        .map_err(log_err("VersionJson 读取任务失败"))??;

    log_info!("[VersionJson] 读取成功: {} 字节", content.len());

    let result = VersionJsonReadResult {
        content,
        path: path_str,
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 保存版本 JSON（先校验合法性，失败返回含具体解析错误的 Err）
pub async fn save(
    state: &AppState,
    params: VersionJsonSaveParams,
) -> Result<serde_json::Value, String> {
    validate_version_id(&params.version_id)?;

    // 先校验是合法 JSON
    if let Err(e) = serde_json::from_str::<serde_json::Value>(&params.content) {
        return Err(format!("JSON 解析失败: {}", e));
    }

    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    let json_path = game_dir
        .join("versions")
        .join(&params.version_id)
        .join(format!("{}.json", params.version_id));

    log_info!("[VersionJson] 保存: {}", json_path.display());

    let json_path_clone = json_path.clone();
    let content = params.content;
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        // 确保父目录存在
        if let Some(parent) = json_path_clone.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建版本目录失败: {}", e))?;
        }
        std::fs::write(&json_path_clone, content.as_bytes())
            .map_err(|e| format!("写入文件失败: {}", e))
    })
    .await
    .map_err(log_err("VersionJson 保存任务失败"))??;

    log_info!("[VersionJson] 保存成功: {}", json_path.display());

    let result = VersionJsonSaveResult { success: true };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 将路径转为字符串（未使用，保留以备后续扩展）
#[allow(dead_code)]
fn path_to_string(path: &Path) -> String {
    path.to_str().unwrap_or("").to_string()
}
