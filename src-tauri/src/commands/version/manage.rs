//! 版本管理：文件补全、重命名、选中版本持久化

use crate::state::AppState;
use crate::{log_error, log_info};
use tauri::{Emitter, State};

use super::sanitize_version_id;

/// 补全版本文件（参考 PCL2 BtnManageCheck，校验并下载缺失的 libraries/assets）
#[tauri::command]
pub async fn fix_version_files(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    version_id: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Fixing version files for: {}", version_id);

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let mirror_url = config.mirror_url.clone();
    let max_threads = config.max_download_threads as usize;
    let chunk_count = config.chunk_count as usize;
    let speed_limit = config.max_download_speed;
    let source_mode =
        crate::minecraft::sources::DownloadSourceMode::from_str(&config.download_source);
    drop(config);

    // 通知前端开始
    let _ = app_handle.emit(
        "version-fix-progress",
        serde_json::json!({
            "version_id": version_id,
            "stage": "starting",
            "message": "开始补全文件"
        }),
    );

    let result = crate::minecraft::download::fix_version_files(
        &version_id,
        &game_dir,
        mirror_url.as_deref(),
        max_threads,
        chunk_count,
        speed_limit,
        source_mode,
    )
    .await;

    match result {
        Ok(_) => {
            log_info!("Version files fixed successfully: {}", version_id);
            let _ = app_handle.emit(
                "version-fix-progress",
                serde_json::json!({
                    "version_id": version_id,
                    "stage": "finished",
                    "message": "补全完成"
                }),
            );
            Ok(())
        }
        Err(e) => {
            let msg = e.to_string();
            log_error!("Failed to fix version files: {}", msg);
            let _ = app_handle.emit(
                "version-fix-progress",
                serde_json::json!({
                    "version_id": version_id,
                    "stage": "failed",
                    "message": msg
                }),
            );
            Err(msg)
        }
    }
}

/// 重命名版本（参考 PCL2 BtnDisplayRename_Click）
#[tauri::command]
pub async fn rename_version(
    state: State<'_, AppState>,
    version_id: String,
    new_name: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    sanitize_version_id(&new_name)?;

    if version_id == new_name {
        return Err("新名称与原名称相同".to_string());
    }

    log_info!("Renaming version: {} -> {}", version_id, new_name);

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;

    let versions_dir = game_dir.join("versions");
    let old_dir = versions_dir.join(&version_id);
    let new_dir = versions_dir.join(&new_name);

    if !old_dir.exists() {
        return Err(format!("版本 {} 不存在", version_id));
    }
    if new_dir.exists() {
        return Err(format!("目标名称 {} 已存在", new_name));
    }

    // 1. 重命名版本文件夹
    std::fs::rename(&old_dir, &new_dir).map_err(|e| {
        log_error!("Failed to rename version dir: {}", e);
        e.to_string()
    })?;

    // 2. 重命名 jar 文件
    let old_jar = new_dir.join(format!("{}.jar", version_id));
    let new_jar = new_dir.join(format!("{}.jar", new_name));
    if old_jar.exists() {
        if let Err(e) = std::fs::rename(&old_jar, &new_jar) {
            log_error!("Failed to rename jar: {}", e);
        }
    }

    // 3. 重命名 JSON 文件
    let old_json = new_dir.join(format!("{}.json", version_id));
    let new_json = new_dir.join(format!("{}.json", new_name));
    if old_json.exists() {
        // 读取 JSON 并更新 id 字段
        if let Ok(content) = std::fs::read_to_string(&old_json) {
            if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                // 更新 id 字段为新版本名
                json["id"] = serde_json::Value::String(new_name.clone());
                if let Ok(new_content) = serde_json::to_string_pretty(&json) {
                    let _ = std::fs::write(&new_json, new_content);
                    let _ = std::fs::remove_file(&old_json);
                }
            }
        }
        if !new_json.exists() {
            // JSON 更新失败时简单重命名
            let _ = std::fs::rename(&old_json, &new_json);
        }
    }

    // 4. 重命名 natives 文件夹
    let old_natives = new_dir.join(format!("{}-natives", version_id));
    let new_natives = new_dir.join(format!("{}-natives", new_name));
    if old_natives.exists() {
        let _ = std::fs::rename(&old_natives, &new_natives);
    }

    log_info!("Version renamed successfully: {} -> {}", version_id, new_name);
    Ok(())
}

/// 获取上次选中的版本（持久化）
#[tauri::command]
pub async fn get_selected_version(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let config = state.config.lock().await;
    Ok(config.selected_version.clone())
}

/// 保存当前选中的版本（持久化到 config.ini）
#[tauri::command]
pub async fn set_selected_version(
    state: State<'_, AppState>,
    version_id: Option<String>,
) -> Result<(), String> {
    crate::commands::system::update_config(&state, |config| {
        config.selected_version = version_id.clone();
    })
    .await?;
    log_info!("Selected version saved: {:?}", version_id);
    Ok(())
}
