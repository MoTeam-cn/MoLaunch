//! 开发者模式相关命令
//! 解锁流程：法律信息中点「MoTeam」7 次（3 秒内）→ `unlock_developer_mode` → 显示「开发者
//! 模式」开关卡片 → `apply_config({developerMode})` → 侧边菜单出现「开发者」项 →
//! `open_devtools` 调 WebView2 DevTools；撤销走 `lock_developer_mode`。存储：Windows 注册表
//! `HKCU\Software\MoLaunch` 下 DeveloperUnlocked/DeveloperMode；DevTools 状态用 `AtomicBool` 维护。

use crate::log_info;
use crate::minecraft::system::{get_os_type, get_system_arch, get_system_memory};
use crate::storage::registry::{reg_get_bool, reg_set_bool};
use crate::storage::Storage;
use crate::utils::cache;
use crate::utils::cache_app;
use crate::utils::cache_stats;
use crate::utils::cache_temp;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};

/// 注册表键名：开发者模式是否已解锁
pub const KEY_DEV_UNLOCKED: &str = "DeveloperUnlocked";

/// 注册表键名：开发者模式是否开启
pub const KEY_DEV_MODE: &str = "DeveloperMode";

/// 注册表键名：是否忽略 TLS 证书校验（仅开发者模式可开启）
///
/// 开启后 `http::build_client` 会调用 `danger_accept_invalid_certs(true)`，
/// 跳过所有证书校验，用于联机服务端自签名证书调试等场景。
pub const KEY_IGNORE_TLS: &str = "IgnoreTls";

/// DevTools 打开状态（全局原子标志）
///
/// WebView2 不提供查询 DevTools 是否打开的 API，Tauri 的
/// `WebviewWindow::is_devtools_open()` 在 Windows 上始终返回 false，
/// 因此由后端自行维护：
/// - `open_devtools()` 成功后置 true
/// - `close_devtools()` 成功后置 false
/// - 主窗口销毁时通过 `reset_devtools_state()` 重置为 false
static DEVTOOLS_OPEN: AtomicBool = AtomicBool::new(false);

/// 查询开发者模式是否已解锁（用户在鸣谢法律信息中触发隐藏字段后解锁）
///
/// 未解锁时返回 false，「高阶配置」页不显示开发者模式开关卡片。
/// 开关的开启状态由 `get_config` / `apply_config` 统一管理（developerMode 字段）。
pub fn is_developer_unlocked() -> bool {
    reg_get_bool(KEY_DEV_UNLOCKED)
}

/// 解锁开发者模式（写入注册表 `DeveloperUnlocked=true`）
///
/// 调用后「高阶配置」页将显示开发者模式开关卡片。
/// 已解锁时重复调用是幂等的。
pub fn unlock_developer_mode() -> Result<(), String> {
    log_info!("[Developer] 开发者模式已解锁");
    reg_set_bool(KEY_DEV_UNLOCKED, true)
}

/// 撤销开发者模式解锁（写注册表 `DeveloperUnlocked=false`）
///
/// 同时重置 `DeveloperMode` 和 `IgnoreTls`，确保开发者能力全部失效；DevTools 已开则尝试关闭。
/// 撤销后：高阶配置开关卡片隐藏、侧边「开发者」项隐藏（emit `developer-mode-changed`）、
/// DevTools 无法调出（`require_dev_mode()` 失败）、IgnoreTls 失效。已撤销时调用幂等。
pub fn lock_developer_mode(app: &AppHandle) -> Result<(), String> {
    // 若 DevTools 已打开，先关闭（不强制要求关闭成功，避免 WebView2 异常阻断撤销）
    if DEVTOOLS_OPEN.load(Ordering::SeqCst) {
        if let Some(window) = app.get_webview_window("main") {
            window.close_devtools();
        }
        DEVTOOLS_OPEN.store(false, Ordering::SeqCst);
    }
    // 顺序：先关 DeveloperMode（触发 require_dev_mode 校验失败），再关 IgnoreTls，
    //       最后清 DeveloperUnlocked。任一步失败均向上抛错，保证状态一致。
    reg_set_bool(KEY_DEV_MODE, false)?;
    reg_set_bool(KEY_IGNORE_TLS, false)?;
    reg_set_bool(KEY_DEV_UNLOCKED, false)?;
    log_info!("[Developer] 开发者模式已撤销解锁");
    Ok(())
}

/// 查询是否忽略 TLS 证书校验（仅在开发者模式已开启时返回 true）
///
/// 与 `is_developer_unlocked` 不同，本函数同时校验 `DeveloperUnlocked`
/// 与 `DeveloperMode` 两个开关，确保仅开发者模式实际开启时才允许忽略 TLS。
/// 任何一层关闭均返回 false，避免开发者模式被关闭后 IgnoreTls 仍生效。
pub fn is_ignore_tls() -> bool {
    let unlocked = reg_get_bool(KEY_DEV_UNLOCKED);
    let mode = reg_get_bool(KEY_DEV_MODE);
    unlocked && mode && reg_get_bool(KEY_IGNORE_TLS)
}

/// 存储目录信息
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageDirs {
    /// 数据根目录（.Molaunch）
    pub base: String,
    /// 配置文件（config.ini 完整路径）
    pub config: String,
    /// 日志目录
    pub logs: String,
    /// 运行路径缓存目录（.Molaunch/cache/）
    pub cache: String,
    /// 临时目录（.Molaunch/temp/）
    pub temp: String,
    /// 系统临时目录缓存（<temp>/MoLaunch/，含 TaskTemp 和 sdk）
    pub cache_temp: String,
    /// AppData 缓存目录（%APPDATA%/.minecraft/，Java Runtime）
    pub cache_app: String,
}

/// 获取所有存储目录路径（开发者页展示用）
pub fn get_storage_dirs() -> StorageDirs {
    let storage = Storage::instance();
    StorageDirs {
        base: storage.base_dir().to_string_lossy().to_string(),
        config: storage.config_path().to_string_lossy().to_string(),
        logs: storage.logs_dir().to_string_lossy().to_string(),
        cache: cache::dir().to_string_lossy().to_string(),
        temp: storage.temp_dir().to_string_lossy().to_string(),
        cache_temp: cache_temp::dir().to_string_lossy().to_string(),
        cache_app: cache_app::dir().to_string_lossy().to_string(),
    }
}

/// 系统信息（开发者页展示用）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    /// 应用版本（Cargo.toml version）
    pub app_version: String,
    /// 操作系统（windows/macos/linux）
    pub os: String,
    /// 系统架构（x86_64/aarch64）
    pub arch: String,
    /// 是否 64 位
    pub is_64bit: bool,
    /// 总内存（字节）
    pub total_memory: u64,
    /// 已用内存（字节）
    pub used_memory: u64,
    /// 可用内存（字节）
    pub available_memory: u64,
    /// 内存使用率（百分比）
    pub memory_usage_percent: f64,
}

/// 获取系统信息（开发者页展示用）
pub fn get_system_info() -> SystemInfo {
    let mem = get_system_memory();
    SystemInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        os: get_os_type(),
        arch: get_system_arch(),
        is_64bit: std::mem::size_of::<usize>() == 8,
        total_memory: mem.total,
        used_memory: mem.used,
        available_memory: mem.available,
        memory_usage_percent: mem.usage_percent,
    }
}

/// 获取所有缓存目录的统计信息（文件数、占用大小、TTL）
///
/// 在 `spawn_blocking` 中执行以避免阻塞主线程。
pub async fn get_cache_stats() -> Result<cache_stats::CacheStatsResult, String> {
    tauri::async_runtime::spawn_blocking(cache_stats::collect_all)
        .await
        .map_err(|e| format!("Failed to collect cache stats: {}", e))
}

// ==================== DevTools 控制 ====================
//
// 安全约束：所有 devtools 控制函数均要求 DeveloperUnlocked=true && DeveloperMode=true
// 双层校验，确保普通用户即使绕过前端按钮直接调 IPC 也无法打开 devtools。
// 任何一层关闭均拒绝调用，避免开发者模式被关闭后 devtools 仍可调出。

/// 校验当前用户是否有权限使用 devtools（开发者模式已解锁且已开启）
fn require_dev_mode() -> Result<(), String> {
    if !is_developer_unlocked() {
        return Err("开发者模式未解锁".to_string());
    }
    if !reg_get_bool(KEY_DEV_MODE) {
        return Err("开发者模式未开启".to_string());
    }
    Ok(())
}

/// 打开主窗口的 WebView2 DevTools
///
/// 在开发者模式已开启时调用 `WebviewWindow::open_devtools()` 调出开发者工具。
/// 重复调用是幂等的（DevTools 已打开时不会重复打开）。
///
/// 状态维护：成功调用后置 `DEVTOOLS_OPEN=true`，供 `is_devtools_open` 查询。
/// WebView2 本身无查询 API，必须由后端自行维护状态。
pub fn open_devtools(app: &AppHandle) -> Result<(), String> {
    require_dev_mode()?;
    if let Some(window) = app.get_webview_window("main") {
        window.open_devtools();
        DEVTOOLS_OPEN.store(true, Ordering::SeqCst);
        log_info!("[Developer] DevTools opened");
        Ok(())
    } else {
        Err("主窗口未找到".to_string())
    }
}

/// 关闭主窗口的 WebView2 DevTools
///
/// 状态维护：成功调用后置 `DEVTOOLS_OPEN=false`。
pub fn close_devtools(app: &AppHandle) -> Result<(), String> {
    require_dev_mode()?;
    if let Some(window) = app.get_webview_window("main") {
        window.close_devtools();
        DEVTOOLS_OPEN.store(false, Ordering::SeqCst);
        log_info!("[Developer] DevTools closed");
        Ok(())
    } else {
        Err("主窗口未找到".to_string())
    }
}

/// 查询主窗口的 DevTools 是否已打开
///
/// 返回 false 的情况：
/// - DevTools 实际未打开（`DEVTOOLS_OPEN=false`）
/// - 开发者模式未开启（拒绝查询，避免绕过校验探测状态）
/// - 主窗口未找到
///
/// 注意：不使用 Tauri 的 `WebviewWindow::is_devtools_open()`，因为该方法
/// 在 Windows WebView2 上始终返回 false（WebView2 不提供查询 API）。
pub fn is_devtools_open(app: &AppHandle) -> Result<bool, String> {
    require_dev_mode()?;
    if app.get_webview_window("main").is_some() {
        Ok(DEVTOOLS_OPEN.load(Ordering::SeqCst))
    } else {
        Ok(false)
    }
}

/// 重置 DevTools 打开状态为 false
///
/// 在主窗口销毁/关闭时调用，防止下次启动前状态泄露（实际上进程退出后
/// `static` 也会重置，但显式调用更稳妥，便于将来支持窗口重建场景）。
pub fn reset_devtools_state() {
    DEVTOOLS_OPEN.store(false, Ordering::SeqCst);
}
