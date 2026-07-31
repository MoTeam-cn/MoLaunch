//! 统一配置更新命令
//!
//! 用单个 `apply_config` IPC 接口取代此前分散在 proxy/download/game/community
//! 等模块的 19 个 `set_*` setter 命令。前端通过传入 `ConfigPatch`（所有字段
//! `Option<T>`，仅传需要改的字段）一次性完成多字段更新，后端在单次
//! `update_config` 闭包内完成字段赋值与联动，避免多次 IPC 往返和多次落盘。
//!
//! 三段式分流：
//! 1. 校验阶段（mirror_url SSRF 防护、download_source/meta_source 枚举校验）
//! 2. 加密字段分流（CurseForge API Key 走 secure_storage，不进 AppConfig）
//! 3. 普通字段统一更新（update_config 闭包内一次性赋值 + 联动 + 副作用）
//!
//! 注：原 `get_config` / `apply_config` 两个分散 Tauri 命令已聚合为 `config_manager`
//! 一个 IPC 入口（注册在 `commands::system::config`），通过请求体的 `action` 字段分发。
//! 子模块函数已去掉 `#[tauri::command]` 标注，改为接收 `&AppState`，
//! 由 `utils::config_manager::dispatch` 反序列化参数后调用。

mod apply;
mod secure;
mod types;
mod validate;

pub use types::{ConfigEntry, ConfigPatch, ConfigSnapshot};

use crate::state::AppState;

/// 读取配置（扁平化数组返回，支持按 keys 过滤）
///
/// - 不传 `keys` 或传空数组：返回全部字段
/// - 传 `keys`：仅返回指定字段（camelCase 名称），未匹配的字段被忽略
///
/// 返回格式：`[{ "key": "proxyMode", "value": "none" }, ...]`
///
/// CurseForge 的 `api_key` 使用异步读取（`get_config_async`），
/// 首次调用会触发 SDK DES 解密并缓存，避免此前懒加载导致首次返回空字符串的 bug。
pub async fn get_config(
    state: &AppState,
    keys: Option<Vec<String>>,
) -> Result<Vec<ConfigEntry>, String> {
    // 异步触发解密（修复懒加载导致 apiKey 不返回的 bug）
    let (cf_enabled, cf_api_key) = secure::read_curseforge().await;
    // 读取开发者模式状态（注册表，不进 AppConfig）：含 ignore_tls
    let (dev_unlocked, dev_mode, ignore_tls) = secure::read_developer();
    // 读取 Java 路径（INI [Java] path 独立存储，不进 AppConfig）
    let java_path = crate::storage::Storage::instance().get_config("Java", "path");

    // 构建全量快照（持有 config 锁的最短时间）
    let snapshot = {
        let config = state.config.lock().await;
        types::build_snapshot(
            &config,
            cf_enabled,
            cf_api_key,
            dev_unlocked,
            dev_mode,
            ignore_tls,
            java_path,
        )
    };

    // 序列化为 JSON 对象，再转为扁平数组
    let value = serde_json::to_value(&snapshot).map_err(|e| format!("序列化配置失败: {}", e))?;
    let map = value
        .as_object()
        .cloned()
        .ok_or_else(|| "配置序列化结果非对象".to_string())?;

    // 按 keys 过滤
    let keys_set: Option<std::collections::HashSet<String>> =
        keys.map(|ks| ks.into_iter().collect());

    let entries: Vec<ConfigEntry> = if let Some(ref filter) = keys_set {
        map.into_iter()
            .filter(|(k, _)| filter.contains(k))
            .map(|(k, v)| ConfigEntry { key: k, value: v })
            .collect()
    } else {
        map.into_iter()
            .map(|(k, v)| ConfigEntry { key: k, value: v })
            .collect()
    };

    Ok(entries)
}

/// 统一配置更新命令（与 `get_config` 格式对称）
///
/// 前端传入 `Vec<ConfigEntry>` 数组，格式与 `get_config` 返回值完全一致：
/// `[{ "key": "proxyMode", "value": "none" }, { "key": "communitySource", "value": 0 }, ...]`
///
/// 后端把数组转为 Map 再反序列化为 `ConfigPatch`，仅包含传入的字段会被更新。
/// `mirror_url` / `selected_version` 使用双层 Option：
/// - `null` 表示清除（Some(None)）
/// - 非空字符串表示设置（Some(Some("xxx"))）
/// - 不存在表示不更新（None）
pub async fn apply_config(
    state: &AppState,
    entries: Vec<ConfigEntry>,
) -> Result<(), String> {
    // 将扁平数组转为 Map，再反序列化为 ConfigPatch
    let mut map = serde_json::Map::new();
    for entry in entries {
        map.insert(entry.key, entry.value);
    }
    let patch: ConfigPatch = serde_json::from_value(serde_json::Value::Object(map))
        .map_err(|e| format!("解析配置补丁失败: {}", e))?;
    apply::apply_config_inner(state, patch).await
}
