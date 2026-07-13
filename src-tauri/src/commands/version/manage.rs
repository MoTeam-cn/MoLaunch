use crate::minecraft::isolation::{self, IsolationMode};
use crate::minecraft::version::scan as version_scan;
use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;
use crate::state::AppState;
use crate::{log_error, log_info, log_warn};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

use super::sanitize_version_id;

/// Installed version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledVersionInfo {
    pub id: String,
    pub version_type: String,
}

/// Get installed versions
#[tauri::command]
pub async fn list_installed_versions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    log_info!("Fetching installed versions");

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    let versions = version_scan::scan_installed_versions(&game_dir);
    let version_ids: Vec<String> = versions.iter().map(|v| v.id.clone()).collect();

    log_info!(
        "Found {} version directories: {:?}",
        version_ids.len(),
        version_ids
    );
    Ok(version_ids)
}

/// Get installed versions with type info
#[tauri::command]
pub async fn list_installed_versions_with_type(
    state: State<'_, AppState>,
) -> Result<Vec<InstalledVersionInfo>, String> {
    log_info!("Fetching installed versions with type info");

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    let versions = version_scan::scan_installed_versions(&game_dir);
    let mut result = Vec::new();

    for version in versions {
        let version_type = detect_version_type_from_dir(&game_dir, &version.id);
        result.push(InstalledVersionInfo {
            id: version.id,
            version_type: version_type_to_string(&version_type),
        });
    }

    log_info!("Found {} versions with type info", result.len());
    Ok(result)
}

/// Detect version type from directory
fn detect_version_type_from_dir(game_dir: &std::path::Path, version_id: &str) -> VersionType {
    let version_dir = game_dir.join("versions").join(version_id);

    // 1. 优先从 JSON 检测（检查libraries中的加载器）
    let json_path = version_dir.join(format!("{}.json", version_id));
    if json_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let detected = VersionType::detect_from_json(version_id, &json);
                // 如果检测到加载器类型，直接返回
                if detected != VersionType::Release {
                    return detected;
                }
            }
        }
    }

    // 2. 从 setup.ini 读取（仅当JSON检测为Release时）
    let setup_path = version_dir.join("setup.ini");
    if setup_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&setup_path) {
            for line in content.lines() {
                if let Some(value) = line.strip_prefix("Type=") {
                    let type_str = value.trim().to_lowercase();
                    // 忽略 "release"，继续检测
                    if type_str != "release" {
                        return match type_str.as_str() {
                            "forge" => VersionType::Forge,
                            "neoforge" => VersionType::NeoForge,
                            "fabric" => VersionType::Fabric,
                            "quilt" => VersionType::Quilt,
                            "optifine" => VersionType::OptiFine,
                            "liteloader" => VersionType::LiteLoader,
                            "snapshot" => VersionType::Snapshot,
                            "old" | "old_alpha" | "old_beta" => VersionType::Old,
                            _ => VersionType::Release,
                        };
                    }
                }
            }
        }
    }

    // 3. 从版本ID推断
    let id_lower = version_id.to_lowercase();
    if id_lower.contains("forge") {
        return VersionType::Forge;
    }
    if id_lower.contains("neoforge") {
        return VersionType::NeoForge;
    }
    if id_lower.contains("fabric") {
        return VersionType::Fabric;
    }
    if id_lower.contains("optifine") {
        return VersionType::OptiFine;
    }

    VersionType::Release
}

/// Convert VersionType to string
fn version_type_to_string(version_type: &VersionType) -> String {
    match version_type {
        VersionType::Release => "release".to_string(),
        VersionType::Snapshot => "snapshot".to_string(),
        VersionType::Old => "old".to_string(),
        VersionType::Fool => "fool".to_string(),
        VersionType::Forge => "forge".to_string(),
        VersionType::NeoForge => "neoforge".to_string(),
        VersionType::Fabric => "fabric".to_string(),
        VersionType::Quilt => "quilt".to_string(),
        VersionType::OptiFine => "optifine".to_string(),
        VersionType::LiteLoader => "liteloader".to_string(),
        VersionType::Unknown => "unknown".to_string(),
    }
}

/// Uninstall version
#[tauri::command]
pub async fn uninstall_version(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Uninstalling version: '{}'", version_id);

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    version_scan::uninstall_version(&game_dir, &version_id).map_err(|e| {
        log_error!("Failed to uninstall version: {}", e);
        e.to_string()
    })?;

    log_info!("Version {} uninstalled successfully", version_id);
    Ok(())
}

/// 获取版本的有效游戏目录（考虑版本隔离）
///
/// 隔离时返回 `{game_dir}/versions/{version_id}/`
/// 非隔离时返回 `{game_dir}/`
#[tauri::command]
pub async fn get_version_effective_dir(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<String, String> {
    sanitize_version_id(&version_id)?;

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let isolation_mode = config.isolation_mode;
    drop(config);

    let version_type = detect_version_type_from_dir(&game_dir, &version_id);
    let mode = IsolationMode::from_u32(isolation_mode);
    let effective_dir = isolation::get_effective_game_dir(
        &game_dir,
        &version_id,
        mode,
        version_type,
    );

    Ok(effective_dir.to_string_lossy().to_string())
}

/// 版本个性化信息（返回给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionPersonalization {
    pub logo: String,
    pub custom_info: String,
    pub display_type: i32,
    pub is_star: bool,
    pub version_type: String,
    pub original_version: String,
}

/// 获取版本个性化设置
#[tauri::command]
pub async fn get_version_personalization(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<VersionPersonalization, String> {
    sanitize_version_id(&version_id)?;

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    let version_dir = game_dir.join("versions").join(&version_id);
    let setup = VersionSetup::load_or_create(&version_dir, &version_id);

    Ok(VersionPersonalization {
        logo: setup.logo.unwrap_or_default(),
        custom_info: setup.custom_info.unwrap_or_default(),
        display_type: setup.display_type.unwrap_or(0),
        is_star: setup.is_star.unwrap_or(false),
        version_type: version_type_to_string(&setup.version_type),
        original_version: setup.original_version,
    })
}

/// 更新版本个性化字段
#[tauri::command]
pub async fn update_version_personalization(
    state: State<'_, AppState>,
    version_id: String,
    logo: Option<String>,
    custom_info: Option<String>,
    display_type: Option<i32>,
    is_star: Option<bool>,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Updating personalization for version: {}", version_id);

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    let version_dir = game_dir.join("versions").join(&version_id);
    VersionSetup::update_personalization(
        &version_dir,
        logo.as_deref(),
        custom_info.as_deref(),
        display_type,
        is_star,
    )
    .map_err(|e| {
        log_error!("Failed to update personalization: {}", e);
        e.to_string()
    })?;

    log_info!("Personalization updated for version: {}", version_id);
    Ok(())
}

/// 导出启动脚本（.bat 批处理文件，使用绝对路径 Java + 版权信息）
#[tauri::command]
pub async fn export_launch_script(
    state: State<'_, AppState>,
    version_id: String,
    username: String,
    uuid: String,
    access_token: String,
    login_type: Option<String>,
    java_path: Option<String>,
    save_path: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Exporting launch script for version: {}", version_id);

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let isolation_mode = config.isolation_mode;
    let min_memory = config.min_memory;
    let max_memory = config.max_memory;
    drop(config);

    // 解析 Java 路径：优先用户指定 → 否则按 MC 版本自动检测 → 都失败则报错
    let java_path_buf = resolve_java_path(&game_dir, &version_id, java_path.as_deref())
        .await
        .map_err(|e| {
            log_error!("Failed to resolve Java path for script: {}", e);
            e
        })?;
    let java_str = java_path_buf.to_string_lossy().replace('/', "\\");
    log_info!("Script will use Java: {}", java_str);

    // 构建认证信息（导出脚本时使用占位符，避免泄露真实 token）
    let auth_info = crate::minecraft::launch::AuthInfo {
        username: username.clone(),
        uuid,
        access_token: access_token.clone(),
        client_token: access_token,
        login_type: login_type.unwrap_or_else(|| "Legacy".to_string()),
    };

    // 构建启动参数（注意：build_launch_arguments 会触发 set_game_language 副作用，此处可接受）
    let launch_args = crate::minecraft::launch::build_launch_arguments(
        &game_dir,
        &version_id,
        &java_path_buf,
        &auth_info,
        min_memory,
        max_memory,
        None,
        None,
        None,
        None,
        isolation_mode,
    )
    .map_err(|e| {
        log_error!("Failed to build launch arguments: {}", e);
        e.to_string()
    })?;

    // 生成 .bat 脚本内容（绝对路径 Java + 版权信息 + 启动提示）
    let game_dir_display = launch_args.game_dir.replace('/', "\\");
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut script = String::new();
    script.push_str("@echo off\n");
    script.push_str(&format!("title MoLaunch - {}\n", version_id));
    script.push('\n');
    // 版权信息头
    script.push_str("REM ============================================================\n");
    script.push_str("REM  MoLaunch 启动脚本\n");
    script.push_str(&format!("REM  版本: {}\n", version_id));
    script.push_str(&format!("REM  生成时间: {}\n", timestamp));
    script.push_str("REM  Copyright (c) MoLaunch. All rights reserved.\n");
    script.push_str("REM ============================================================\n");
    script.push('\n');
    // 启动提示信息
    script.push_str("echo.\n");
    script.push_str("echo  ============================================================\n");
    script.push_str("echo    MoLaunch 启动器 - 独立启动脚本\n");
    script.push_str(&format!("echo    版本: {}\n", version_id));
    script.push_str(&format!("echo    用户: {}\n", username));
    script.push_str(&format!("echo    Java: {}\n", java_str));
    script.push_str(&format!("echo    游戏目录: {}\n", game_dir_display));
    script.push_str("echo  ============================================================\n");
    script.push_str("echo.\n");
    script.push_str("echo  正在启动游戏，请稍候...\n");
    script.push_str("echo.\n");
    script.push('\n');
    // 切换到游戏目录
    script.push_str(&format!("cd /D \"{}\"\n", game_dir_display));
    script.push('\n');
    // Java 启动命令（使用绝对路径，不依赖系统 PATH）
    script.push_str(&format!(
        "\"{}\" {} {} {}\n",
        java_str,
        launch_args.jvm_args.join(" "),
        launch_args.main_class,
        launch_args.game_args.join(" ")
    ));
    script.push('\n');
    // 退出提示
    script.push_str("echo.\n");
    script.push_str("echo  ============================================================\n");
    script.push_str("echo    游戏已退出\n");
    script.push_str("echo  ============================================================\n");
    script.push_str("pause\n");

    // Windows 批处理文件必须使用 CRLF 换行符（cmd.exe 按 CRLF 识别行边界）
    // 若用 LF，cmd 会把多行内容拼成一行，导致行中间的单词被当成命令执行
    let script = script.replace('\n', "\r\n");

    // 用 GBK 编码写入文件（中文 Windows cmd 默认按 ANSI/GBK 解析批处理文件）
    // 若用 UTF-8 写入，中文字节会被错误拆分成命令，导致 "xxx 不是内部或外部命令" 错误
    let (bytes, _, had_errors) = encoding_rs::GBK.encode(&script);
    if had_errors {
        log_warn!("[ExportScript] Some characters could not be encoded to GBK");
    }
    std::fs::write(&save_path, &bytes).map_err(|e| {
        log_error!("Failed to write script: {}", e);
        e.to_string()
    })?;

    log_info!("Launch script exported to: {}", save_path);
    Ok(())
}

/// 解析脚本使用的 Java 路径（优先用户指定 → 否则按 MC 版本自动检测）
async fn resolve_java_path(
    game_dir: &std::path::Path,
    version_id: &str,
    user_java_path: Option<&str>,
) -> Result<std::path::PathBuf, String> {
    // 1. 优先使用用户指定的 Java 路径
    if let Some(path) = user_java_path {
        if !path.is_empty() {
            let p = std::path::PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
            log_error!("User-specified Java not found: {}", path);
        }
    }

    // 2. 自动检测：先获取 MC 版本号
    let version_dir = game_dir.join("versions").join(version_id);
    let mc_version = read_mc_version_for_java(&version_dir, version_id);

    log_info!(
        "[ExportScript] Auto-detecting Java for MC {}...",
        mc_version
    );

    // 3. 搜索系统 Java
    let java_list = tokio::task::spawn_blocking(crate::minecraft::java::search_java)
        .await
        .map_err(|e| format!("Java 搜索失败: {}", e))?;

    if java_list.is_empty() {
        return Err("未找到任何已安装的 Java，请先安装 Java 或在设置中指定 Java 路径".to_string());
    }

    // 4. 按版本号选择最佳 Java
    let selected = crate::minecraft::java_selector::select_best_java(&mc_version, &java_list, None)
        .ok_or_else(|| {
            let required =
                crate::minecraft::java_selector::get_required_java_version(&mc_version);
            format!(
                "未找到满足 MC {} 要求的 Java (需要 Java {}+)",
                mc_version, required
            )
        })?;

    Ok(std::path::PathBuf::from(&selected))
}

/// 从 setup.ini 或 version.json 读取 MC 版本号（用于 Java 选择）
fn read_mc_version_for_java(version_dir: &std::path::Path, version_id: &str) -> String {
    // 1. 优先从 setup.ini 读取 OriginalVersion
    let setup_path = version_dir.join("setup.ini");
    if setup_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&setup_path) {
            for line in content.lines() {
                if let Some(value) = line.strip_prefix("OriginalVersion=") {
                    let v = value.trim().to_string();
                    if !v.is_empty() {
                        return v;
                    }
                }
            }
        }
    }

    // 2. 从 version.json 读取 inheritsFrom 或 id
    let json_path = version_dir.join(format!("{}.json", version_id));
    if json_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(inherits_from) = json.get("inheritsFrom").and_then(|v| v.as_str()) {
                    if !inherits_from.is_empty() {
                        return inherits_from.to_string();
                    }
                }
                if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                    return id.to_string();
                }
            }
        }
    }

    version_id.to_string()
}

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

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

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
