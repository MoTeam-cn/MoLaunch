//! `AuthStorage::load` 读取分发：优先返回内存缓存，再按平台分支读取

use super::super::types::PersistedAuthState;
use super::super::AuthStorage;

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