//! 下载设置相关命令

use crate::log_info;
use crate::state::AppState;
use tauri::State;

/// 设置镜像源
#[tauri::command]
pub async fn set_mirror_url(
    state: State<'_, AppState>,
    mirror_url: Option<String>,
    _skip_reinit: Option<bool>,
) -> Result<(), String> {
    log_info!("Mirror URL changed to: {:?}", mirror_url);
    super::update_config(&state, |config| {
        config.mirror_url = mirror_url;
    }).await
}

/// 获取镜像源
#[tauri::command]
pub async fn get_mirror_url(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let config = state.config.lock().await;
    Ok(config.mirror_url.clone())
}

/// 设置下载源模式
#[tauri::command]
pub async fn set_download_source(
    state: State<'_, AppState>,
    source: String,
    _skip_reinit: Option<bool>,
) -> Result<(), String> {
    let bmclapi = crate::minecraft::sources::BMCLAPI_BASE;

    log_info!("Download source changed to: {}", source);
    super::update_config(&state, |config| {
        match source.as_str() {
            "mirror" => {
                config.mirror_url_meta = Some(bmclapi.to_string());
                config.mirror_url_download = Some(bmclapi.to_string());
                config.mirror_url = Some(bmclapi.to_string());
                config.mirror_mode = 0;
            }
            "official" => {
                config.mirror_url_meta = None;
                config.mirror_url_download = None;
                config.mirror_url = None;
                config.mirror_mode = 0;
            }
            "smart" => {
                config.mirror_url_meta = None;
                config.mirror_url_download = None;
                config.mirror_url = None;
                config.mirror_mode = 1;
            }
            _ => {}
        }
        config.download_source = source;
    }).await
}

/// 获取下载源模式
#[tauri::command]
pub async fn get_download_source(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(config.download_source.clone())
}

/// 设置最大下载速度
#[tauri::command]
pub async fn set_max_download_speed(
    state: State<'_, AppState>,
    speed: u64,
    _skip_reinit: Option<bool>,
) -> Result<(), String> {
    log_info!("Max download speed changed to: {} bytes/sec", speed);
    super::update_config(&state, |config| {
        config.max_download_speed = speed;
    }).await
}

/// 获取最大下载速度
#[tauri::command]
pub async fn get_max_download_speed(state: State<'_, AppState>) -> Result<u64, String> {
    let config = state.config.lock().await;
    Ok(config.max_download_speed)
}

/// 设置下载线程数
#[tauri::command]
pub async fn set_max_download_threads(
    state: State<'_, AppState>,
    threads: u32,
) -> Result<(), String> {
    log_info!("Max download threads changed to: {}", threads);
    super::update_config(&state, |config| {
        config.max_download_threads = threads;
    }).await
}

/// 获取下载线程数
#[tauri::command]
pub async fn get_max_download_threads(state: State<'_, AppState>) -> Result<u32, String> {
    let config = state.config.lock().await;
    Ok(config.max_download_threads)
}

/// 设置分片数量
#[tauri::command]
pub async fn set_chunk_count(
    state: State<'_, AppState>,
    count: u32,
) -> Result<(), String> {
    log_info!("Chunk count changed to: {}", count);
    super::update_config(&state, |config| {
        config.chunk_count = count;
    }).await
}

/// 获取分片数量
#[tauri::command]
pub async fn get_chunk_count(state: State<'_, AppState>) -> Result<u32, String> {
    let config = state.config.lock().await;
    Ok(config.chunk_count)
}
