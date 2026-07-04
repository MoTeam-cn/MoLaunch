//! Java 管理命令

use crate::log_info;
use crate::minecraft::java;
use crate::minecraft::java_selector;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

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
    /// 最低 Java 版本
    pub min_java_version: u32,
    /// 推荐 Java 版本
    pub recommended_java_version: u32,
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
    let best_java = java::select_best_java(&java_list, None, None)
        .ok_or("No suitable Java found")?;
    
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
    
    let result: Vec<JavaRuntimeInfo> = java_list.iter().map(|j| JavaRuntimeInfo {
        executable: j.executable.clone(),
        path_folder: j.path_folder.clone(),
        is_user_import: j.is_user_import,
        version: j.version.clone(),
        major_version: j.major_version,
        is_jre: j.is_jre,
        is_64bit: j.is_64bit,
    }).collect();

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

    let best_java = java_selector::select_best_java(
        &mc_version,
        &java_list,
        user_java_path.as_deref(),
    )
    .ok_or_else(|| {
        let required = java_selector::get_required_java_version(&mc_version);
        format!("No suitable Java found for MC {} (requires Java {}+)", mc_version, required)
    })?;

    let java = java_list.iter().find(|j| j.executable == best_java)
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

    log_info!("Selected Java for MC {}: {} ({})", mc_version, result.version, result.executable);
    Ok(result)
}

/// 获取 MC 版本的 Java 需求
#[tauri::command]
pub async fn get_java_requirements(mc_version: String) -> Result<JavaRequirements, String> {
    let required = java_selector::get_required_java_version(&mc_version);
    let recommended = java_selector::get_recommended_java_version(&mc_version);

    Ok(JavaRequirements {
        mc_version,
        min_java_version: required,
        recommended_java_version: recommended,
    })
}
