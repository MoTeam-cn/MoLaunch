//! 版本启动命令

use crate::log_info;
use crate::minecraft::launch::{self, AuthInfo};
use crate::state::{AppState, resolve_game_dir};
use tauri::State;

/// 启动游戏
#[tauri::command]
pub async fn launch_game(
    state: State<'_, AppState>,
    version_id: String,
    java_path: String,
    username: String,
    uuid: String,
    access_token: String,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<String>,
    server_port: Option<u32>,
) -> Result<u32, String> {
    log_info!("Launching game version: {}", version_id);

    let config = state.config.lock().await;
    let game_dir = resolve_game_dir(&config.game_dir);

    // 构建认证信息
    let auth_info = AuthInfo {
        username,
        uuid,
        access_token: access_token.clone(),
        client_token: access_token,
        login_type: "Legacy".to_string(),
    };

    let java = std::path::PathBuf::from(&java_path);

    // 构建启动参数（包含版本隔离逻辑）
    let args = launch::build_launch_arguments(
        &game_dir,
        &version_id,
        &java,
        &auth_info,
        config.min_memory,
        config.max_memory,
        window_width,
        window_height,
        server_address.as_deref(),
        server_port,
        config.isolation_mode,
    )
    .map_err(|e| format!("Failed to build launch arguments: {}", e))?;

    log_info!("Launch arguments built successfully");

    // 启动游戏进程
    // 注意：current_dir 使用原始 game_dir（.minecraft/），而不是隔离目录
    // 隔离目录通过 game_args 中的 --gameDir 参数传递给 Minecraft
    let pid = launch::launch_game(&java, &args, &game_dir)
        .map_err(|e| format!("Failed to launch game: {}", e))?;

    log_info!("Game launched with PID: {}", pid);
    Ok(pid)
}
