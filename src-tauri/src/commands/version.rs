//! 版本管理命令

use crate::sdk::VersionList;
use crate::state::AppState;
use tauri::{Manager, State};

/// 下载进度事件
#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub percentage: f64,
}

/// 获取版本列表
#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<VersionList, String> {
    log::info!("Fetching version list");

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    let versions = sdk.list_versions().map_err(|e| {
        log::error!("Failed to list versions: {}", e);
        e.to_string()
    })?;

    log::info!("Found {} versions", versions.versions.len());
    Ok(versions)
}

/// 下载版本（同步阻塞，通过轮询获取进度）
#[tauri::command]
pub async fn download_version(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    log::info!("Downloading version: {}", version_id);

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    // 下载前重置进度
    let _ = sdk.reset_progress();

    let handle_addr = sdk.handle_ptr() as usize;
    let download_fn_addr = sdk.download_fn_addr();
    drop(sdk_guard);

    if handle_addr == 0 {
        return Err("SDK handle is null".to_string());
    }

    let version_id_clone = version_id.clone();
    let config_guard = state.config.lock().await;
    let game_dir = config_guard.game_dir.clone();
    drop(config_guard);

    let result = std::thread::Builder::new()
        .name("sdk-download".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let version_cstr = std::ffi::CString::new(version_id_clone.as_str())
                .map_err(|e| format!("Invalid version id: {}", e))?;

            // mc_download_version(handle, version_id, callback, user_data)
            // 不传回调，SDK 不支持
            let handle = handle_addr as *const std::ffi::c_void;
            let download_fn: unsafe extern "C" fn(
                *const std::ffi::c_void,
                *const std::ffi::c_char,
                *const std::ffi::c_void,
                *mut std::ffi::c_void,
            ) -> i32 = unsafe { std::mem::transmute(download_fn_addr) };

            eprintln!("[download] 调用 mc_download_version(version={:?})...", version_cstr);
            log::info!("[download] Calling mc_download_version for {:?}", version_cstr);
            let code = unsafe {
                download_fn(handle, version_cstr.as_ptr(), std::ptr::null(), std::ptr::null_mut())
            };
            eprintln!("[download] mc_download_version 返回 code={}", code);
            log::info!("[download] mc_download_version returned code={}", code);

            if code != 0 {
                Err(format!("SDK download failed with code: {}", code))
            } else {
                Ok(())
            }
        })
        .map_err(|e| format!("Failed to spawn download thread: {}", e))?
        .join()
        .map_err(|_| "Download thread panicked".to_string())?;

    result?;

    // 下载完成，检查版本目录是否存在
    let version_dir = std::path::Path::new(&game_dir).join("versions").join(&version_id);
    eprintln!("[download] 版本目录: {:?}, 存在: {}", version_dir, version_dir.exists());

    let _ = app.emit_all(
        "download-complete",
        serde_json::json!({ "version_id": version_id }),
    );

    log::info!("Version {} download command completed", version_id);
    Ok(())
}

/// 获取已安装版本列表
#[tauri::command]
pub async fn list_installed_versions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    log::info!("Fetching installed versions");

    let config = state.config.lock().await;
    let game_dir = config.game_dir.clone();
    drop(config);

    log::info!("Game directory: {}", game_dir);

    let versions_dir = std::path::Path::new(&game_dir).join("versions");
    log::info!("Versions directory: {}", versions_dir.display());

    if !versions_dir.exists() {
        log::warn!("Versions directory does not exist");
        return Ok(Vec::new());
    }

    // 读取版本目录
    let mut versions = Vec::new();
    match std::fs::read_dir(&versions_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy().to_string();
                        versions.push(name_str);
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to read versions directory: {}", e);
            return Err(format!("Failed to read: {}", e));
        }
    }

    log::info!("Found {} version directories: {:?}", versions.len(), versions);
    Ok(versions)
}

/// 卸载版本
#[tauri::command]
pub async fn uninstall_version(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    log::info!("Uninstalling version: '{}'", version_id);
    log::info!("Version ID length: {}, bytes: {:?}", version_id.len(), version_id.as_bytes());

    let config = state.config.lock().await;
    let game_dir = config.game_dir.clone();
    drop(config);

    log::info!("Game directory: {}", game_dir);

    // 构建版本目录路径
    let version_dir = std::path::Path::new(&game_dir)
        .join("versions")
        .join(&version_id);

    log::info!("Version directory: {}", version_dir.display());
    log::info!("Version directory exists: {}", version_dir.exists());

    // 列出versions目录下的所有子目录
    let versions_dir = std::path::Path::new(&game_dir).join("versions");
    if versions_dir.exists() {
        log::info!("Listing versions directory:");
        if let Ok(entries) = std::fs::read_dir(&versions_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    log::info!("  - {}", entry.file_name().to_string_lossy());
                }
            }
        }
    }

    if version_dir.exists() {
        std::fs::remove_dir_all(&version_dir).map_err(|e| {
            log::error!("Failed to remove version directory: {}", e);
            format!("Failed to remove version: {}", e)
        })?;
        log::info!("Version {} uninstalled successfully", version_id);
        Ok(())
    } else {
        log::warn!("Version directory not found: {}", version_dir.display());
        Err(format!(
            "Version directory not found: {}",
            version_dir.display()
        ))
    }
}

/// 获取下载进度快照
#[tauri::command]
pub async fn get_download_progress(state: State<'_, AppState>) -> Result<crate::sdk::ProgressSnapshot, String> {
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;
    let progress = sdk.get_progress().map_err(|e| {
        log::error!("[Progress] Failed to get progress: {}", e);
        e.to_string()
    })?;
    
    log::debug!("[Progress] stage={}, current={}, total={}, bytes_downloaded={}, bytes_total={}, speed={}, files_remaining={}, is_active={}, is_complete={}, error_code={}",
        progress.stage, progress.current, progress.total, 
        progress.bytes_downloaded, progress.bytes_total, progress.speed,
        progress.files_remaining, progress.is_active, progress.is_complete, progress.error_code);
    
    Ok(progress)
}

/// 检查是否正在下载
#[tauri::command]
pub async fn is_downloading(state: State<'_, AppState>) -> Result<bool, String> {
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;
    let downloading = sdk.is_downloading().map_err(|e| {
        log::error!("[Downloading] Failed to check downloading status: {}", e);
        e.to_string()
    })?;
    
    log::debug!("[Downloading] is_downloading={}", downloading);
    
    Ok(downloading)
}

/// 重置下载进度
#[tauri::command]
pub async fn reset_download_progress(state: State<'_, AppState>) -> Result<(), String> {
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;
    sdk.reset_progress().map_err(|e| e.to_string())
}

/// 查询 Forge 版本列表
#[tauri::command]
pub async fn list_forge_versions(state: State<'_, AppState>, mc_version: String) -> Result<String, String> {
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;
    sdk.list_forge_versions(&mc_version).map_err(|e| e.to_string())
}

/// 查询 NeoForge 版本列表
#[tauri::command]
pub async fn list_neoforge_versions(state: State<'_, AppState>, mc_version: String) -> Result<String, String> {
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;
    sdk.list_neoforge_versions(&mc_version).map_err(|e| e.to_string())
}

/// 查询 Fabric 版本列表
#[tauri::command]
pub async fn list_fabric_versions(state: State<'_, AppState>) -> Result<String, String> {
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;
    sdk.list_fabric_versions().map_err(|e| e.to_string())
}

/// 查询 OptiFine 版本列表
#[tauri::command]
pub async fn list_optifine_versions(state: State<'_, AppState>) -> Result<String, String> {
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;
    sdk.list_optifine_versions().map_err(|e| e.to_string())
}

/// 查询 LiteLoader 版本列表
#[tauri::command]
pub async fn list_liteloader_versions(state: State<'_, AppState>, mc_version: String) -> Result<String, String> {
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;
    sdk.list_liteloader_versions(&mc_version).map_err(|e| e.to_string())
}

/// 校验加载器兼容性
#[tauri::command]
pub async fn validate_loaders(
    mc_version: String,
    forge_version: Option<String>,
    neoforge_version: Option<String>,
    fabric_version: Option<String>,
    optifine_version: Option<String>,
) -> Result<bool, String> {
    match crate::sdk::validate_loaders(
        &mc_version,
        forge_version.as_deref(),
        neoforge_version.as_deref(),
        fabric_version.as_deref(),
        optifine_version.as_deref(),
    ) {
        Ok(()) => Ok(true),
        Err(crate::sdk::SdkError::FfiFailed(_)) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

/// 合并安装（MC + 加载器）
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn install_merged(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    mc_version: String,
    forge_version: Option<String>,
    neoforge_version: Option<String>,
    fabric_version: Option<String>,
    optifine_version: Option<String>,
    liteloader_version: Option<String>,
    instance_name: Option<String>,
) -> Result<(), String> {
    log::info!("Merged install: mc={}, forge={:?}, neoforge={:?}, fabric={:?}, optifine={:?}",
        mc_version, forge_version, neoforge_version, fabric_version, optifine_version);

    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;
    let _ = sdk.reset_progress();
    let handle_addr = sdk.handle_ptr() as usize;
    let install_fn_addr = sdk.install_merged_fn_addr();
    drop(sdk_guard);

    let mc_version_clone = mc_version.clone();
    let instance = instance_name.clone().unwrap_or_else(|| mc_version.clone());
    let instance_clone = instance.clone();

    let result = std::thread::Builder::new()
        .name("sdk-install".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            use crate::sdk::FFIMergedInstallRequest;

            let mc_c = std::ffi::CString::new(mc_version_clone.as_str())
                .map_err(|e| format!("Invalid mc version: {}", e))?;
            let forge_c = forge_version.map(|v| std::ffi::CString::new(v).map_err(|e| format!("Invalid forge version: {}", e))).transpose()?;
            let neoforge_c = neoforge_version.map(|v| std::ffi::CString::new(v).map_err(|e| format!("Invalid neoforge version: {}", e))).transpose()?;
            let fabric_c = fabric_version.map(|v| std::ffi::CString::new(v).map_err(|e| format!("Invalid fabric version: {}", e))).transpose()?;
            let optifine_c = optifine_version.map(|v| std::ffi::CString::new(v).map_err(|e| format!("Invalid optifine version: {}", e))).transpose()?;
            let liteloader_c = liteloader_version.map(|v| std::ffi::CString::new(v).map_err(|e| format!("Invalid liteloader version: {}", e))).transpose()?;
            let instance_c = std::ffi::CString::new(instance_clone.as_str())
                .map_err(|e| format!("Invalid instance name: {}", e))?;

            let request = FFIMergedInstallRequest {
                mc_version: mc_c.as_ptr(),
                forge_version: forge_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
                neoforge_version: neoforge_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
                fabric_version: fabric_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
                optifine_version: optifine_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
                liteloader_version: liteloader_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
                instance_name: instance_c.as_ptr(),
            };

            let handle = handle_addr as *const std::ffi::c_void;
            let install_fn: unsafe extern "C" fn(
                *const std::ffi::c_void,
                *const FFIMergedInstallRequest,
                *const std::ffi::c_void,
                *mut std::ffi::c_void,
            ) -> i32 = unsafe { std::mem::transmute(install_fn_addr) };

            log::info!("[Install] Calling mc_install_merged...");
            eprintln!("[install] Calling mc_install_merged...");
            
            let code = unsafe { install_fn(handle, &request, std::ptr::null(), std::ptr::null_mut()) };
            
            log::info!("[Install] mc_install_merged returned code={}", code);
            eprintln!("[install] mc_install_merged returned code={}", code);

            if code != 0 {
                log::error!("[Install] SDK install failed with code: {}", code);
                Err(format!("SDK install failed with code: {}", code))
            } else {
                log::info!("[Install] SDK install completed successfully");
                Ok(())
            }
        })
        .map_err(|e| format!("Failed to spawn install thread: {}", e))?
        .join()
        .map_err(|_| "Install thread panicked".to_string())?;

    result?;

    let _ = app.emit_all("install-complete", serde_json::json!({ "instance_name": instance }));
    log::info!("Merged install completed");
    Ok(())
}
