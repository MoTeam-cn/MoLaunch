//! AI 配置持久化（config.ini [AI] 段 + SDK DES 加密 api_key）
//!
//! api_key 用 SDK DES 加密后存 INI，其余字段明文。懒加载：不在启动时解密，
//! 避免触发杀软启发式；首次 async 请求时解密并缓存（参照 CurseForge 模式）。

use std::sync::{Arc, OnceLock, RwLock};

use tokio::sync::Mutex as TokioMutex;

use super::config::AiConfig;
use crate::sdk::SdkInstance;
use crate::storage::Storage;

/// INI 段名
const SECTION: &str = "AI";
/// INI key：服务地址（明文）
const KEY_BASE_URL: &str = "base_url";
/// INI key：API Key（DES 加密后字符串）
const KEY_API_KEY: &str = "api_key";
/// INI key：超时秒数（明文）
const KEY_TIMEOUT: &str = "timeout_secs";
/// INI key：已启用模型列表（JSON 数组）
const KEY_MODELS: &str = "models";
/// INI key：默认模型名
const KEY_DEFAULT_MODEL: &str = "default_model";

/// api_key 解密缓存状态
struct ApiKeyCache {
    /// 是否已尝试解密过（避免重复解密）
    decrypted: bool,
    /// 解密后的 api_key（未配置/无 SDK 时为 None）
    value: Option<String>,
}

/// SDK 引用：首次请求时通过 `set_sdk` 注入，避免启动时持有
static SDK_REF: OnceLock<Arc<TokioMutex<Option<SdkInstance>>>> = OnceLock::new();

/// 解密后的 api_key 缓存（懒加载，避免启动时 DES 解密触发杀软）
static API_KEY_CACHE: OnceLock<RwLock<ApiKeyCache>> = OnceLock::new();

/// 注入 SDK 引用（lib.rs 启动时调用）
pub fn set_sdk(sdk: Arc<TokioMutex<Option<SdkInstance>>>) {
    let _ = SDK_REF.set(sdk);
}

/// 懒加载解密 api_key 并缓存（首次 async 请求时调用）
async fn ensure_api_key_decrypted() {
    if API_KEY_CACHE.get().is_some_and(|s| s.read().unwrap().decrypted) {
        return;
    }

    let storage = Storage::instance();
    let encrypted = storage.get_config(SECTION, KEY_API_KEY).unwrap_or_default();
    let api_key = if encrypted.is_empty() {
        None
    } else {
        match SDK_REF.get() {
            Some(sdk_arc) => {
                crate::utils::sdk_crypto::decrypt_with_sdk_optional(sdk_arc, &encrypted, "AI api_key")
                    .await
            }
            None => {
                crate::log_warn!("[AI] SDK 未注入，api_key 无法解密");
                None
            }
        }
    };

    let cache = API_KEY_CACHE.get_or_init(|| {
        RwLock::new(ApiKeyCache {
            decrypted: false,
            value: None,
        })
    });
    let mut guard = cache.write().unwrap();
    if !guard.decrypted {
        guard.value = api_key;
        guard.decrypted = true;
    }
}

/// 同步读取配置（api_key 返回缓存值，未解密过则返回空串）
pub fn load() -> AiConfig {
    let cached_key = API_KEY_CACHE
        .get()
        .and_then(|c| c.read().unwrap().value.clone())
        .unwrap_or_default();
    let storage = Storage::instance();
    AiConfig {
        base_url: storage.get_config(SECTION, KEY_BASE_URL).unwrap_or_default(),
        api_key: cached_key,
        timeout_secs: storage
            .get_config(SECTION, KEY_TIMEOUT)
            .and_then(|v| v.parse().ok())
            .unwrap_or(60),
        models: storage
            .get_config(SECTION, KEY_MODELS)
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default(),
        default_model: storage
            .get_config(SECTION, KEY_DEFAULT_MODEL)
            .unwrap_or_default(),
    }
}

/// 异步读取配置（首次触发懒加载解密 api_key）
pub async fn load_async() -> AiConfig {
    ensure_api_key_decrypted().await;
    load()
}

/// 保存配置到 INI（api_key 经 SDK DES 加密）并更新内存缓存
pub async fn save(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    config: &AiConfig,
) -> Result<(), String> {
    let storage = Storage::instance();

    storage
        .set_config(SECTION, KEY_BASE_URL, &config.base_url)
        .map_err(|e| format!("保存 AI 服务地址失败: {}", e))?;
    storage
        .set_config(SECTION, KEY_TIMEOUT, &config.timeout_secs.to_string())
        .map_err(|e| format!("保存 AI 超时配置失败: {}", e))?;

    let models_json = serde_json::to_string(&config.models).map_err(|e| e.to_string())?;
    storage
        .set_config(SECTION, KEY_MODELS, &models_json)
        .map_err(|e| format!("保存 AI 模型列表失败: {}", e))?;
    storage
        .set_config(SECTION, KEY_DEFAULT_MODEL, &config.default_model)
        .map_err(|e| format!("保存 AI 默认模型失败: {}", e))?;

    // api_key 加密后写入（空串表示清空）
    let stored_key = if config.api_key.is_empty() {
        String::new()
    } else {
        crate::utils::sdk_crypto::encrypt_with_sdk(sdk_arc, &config.api_key, "AI API Key").await?
    };
    storage
        .set_config(SECTION, KEY_API_KEY, &stored_key)
        .map_err(|e| format!("保存 AI API Key 失败: {}", e))?;

    // 更新内存缓存
    let cache = API_KEY_CACHE.get_or_init(|| {
        RwLock::new(ApiKeyCache {
            decrypted: false,
            value: None,
        })
    });
    let mut guard = cache.write().unwrap();
    guard.value = if config.api_key.is_empty() {
        None
    } else {
        Some(config.api_key.clone())
    };
    guard.decrypted = true;

    crate::log_info!(
        "[AI] 配置已保存: base_url={}, api_key={}, models={}, default_model={}",
        if config.base_url.is_empty() { "(空)" } else { "已配置" },
        if config.api_key.is_empty() { "未配置" } else { "已更新" },
        config.models.len(),
        if config.default_model.is_empty() {
            "(未设置)"
        } else {
            &config.default_model
        }
    );

    Ok(())
}