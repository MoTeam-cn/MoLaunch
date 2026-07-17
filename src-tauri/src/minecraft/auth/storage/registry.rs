//! 认证专用注册表键名常量
//!
//! 注册表低层操作（reg_key/reg_get/reg_set/reg_delete）已提升至
//! `crate::storage::registry` 模块供 crate 级共享。本文件仅保留认证专用键名常量。

/// 所有微软账号列表 JSON（加密）
pub(super) const KEY_MS_ACCOUNTS: &str = "MsAccounts";

/// 所有离线账号列表 JSON（加密）
pub(super) const KEY_OFFLINE_ACCOUNTS: &str = "OfflineAccounts";

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
