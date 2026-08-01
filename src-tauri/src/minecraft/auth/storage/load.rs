//! `AuthStorage::load` 实现
//!
//! 从文件读取认证状态（SDK DES 解密）。
//! 优先返回内存缓存，避免每次命令都重新读文件+解密+打日志。
//! `save` 方法会自动刷新缓存；如需强制从文件读取，调用 `invalidate` 后再 load。

use crate::log_info;
use crate::log_warn;

use super::types::PersistedAuthState;
use super::AuthStorage;

impl AuthStorage {
    /// 加载持久化的认证状态
    ///
    /// 优先返回内存缓存，避免每次命令都重新读文件+解密+打日志。
    /// save 系列方法会自动刷新缓存；如需强制从文件读取，调用 `invalidate` 后再 load。
    pub async fn load(&self) -> Result<PersistedAuthState, String> {
        // 优先返回缓存
        if let Some(cached) = self.cache.lock().await.clone() {
            return Ok(cached);
        }

        let path = match Self::storage_path() {
            Ok(p) => p,
            Err(e) => {
                // 环境变量缺失（无 APPDATA/HOME），返回空状态而非报错
                log_warn!("[Auth] 解析认证存储路径失败，返回空状态: {}", e);
                let state = PersistedAuthState::default();
                *self.cache.lock().await = Some(state.clone());
                return Ok(state);
            }
        };

        // 文件不存在 → 返回空状态（首次启动或未登录）
        if !path.exists() {
            let state = PersistedAuthState::default();
            *self.cache.lock().await = Some(state.clone());
            return Ok(state);
        }

        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取认证文件失败: {}", e))?;

        // 尝试解密（SDK 可用时）；SDK 不可用时降级为明文 JSON
        let json = match self.decrypt(&raw).await {
            Ok(s) => s,
            Err(e) => {
                log_warn!("[Auth] SDK 解密失败，尝试明文解析: {}", e);
                raw
            }
        };

        let state: PersistedAuthState = serde_json::from_str(&json)
            .map_err(|e| format!("解析认证状态 JSON 失败: {}", e))?;

        log_info!(
            "[Auth] Loaded persisted auth state: current_user={}, ms_accounts={}, offline_accounts={}, authlib_accounts={}",
            state
                .current_user
                .as_ref()
                .map(|u| u.name.as_str())
                .unwrap_or("none"),
            state.ms_accounts.len(),
            state.offline_accounts.len(),
            state.authlib_accounts.len(),
        );

        // 写入缓存，后续 load 直接返回
        *self.cache.lock().await = Some(state.clone());

        Ok(state)
    }
}
