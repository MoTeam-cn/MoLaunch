//! 代理设置相关命令

use crate::log_info;
use crate::state::AppState;
use tauri::State;

/// 获取代理模式
#[tauri::command]
pub async fn get_proxy_mode(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(config.proxy_mode.clone())
}

/// 设置代理模式
#[tauri::command]
pub async fn set_proxy_mode(
    state: State<'_, AppState>,
    mode: String,
) -> Result<(), String> {
    log_info!("Proxy mode changed to: {}", mode);
    super::update_config(&state, |config| {
        config.proxy_mode = mode;
    }).await
}

/// 获取代理类型
#[tauri::command]
pub async fn get_proxy_type(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(config.proxy_type.clone())
}

/// 设置代理类型
#[tauri::command]
pub async fn set_proxy_type(
    state: State<'_, AppState>,
    proxy_type: String,
) -> Result<(), String> {
    log_info!("Proxy type changed to: {}", proxy_type);
    super::update_config(&state, |config| {
        config.proxy_type = proxy_type;
    }).await
}

/// 获取代理地址
#[tauri::command]
pub async fn get_proxy_url(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(config.proxy_url.clone())
}

/// 设置代理地址
#[tauri::command]
pub async fn set_proxy_url(
    state: State<'_, AppState>,
    url: String,
) -> Result<(), String> {
    log_info!("Proxy URL changed to: {}", url);
    super::update_config(&state, |config| {
        config.proxy_url = url;
    }).await
}
