//! 系统命令

use crate::state::AppState;
use tauri::State;

/// 打开游戏目录
#[tauri::command]
pub async fn open_game_dir(state: State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().await;
    let game_dir = &config.game_dir;

    log::info!("Opening game directory: {}", game_dir);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(game_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(game_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(game_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    Ok(())
}

/// 获取游戏目录
#[tauri::command]
pub async fn get_game_dir(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(config.game_dir.clone())
}
