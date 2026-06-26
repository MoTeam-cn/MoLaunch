//! MoLaunch 主入口

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mo_launch_lib::commands;
use mo_launch_lib::state::AppState;

fn main() {
    // 初始化日志
    env_logger::init();
    
    log::info!("Starting MoLaunch v{}", env!("CARGO_PKG_VERSION"));
    
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // SDK 命令
            commands::sdk::get_platform_info,
            commands::sdk::initialize_sdk,
            commands::sdk::get_sdk_version,
            commands::sdk::is_sdk_initialized,
            // 认证命令
            commands::auth::login_offline,
            commands::auth::get_login_status,
            commands::auth::logout,
            // 版本命令
            commands::version::list_versions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MoLaunch");
}
