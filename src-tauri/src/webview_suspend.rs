//! WebView2 挂起/恢复：关闭到托盘时释放渲染资源（保留界面状态，恢复秒回）。
//!
//! 仅 Windows（WebView2）；必须在 UI 线程调用（窗口事件 / 托盘事件回调）。
//! 挂起后 WebView2 渲染进程资源释放、内存换出（进程保留），DOM/JS 状态完整保留，
//! 托盘恢复时 Resume() 同步恢复，无需重建窗口或重载前端。

use crate::log_debug;
use crate::log_warn;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::WebviewWindow;
use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2Controller, ICoreWebView2_4};
use windows_0_61::core::Interface;

/// 主窗口当前是否处于 WebView2 挂起状态
static SUSPENDED: AtomicBool = AtomicBool::new(false);

/// 挂起主窗口 WebView2（关闭到托盘时调用；TrySuspend 为异步，handler 传 None 不等待回调）
pub fn suspend(window: &WebviewWindow) {
    if SUSPENDED.load(Ordering::Relaxed) {
        return;
    }
    let result = std::sync::Arc::new(std::sync::Mutex::new(None::<Result<(), String>>));
    let slot = result.clone();
    if let Err(e) = window.with_webview(move |platform_webview| {
        // Windows 平台：PlatformWebview = wry::WebView，controller() 返回 ICoreWebView2Controller
        let out = try_suspend(&platform_webview.controller());
        *slot.lock().unwrap() = Some(out);
    }) {
        log_warn!("[WebView2] with_webview 回调失败，跳过挂起: {}", e);
        return;
    }
    let outcome = result.lock().unwrap().take();
    match outcome {
        Some(Ok(())) => {
            SUSPENDED.store(true, Ordering::Relaxed);
            log_debug!("[WebView2] 已挂起，释放渲染资源");
        }
        Some(Err(e)) => log_warn!("[WebView2] 挂起失败: {}", e),
        None => log_warn!("[WebView2] 挂起回调未执行"),
    }
}

/// 恢复主窗口 WebView2（托盘 / 单实例恢复窗口时调用；Resume 为同步操作）
pub fn resume(window: &WebviewWindow) {
    if !SUSPENDED.load(Ordering::Relaxed) {
        return;
    }
    let result = std::sync::Arc::new(std::sync::Mutex::new(None::<Result<(), String>>));
    let slot = result.clone();
    if let Err(e) = window.with_webview(move |platform_webview| {
        let out = try_resume(&platform_webview.controller());
        *slot.lock().unwrap() = Some(out);
    }) {
        log_warn!("[WebView2] with_webview 回调失败，跳过恢复: {}", e);
        return;
    }
    let outcome = result.lock().unwrap().take();
    match outcome {
        Some(Ok(())) => {
            SUSPENDED.store(false, Ordering::Relaxed);
            log_debug!("[WebView2] 已恢复");
        }
        Some(Err(e)) => log_warn!("[WebView2] 恢复失败: {}", e),
        None => log_warn!("[WebView2] 恢复回调未执行"),
    }
}

/// cast 到 ICoreWebView2_4 并执行 TrySuspend
fn try_suspend(controller: &ICoreWebView2Controller) -> Result<(), String> {
    let core = controller
        .cast::<ICoreWebView2_4>()
        .map_err(|e| format!("cast 到 ICoreWebView2_4 失败: {}", e))?;
    // SAFETY: TrySuspend 为 COM 调用，handler 传 None；UI 线程调用满足 COM 线程要求
    unsafe { core.TrySuspend(None) }.map_err(|e| format!("TrySuspend 失败: {}", e))
}

/// cast 到 ICoreWebView2_4 并执行 Resume
fn try_resume(controller: &ICoreWebView2Controller) -> Result<(), String> {
    let core = controller
        .cast::<ICoreWebView2_4>()
        .map_err(|e| format!("cast 到 ICoreWebView2_4 失败: {}", e))?;
    // SAFETY: Resume 为 COM 调用；UI 线程调用满足 COM 线程要求
    unsafe { core.Resume() }.map_err(|e| format!("Resume 失败: {}", e))
}
