//! MoLaunch 主入口

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mo_launch_lib::commands;
use mo_launch_lib::state::AppState;
use tauri::Manager;

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
            commands::sdk::get_device_id,
            // 认证命令
            commands::auth::login_offline,
            commands::auth::get_login_status,
            commands::auth::logout,
            // 版本命令
            commands::version::list_versions,
            commands::version::download_version,
            commands::version::list_installed_versions,
            commands::version::uninstall_version,
            commands::version::get_download_progress,
            commands::version::is_downloading,
            commands::version::reset_download_progress,
            commands::version::list_forge_versions,
            commands::version::list_neoforge_versions,
            commands::version::list_fabric_versions,
            commands::version::list_optifine_versions,
            commands::version::list_liteloader_versions,
            commands::version::validate_loaders,
            commands::version::install_merged,
            // Java 命令
            commands::java::detect_java,
            commands::java::list_java,
            // 系统命令
            commands::system::open_game_dir,
            commands::system::get_game_dir,
            commands::system::select_folder,
            commands::system::select_file,
            commands::system::set_game_dir,
            commands::system::set_mirror_url,
            commands::system::get_mirror_url,
            commands::system::set_download_source,
            commands::system::get_download_source,
            commands::system::set_max_download_speed,
            commands::system::get_max_download_speed,
            commands::system::get_system_memory,
            commands::system::get_config_path,
            commands::system::save_config_to_file,
            commands::system::set_min_memory,
            commands::system::set_max_memory,
            commands::system::get_memory_config,
            commands::system::set_max_download_threads,
            commands::system::get_max_download_threads,
            commands::system::reinitialize_sdk_command,
        ])
        .on_window_event(|event| {
            // 窗口关闭时保存配置
            if let tauri::WindowEvent::CloseRequested { .. } = event.event() {
                log::info!("Window close requested, saving config...");
                let state = event.window().state::<AppState>();
                let config = state.config.blocking_lock();
                if let Err(e) = mo_launch_lib::config::save_config(&config) {
                    log::error!("Failed to save config on exit: {}", e);
                } else {
                    log::info!("Config saved successfully on exit");
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running MoLaunch");
}
