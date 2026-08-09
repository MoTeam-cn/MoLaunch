//! CurseForge API Key 加密存储（INI + 懒加载）
//!
//! enabled 明文存 INI；api_key 用文件级强加密（AES-256-GCM）后存 INI。
//! 懒加载：不在启动时解密，避免触发杀软启发式；首次 CF 请求时于异步上下文解密并缓存。

use crate::error_util::log_err;
use crate::sdk::SdkInstance;
use std::sync::{Arc, OnceLock, RwLock};

use tokio::sync::Mutex as TokioMutex;

/// INI 段名
const SECTION: &str = "CurseForge";
/// INI key：启用开关（明文）
const KEY_ENABLED: &str = "enabled";
/// INI key：API Key（文件级强加密后字符串）
const KEY_API_KEY: &str = "api_key";

/// 内存状态
struct State {
    /// enabled 开关（启动时同步从 INI 读，无需 SDK）
    enabled: bool,
    /// 解密后的 api_key（首次请求时懒加载解密）
    api_key: Option<String>,
    /// 是否已尝试解密过（避免重复解密）
    decrypted: bool,
}

/// 全局状态：启动时只初始化 enabled（从 INI 同步读），api_key 延迟到首次请求
static STATE: OnceLock<RwLock<State>> = OnceLock::new();

/// SDK 引用：首次请求时通过 ensure_initialized 注入，避免启动时持有
static SDK_REF: OnceLock<Arc<TokioMutex<Option<SdkInstance>>>> = OnceLock::new();

/// 在启动时同步初始化 enabled（不调用 SDK，不触发杀软）
///
/// 必须在 tauri::Builder 之前调用。
pub fn init_enabled() {
    let storage = crate::storage::Storage::instance();
    let enabled = storage
        .get_config(SECTION, KEY_ENABLED)
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    crate::log_info!("[Community] CF enabled={} (从 INI 同步读取)", enabled);

    let _ = STATE.set(RwLock::new(State {
        enabled,
        api_key: None,
        decrypted: false,
    }));
}

/// 注入 SDK 引用（由 Tauri 命令或首次请求时传入）
pub fn set_sdk(sdk: Arc<TokioMutex<Option<SdkInstance>>>) {
    let _ = SDK_REF.set(sdk);
}

/// 首次请求时异步解密 api_key 并缓存（与 AuthStorage 的按需模式一致）
///
/// 在 curseforge.rs 的请求路径首次调用；后续直接读缓存。
/// 注意：不跨 .await 持有非 Send 的 guard（避免 future 非 Send）
async fn ensure_api_key_decrypted() {
    // 快速路径：已解密过（读完立即释放 guard）
    if STATE.get().is_some_and(|s| s.read().unwrap().decrypted) {
        return;
    }

    // 读取加密的 api_key（不持有 guard）
    let storage = crate::storage::Storage::instance();
    let encrypted_key = storage.get_config(SECTION, KEY_API_KEY);

    // await SDK DES 解密（不持有任何 guard）
    let api_key = match encrypted_key {
        Some(ref enc) if !enc.is_empty() => match SDK_REF.get() {
            Some(sdk_arc) => {
                crate::utils::sdk_crypto::decrypt_with_sdk_optional(sdk_arc, enc, "CF api_key")
                    .await
            }
            None => {
                crate::log_warn!("[Community] SDK 未注入，CF api_key 无法解密");
                None
            }
        },
        _ => None,
    };

    crate::log_info!(
        "[Community] CF api_key 解密完成: {}",
        if api_key.is_some() {
            "已配置"
        } else {
            "未配置"
        }
    );

    // 写入结果（double-check 防止竞态）
    let state = STATE.get_or_init(|| {
        RwLock::new(State {
            enabled: false,
            api_key: None,
            decrypted: false,
        })
    });
    let mut guard = state.write().unwrap();
    if !guard.decrypted {
        guard.api_key = api_key;
        guard.decrypted = true;
    }
}

/// 异步获取完整配置（首次调用会触发解密）
///
/// 供 curseforge.rs 使用：必须在 async 上下文调用，首次会阻塞解密。
pub async fn get_config_async() -> (bool, Option<String>) {
    ensure_api_key_decrypted().await;
    let state = STATE
        .get()
        .map(|s| s.read().unwrap())
        .expect("secure_storage not initialized");
    (state.enabled, state.api_key.clone())
}

/// 同步读取缓存配置（供无法 async 的场景）
///
/// 若 api_key 尚未解密，返回 (enabled, None)——调用方需在 async 上下文
/// 先调用 get_config_async() 触发解密。
pub fn get_cached() -> (bool, Option<String>) {
    if let Some(state) = STATE.get() {
        let g = state.read().unwrap();
        (g.enabled, g.api_key.clone())
    } else {
        (false, None)
    }
}

/// 保存配置到 INI（api_key 加密）并更新内存缓存
///
/// SDK 通过参数传入（由 Tauri 命令从 AppState 获取）
pub async fn save(
    sdk_arc: Arc<TokioMutex<Option<SdkInstance>>>,
    enabled: bool,
    api_key: &str,
) -> Result<(), String> {
    let storage = crate::storage::Storage::instance();

    // enabled 明文写入 INI
    storage
        .set_config(SECTION, KEY_ENABLED, if enabled { "true" } else { "false" })
        .map_err(log_err("Failed to save CurseForge enabled flag"))?;

    // api_key 加密后写入 INI
    let stored_key = if api_key.is_empty() {
        String::new()
    } else {
        crate::utils::sdk_crypto::encrypt_with_secure_sdk(&sdk_arc, api_key, "CF API Key").await?
    };
    storage
        .set_config(SECTION, KEY_API_KEY, &stored_key)
        .map_err(log_err("Failed to save CurseForge API key"))?;

    // 更新内存缓存
    let state = STATE.get_or_init(|| {
        RwLock::new(State {
            enabled: false,
            api_key: None,
            decrypted: false,
        })
    });
    {
        let mut g = state.write().unwrap();
        g.enabled = enabled;
        g.api_key = if api_key.is_empty() {
            None
        } else {
            Some(api_key.to_string())
        };
        g.decrypted = true;
    }

    crate::log_info!(
        "[Community] CF 配置已保存: enabled={}, api_key={}",
        enabled,
        if api_key.is_empty() {
            "已清空"
        } else {
            "已更新"
        }
    );

    Ok(())
}
