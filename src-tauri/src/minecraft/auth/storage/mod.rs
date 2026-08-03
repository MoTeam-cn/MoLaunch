//! 认证持久化模块：Windows 注册表 / 非 Windows JSON 文件双轨制存储（敏感字段 SDK 加密）

mod load;
mod manager;
mod operations;
#[cfg(windows)]
mod registry;
mod save;
mod types;

pub use manager::AuthStorage;
pub use types::{
    CurrentUser, PersistedAuthState, StoredAuthlibAccount, StoredMsAccount, StoredOfflineAccount,
};
