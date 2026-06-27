//! SDK 管理命令

use crate::sdk::SdkInstance;
use crate::state::AppState;
use tauri::State;

/// 获取 SDK 状态信息
#[derive(serde::Serialize)]
pub struct SdkStatus {
    pub loaded: bool,
    pub version: Option<String>,
    pub platform: String,
    pub library_path: String,
}

/// 获取当前平台信息
#[tauri::command]
pub async fn get_platform_info() -> Result<SdkStatus, String> {
    let platform = if cfg!(target_os = "windows") {
        "windows-x86_64"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "macos-aarch64"
        } else {
            "macos-x86_64"
        }
    } else if cfg!(target_os = "linux") {
        "linux-x86_64"
    } else {
        "unknown"
    };

    let library_path = crate::sdk::get_sdk_library_path()
        .to_string_lossy()
        .to_string();

    Ok(SdkStatus {
        loaded: false,
        version: None,
        platform: platform.to_string(),
        library_path,
    })
}

/// 初始化 SDK
#[tauri::command]
pub async fn initialize_sdk(
    state: State<'_, AppState>,
    game_dir: Option<String>,
) -> Result<String, String> {
    log::info!("Initializing SDK...");

    let config = state.config.lock().await;
    let game_dir = game_dir.unwrap_or_else(|| config.game_dir.clone());
    let max_threads = config.max_download_threads;
    let log_level = config.log_level;
    let mirror_url = config.mirror_url.clone();
    let mirror_url_meta = config.mirror_url_meta.clone();
    let mirror_url_download = config.mirror_url_download.clone();
    let max_download_speed = config.max_download_speed;
    drop(config);

    log::info!("Game directory: {}", game_dir);
    log::info!("Mirror URL: {:?}", mirror_url);
    log::info!("Mirror meta: {:?}, download: {:?}", mirror_url_meta, mirror_url_download);
    log::info!("Max download speed: {} bytes/sec", max_download_speed);

    // 加载 SDK 库
    let mut sdk = SdkInstance::load().map_err(|e| {
        log::error!("Failed to load SDK: {}", e);
        e.to_string()
    })?;

    // 初始化 SDK
    sdk.init(
        &game_dir,
        max_threads,
        log_level,
        mirror_url.as_deref(),
        mirror_url_meta.as_deref(),
        mirror_url_download.as_deref(),
        max_download_speed,
    ).map_err(|e| {
        log::error!("Failed to initialize SDK: {}", e);
        e.to_string()
    })?;

    let version = sdk.version();
    log::info!("SDK loaded successfully, version: {}", version);

    // 保存 SDK 实例
    let mut sdk_guard = state.sdk.lock().await;
    *sdk_guard = Some(sdk);

    Ok(version)
}

/// 获取 SDK 版本
#[tauri::command]
pub async fn get_sdk_version(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let sdk_guard = state.sdk.lock().await;
    Ok(sdk_guard.as_ref().map(|sdk| sdk.version()))
}

/// 检查 SDK 是否已初始化
#[tauri::command]
pub async fn is_sdk_initialized(state: State<'_, AppState>) -> Result<bool, String> {
    let sdk_guard = state.sdk.lock().await;
    Ok(sdk_guard.is_some())
}

/// 获取设备 ID
#[tauri::command]
pub async fn get_device_id(state: State<'_, AppState>) -> Result<String, String> {
    let sdk_guard = state.sdk.lock().await;
    let sdk = sdk_guard.as_ref().ok_or("SDK not initialized")?;

    sdk.get_device_id().map_err(|e| {
        log::error!("Failed to get device ID: {}", e);
        e.to_string()
    })
}
