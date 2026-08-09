//! 加密格式迁移：存量 SDK DES(v1) 数据重加密为 AES(v2)。
//!
//! 通过 `mc_decrypt_token_ex` 检测算法版本（1=DES 旧密文，2=AES 当前），
//! 仅 v1 数据解密后重加密为 v2；v2 / 无法解密条目保持原样。
//! 升级后首次启动执行一次（后台，不阻塞 UI），完成后写标记文件。

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use crate::log_info;
use crate::log_warn;
use crate::sdk::SdkInstance;

/// 迁移完成标记文件名（AppData 根目录）
const DONE_MARK: &str = "crypto_v3.done";

/// 将单条加密值迁移为 SDK AES 格式；仅 DES(v1) 旧密文重加密，其余（v2/明文/损坏）保持原样
async fn migrate_value(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    value: &str,
    ctx: &str,
) -> Option<String> {
    let (plain, version) =
        match crate::utils::sdk_crypto::decrypt_with_sdk_version(sdk_arc, value, ctx).await {
            Ok(r) => r,
            Err(_) => return None,
        };
    if version != 1 {
        return None;
    }
    match crate::utils::sdk_crypto::encrypt_with_sdk(sdk_arc, &plain, ctx).await {
        Ok(enc) => Some(enc),
        Err(e) => {
            log_warn!("[CryptoV3] {}重加密失败（保持原样）: {}", ctx, e);
            None
        }
    }
}

/// 迭代迁移 JSON 中所有字符串字段，返回是否有字段被改写
#[cfg(not(windows))]
async fn migrate_json_value(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    value: &mut serde_json::Value,
) -> bool {
    let mut changed = false;
    let mut stack = vec![value];
    while let Some(node) = stack.pop() {
        match node {
            serde_json::Value::String(s) => {
                if !s.is_empty() {
                    if let Some(new_s) = migrate_value(sdk_arc, s, "auth.json 字段").await {
                        *s = new_s;
                        changed = true;
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    stack.push(item);
                }
            }
            serde_json::Value::Object(map) => {
                for v in map.values_mut() {
                    stack.push(v);
                }
            }
            _ => {}
        }
    }
    changed
}

/// Windows 注册表认证字段逐键迁移
#[cfg(windows)]
async fn migrate_registry_auth(sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>) {
    use crate::minecraft::auth::storage::registry::ALL_KEYS;
    use crate::storage::registry::{reg_get, reg_key, reg_set};

    let key = match reg_key() {
        Ok(k) => k,
        Err(e) => {
            log_warn!("[CryptoV3] 打开认证注册表失败: {}", e);
            return;
        }
    };
    for name in ALL_KEYS {
        let Some(value) = reg_get(&key, name) else {
            continue;
        };
        if value.is_empty() {
            continue;
        }
        if let Some(new_value) = migrate_value(sdk_arc, &value, "认证注册表字段").await {
            if let Err(e) = reg_set(&key, name, &new_value) {
                log_warn!("[CryptoV3] 注册表字段 {} 迁移写回失败: {}", name, e);
            }
        }
    }
}

/// 非 Windows auth.json 逐字段迁移
#[cfg(not(windows))]
async fn migrate_auth_file(sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>) {
    let path = match crate::storage::appdata::appdata_root() {
        Ok(root) => root.join("auth.json"),
        Err(e) => {
            log_warn!("[CryptoV3] 解析 AppData 路径失败: {}", e);
            return;
        }
    };
    if !path.exists() {
        return;
    }
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            log_warn!("[CryptoV3] 读取 auth.json 失败: {}", e);
            return;
        }
    };
    let mut root: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            log_warn!("[CryptoV3] 解析 auth.json 失败: {}", e);
            return;
        }
    };
    if migrate_json_value(sdk_arc, &mut root).await {
        if let Ok(content) = serde_json::to_string_pretty(&root) {
            if let Err(e) = std::fs::write(&path, content) {
                log_warn!("[CryptoV3] 写回 auth.json 失败: {}", e);
            }
        }
    }
}

/// 联机设备凭证整文件迁移
async fn migrate_device_json(sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>) {
    migrate_file(sdk_arc, "online", "device.json", "联机设备凭证").await;
}

/// FRP 厂商 token 目录下所有 json 文件迁移
async fn migrate_frp_auth(sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>) {
    let dir = match crate::storage::appdata::appdata_subdir("frp_auth") {
        Ok(d) => d,
        Err(e) => {
            log_warn!("[CryptoV3] 解析 frp_auth 目录失败: {}", e);
            return;
        }
    };
    if !dir.exists() {
        return;
    }
    let entries = match std::fs::read_dir(&dir) {
        Ok(it) => it,
        Err(e) => {
            log_warn!("[CryptoV3] 读取 frp_auth 目录失败: {}", e);
            return;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        migrate_file_at(sdk_arc, &path, "FRP 厂商 token").await;
    }
}

/// 读取 AppData 子目录下的加密文件并迁移
async fn migrate_file(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    subdir: &str,
    file_name: &str,
    ctx: &str,
) {
    let path = match crate::storage::appdata::appdata_subdir(subdir) {
        Ok(dir) => dir.join(file_name),
        Err(e) => {
            log_warn!("[CryptoV3] 解析 {} 目录失败: {}", subdir, e);
            return;
        }
    };
    migrate_file_at(sdk_arc, &path, ctx).await;
}

/// 单个加密文件整体迁移（解密→重加密→写回）
async fn migrate_file_at(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    path: &PathBuf,
    ctx: &str,
) {
    if !path.exists() {
        return;
    }
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            log_warn!("[CryptoV3] 读取 {} 失败: {}", path.display(), e);
            return;
        }
    };
    if raw.is_empty() {
        return;
    }
    if let Some(new_raw) = migrate_value(sdk_arc, &raw, ctx).await {
        if let Err(e) = std::fs::write(path, &new_raw) {
            log_warn!("[CryptoV3] {} 迁移写回失败: {}", path.display(), e);
        }
    }
}

/// INI 中的 API Key 迁移（CurseForge / AI）
async fn migrate_ini_keys(sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>) {
    let storage = crate::storage::Storage::instance();
    for (section, key, ctx) in [
        ("CurseForge", "api_key", "CurseForge api_key"),
        ("AI", "api_key", "AI api_key"),
    ] {
        let Some(enc) = storage.get_config(section, key) else {
            continue;
        };
        if enc.is_empty() {
            continue;
        }
        if let Some(new_enc) = migrate_value(sdk_arc, &enc, ctx).await {
            if let Err(e) = storage.set_config(section, key, &new_enc) {
                log_warn!("[CryptoV3] {} 迁移写回失败: {}", ctx, e);
            }
        }
    }
}

/// 启动时加密格式迁移入口（升级后首次启动执行一次）
pub async fn migrate(sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>) {
    if sdk_arc.lock().await.is_none() {
        log_warn!("[CryptoV3] SDK 未加载，跳过加密迁移（下次启动重试）");
        return;
    }
    let mark = match crate::storage::appdata::appdata_root() {
        Ok(root) => root.join(DONE_MARK),
        Err(e) => {
            log_warn!("[CryptoV3] 解析 AppData 路径失败，跳过迁移: {}", e);
            return;
        }
    };
    if mark.exists() {
        return;
    }

    log_info!("[CryptoV3] 开始迁移加密数据（DES v1 → SDK AES v2）");

    #[cfg(windows)]
    migrate_registry_auth(sdk_arc).await;
    #[cfg(not(windows))]
    migrate_auth_file(sdk_arc).await;
    migrate_device_json(sdk_arc).await;
    migrate_frp_auth(sdk_arc).await;
    migrate_ini_keys(sdk_arc).await;

    match std::fs::write(&mark, "done") {
        Ok(()) => log_info!("[CryptoV3] 加密数据迁移完成"),
        Err(e) => log_warn!("[CryptoV3] 写入完成标记失败（下次启动重试）: {}", e),
    }
}
