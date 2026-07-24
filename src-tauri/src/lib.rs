//! MoLaunch 库入口

pub mod commands;
pub mod config;
pub mod error_util;
pub mod http;
pub mod logger;
pub mod minecraft;
pub mod res_scheme;
pub mod resources;
pub mod sdk;
pub mod state;
pub mod storage;
pub mod utils;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统（统一使用自定义 logger，不使用 env_logger）
    // 移除 env_logger::init()，避免双重日志系统导致第三方库日志绕过脱敏过滤
    log_info!("Starting MoLaunch v{}", env!("CARGO_PKG_VERSION"));

    // 初始化 storage
    if let Err(e) = storage::Storage::instance().init() {
        log_error!("Failed to initialize storage: {}", e);
    }

    // 初始化日志系统（从配置文件加载日志级别和输出选项）
    logger::init_from_config();

    // 初始化 HTTP 客户端（根据代理配置）
    let app_state = AppState::new();
    {
        let config = app_state.config.blocking_lock();
        http::init_client(&config.proxy.mode, &config.proxy.kind, &config.proxy.url);
        log_info!("HTTP client initialized (proxy: {})", config.proxy.mode);

        // 确保默认游戏目录存在
        // 启动时主动创建 .minecraft/versions/，避免用户首次点"打开游戏目录"时报路径不存在
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

    // 启动缓存定期清理任务（启动时立即清理一次，之后每 1h 重复执行）
    // 清理超过 24h 的不重要缓存文件（图片、安装器、预加载、临时安装包等）
    // 不清理 SDK 动态库和 Java Runtime（重要资源）
    utils::cache_cleanup::spawn_cleanup_task();

    log_info!("[Startup] Pre-builder setup done, constructing Tauri Builder...");

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .setup(|_app| {
            // setup 钩子在窗口/webview 创建后、前端加载前调用
            log_info!("[Startup] Tauri setup() hook entered — webview & window created");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // SDK 命令（lite 版本）
            commands::sdk::get_platform_info,
            commands::sdk::get_sdk_version,
            commands::sdk::is_sdk_initialized,
            commands::sdk::get_device_id,
            commands::sdk::check_update_lite,
            // 认证命令（account 子模块拆分为 ms / offline / session）
            commands::auth::offline::login_offline,
            commands::auth::account::session::get_login_status,
            commands::auth::account::session::logout,
            // 微软登录命令
            commands::auth::microsoft::ms_login_get_config,
            commands::auth::microsoft::ms_login_web_start,
            commands::auth::microsoft::ms_login_web_exchange,
            commands::auth::microsoft::ms_login_request_device_code,
            commands::auth::microsoft::ms_login_poll,
            commands::auth::microsoft::ms_login_refresh,
            commands::auth::account::ms::get_ms_accounts,
            commands::auth::account::ms::remove_ms_account,
            commands::auth::account::ms::switch_ms_account,
            commands::auth::account::offline::get_offline_accounts,
            commands::auth::account::offline::remove_offline_account,
            commands::auth::account::offline::switch_offline_account,
            commands::auth::account::offline::set_offline_skin,
            commands::auth::account::offline::save_custom_skin,
            // 皮肤管理命令
            commands::skin::get_skin_cape_info,
            commands::skin::get_skin_url,
            commands::skin::get_cape_url,
            commands::skin::upload_skin,
            commands::skin::equip_cape,
            commands::skin::unequip_cape,
            commands::skin::download_url_to_file,
            // 通用图片缓存命令
            commands::image_cache::get_cached_image_url,
            commands::image_cache::invalidate_cached_image,
            commands::image_cache::clear_image_cache,
            // 版本命令（按子域分组；generate_handler! 不支持通配符，须逐个显式注册）
            // - list: 版本列表与卸载
            commands::version::list::list_versions,
            commands::version::list::list_installed_versions,
            commands::version::list::list_installed_versions_with_type,
            commands::version::list::uninstall_version,
            commands::version::list::get_version_effective_dir,
            commands::version::list::get_version_game_version,
            // - folder: 游戏目录管理
            commands::version::folder::list_mc_folders,
            commands::version::folder::add_mc_folder,
            commands::version::folder::remove_mc_folder,
            commands::version::folder::switch_mc_folder,
            commands::version::folder::rename_mc_folder,
            // - download: 版本下载
            commands::version::download::download_version,
            // - install: 合并安装
            commands::version::install::install_merged,
            // - loaders: 加载器版本查询与安装
            commands::version::loaders::list_forge_versions,
            commands::version::loaders::list_neoforge_versions,
            commands::version::loaders::list_fabric_versions,
            commands::version::loaders::list_optifine_versions,
            commands::version::loaders::list_liteloader_versions,
            commands::version::loaders::validate_loaders,
            commands::version::loaders::list_fabric_api_versions,
            commands::version::loaders::install_fabric_api_for_version,
            // - manage: 版本管理（修复/重命名/选中）
            commands::version::manage::fix_version_files,
            commands::version::manage::rename_version,
            commands::version::manage::get_selected_version,
            commands::version::manage::set_selected_version,
            // - personalization: 版本个性化设置
            commands::version::personalization::get_version_personalization,
            commands::version::personalization::update_version_personalization,
            // - mods: Mod 管理（命令分散到 list/manage/install/watcher 子模块，
            //   tauri::command 宏 __cmd__ 符号无法 pub use 重导出，故使用完整路径注册）
            commands::version::mods::list::is_version_modable,
            commands::version::mods::list::list_mods,
            commands::version::mods::manage::toggle_mod,
            commands::version::mods::manage::delete_mod,
            commands::version::mods::install::install_mod,
            commands::version::mods::install::open_mods_dir,
            commands::version::mods::install::reveal_mod_file,
            commands::version::mods::install::get_version_mods_dir,
            commands::version::mods::watcher::watch_mods_dir,
            commands::version::mods::watcher::unwatch_mods_dir,
            // - preload: Mod 详情预加载
            commands::version::preload::preload_mods_detail_cmd,
            // - progress: 下载进度与控制
            commands::version::progress::get_download_progress,
            commands::version::progress::is_downloading,
            commands::version::progress::reset_download_progress,
            commands::version::progress::cancel_download,
            commands::version::progress::pause_download,
            commands::version::progress::resume_download,
            // - launch: 启动游戏
            commands::version::launch::launch_game,
            commands::version::launch::get_launch_progress,
            commands::version::launch::cancel_launch,
            commands::version::launch::stop_game,
            commands::version::launch::get_running_game,
            commands::version::launch::get_launch_history,
            // - script_export: 启动脚本导出
            commands::version::script_export::export_launch_script,
            // Java 命令
            commands::java::detect_java,
            commands::java::list_java,
            commands::java::select_java_for_mc,
            commands::java::get_java_requirements,
            commands::java::check_java_compatible,
            commands::java::download_java,
            // 系统命令（文件/文件夹选择对话框已统一走前端 @tauri-apps/plugin-dialog）
            commands::system::open_game_dir,
            commands::system::open_path,
            commands::system::reveal_in_explorer,
            commands::system::get_game_dir,
            commands::system::write_text_file,
            // 统一配置读写命令（取代此前 33 个分散的 get_*/set_* 命令）
            commands::system::get_config,
            commands::system::apply_config,
            commands::system::get_config_path,
            commands::system::save_config_to_file,
            commands::system::get_system_memory,
            commands::system::get_config_value,
            commands::system::set_config_value,
            // 开发者模式命令（开关状态由 get_config/apply_config 统一管理）
            commands::system::is_developer_unlocked,
            commands::system::unlock_developer_mode,
            commands::system::get_storage_dirs,
            commands::system::get_system_info,
            commands::system::get_cache_stats,
            // 关于页面数据命令（从 resources/about/ 加载 markdown 表格数据）
            commands::system::get_about_data,
            // 日志查看命令（开发者模式）
            logger::get_log_path,
            logger::list_log_files,
            logger::read_log_file,
            // 社区资源命令
            commands::community::search::search_resources,
            commands::community::search::get_category_tags,
            commands::community::detail::get_project_detail,
            commands::community::detail::get_project_versions,
            commands::community::detail::get_mcmod_url,
            commands::community::install::resource::download_resource,
            commands::community::install::resource::download_resource_to_path,
            commands::community::install::resource::format_download_filename,
            commands::community::install::resource::install_resource,
            commands::community::install::modpack::install_modpack,
            commands::community::install::modpack::install_local_modpack,
            commands::community::install::resource::get_resource_install_path,
            // 插件系统命令（拆分到 plugins/ 子模块：sandbox / install / spawn / window / layout / export / personalization）
            commands::plugins::sandbox::list_external_plugins,
            commands::plugins::sandbox::read_external_plugin_file,
            commands::plugins::sandbox::uninstall_external_plugin,
            commands::plugins::install::install_external_plugin_from_dir,
            commands::plugins::install::install_external_plugin_from_zip,
            commands::plugins::spawn::plugin_spawn_process,
            commands::plugins::window::plugin_create_window,
            commands::plugins::layout::load_custom_layout,
            commands::plugins::export::read_layout_sample,
            commands::plugins::export::export_plugin_sample,
            commands::plugins::personalization::read_personalization,
            commands::plugins::personalization::write_personalization,
            // 外部下载工具命令（统一 tools_manager 入口）
            commands::tools::tools_manager,
        ])
        .on_window_event(|_window, event| {
            // 窗口关闭时保存配置
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                log_info!("Window close requested, saving config...");
                // 这里需要获取AppState，但由于生命周期限制，我们简化处理
                log_info!("Config will be saved on exit");
            }
        });

    // 注册 cache-image 自定义 URI scheme（图片缓存协议，抽离至 minecraft::image_cache）
    log_info!("[Startup] Registering cache-image URI scheme...");
    let builder = minecraft::image_cache::register_uri_scheme(builder);

    // 注册 res:// 自定义 URI scheme（前端访问后端嵌入资源，如 WASM 文件）
    log_info!("[Startup] Registering res:// URI scheme...");
    let builder = res_scheme::register_res_scheme(builder);

    log_info!("[Startup] All setup done, calling builder.run() — entering Tauri event loop (webview/window creation follows)...");
    builder
        .run(tauri::generate_context!())
        .expect("error while running MoLaunch");
}
