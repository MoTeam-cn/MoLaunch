//! MoLaunch 库入口

pub mod certs;
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
pub mod ws;

use state::AppState;
use tauri::Manager;

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

    // 初始化 HTTP 客户端（根据代理 + TLS 配置）
    // ignore_tls 走注册表（DeveloperUnlocked + DeveloperMode + IgnoreTls 三层判定），
    // 仅开发者模式实际开启时才返回 true
    let app_state = AppState::new();
    {
        let config = app_state.config.blocking_lock();
        let ignore_tls = commands::system::developer::is_ignore_tls();
        http::init_client(
            &config.proxy.mode,
            &config.proxy.kind,
            &config.proxy.url,
            &config.proxy.ip_version,
            &config.tls.trust_mode,
            ignore_tls,
        );
        log_info!(
            "HTTP client initialized (proxy: {}, ip_version: {}, trust_mode: {}, ignore_tls: {})",
            config.proxy.mode,
            config.proxy.ip_version,
            config.tls.trust_mode,
            ignore_tls
        );

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
        // 自动更新 plugin（检测/下载/校验/启动 NSIS installer 子程序）
        // See: docs/updater/design.md §4.1.4
        .plugin(tauri_plugin_updater::Builder::new().build())
        // 重启主进程 plugin（更新文件替换完成后调用 relaunch）
        .plugin(tauri_plugin_process::init())
        .manage(app_state)
        .setup(|app| {
            // setup 钩子在窗口/webview 创建后、前端加载前调用
            log_info!("[Startup] Tauri setup() hook entered — webview & window created");

            // 启动 WebSocket 服务器（下载进度推送，替代前端轮询）
            // 监听 127.0.0.1:0 随机端口，端口写入 AppState.ws_port 供前端查询
            let app_handle = app.handle().clone();
            let state = app.state::<AppState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                ws::start_server(app_handle, state).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // SDK 命令（5 个 action，已聚合为 sdk_manager 单一入口）
            commands::sdk::sdk_manager,
            // 认证命令（28 个 action，已聚合为 meta_manager 单一入口）
            commands::auth::meta_manager,
            // 皮肤管理命令（7 个 action，已聚合为 skin_manager 单一入口）
            commands::skin::skin_manager,
            // 通用图片缓存命令（3 个 action，已聚合为 image_cache_manager 单一入口）
            commands::image_cache::image_cache_manager,
            // 版本命令（按子域拆分为 5 个 manager）
            // - version_list_manager：list + folder + manage + personalization（17 个 action）
            commands::version::version_list_manager,
            // - version_install_manager：download + install + loaders + preload（11 个 action）
            commands::version::version_install_manager,
            // - version_mods_manager：mods 下 list/manage/install/update/watcher（11 个 action）
            commands::version::mods::version_mods_manager,
            // - version_progress_manager：下载进度与控制（6 个 action）
            commands::version::progress::version_progress_manager,
            // - version_launch_manager：launch + script_export（7 个 action）
            commands::version::launch::version_launch_manager,
            // - version_export_manager：导出整合包（4 个 action）
            commands::version::version_export_manager,
            // Java 命令（6 个 action，已聚合为 java_manager 单一入口）
            commands::java::java_manager,
            // 系统命令（17 个 action，含 game_dir/config/developer/about/logger，已聚合为 system_manager 单一入口）
            commands::system::system_manager,
            // 配置命令（4 个 action，已聚合为 config_manager 单一入口）
            commands::system::config_manager,
            // 社区资源命令（13 个 action，已聚合为 community_manager 单一入口）
            commands::community::community_manager,
            // 插件系统命令（12 个 action，已聚合为 plugins_manager 单一入口）
            commands::plugins::plugins_manager,
            // 外部下载工具命令（25 个 action，已聚合为 tools_manager 单一入口）
            commands::tools::tools_manager,
            // 联机功能命令（6 个 action，阶段一认证相关，已聚合为 online_manager 单一入口）
            commands::online::online_manager,
        ])
        .on_window_event(|_window, event| {
            // 窗口关闭时保存配置
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                log_info!("Window close requested, saving config...");
                // 这里需要获取AppState，但由于生命周期限制，我们简化处理
                log_info!("Config will be saved on exit");
            }
            // 主窗口销毁时重置 DevTools 打开状态
            // WebView2 不提供查询 API，后端用 AtomicBool 维护状态；
            // 窗口销毁后状态应重置，避免下次启动前状态泄露
            if let tauri::WindowEvent::Destroyed { .. } = event {
                commands::system::developer::reset_devtools_state();
                log_info!("[Developer] Window destroyed, devtools state reset");
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
