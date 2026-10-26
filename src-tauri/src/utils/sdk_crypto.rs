//! SDK Token 加解密封装：完全依赖 RunSDK 内置实现
//! （加密 AES-256-CBC，解密自动兼容旧版 DES）。
//!
//! SDK 未加载时会做一次懒加载重试（启动阶段被杀软拦截/临时目录异常都可能失败），
//! 仍失败则把原因与动态库路径一并写进错误信息，便于用户自助排查。

use crate::sdk::SdkInstance;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

/// 最近一次 SDK 加载失败的原因（启动时写入，懒重试失败时覆盖）
static SDK_LOAD_ERROR: Mutex<Option<String>> = Mutex::new(None);

/// 记录 SDK 加载失败原因（供错误信息展示）
pub fn set_load_error(message: String) {
    if let Ok(mut slot) = SDK_LOAD_ERROR.lock() {
        *slot = Some(message);
    }
}

/// 读取最近一次 SDK 加载失败原因
pub fn load_error() -> Option<String> {
    SDK_LOAD_ERROR.lock().ok().and_then(|slot| slot.clone())
}

fn clear_load_error() {
    if let Ok(mut slot) = SDK_LOAD_ERROR.lock() {
        *slot = None;
    }
}

/// SDK 不可用时拼装可操作的错误信息
pub(super) fn unavailable_message(action: &str, ctx: &str) -> String {
    let path = crate::sdk::get_sdk_library_path();
    let reason = load_error().unwrap_or_else(|| "未知原因".to_string());
    format!(
        "SDK 未加载，无法{}{}（原因：{}；动态库：{}）。常见原因是杀毒软件拦截该动态库或临时目录不可写，可把该文件加入白名单后重启启动器",
        action,
        ctx,
        reason,
        path.display()
    )
}

/// SDK 未加载时尝试懒加载一次（启动阶段失败的用户可在此自愈）
fn retry_load_if_needed(guard: &mut Option<SdkInstance>) {
    if guard.is_some() {
        return;
    }
    match SdkInstance::load() {
        Ok(instance) => {
            crate::log_info!("[SDK] 懒加载成功");
            clear_load_error();
            *guard = Some(instance);
        }
        Err(e) => {
            crate::log_warn!("[SDK] 懒加载仍失败: {}", e);
            set_load_error(e.to_string());
        }
    }
}

/// 加密字符串（SDK AES-256-CBC）
pub async fn encrypt_with_sdk(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Result<String, String> {
    let mut guard = sdk_arc.lock().await;
    retry_load_if_needed(&mut guard);
    match guard.as_ref() {
        Some(sdk) => sdk
            .encrypt_token(data)
            .map_err(|e| format!("{}加密失败: {}", ctx, e)),
        None => Err(unavailable_message("加密", ctx)),
    }
}

/// 解密字符串（SDK 内部自动 AES→DES 回退），失败返回 Err
pub async fn decrypt_with_sdk(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Result<String, String> {
    let mut guard = sdk_arc.lock().await;
    retry_load_if_needed(&mut guard);
    match guard.as_ref() {
        Some(sdk) => sdk
            .decrypt_token(data)
            .map_err(|e| format!("{}解密失败: {}", ctx, e)),
        None => Err(unavailable_message("解密", ctx)),
    }
}

/// 解密字符串并返回算法版本（1=DES(v1) 旧密文，2=AES(v2) 当前；供迁移判定）
pub async fn decrypt_with_sdk_version(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Result<(String, i32), String> {
    let mut guard = sdk_arc.lock().await;
    retry_load_if_needed(&mut guard);
    match guard.as_ref() {
        Some(sdk) => sdk
            .decrypt_token_with_version(data)
            .map_err(|e| format!("{}解密失败: {}", ctx, e)),
        None => Err(unavailable_message("解密", ctx)),
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
