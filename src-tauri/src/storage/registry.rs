//! 跨平台系统注册表（KV）：Windows 写注册表，macOS/Linux 写全局共用 JSON 文件
//!
//! 提供 AuthStorage（Windows 分支）、开发者模式、正版购买提示等多处复用的低层 KV 操作。
//! 认证专用键名常量仍保留在 `minecraft::auth::storage::registry`。
//! - Windows：注册表 `HKEY_CURRENT_USER\Software\MoLaunch`
//! - 其他平台：全局共用文件 `~/.config/Molaunch/system.json`（XDG 配置系统目录，
//!   跨启动器实例共享；目录缺失自动创建，文件损坏自动回退默认值，即"有保底"）。
//!   所有"注册表字段"在此平台统一收敛到这一个文件，避免各功能分散建文件导致紊乱。

#[cfg(windows)]
use crate::error_util::log_err;
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;
#[cfg(not(windows))]
use std::collections::HashMap;
#[cfg(not(windows))]
use std::path::PathBuf;

/// 注册表子键路径（所有 MoLaunch 数据均存于此键下）
#[cfg(windows)]
pub(crate) const REG_SUBKEY: &str = "Software\\MoLaunch";

/// 打开或创建注册表子键
#[cfg(windows)]
pub(crate) fn reg_key() -> Result<RegKey, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey_with_flags(REG_SUBKEY, KEY_SET_VALUE | KEY_READ)
        .or_else(|_| {
            hkcu.create_subkey(REG_SUBKEY)
                .map(|(k, _)| k)
                .map_err(log_err("Failed to create registry subkey"))
        })
        .map_err(|e| format!("打开注册表失败: {}", e))
}

/// 读取注册表字符串值
#[cfg(windows)]
pub(crate) fn reg_get(key: &RegKey, name: &str) -> Option<String> {
    key.get_value::<String, _>(name).ok()
}

/// 写入注册表字符串值
#[cfg(windows)]
pub(crate) fn reg_set(key: &RegKey, name: &str, value: &str) -> Result<(), String> {
    key.set_value(name, &value)
        .map_err(|e| format!("写入注册表失败: {}", e))
}

/// 删除注册表值（不存在不算错误）
#[cfg(windows)]
#[allow(dead_code)] // auth/storage 改为文件存储后暂无调用方，保留以备 crate 级复用
pub(crate) fn reg_delete(key: &RegKey, name: &str) -> Result<(), String> {
    match key.delete_value(name) {
        Ok(()) => Ok(()),
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("删除注册表失败: {}", e)),
    }
}

// ==================== 非 Windows：统一 JSON 文件实现 ====================
//
// 非 Windows 平台无注册表，所有"注册表字段"读写 `~/.config/Molaunch/system.json`，
// 值统一以字符串存储（与 Windows REG_SZ 语义一致），函数签名与 Windows 版一致。

/// 非 Windows 系统存储文件路径（`appdata_root()/system.json`）
#[cfg(not(windows))]
fn sys_file_path() -> Result<PathBuf, String> {
    Ok(crate::storage::appdata::appdata_root()?.join("system.json"))
}

/// 读取全部键值（文件缺失/损坏时返回空表）
#[cfg(not(windows))]
fn read_all() -> HashMap<String, String> {
    sys_file_path()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// 写回全部键值（保底：自动创建目录）
#[cfg(not(windows))]
fn write_all(map: &HashMap<String, String>) -> Result<(), String> {
    let path = sys_file_path()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("创建系统存储目录失败: {}", e))?;
    }
    let json = serde_json::to_string_pretty(map)
        .map_err(|e| format!("序列化系统存储失败: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("写入系统存储失败: {}", e))?;
    Ok(())
}

/// 打开存储（哨兵：非 Windows 无 key 概念，仅保持与 Windows 版签名一致）
#[cfg(not(windows))]
pub(crate) fn reg_key() -> Result<(), String> {
    Ok(())
}

/// 读取字符串值（键不存在返回 None）
#[cfg(not(windows))]
pub(crate) fn reg_get(_key: &(), name: &str) -> Option<String> {
    read_all().get(name).cloned()
}

/// 写入字符串值
#[cfg(not(windows))]
pub(crate) fn reg_set(_key: &(), name: &str, value: &str) -> Result<(), String> {
    let mut map = read_all();
    map.insert(name.to_string(), value.to_string());
    write_all(&map)
}

/// 删除值（不存在不算错误）
#[cfg(not(windows))]
#[allow(dead_code)] // auth 非 Windows 走文件存储，暂无调用方；保留与 Windows 版对称
pub(crate) fn reg_delete(_key: &(), name: &str) -> Result<(), String> {
    let mut map = read_all();
    if map.remove(name).is_some() {
        write_all(&map)?;
    }
    Ok(())
}

/// 读取注册表 bool 值（"true"/"1" 视为 true，其余为 false）
///
/// 返回 `Option<bool>` 以显式区分三种状态：
/// - `Some(true)`：值为 "true" / "1"
/// - `Some(false)`：值为其他字符串（如 "false" / "0" / "yes" 等）
/// - `None`：键不存在
///
/// 调用方通常用 `.unwrap_or(false)` 保持原"键不存在视为 false"语义；
/// 需要区分"不存在"与"false"的场景可 `match` 显式处理。
#[cfg(windows)]
pub(crate) fn reg_get_bool(name: &str) -> Option<bool> {
    reg_key()
        .ok()
        .and_then(|k| reg_get(&k, name))
        .map(|v| v == "true" || v == "1")
}

/// 写入注册表 bool 值（存储为 "true"/"false" 字符串）
#[cfg(windows)]
pub(crate) fn reg_set_bool(name: &str, value: bool) -> Result<(), String> {
    let key = reg_key()?;
    reg_set(&key, name, if value { "true" } else { "false" })
}

/// 非 Windows：读取 bool 值（统一 JSON 文件，与注册表"true"/"1"语义一致）
#[cfg(not(windows))]
pub(crate) fn reg_get_bool(name: &str) -> Option<bool> {
    reg_get(&(), name).map(|v| v == "true" || v == "1")
}

/// 非 Windows：写入 bool 值（存储为 "true"/"false" 字符串）
#[cfg(not(windows))]
pub(crate) fn reg_set_bool(name: &str, value: bool) -> Result<(), String> {
    reg_set(&(), name, if value { "true" } else { "false" })
}
