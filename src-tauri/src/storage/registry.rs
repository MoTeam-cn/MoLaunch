//! Windows 注册表通用操作（storage 子模块）
//!
//! 将原本位于 `minecraft::auth::storage::registry` 的注册表低层操作
//! 提升至 `storage::registry` 模块，供 AuthStorage 与开发者模式等
//! 多处复用。认证专用键名常量仍保留在 `minecraft::auth::storage::registry`。
//!
//! 注册表路径：`HKEY_CURRENT_USER\Software\MoLaunch`

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// 注册表子键路径（所有 MoLaunch 数据均存于此键下）
pub(crate) const REG_SUBKEY: &str = "Software\\MoLaunch";

/// 打开或创建注册表子键
#[cfg(windows)]
pub(crate) fn reg_key() -> Result<RegKey, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey_with_flags(REG_SUBKEY, KEY_SET_VALUE | KEY_READ)
        .or_else(|_| {
            hkcu.create_subkey(REG_SUBKEY)
                .map(|(k, _)| k)
                .map_err(|e| e.to_string())
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
pub(crate) fn reg_delete(key: &RegKey, name: &str) -> Result<(), String> {
    match key.delete_value(name) {
        Ok(()) => Ok(()),
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("删除注册表失败: {}", e)),
    }
}

// ============================================================
// 非_windows 平台桩实现（保证跨平台编译通过）
// ============================================================

#[cfg(not(windows))]
pub(crate) fn reg_key() -> Result<(), String> {
    Err("注册表仅在 Windows 平台可用".to_string())
}

#[cfg(not(windows))]
pub(crate) fn reg_get(_key: &(), _name: &str) -> Option<String> {
    None
}

#[cfg(not(windows))]
pub(crate) fn reg_set(_key: &(), _name: &str, _value: &str) -> Result<(), String> {
    Err("注册表仅在 Windows 平台可用".to_string())
}

#[cfg(not(windows))]
pub(crate) fn reg_delete(_key: &(), _name: &str) -> Result<(), String> {
    Err("注册表仅在 Windows 平台可用".to_string())
}

// ============================================================
// 高层便捷 API（供 commands/system/developer.rs 等使用）
// ============================================================

/// 读取注册表 bool 值（"true"/"1" 视为 true，其余为 false）
///
/// 供开发者模式开关等纯布尔状态读取使用。
/// 不存在时返回 false（而非 None），便于调用方直接判断。
#[cfg(windows)]
pub(crate) fn reg_get_bool(name: &str) -> bool {
    reg_key()
        .ok()
        .and_then(|k| reg_get(&k, name))
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// 写入注册表 bool 值（存储为 "true"/"false" 字符串）
#[cfg(windows)]
pub(crate) fn reg_set_bool(name: &str, value: bool) -> Result<(), String> {
    let key = reg_key()?;
    reg_set(&key, name, if value { "true" } else { "false" })
}

#[cfg(not(windows))]
pub(crate) fn reg_get_bool(_name: &str) -> bool {
    false
}

#[cfg(not(windows))]
pub(crate) fn reg_set_bool(_name: &str, _value: bool) -> Result<(), String> {
    Err("注册表仅在 Windows 平台可用".to_string())
}
