//! `AuthStorage::load` 实现：注册表 / JSON 文件双轨制读取（优先返回内存缓存）
//!
//! 子模块：file（非 Windows JSON）/ registry（Windows 注册表）。

#[cfg(not(windows))]
mod file;
#[cfg(windows)]
mod registry;

use super::types::PersistedAuthState;
use super::AuthStorage;

impl AuthStorage {
    /// 加载持久化的认证状态
    ///
    /// 优先返回内存缓存，避免每次命令都重新读存储+解密+打日志。
    /// save 系列方法会自动刷新缓存；如需强制从存储读取，调用 `invalidate` 后再 load。
    pub async fn load(&self) -> Result<PersistedAuthState, String> {
        // 优先返回缓存
        if let Some(cached) = self.cache.lock().await.clone() {
            return Ok(cached);
        }

        #[cfg(not(windows))]
        {
            self.load_from_file().await
        }

        #[cfg(windows)]
        {
            self.load_from_registry().await
        }
    }
}