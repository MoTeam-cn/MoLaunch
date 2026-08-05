//! MoLaunch 库入口

pub mod api_paths;
pub mod ai_core;
pub mod certs;
pub mod commands;
pub mod config;
pub mod deeplink;
pub mod error_util;
pub mod http;
pub mod logger;
pub mod migrations;
pub mod minecraft;
pub mod res_scheme;
pub mod resources;
pub mod sdk;
pub mod state;
pub mod storage;
pub mod tray;
pub mod utils;
pub mod ws;

use state::AppState;
use tauri::Emitter;
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

    // 注入 Frp 认证存储 SDK 引用（token 用 SDK DES 加密后写文件，替代原 keyring）
    commands::frp::auth::set_sdk(app_state.sdk.clone());

    // 注入 AI 配置存储 SDK 引用（api_key 用 SDK DES 加密后写 config.ini [AI] 段）
    ai_core::storage::set_sdk(app_state.sdk.clone());

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
        // 单实例插件（带 deep-link feature）：保证只有一个主进程实例；
        // Windows/Linux 上第二次点击 molaunch:// 链接时，新进程把 URL 作为
        // CLI 参数交给本回调，再由 deep-link 插件转发为 deep-link://new-url 事件。
        // 注意：必须注册在 deep-link 插件之前。
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            // 新实例启动时，主实例在这里收到 argv（含 deeplink URL）。
            // deeplink 插件已接管 deep-link://new-url 事件分发，此处仅需聚焦窗口。
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
            log_info!("[SingleInstance] 新实例 argv: {:?}", argv);
        }))
        // 深度链接插件（molaunch:// 协议注册与事件解析）
        .plugin(tauri_plugin_deep_link::init())
        // 重启主进程 plugin（更新文件替换完成后调用 relaunch）
        .plugin(tauri_plugin_process::init())
        .manage(app_state);

    // 自动更新官方 plugin：仅 macOS/Linux 使用（Windows 便携版走自实现 updater，
    // 见 commands/system/updater/install_windows.rs，官方 plugin 不链接）
    #[cfg(not(target_os = "windows"))]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    let builder = builder
        .setup(|app| {
            // setup 钩子在窗口/webview 创建后、前端加载前调用
            log_info!("[Startup] Tauri setup() hook entered — webview & window created");

            // 初始化深度链接模块（molaunch:// 协议监听 + 内置 handler 注册）
            // 返回 EventId（Copy 值），监听由插件内部持有，无需托管
            match deeplink::init(app.handle()) {
                Ok(_event_id) => {}
                Err(e) => {
                    log_error!("[Deeplink] 初始化失败: {}", e);
                }
            }

            // 启动 WebSocket 服务器（下载进度推送，替代前端轮询）
            // 监听 127.0.0.1:0 随机端口，端口写入 AppState.ws_port 供前端查询
            let app_handle = app.handle().clone();
            let state = app.state::<AppState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                ws::start_server(app_handle, state).await;
            });

            // 创建系统托盘（右键菜单：打开主页面 / 检查更新 / 退出）
            if let Err(e) = tray::setup_tray(app.handle()) {
                log_error!("[Tray] 托盘创建失败: {}", e);
            }

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
            // AI 分析命令（本地 OpenAI 兼容 API，action：analyze_crash/check_status/save_config/load_config）
            commands::ai::ai_manager,
            // 联机功能命令（6 个 action，阶段一认证相关，已聚合为 online_manager 单一入口）
            commands::online::online_manager,
            // Frp 内网穿透命令（8 个 action，厂商/隧道/进程管理，已聚合为 frp_manager 单一入口）
            commands::frp::frp_manager,
            // 托盘退出命令（前端完成联机退房等清理后调用，后端再统一清理 frpc/TUN 后退出）
            tray::request_exit,
        ])
        .on_window_event(|window, event| {
            // 仅拦截主窗口的关闭请求；picker:// 等子窗口关闭时直接放行（正常销毁）
            if window.label() != "main" {
                return;
            }
            // 拦截关闭请求，按 close_behavior 分流：
            // - tray：隐藏窗口（保留托盘运行）
            // - exit：直接执行退出清理 + 退出进程
            // - ask：通知前端弹出"直接退出 / 保留托盘"选择框
            // 该钩子覆盖 Alt+F4 / 任务栏关闭等绕过前端 handleClose 的路径，
            // 补齐此前关闭流程的清理缺口（frpc 残留 / TUN 未停止 / 跳过配置保存）。
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let app = window.app_handle();
                let behavior = {
                    let state = app.state::<AppState>();
                    let config = state.config.blocking_lock();
                    config.close_behavior.clone()
                };
                match behavior.as_str() {
                    "tray" => {
                        // 保留托盘：隐藏主界面，进程 / 全局 keepalive 定时器继续运行
                        let _ = window.hide();
                        log_info!("[Window] 关闭请求：关闭到托盘");
                    }
                    "exit" => {
                        log_info!("[Window] 关闭请求：直接退出");
                        tray::cleanup_and_exit(app);
                    }
                    _ => {
                        // 每次询问：通知前端弹出选择框
                        log_info!("[Window] 关闭请求：通知前端弹出退出选择框");
                        let _ = app.emit("window-close-requested", ());
                    }
                }
                return;
            }
            // 主窗口销毁时重置 DevTools 打开状态
            // WebView2 不提供查询 API，后端用 AtomicBool 维护状态；
            // 窗口销毁后状态应重置，避免下次启动前状态泄露
            if let tauri::WindowEvent::Destroyed = event {
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

    // 注册 picker:// 自定义 URI scheme（选择器子窗口 HTML 渲染）
    log_info!("[Startup] Registering picker URI scheme...");
    let builder = commands::tools::picker_window::register_picker_scheme(builder);

    log_info!("[Startup] All setup done, calling builder.run() — entering Tauri event loop (webview/window creation follows)...");
    builder
        .run(tauri::generate_context!())
        .expect("error while running MoLaunch");
}
