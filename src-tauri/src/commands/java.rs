//! Java 管理命令

use crate::log_info;
use crate::minecraft::java;
use crate::minecraft::java_selector;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

/// Java 运行时信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaRuntimeInfo {
    /// java.exe完整路径
    pub executable: String,
    /// 所在文件夹
    pub path_folder: String,
    /// 是否手动导入
    pub is_user_import: bool,
    /// 详细版本号
    pub version: String,
    /// 大版本号
    pub major_version: u32,
    /// 是否为JRE
    pub is_jre: bool,
    /// 是否64位
    pub is_64bit: bool,
}

/// Java 需求信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaRequirements {
    /// MC 版本号
    pub mc_version: String,
    /// 最低 Java 版本（无约束时为 0）
    pub min_java_version: u32,
    /// 最高 Java 版本（无约束时为 0 表示无上限）
    pub max_java_version: u32,
    /// 推荐 Java 版本
    pub recommended_java_version: u32,
    /// 加载器类型（可选）
    pub loader: Option<String>,
}

/// Java 兼容性检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaCompatResult {
    /// 是否兼容
    pub compatible: bool,
    /// 当前 Java 大版本号（无法检测时为 0）
    pub current_version: u32,
    /// 最低要求（无约束时为 0）
    pub min_required: u32,
    /// 最高要求（无约束时为 0 表示无上限）
    pub max_required: u32,
    /// 人类可读的警告信息（不兼容时）
    pub warning: String,
}

/// 检测 Java
#[tauri::command]
pub async fn detect_java(_state: State<'_, AppState>) -> Result<JavaRuntimeInfo, String> {
    log_info!("Detecting Java...");

    // 从环境变量中查找Java
    let java_list = java::search_java();

    if java_list.is_empty() {
        return Err("No Java found".to_string());
    }

    // 选择最佳Java
    let best_java =
        java::select_best_java(&java_list, None, None).ok_or("No suitable Java found")?;

    let result = JavaRuntimeInfo {
        executable: best_java.executable.clone(),
        path_folder: best_java.path_folder.clone(),
        is_user_import: best_java.is_user_import,
        version: best_java.version.clone(),
        major_version: best_java.major_version,
        is_jre: best_java.is_jre,
        is_64bit: best_java.is_64bit,
    };

    log_info!("Java detected: {} ({})", result.version, result.executable);
    Ok(result)
}

/// 列出所有 Java
#[tauri::command]
pub async fn list_java(_state: State<'_, AppState>) -> Result<Vec<JavaRuntimeInfo>, String> {
    log_info!("Listing all Java runtimes...");

    let java_list = java::search_java();

    let result: Vec<JavaRuntimeInfo> = java_list
        .iter()
        .map(|j| JavaRuntimeInfo {
            executable: j.executable.clone(),
            path_folder: j.path_folder.clone(),
            is_user_import: j.is_user_import,
            version: j.version.clone(),
            major_version: j.major_version,
            is_jre: j.is_jre,
            is_64bit: j.is_64bit,
        })
        .collect();

    log_info!("Found {} Java runtimes", result.len());
    Ok(result)
}

/// 根据 MC 版本选择最佳 Java
#[tauri::command]
pub async fn select_java_for_mc(
    mc_version: String,
    user_java_path: Option<String>,
    _state: State<'_, AppState>,
) -> Result<JavaRuntimeInfo, String> {
    log_info!("Selecting Java for MC {}...", mc_version);

    let java_list = java::search_java();

    if java_list.is_empty() {
        return Err("No Java found".to_string());
    }

    let best_java =
        java_selector::select_best_java(&mc_version, &java_list, user_java_path.as_deref())
            .ok_or_else(|| {
                let required = java_selector::get_required_java_version(&mc_version);
                format!(
                    "No suitable Java found for MC {} (requires Java {}+)",
                    mc_version, required
                )
            })?;

    let java = java_list
        .iter()
        .find(|j| j.executable == best_java)
        .ok_or("Selected Java not found in list")?;

    let result = JavaRuntimeInfo {
        executable: java.executable.clone(),
        path_folder: java.path_folder.clone(),
        is_user_import: java.is_user_import,
        version: java.version.clone(),
        major_version: java.major_version,
        is_jre: java.is_jre,
        is_64bit: java.is_64bit,
    };

    log_info!(
        "Selected Java for MC {}: {} ({})",
        mc_version,
        result.version,
        result.executable
    );
    Ok(result)
}

/// 获取 MC 版本的 Java 需求（支持加载器约束）
#[tauri::command]
pub async fn get_java_requirements(
    mc_version: String,
    loader: Option<String>,
) -> Result<JavaRequirements, String> {
    let (min, max) = java_selector::get_java_version_range(&mc_version, loader.as_deref());
    let recommended = java_selector::get_recommended_java_version(&mc_version);

    Ok(JavaRequirements {
        mc_version,
        min_java_version: min.unwrap_or(0),
        max_java_version: max.unwrap_or(0),
        recommended_java_version: recommended,
        loader,
    })
}

/// 检查指定 Java 是否兼容 MC 版本需求
///
/// - `java_path`: Java 可执行文件路径（如 "C:\\jdk-17\\bin\\java.exe"）
/// - `mc_version`: MC 版本号
/// - `loader`: 加载器类型（可选）
#[tauri::command]
pub async fn check_java_compatible(
    java_path: String,
    mc_version: String,
    loader: Option<String>,
) -> Result<JavaCompatResult, String> {
    use std::path::Path;

    let path = Path::new(&java_path);
    if !path.exists() {
        return Ok(JavaCompatResult {
            compatible: false,
            current_version: 0,
            min_required: 0,
            max_required: 0,
            warning: format!("Java 路径不存在: {}", java_path),
        });
    }

    // 调用 java -version 检测版本
    let current_version = match crate::minecraft::java::detect_java_version(&java_path) {
        Some(v) => v,
        None => {
            return Ok(JavaCompatResult {
                compatible: true, // 无法检测版本时不阻断，仅警告
                current_version: 0,
                min_required: 0,
                max_required: 0,
                warning: "无法检测 Java 版本，已跳过兼容性检查".to_string(),
            });
        }
    };

    let (min, max) = java_selector::get_java_version_range(&mc_version, loader.as_deref());

    let check =
        java_selector::check_java_compatible(current_version, &mc_version, loader.as_deref());

    let warning = match &check {
        Ok(()) => String::new(),
        Err((cur, min_req, max_req)) => {
            let req_desc =
                crate::minecraft::java_selector::describe_java_requirement(*min_req, *max_req);
            format!("当前版本{}，{}，可能导致游戏崩溃", cur, req_desc)
        }
    };

    Ok(JavaCompatResult {
        compatible: check.is_ok(),
        current_version,
        min_required: min.unwrap_or(0),
        max_required: max.unwrap_or(0),
        warning,
    })
}

/// 下载 Java Runtime（从 Mojang 官方 Java Runtime 索引）
///
/// - `target_major`: 目标 Java 大版本号（如 21、17、8）
///
/// 推送 `java-download-progress` 事件，payload 为 `JavaDownloadProgress`：
/// ```json
/// { "stage": "fetching|matching|manifest|downloading|verifying|done",
///   "current": 0, "total": 0, "bytes_downloaded": 0, "bytes_total": 0, "message": "" }
/// ```
///
/// 返回下载的 java.exe 完整路径
#[tauri::command]
pub async fn download_java(
    target_major: u32,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log_info!("[JavaDownload] Start downloading Java {}", target_major);

    let config = state.config.lock().await;
    let dl_mode = crate::minecraft::sources::DownloadSourceMode::from_str(&config.download_source);
    let mirror_url = config.mirror_url.clone();
    drop(config);

    let java_exe = java::download::download_java_runtime(
        target_major,
        dl_mode,
        mirror_url.as_deref(),
        Some(&app),
    )
    .await?;

    let path_str = java_exe.to_string_lossy().to_string();
    log_info!("[JavaDownload] Done: {}", path_str);
    Ok(path_str)
}
