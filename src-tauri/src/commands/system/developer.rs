//! 开发者模式相关命令
//!
//! 触发流程：
//! 1. 用户在「其他」页连续点击应用版本号 5 次 → 调用 `unlock_developer_mode`
//! 2. 解锁后「高阶配置」顶部显示「开发者模式」开关卡片 → 调用 `set_developer_mode`
//! 3. 开关开启后「设置」侧边菜单出现「开发者」项 → 进入 SettingsDeveloper.vue
//!
//! 存储位置：Windows 注册表 `HKCU\Software\MoLaunch` 下的两个布尔值
//! - `DeveloperUnlocked`：是否已解锁（决定开关卡片是否显示）
//! - `DeveloperMode`：开关是否开启（决定侧边菜单 developer 项是否显示）

use crate::log_info;
use crate::minecraft::system::{get_os_type, get_system_arch, get_system_memory};
use crate::storage::registry::{reg_get_bool, reg_set_bool};
use crate::storage::Storage;
use serde::Serialize;

/// 注册表键名：开发者模式是否已解锁
const KEY_DEV_UNLOCKED: &str = "DeveloperUnlocked";

/// 注册表键名：开发者模式是否开启
const KEY_DEV_MODE: &str = "DeveloperMode";

// ============================================================
// 解锁与开关
// ============================================================

/// 查询开发者模式是否已解锁（用户连续点击版本号 5 次后解锁）
///
/// 未解锁时返回 false，「高阶配置」页不显示开发者模式开关卡片。
#[tauri::command]
pub fn is_developer_unlocked() -> bool {
    reg_get_bool(KEY_DEV_UNLOCKED)
}

/// 解锁开发者模式（写入注册表 `DeveloperUnlocked=true`）
///
/// 调用后「高阶配置」页将显示开发者模式开关卡片。
/// 已解锁时重复调用是幂等的。
#[tauri::command]
pub fn unlock_developer_mode() -> Result<(), String> {
    log_info!("[Developer] 开发者模式已解锁");
    reg_set_bool(KEY_DEV_UNLOCKED, true)
}

/// 查询开发者模式是否开启
///
/// 开启后「设置」侧边菜单将出现「开发者」项。
/// 未解锁时也返回 false（开关未显示自然不会开启）。
#[tauri::command]
pub fn is_developer_mode() -> bool {
    is_developer_unlocked() && reg_get_bool(KEY_DEV_MODE)
}

/// 设置开发者模式开关
///
/// 仅在已解锁时可生效。未解锁时调用返回错误。
#[tauri::command]
pub fn set_developer_mode(enabled: bool) -> Result<(), String> {
    if !is_developer_unlocked() {
        return Err("开发者模式尚未解锁".to_string());
    }
    log_info!("[Developer] 开发者模式开关: {}", enabled);
    reg_set_bool(KEY_DEV_MODE, enabled)
}

// ============================================================
// 存储目录与系统信息（开发者页展示用）
// ============================================================

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
    /// 缓存目录
    pub cache: String,
    /// 临时目录
    pub temp: String,
}

/// 获取所有存储目录路径（开发者页展示用）
#[tauri::command]
pub fn get_storage_dirs() -> StorageDirs {
    let storage = Storage::instance();
    StorageDirs {
        base: storage.base_dir().to_string_lossy().to_string(),
        config: storage.config_path().to_string_lossy().to_string(),
        logs: storage.logs_dir().to_string_lossy().to_string(),
        cache: storage.cache_dir().to_string_lossy().to_string(),
        temp: storage.temp_dir().to_string_lossy().to_string(),
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
#[tauri::command]
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
