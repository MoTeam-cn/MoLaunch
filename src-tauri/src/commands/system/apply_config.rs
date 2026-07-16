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

use crate::log_info;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// 配置补丁：所有字段可选，仅传需要更新的字段
///
/// 字段命名采用 camelCase 序列化（前端约定），与 `AppConfig` 的 snake_case
/// 字段一一对应（通过 `#[serde(rename_all = "camelCase")]` 映射）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPatch {
    // ===== 代理 =====
    pub proxy_mode: Option<String>,
    pub proxy_type: Option<String>,
    pub proxy_url: Option<String>,

    // ===== 下载 =====
    /// "official" / "mirror" / "smart"
    pub download_source: Option<String>,
    pub meta_source: Option<String>,
    pub max_download_speed: Option<u64>,
    pub max_download_threads: Option<u32>,
    pub chunk_count: Option<u32>,
    /// 双层 Option：外层 Some 表示"要更新此字段"，内层 None 表示"清空"
    pub mirror_url: Option<Option<String>>,

    // ===== 内存 =====
    pub memory_mode: Option<String>,
    pub min_memory: Option<u32>,
    pub max_memory: Option<u32>,

    // ===== 启动器 =====
    pub game_dir: Option<String>,
    pub isolation_mode: Option<u32>,
    pub log_level: Option<u32>,
    pub selected_version: Option<Option<String>>,

    // ===== 社区资源（INI 明文，进 AppConfig）=====
    pub community_source: Option<u8>,
    pub community_filename_format: Option<u8>,
    pub community_mod_local_name_style: Option<u8>,
    pub community_ignore_quilt: Option<bool>,

    // ===== CurseForge（加密存储，不进 AppConfig，内部分流到 secure_storage）=====
    pub curseforge_enabled: Option<bool>,
    pub curseforge_api_key: Option<String>,
}

/// 配置快照：返回所有配置字段的当前值
///
/// 用于前端一次性读取全部配置，取代此前分散的 14 个 get_* 命令。
/// CurseForge 的 api_key 从 secure_storage 缓存读取（已解密），
/// 若首次未解密则返回空字符串（懒加载，避免触发杀软误报）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSnapshot {
    // 代理
    pub proxy_mode: String,
    pub proxy_type: String,
    pub proxy_url: String,
    // 下载
    pub mirror_url: Option<String>,
    pub download_source: String,
    pub meta_source: String,
    pub max_download_speed: u64,
    pub max_download_threads: u32,
    pub chunk_count: u32,
    // 内存
    pub memory_mode: String,
    pub min_memory: u32,
    pub max_memory: u32,
    // 启动器
    pub game_dir: String,
    pub isolation_mode: u32,
    pub log_level: u32,
    pub selected_version: Option<String>,
    // 社区资源（INI 明文）
    pub community_source: u8,
    pub community_filename_format: u8,
    pub community_mod_local_name_style: u8,
    pub community_ignore_quilt: bool,
    // CurseForge（从 secure_storage 缓存读，已解密）
    pub curseforge_enabled: bool,
    pub curseforge_api_key: String,
}

/// 配置项：扁平化 key-value 对
///
/// `get_config` 返回 `Vec<ConfigEntry>`，`apply_config` 接收同样的 `Vec<ConfigEntry>`，
/// 前后端格式完全对称。每项形如 `{ "key": "proxyMode", "value": "none" }`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: serde_json::Value,
}

/// 读取配置（扁平化数组返回，支持按 keys 过滤）
///
/// - 不传 `keys` 或传空数组：返回全部字段
/// - 传 `keys`：仅返回指定字段（camelCase 名称），未匹配的字段被忽略
///
/// 返回格式：`[{ "key": "proxyMode", "value": "none" }, ...]`
///
/// CurseForge 的 `api_key` 使用异步读取（`get_config_async`），
/// 首次调用会触发 SDK DES 解密并缓存，避免此前懒加载导致首次返回空字符串的 bug。
#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
    keys: Option<Vec<String>>,
) -> Result<Vec<ConfigEntry>, String> {
    // 异步触发解密（修复懒加载导致 apiKey 不返回的 bug）
    let (cf_enabled, cf_api_key) =
        crate::minecraft::community::secure_storage::get_config_async().await;

    // 构建全量快照（持有 config 锁的最短时间）
    let snapshot = {
        let config = state.config.lock().await;
        ConfigSnapshot {
            proxy_mode: config.proxy_mode.clone(),
            proxy_type: config.proxy_type.clone(),
            proxy_url: config.proxy_url.clone(),
            mirror_url: config.mirror_url.clone(),
            download_source: config.download_source.clone(),
            meta_source: config.meta_source.clone(),
            max_download_speed: config.max_download_speed,
            max_download_threads: config.max_download_threads,
            chunk_count: config.chunk_count,
            memory_mode: config.memory_mode.clone(),
            min_memory: config.min_memory,
            max_memory: config.max_memory,
            game_dir: config.game_dir.clone(),
            isolation_mode: config.isolation_mode,
            log_level: config.log_level,
            selected_version: config.selected_version.clone(),
            community_source: config.community_source,
            community_filename_format: config.community_filename_format,
            community_mod_local_name_style: config.community_mod_local_name_style,
            community_ignore_quilt: config.community_ignore_quilt,
            curseforge_enabled: cf_enabled,
            curseforge_api_key: cf_api_key.unwrap_or_default(),
        }
    };

    // 序列化为 JSON 对象，再转为扁平数组
    let value = serde_json::to_value(&snapshot)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
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
fn validate_mirror_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("镜像 URL 必须以 http:// 或 https:// 开头".to_string());
    }
    let after_scheme = if let Some(rest) = url.strip_prefix("https://") {
        rest
    } else {
        url.strip_prefix("http://").unwrap_or(url)
    };
    let host_part = after_scheme.split('@').last().unwrap_or(after_scheme);
    let host_end = host_part
        .find(|c| c == '/' || c == ':' || c == '?' || c == '#')
        .unwrap_or(host_part.len());
    let host = &host_part[..host_end];
    let host = host.trim_start_matches('[').trim_end_matches(']');

    if host.is_empty() {
        return Err("镜像 URL 主机不能为空".to_string());
    }
    if host == "localhost" || host == "127.0.0.1" || host == "::1" {
        return Err("镜像 URL 不能指向环回地址".to_string());
    }
    if host.starts_with("169.254.") {
        return Err("镜像 URL 不能指向链路本地地址".to_string());
    }
    if host.starts_with("10.") || host.starts_with("192.168.") {
        return Err("镜像 URL 不能指向私有网络地址".to_string());
    }
    if host.starts_with("172.") {
        if let Some(second) = host.split('.').nth(1) {
            if let Ok(n) = second.parse::<u32>() {
                if (16..=31).contains(&n) {
                    return Err("镜像 URL 不能指向私有网络地址".to_string());
                }
            }
        }
    }
    Ok(())
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
///
/// 处理顺序：
/// 1. 校验（mirror_url SSRF、download_source/meta_source 枚举）
/// 2. 加密字段分流（CurseForge 走 secure_storage，不进 AppConfig）
/// 3. 普通字段统一更新（update_config 闭包内赋值 + 联动 + 副作用）
#[tauri::command]
pub async fn apply_config(
    state: State<'_, AppState>,
    entries: Vec<ConfigEntry>,
) -> Result<(), String> {
    // 将扁平数组转为 Map，再反序列化为 ConfigPatch
    let mut map = serde_json::Map::new();
    for entry in entries {
        map.insert(entry.key, entry.value);
    }
    let patch: ConfigPatch = serde_json::from_value(serde_json::Value::Object(map))
        .map_err(|e| format!("解析配置补丁失败: {}", e))?;
    apply_config_inner(state, patch).await
}

/// 配置更新核心逻辑（从扁平参数构建 ConfigPatch 后调用）
async fn apply_config_inner(state: State<'_, AppState>, patch: ConfigPatch) -> Result<(), String> {
    // ===== 1. 校验阶段 =====
    if let Some(Some(ref url)) = patch.mirror_url {
        validate_mirror_url(url)?;
    }
    if let Some(ref s) = patch.download_source {
        if !matches!(s.as_str(), "official" | "mirror" | "smart") {
            return Err(format!("无效的 download_source: {}", s));
        }
    }
    if let Some(ref s) = patch.meta_source {
        if !matches!(s.as_str(), "official" | "mirror" | "smart") {
            return Err(format!("无效的 meta_source: {}", s));
        }
    }

    // ===== 2. 加密字段分流（CurseForge API Key）=====
    if patch.curseforge_enabled.is_some() || patch.curseforge_api_key.is_some() {
        // 至少一个 CF 字段要更新：取 patch 提供的值，未提供的字段异步读取旧值
        // 使用 get_config_async 确保首次保存时 api_key 已解密（避免误清空）
        let (old_enabled, old_key) =
            crate::minecraft::community::secure_storage::get_config_async().await;
        let enabled = patch.curseforge_enabled.unwrap_or(old_enabled);
        let api_key = match &patch.curseforge_api_key {
            Some(k) => k.clone(),
            None => old_key.unwrap_or_default(),
        };
        log_info!("[Config] CurseForge 配置更新: enabled={}", enabled);
        crate::minecraft::community::secure_storage::save(state.sdk.clone(), enabled, &api_key)
            .await?;
    }

    // ===== 3. 普通字段统一更新 =====
    // 收集需要触发副作用的标志（闭包外执行，避免跨 await 持有锁）
    let mut need_log_level_apply = false;
    let mut log_level_value: u32 = 0;

    super::update_config(&state, |config| {
        // 代理
        if let Some(ref mode) = patch.proxy_mode {
            log_info!("[Config] proxy_mode = {}", mode);
            config.proxy_mode = mode.clone();
        }
        if let Some(ref t) = patch.proxy_type {
            log_info!("[Config] proxy_type = {}", t);
            config.proxy_type = t.clone();
        }
        if let Some(ref url) = patch.proxy_url {
            log_info!("[Config] proxy_url = {}", url);
            config.proxy_url = url.clone();
        }

        // 下载
        if let Some(ref source) = patch.download_source {
            log_info!("[Config] download_source = {}", source);
            let bmclapi = crate::minecraft::sources::BMCLAPI_BASE;
            match source.as_str() {
                "mirror" => {
                    config.mirror_url_download = Some(bmclapi.to_string());
                    config.mirror_url = Some(bmclapi.to_string());
                    config.mirror_mode = 0;
                }
                "official" => {
                    config.mirror_url_download = None;
                    config.mirror_url = None;
                    config.mirror_mode = 0;
                }
                "smart" => {
                    config.mirror_url_download = None;
                    config.mirror_url = None;
                    config.mirror_mode = 1;
                }
                _ => {}
            }
            config.download_source = source.clone();
        }
        if let Some(ref source) = patch.meta_source {
            log_info!("[Config] meta_source = {}", source);
            let bmclapi = crate::minecraft::sources::BMCLAPI_BASE;
            match source.as_str() {
                "mirror" => config.mirror_url_meta = Some(bmclapi.to_string()),
                "official" | "smart" => config.mirror_url_meta = None,
                _ => {}
            }
            config.meta_source = source.clone();
        }
        if let Some(speed) = patch.max_download_speed {
            log_info!("[Config] max_download_speed = {}", speed);
            config.max_download_speed = speed;
        }
        if let Some(threads) = patch.max_download_threads {
            log_info!("[Config] max_download_threads = {}", threads);
            config.max_download_threads = threads;
        }
        if let Some(count) = patch.chunk_count {
            log_info!("[Config] chunk_count = {}", count);
            config.chunk_count = count;
        }
        if let Some(ref url_opt) = patch.mirror_url {
            log_info!("[Config] mirror_url = {:?}", url_opt);
            config.mirror_url = url_opt.clone();
        }

        // 内存
        if let Some(ref mode) = patch.memory_mode {
            log_info!("[Config] memory_mode = {}", mode);
            config.memory_mode = mode.clone();
            if mode == "auto" {
                // 切换到自动模式时，清零内存值（保留原有联动）
                config.min_memory = 0;
                config.max_memory = 0;
            }
        }
        if let Some(mem) = patch.min_memory {
            log_info!("[Config] min_memory = {}", mem);
            config.min_memory = mem;
        }
        if let Some(mem) = patch.max_memory {
            log_info!("[Config] max_memory = {}", mem);
            config.max_memory = mem;
        }

        // 启动器
        if let Some(ref dir) = patch.game_dir {
            log_info!("[Config] game_dir = {}", dir);
            config.game_dir = dir.clone();
        }
        if let Some(mode) = patch.isolation_mode {
            log_info!("[Config] isolation_mode = {}", mode);
            config.isolation_mode = mode;
        }
        if let Some(level) = patch.log_level {
            log_info!("[Config] log_level = {}", level);
            config.log_level = level;
            need_log_level_apply = true;
            log_level_value = level;
        }
        if let Some(ref version) = patch.selected_version {
            log_info!("[Config] selected_version = {:?}", version);
            config.selected_version = version.clone();
        }

        // 社区资源
        if let Some(source) = patch.community_source {
            log_info!("[Config] community_source = {}", source);
            config.community_source = source;
        }
        if let Some(fmt) = patch.community_filename_format {
            log_info!("[Config] community_filename_format = {}", fmt);
            config.community_filename_format = fmt;
        }
        if let Some(style) = patch.community_mod_local_name_style {
            log_info!("[Config] community_mod_local_name_style = {}", style);
            config.community_mod_local_name_style = style;
        }
        if let Some(ignore) = patch.community_ignore_quilt {
            log_info!("[Config] community_ignore_quilt = {}", ignore);
            config.community_ignore_quilt = ignore;
        }
    })
    .await?;

    // ===== 副作用阶段（闭包外执行）=====
    // log_level 变更需要立即生效（参考此前 set_config_value 的特例补丁）
    if need_log_level_apply {
        let log_level = match log_level_value {
            0 | 1 => crate::logger::LogLevel::Error,
            2 => crate::logger::LogLevel::Warn,
            3 => crate::logger::LogLevel::Info,
            4 => crate::logger::LogLevel::Debug,
            5 => crate::logger::LogLevel::Trace,
            _ => crate::logger::LogLevel::Info,
        };
        crate::logger::set_level(log_level);
    }

    Ok(())
}
