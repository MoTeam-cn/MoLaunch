//! `AuthStorage::save` 实现
//!
//! 将 `PersistedAuthState` 整体序列化为 JSON → SDK DES 加密 → 写入单文件。
//! Unix 显式设置 0o600 权限保护敏感字段；写完后刷新内存缓存。

use crate::log_warn;

use super::types::PersistedAuthState;
use super::AuthStorage;

impl AuthStorage {
    /// 保存认证状态到文件
    ///
    /// 将 `PersistedAuthState` 整体序列化为 JSON（通过 `to_storage_json()` 避免派生 `Serialize`
    /// 误暴露敏感字段到 IPC），SDK DES 加密后写入单文件。Unix 显式设置 0o600 权限。
    pub async fn save(&self, state: &PersistedAuthState) -> Result<(), String> {
        // 通过 to_storage_json 手动序列化，避免派生 Serialize 误暴露 token
        let json = serde_json::to_string(&state.to_storage_json())
            .map_err(|e| format!("序列化认证状态失败: {}", e))?;

        // 优先加密存储；SDK 不可用时降级为明文（带警告）
        let stored = match self.encrypt(&json).await {
            Ok(s) => s,
            Err(e) => {
                log_warn!("[Auth] SDK 加密失败，降级明文存储: {}", e);
                json
            }
        };

        let path = Self::storage_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建认证存储目录失败: {}", e))?;
        }
        std::fs::write(&path, &stored)
            .map_err(|e| format!("写入认证文件失败: {}", e))?;

        // Unix 下显式设置文件权限为 0o600（仅当前用户可读写），防止其他用户读取 token
        #[cfg(unix)]
        super::restrict_file_permissions(&path);

        // 更新内存缓存
        *self.cache.lock().await = Some(state.clone());

        Ok(())
    }
}
