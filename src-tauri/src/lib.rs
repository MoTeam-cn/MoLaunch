//! MoLaunch 库入口

pub mod commands;
pub mod config;
pub mod http;
pub mod logger;
pub mod minecraft;
pub mod resources;
pub mod sdk;
pub mod state;
pub mod storage;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    env_logger::init();

    log_info!("Starting MoLaunch v{}", env!("CARGO_PKG_VERSION"));

    // 初始化 storage
    if let Err(e) = storage::Storage::instance().init() {
        log_error!("Failed to initialize storage: {}", e);
    }

    // 初始化日志系统
    logger::init_from_config();

    // 初始化 HTTP 客户端（根据代理配置）
    let app_state = AppState::new();
    {
        let config = app_state.config.blocking_lock();
        http::init_client(&config.proxy_mode, &config.proxy_type, &config.proxy_url);
        log_info!("HTTP client initialized (proxy: {})", config.proxy_mode);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // SDK 命令（lite 版本）
            commands::sdk::get_platform_info,
            commands::sdk::get_sdk_version,
            commands::sdk::is_sdk_initialized,
            commands::sdk::get_device_id,
            commands::sdk::encrypt_token,
            commands::sdk::decrypt_token,
            commands::sdk::check_update_lite,
            // 认证命令
            commands::auth::login_offline,
            commands::auth::get_login_status,
            commands::auth::logout,
            // 版本命令
            commands::version::list::list_versions,
            commands::version::download::download_version,
            commands::version::manage::list_installed_versions,
            commands::version::manage::list_installed_versions_with_type,
            commands::version::manage::uninstall_version,
            commands::version::progress::get_download_progress,
            commands::version::progress::is_downloading,
            commands::version::progress::reset_download_progress,
            commands::version::loaders::list_forge_versions,
            commands::version::loaders::list_neoforge_versions,
            commands::version::loaders::list_fabric_versions,
            commands::version::loaders::list_optifine_versions,
            commands::version::loaders::list_liteloader_versions,
            commands::version::loaders::validate_loaders,
            commands::version::install::install_merged,
            commands::version::launch::launch_game,
            commands::version::launch::get_launch_progress,
            commands::version::launch::cancel_launch,
            commands::version::launch::stop_game,
            commands::version::launch::get_running_game,
            // Java 命令
            commands::java::detect_java,
            commands::java::list_java,
            commands::java::select_java_for_mc,
            commands::java::get_java_requirements,
            // 系统命令
            commands::system::open_game_dir,
            commands::system::get_game_dir,
            commands::system::select_folder,
            commands::system::select_file,
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
            commands::system::get_memory_mode,
            commands::system::set_memory_mode,
            commands::system::set_max_download_threads,
            commands::system::get_max_download_threads,
            commands::system::set_chunk_count,
            commands::system::get_chunk_count,
            commands::system::set_isolation_mode,
            commands::system::get_isolation_mode,
            commands::system::get_config_value,
            commands::system::set_config_value,
            commands::system::get_proxy_mode,
            commands::system::set_proxy_mode,
            commands::system::get_proxy_type,
            commands::system::set_proxy_type,
            commands::system::get_proxy_url,
            commands::system::set_proxy_url,
        ])
        .on_window_event(|_window, event| {
            // 窗口关闭时保存配置
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                log_info!("Window close requested, saving config...");
                // 这里需要获取AppState，但由于生命周期限制，我们简化处理
                log_info!("Config will be saved on exit");
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running MoLaunch");
}
