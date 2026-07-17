//! Windows 注册表低层操作（常量 + 自由函数）
//!
//! 将原本是 `AuthStorage` 静态方法的 `reg_key`/`reg_get`/`reg_set`/`reg_delete`
//! 抽出为模块级自由函数，便于 `storage/mod.rs` 与 `storage/operations.rs` 复用。

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

// ============================================================
// 注册表键名定义（参考 PCL2 的命名风格）
// ============================================================

/// 注册表子键路径
pub(super) const REG_SUBKEY: &str = "Software\\MoLaunch";

/// 登录类型（明文）："Legacy" 或 "Microsoft"
pub(super) const KEY_LOGIN_TYPE: &str = "LoginType";

/// 离线登录用户名（加密）
pub(super) const KEY_LEGACY_NAME: &str = "LoginLegacyName";
/// 离线登录 UUID（加密）
pub(super) const KEY_LEGACY_UUID: &str = "LoginLegacyUuid";

/// 当前微软账号用户名（加密）
pub(super) const KEY_MS_CURRENT_NAME: &str = "MsCurrentName";
/// 当前微软账号 UUID（加密）
pub(super) const KEY_MS_CURRENT_UUID: &str = "MsCurrentUuid";
/// 当前微软账号 access_token（加密）
pub(super) const KEY_MS_CURRENT_ACCESS: &str = "MsCurrentAccess";
/// 当前微软账号 refresh_token（加密）
pub(super) const KEY_MS_CURRENT_REFRESH: &str = "MsCurrentRefresh";
/// 当前微软账号过期时间戳（加密，字符串形式的 u64）
pub(super) const KEY_MS_CURRENT_EXPIRES: &str = "MsCurrentExpires";
/// 当前微软账号档案 JSON（加密）
pub(super) const KEY_MS_CURRENT_PROFILE: &str = "MsCurrentProfile";

/// 所有微软账号列表 JSON（加密）
pub(super) const KEY_MS_ACCOUNTS: &str = "MsAccounts";

/// 所有离线账号列表 JSON（加密）
pub(super) const KEY_OFFLINE_ACCOUNTS: &str = "OfflineAccounts";

/// 所有注册表键名（用于清理）
#[cfg(windows)]
pub(super) const ALL_KEYS: &[&str] = &[
    KEY_LOGIN_TYPE,
    KEY_LEGACY_NAME,
    KEY_LEGACY_UUID,
    KEY_MS_CURRENT_NAME,
    KEY_MS_CURRENT_UUID,
    KEY_MS_CURRENT_ACCESS,
    KEY_MS_CURRENT_REFRESH,
    KEY_MS_CURRENT_EXPIRES,
    KEY_MS_CURRENT_PROFILE,
    KEY_MS_ACCOUNTS,
    KEY_OFFLINE_ACCOUNTS,
];

// ============================================================
// 注册表自由函数（原 AuthStorage 静态方法）
// ============================================================

/// 打开或创建注册表子键
#[cfg(windows)]
pub(super) fn reg_key() -> Result<RegKey, String> {
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
pub(super) fn reg_get(key: &RegKey, name: &str) -> Option<String> {
    key.get_value::<String, _>(name).ok()
}

/// 写入注册表字符串值
#[cfg(windows)]
pub(super) fn reg_set(key: &RegKey, name: &str, value: &str) -> Result<(), String> {
    key.set_value(name, &value)
        .map_err(|e| format!("写入注册表失败: {}", e))
}

/// 删除注册表值（不存在不算错误）
#[cfg(windows)]
pub(super) fn reg_delete(key: &RegKey, name: &str) -> Result<(), String> {
    match key.delete_value(name) {
        Ok(()) => Ok(()),
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("删除注册表失败: {}", e)),
    }
}
