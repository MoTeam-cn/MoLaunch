//! 开屏动画（splashscreen）窗口控制
//!
//! 启动时序：
//! 1. Tauri 先创建 `splashscreen` 窗口（加载 `splash.html` 开屏动画，无边框透明居中）
//! 2. `main` 窗口以 `visible: false` 后台加载（Rust 初始化 + WebView 加载大 bundle 期间
//!    用户看到的是开屏动画而非卡顿白屏）
//! 3. 前端 Vue app 就绪后（或开屏动画播完兜底）调用 `frontend_ready`
//! 4. 本命令关闭开屏窗口、显示并聚焦主窗口
//!
//! 开屏页位于 `public/splash.html`（Vite 构建时复制到 dist，dev 模式经 devUrl 提供），

use crate::log_info;
use tauri::Manager;

/// 前端就绪：关闭开屏窗口并显示主窗口
///
/// 幂等：开屏窗口已关闭 / 主窗口已显示时静默跳过，重复调用无副作用。
/// 调用方：前端 Vue 就绪后主动调用；开屏页动画播完后也做兜底调用。
#[tauri::command]
pub fn frontend_ready(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(splash) = app.get_webview_window("splashscreen") {
        let _ = splash.close();
        log_info!("[Splash] 开屏窗口已关闭");
    }
    if let Some(main_window) = app.get_webview_window("main") {
        let _ = main_window.show();
        let _ = main_window.unminimize();
        let _ = main_window.set_focus();
        log_info!("[Splash] 主窗口已显示");
    }
    Ok(())
}
