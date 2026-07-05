//! 游戏设置相关命令

use crate::log_info;
use crate::state::AppState;
use tauri::State;

/// 设置最小内存
#[tauri::command]
pub async fn set_min_memory(
    state: State<'_, AppState>,
    memory: u32,
) -> Result<(), String> {
    log_info!("Min memory changed to: {} MB", memory);
    super::update_config(&state, |config| {
        config.min_memory = memory;
    }).await
}

/// 设置最大内存
#[tauri::command]
pub async fn set_max_memory(
    state: State<'_, AppState>,
    memory: u32,
) -> Result<(), String> {
    log_info!("Max memory changed to: {} MB", memory);
    super::update_config(&state, |config| {
        config.max_memory = memory;
    }).await
}

/// 获取内存配置
#[tauri::command]
pub async fn get_memory_config(state: State<'_, AppState>) -> Result<(u32, u32), String> {
    let config = state.config.lock().await;
    Ok((config.min_memory, config.max_memory))
}

/// 设置版本隔离模式
#[tauri::command]
pub async fn set_isolation_mode(
    state: State<'_, AppState>,
    mode: u32,
) -> Result<(), String> {
    log_info!("Isolation mode changed to: {}", mode);
    super::update_config(&state, |config| {
        config.isolation_mode = mode;
    }).await
}

/// 获取版本隔离模式
#[tauri::command]
pub async fn get_isolation_mode(state: State<'_, AppState>) -> Result<u32, String> {
    let config = state.config.lock().await;
    Ok(config.isolation_mode)
}

/// 获取日志级别
#[tauri::command]
pub async fn get_log_level(state: State<'_, AppState>) -> Result<u32, String> {
    let config = state.config.lock().await;
    Ok(config.log_level)
}

/// 设置日志级别
#[tauri::command]
pub async fn set_log_level(
    state: State<'_, AppState>,
    level: u32,
) -> Result<(), String> {
    log_info!("Log level changed to: {}", level);
    super::update_config(&state, |config| {
        config.log_level = level;
    }).await
}
