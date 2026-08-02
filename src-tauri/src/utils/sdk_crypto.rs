//! SDK DES 加解密公共封装
//!
//! 统一 `Arc<TokioMutex<Option<SdkInstance>>>` 取锁 → encrypt_token/decrypt_token → 错误映射
//! 的样板，供各域存储模块（frp/auth/online/community）复用，消除 4 份复制实现。

use crate::sdk::SdkInstance;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// 加密字符串（SDK DES）
///
/// `ctx` 用于错误消息前缀（如 "认证数据" / "FRP token" / "API Key"），
/// 便于定位具体调用方。
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

/// 解密字符串（SDK DES），失败返回 Err
///
/// 供需要区分「解密失败」与「无数据」的调用方使用（如 auth/online 存储）。
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

/// 解密字符串（SDK DES），失败记 warn 并返回 None
///
/// 供「SDK 不可用时视为无数据」的调用方使用（如 frp/community 存储，
/// 解密失败即视为未认证/未配置）。
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
