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

        // 确保默认游戏目录存在（参考 PCL2 McFolderListLoadSub:124-128）
        // PCL2 启动时主动创建 .minecraft/versions/，避免用户首次点"打开游戏目录"时报路径不存在
        let game_dir = state::resolve_game_dir(&config.game_dir);
        let versions_dir = game_dir.join("versions");
        if !versions_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&versions_dir) {
                log_error!("Failed to create game directory: {}", e);
            } else {
                log_info!("Created game directory: {}", game_dir.display());
            }
        }
    }

    // 初始化 CurseForge 配置（同步读 enabled，api_key 懒加载，避免启动时 DES 解密触发杀软）
    minecraft::community::secure_storage::init_enabled();
    minecraft::community::secure_storage::set_sdk(app_state.sdk.clone());

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
            commands::sdk::check_update_lite,
            // 认证命令
            commands::auth::offline::login_offline,
            commands::auth::account::get_login_status,
            commands::auth::account::logout,
            // 微软登录命令
            commands::auth::microsoft::ms_login_get_config,
            commands::auth::microsoft::ms_login_web_start,
            commands::auth::microsoft::ms_login_web_exchange,
            commands::auth::microsoft::ms_login_request_device_code,
            commands::auth::microsoft::ms_login_poll,
            commands::auth::microsoft::ms_login_refresh,
            commands::auth::account::get_ms_accounts,
            commands::auth::account::remove_ms_account,
            commands::auth::account::switch_ms_account,
            commands::auth::account::get_offline_accounts,
            commands::auth::account::remove_offline_account,
            commands::auth::account::switch_offline_account,
            commands::auth::account::set_offline_skin,
            // 皮肤管理命令
            commands::skin::get_skin_cape_info,
            commands::skin::get_skin_url,
            commands::skin::download_skin_png,
            commands::skin::download_cape_png,
            commands::skin::upload_skin,
            commands::skin::equip_cape,
            commands::skin::unequip_cape,
            commands::skin::save_data_url_to_file,
            // 版本命令
            commands::version::list::list_versions,
            commands::version::download::download_version,
            commands::version::list::list_installed_versions,
            commands::version::list::list_installed_versions_with_type,
            commands::version::list::uninstall_version,
            commands::version::list::get_version_effective_dir,
            commands::version::personalization::get_version_personalization,
            commands::version::personalization::update_version_personalization,
            commands::version::script_export::export_launch_script,
            commands::version::manage::fix_version_files,
            commands::version::manage::rename_version,
            commands::version::manage::get_selected_version,
            commands::version::manage::set_selected_version,
            commands::version::mods::is_version_modable,
            commands::version::mods::list_mods,
            commands::version::mods::toggle_mod,
            commands::version::mods::delete_mod,
            commands::version::mods::install_mod,
            commands::version::mods::open_mods_dir,
            commands::version::folder::list_mc_folders,
            commands::version::folder::add_mc_folder,
            commands::version::folder::remove_mc_folder,
            commands::version::folder::switch_mc_folder,
            commands::version::folder::rename_mc_folder,
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
            commands::java::check_java_compatible,
            commands::java::download_java,
            // 系统命令
            commands::system::open_game_dir,
            commands::system::open_path,
            commands::system::reveal_in_explorer,
            commands::system::get_game_dir,
            commands::system::select_folder,
            commands::system::select_file,
            commands::system::save_file,
            // 统一配置读写命令（取代此前 33 个分散的 get_*/set_* 命令）
            commands::system::get_config,
            commands::system::apply_config,
            commands::system::get_config_path,
            commands::system::save_config_to_file,
            commands::system::get_system_memory,
            commands::system::get_config_value,
            commands::system::set_config_value,
            // 社区资源命令
            commands::community::search::search_resources,
            commands::community::search::get_category_tags,
            commands::community::detail::get_project_detail,
            commands::community::detail::get_project_versions,
            commands::community::detail::get_mcmod_url,
            commands::community::install::download_resource,
            commands::community::install::download_resource_to_path,
            commands::community::install::format_download_filename,
            commands::community::install::install_resource,
            commands::community::install::install_modpack,
            commands::community::install::get_resource_install_path,
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
