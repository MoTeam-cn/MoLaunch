//! SDK Token 加解密封装：完全依赖 RunSDK 内置实现
//! （加密 AES-256-CBC，解密自动兼容旧版 DES，协议见 docs/token-encryption.md）。

use crate::sdk::SdkInstance;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// 加密字符串（SDK AES-256-CBC，协议见 docs/token-encryption.md）
pub async fn encrypt_with_sdk(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Result<String, String> {
    let sdk = sdk_arc.lock().await;
    match sdk.as_ref() {
        Some(sdk) => sdk
            .encrypt_token(data)
            .map_err(|e| format!("{}加密失败: {}", ctx, e)),
        None => Err(format!("SDK 未加载，无法加密{}", ctx)),
    }
}

/// 解密字符串（SDK 内部自动 AES→DES 回退），失败返回 Err
pub async fn decrypt_with_sdk(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Result<String, String> {
    let sdk = sdk_arc.lock().await;
    match sdk.as_ref() {
        Some(sdk) => sdk
            .decrypt_token(data)
            .map_err(|e| format!("{}解密失败: {}", ctx, e)),
        None => Err(format!("SDK 未加载，无法解密{}", ctx)),
    }
}

/// 解密字符串并返回算法版本（1=DES(v1) 旧密文，2=AES(v2) 当前；供迁移判定）
pub async fn decrypt_with_sdk_version(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Result<(String, i32), String> {
    let sdk = sdk_arc.lock().await;
    match sdk.as_ref() {
        Some(sdk) => sdk
            .decrypt_token_with_version(data)
            .map_err(|e| format!("{}解密失败: {}", ctx, e)),
        None => Err(format!("SDK 未加载，无法解密{}", ctx)),
    }
}

/// 解密字符串，失败记 warn 并返回 None（供 frp/community 使用）
pub async fn decrypt_with_sdk_optional(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Option<String> {
    match decrypt_with_sdk(sdk_arc, data, ctx).await {
        Ok(s) => Some(s),
        Err(e) => {
            crate::log_warn!("[SDK Crypto] {}: {}", ctx, e);
            None
        }
    }
}
