//! 系统托盘：托盘图标 + 右键菜单（打开主页面 / 检查更新 / 退出）+ 退出统一清理
//!
//! 关闭缺口修复的落点：
//! - `CloseRequested` 被 `on_window_event` 拦截后按 `close_behavior` 分流（hide / exit / 通知前端弹框）
//! - `cleanup_and_exit` 在真正退出时统一清理 frpc 隧道与 TUN 虚拟网卡，
//!   避免 Alt+F4 / 托盘退出绕过前端 handleClose 导致的进程残留

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
/// 2. 停止 TUN 虚拟网卡桥接器（此前依赖 Drop 兜底）
/// 3. 保存配置到文件
pub fn cleanup_and_exit(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        crate::commands::frp::process::stop_all_tunnels().await;

        if let Some(state) = app.try_state::<AppState>() {
            let bridge = {
                let mut guard = state.virtual_lan_bridge.lock().await;
                guard.take()
            };
            if let Some(bridge) = bridge {
                log_info!("[Exit] 停止 TUN 虚拟网卡");
                bridge.stop().await;
            }

            let config = state.config.lock().await;
            if let Err(e) = crate::config::save_config(&config) {
                log_warn!("[Exit] 退出时保存配置失败: {}", e);
            }
        }

        log_info!("[Exit] 清理完成，退出进程");
        app.exit(0);
    });
}
