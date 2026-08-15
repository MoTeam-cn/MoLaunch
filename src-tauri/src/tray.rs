//! 系统托盘：提供主界面打开、更新检查、退出菜单及统一退出清理。
//! 窗口关闭与托盘退出最终复用后端清理流程。

use crate::log_info;
use crate::log_warn;
use crate::state::AppState;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

/// 创建系统托盘（在 setup 中调用）
pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "打开主页面", true, None::<&str>)?;
    let check = MenuItem::with_id(app, "check-update", "检查更新", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &check, &quit])?;

    let icon = app.default_window_icon().cloned().unwrap_or_else(|| {
        tauri::image::Image::from_bytes(include_bytes!("../Images/icon.ico"))
            .expect("failed to load tray icon")
    });

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("MoLaunch")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => open_main_window(app),
            "check-update" => {
                log_info!("[Tray] 检查更新");
                // 通知前端复用现有检查更新流程（弹窗/提示由前端掌控）
                let _ = app.emit("tray-check-update", ());
            }
            "quit" => {
                log_info!("[Tray] 托盘退出，执行退出清理");
                // 托盘退出即完全退出，不弹确认框：直接走后端统一清理
                // （frpc 隧道 / TUN 虚拟网卡 / 保存配置）后退出进程。
                // 注意：此处不转交前端 doExit 流程（跳过联机退房/待安装更新），
                // 托盘退出语义为"无条件退出"，与 X 按钮的"优雅退出"区分。
                cleanup_and_exit(app);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // 左键单击托盘图标：打开主界面（右键弹菜单由 show_menu_on_left_click(false) 保证）
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                open_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    log_info!("[Tray] 系统托盘创建成功");
    Ok(())
}

/// 打开主界面：显示窗口并聚焦（窗口可能被最小化或隐藏到托盘）
fn open_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        log_info!("[Tray] 打开主界面");
    }
}

/// 前端直接退出时调用的命令（前端已完成配置保存/联机退房/待安装更新等清理）
#[tauri::command]
pub fn request_exit(app: AppHandle) {
    log_info!("[Exit] 前端已确认直接退出，执行后端清理");
    cleanup_and_exit(&app);
}

/// 退出统一清理 + 退出进程
///
/// 补齐此前退出路径的清理缺口：
/// 1. 遍历 frpc 全局进程表停止所有隧道（避免残留 frpc.exe）
/// 2. 停止 easytier 虚拟网络与联机中心服务（此前依赖 Drop 兜底）
/// 3. 保存配置到文件
pub fn cleanup_and_exit(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        crate::commands::frp::process::stop_all_tunnels().await;

        if let Some(state) = app.try_state::<AppState>() {
            // 停止 easytier 虚拟网络
            let easytier = {
                let mut guard = state.easytier.lock().await;
                guard.take()
            };
            if let Some(easytier) = easytier {
                log_info!("[Exit] 停止 easytier 虚拟网络");
                easytier.stop().await;
            }
            // 停止联机中心 TCP 服务
            let server = {
                let mut guard = state.scaffolding_server.lock().await;
                guard.take()
            };
            if let Some(server) = server {
                log_info!("[Exit] 停止联机中心服务");
                server.stop().await;
            }

            let config = state.config.lock().await;
            if let Err(e) = crate::config::save_config(&config) {
                log_warn!("[Exit] 退出时保存配置失败: {}", e);
            }
        }

        log_info!("[Exit] 清理完成，销毁 WebView2 窗口后退出进程");

        // 先销毁所有窗口（含 WebView2），并等待其内部异步清理完成，再退出进程。
        // 直接 app.exit(0) 会在 WebView2 的 Chromium 窗口（Chrome_WidgetWin_0 类）
        // 尚未完成 teardown 时强制退出，触发
        // "Failed to unregister class Chrome_WidgetWin_0. Error = 1412" 竞态报错。
        // destroy() 走强制销毁路径，不触发 on_window_event 的 CloseRequested 拦截。
        for (_, window) in app.webview_windows() {
            let _ = window.destroy();
        }
        // 给 WebView2 一段内部清理时间，避免 Chromium 注销窗口类时与进程退出竞争
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        app.exit(0);
    });
}
