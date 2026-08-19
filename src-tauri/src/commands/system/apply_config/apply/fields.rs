//! 配置更新 9 个域子函数（在 `update_config` 闭包内被 `apply_config_inner` 调用）

use super::super::secure;
use super::super::types::ConfigPatch;
use crate::log_info;
use crate::log_warn;

/// 代理域：proxy.mode / proxy.kind / proxy.url / proxy.ip_version
///
/// 任一字段变更即收集完整四元组到 `proxy_pending`，供闭包外重建 HTTP 客户端。
pub(super) fn apply_proxy(
    config: &mut crate::state::AppConfig,
    patch: &ConfigPatch,
    proxy_pending: &mut Option<(String, String, String, String)>,
) {
    let mut changed = false;
    if let Some(ref mode) = patch.proxy.mode {
        log_info!("[Config] proxy_mode = {}", mode);
        config.proxy.mode = mode.clone();
        changed = true;
    }
    if let Some(ref t) = patch.proxy.kind {
        log_info!("[Config] proxy_type = {}", t);
        config.proxy.kind = t.clone();
        changed = true;
    }
    if let Some(ref url) = patch.proxy.url {
        log_info!("[Config] proxy_url = {}", url);
        config.proxy.url = url.clone();
        changed = true;
    }
    if let Some(ref v) = patch.proxy.ip_version {
        log_info!("[Config] ip_version = {}", v);
        config.proxy.ip_version = v.clone();
        changed = true;
    }
    if changed {
        *proxy_pending = Some((
            config.proxy.mode.clone(),
            config.proxy.kind.clone(),
            config.proxy.url.clone(),
            config.proxy.ip_version.clone(),
        ));
    }
}

/// 下载域：download.source / meta_source / max_speed / max_threads / chunk_count / mirror_url
pub(super) fn apply_download(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
    if let Some(ref source) = patch.download.source {
        log_info!("[Config] download_source = {}", source);
        let bmclapi = crate::minecraft::sources::BMCLAPI_BASE;
        match source.as_str() {
            "mirror" => {
                config.download.mirror_url_download = Some(bmclapi.to_string());
                config.download.mirror_url = Some(bmclapi.to_string());
                config.download.mirror_mode = 0;
            }
            "official" => {
                config.download.mirror_url_download = None;
                config.download.mirror_url = None;
                config.download.mirror_mode = 0;
            }
            "smart" => {
                config.download.mirror_url_download = None;
                config.download.mirror_url = None;
                config.download.mirror_mode = 1;
            }
            _ => {}
        }
        config.download.source = source.clone();
    }
    if let Some(ref source) = patch.download.meta_source {
        log_info!("[Config] meta_source = {}", source);
        let bmclapi = crate::minecraft::sources::BMCLAPI_BASE;
        match source.as_str() {
            "mirror" => config.download.mirror_url_meta = Some(bmclapi.to_string()),
            "official" | "smart" => config.download.mirror_url_meta = None,
            _ => {}
        }
        config.download.meta_source = source.clone();
    }
    if let Some(speed) = patch.download.max_speed {
        log_info!("[Config] max_download_speed = {}", speed);
        config.download.max_speed = speed;
    }
    if let Some(threads) = patch.download.max_threads {
        log_info!("[Config] max_download_threads = {}", threads);
        config.download.max_threads = threads;
    }
    if let Some(count) = patch.download.chunk_count {
        log_info!("[Config] chunk_count = {}", count);
        config.download.chunk_count = count;
    }
    if let Some(ref url_opt) = patch.download.mirror_url {
        log_info!("[Config] mirror_url = {:?}", url_opt);
        config.download.mirror_url = url_opt.clone();
    }
    if let Some(v) = patch.download.modrinth_cdn_raw_enabled {
        log_info!("[Config] modrinth_cdn_raw_enabled = {}", v);
        config.download.modrinth_cdn_raw_enabled = v;
    }
}

/// 内存域：memory.mode（auto 联动清零）/ memory.min / memory.max
pub(super) fn apply_memory(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
    if let Some(ref mode) = patch.memory.mode {
        log_info!("[Config] memory_mode = {}", mode);
        config.memory.mode = mode.clone();
        if mode == "auto" {
            // 切换到自动模式时，清零内存值（保留原有联动）
            config.memory.min = 0;
            config.memory.max = 0;
        }
    }
    if let Some(mem) = patch.memory.min {
        log_info!("[Config] min_memory = {}", mem);
        config.memory.min = mem;
    }
    if let Some(mem) = patch.memory.max {
        log_info!("[Config] max_memory = {}", mem);
        config.memory.max = mem;
    }
}

/// 启动器域：game_dir / isolation_mode / log_level（收集待应用值）/ selected_version / game_language / primary_color
pub(super) fn apply_launcher(
    config: &mut crate::state::AppConfig,
    patch: &ConfigPatch,
    log_level_pending: &mut Option<u32>,
) {
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
        *log_level_pending = Some(level);
    }
    if let Some(ref version) = patch.selected_version {
        log_info!("[Config] selected_version = {:?}", version);
        config.selected_version = version.clone();
    }
    if let Some(ref lang) = patch.game_language {
        log_info!("[Config] game_language = {}", lang);
        config.game_language = lang.clone();
    }
    if let Some(ref color) = patch.primary_color {
        log_info!("[Config] primary_color = {}", color);
        config.primary_color = color.clone();
    }
    if let Some(ref behavior) = patch.close_behavior {
        log_info!("[Config] close_behavior = {}", behavior);
        config.close_behavior = behavior.clone();
    }
}

/// 实验性功能域：experimental_enabled
///
/// 首次开启时惰性初始化 SQLite 聊天存储（建库建表，幂等）；
/// 关闭时仅隐藏入口，不删除已有数据，避免误操作丢失聊天记录。
pub(super) fn apply_experimental(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
    if let Some(enabled) = patch.experimental_enabled {
        log_info!("[Config] experimental_enabled = {}", enabled);
        if enabled && !config.experimental_enabled {
            if let Err(e) = crate::commands::experimental::db::ensure_initialized() {
                log_warn!("[Experimental] 初始化聊天存储失败: {}", e);
            }
        }
        config.experimental_enabled = enabled;
    }
}

/// 社区资源域：community.source / filename_format / mod_local_name_style / ignore_quilt
pub(super) fn apply_community(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
    if let Some(source) = patch.community.source {
        log_info!("[Config] community_source = {}", source);
        config.community.source = source;
    }
    if let Some(fmt) = patch.community.filename_format {
        log_info!("[Config] community_filename_format = {}", fmt);
        config.community.filename_format = fmt;
    }
    if let Some(style) = patch.community.mod_local_name_style {
        log_info!("[Config] community_mod_local_name_style = {}", style);
        config.community.mod_local_name_style = style;
    }
    if let Some(ignore) = patch.community.ignore_quilt {
        log_info!("[Config] community_ignore_quilt = {}", ignore);
        config.community.ignore_quilt = ignore;
    }
}

/// 启动高级选项域：launch_advanced.disable_jlw / disable_lua / use_dedicated_gpu
pub(super) fn apply_launch_advanced(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
    if let Some(v) = patch.launch_advanced.disable_jlw {
        log_info!("[Config] launch_disable_jlw = {}", v);
        config.launch_advanced.disable_jlw = v;
    }
    if let Some(v) = patch.launch_advanced.disable_lua {
        log_info!("[Config] launch_disable_lua = {}", v);
        config.launch_advanced.disable_lua = v;
    }
    if let Some(v) = patch.launch_advanced.use_dedicated_gpu {
        log_info!("[Config] launch_use_dedicated_gpu = {}", v);
        config.launch_advanced.use_dedicated_gpu = v;
    }
}

/// 外部下载工具域：external_download_dir
///
/// 双层 Option 语义：
/// - `None`：不更新（保持原值）
/// - `Some(None)`：清空（回退到默认 .Molaunch/Download/）
/// - `Some(Some(dir))`：设置为指定目录
pub(super) fn apply_external_download(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
    if let Some(ref dir_opt) = patch.external_download_dir {
        log_info!("[Config] external_download_dir = {:?}", dir_opt);
        config.external_download_dir = dir_opt.clone();
    }
}

/// 联机域：online.api_server_url + online.custom_turn_servers
///
/// - `api_server_url`：空字符串视为不更新（避免前端误传空值清空配置）
///   **开发者模式校验**：仅在开发者模式已开启时允许更新（防止用户误改 + config.ini 直改保护）；
///   关闭状态下静默忽略，不写入 config.ini，不报错（与 ignore_tls 关闭联动语义一致）。
/// - `custom_turn_servers`：`Some` 即更新（含空数组，表示清空所有自定义 TURN）
pub(super) fn apply_online(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
    if let Some(ref url) = patch.online.api_server_url {
        if !url.is_empty() {
            // 开发者模式校验：未开启时静默忽略，保护 config.ini 不被写入
            let (_, dev_mode, _) = secure::read_developer();
            if dev_mode {
                log_info!("[Config] online_api_server_url = {}", url);
                config.online.api_server_url = url.clone();
            } else {
                log_warn!("[Config] 开发者模式未开启，禁止更新 online_api_server_url（保持原值）");
            }
        }
    }
    if let Some(ref servers) = patch.online.custom_turn_servers {
        log_info!(
            "[Config] online_custom_turn_servers count = {}",
            servers.len()
        );
        config.online.custom_turn_servers = servers.clone();
    }
    if let Some(ref peers) = patch.online.easytier_public_peers {
        log_info!(
            "[Config] online_easytier_public_peers count = {}",
            peers.len()
        );
        config.online.easytier_public_peers = peers.clone();
    }
    if let Some(ref proxies) = patch.online.github_proxies {
        log_info!("[Config] online_github_proxies count = {}", proxies.len());
        config.online.github_proxies = proxies.clone();
    }
}

/// TLS 域：tls.trust_mode
///
/// `trust_mode` 变更收集到 `tls_pending`，供闭包外重建 HTTP 客户端。
/// `ignore_tls` 不在此处（走注册表，由 `secure::apply_ignore_tls` 处理）。
pub(super) fn apply_tls(
    config: &mut crate::state::AppConfig,
    patch: &ConfigPatch,
    tls_pending: &mut Option<bool>,
) {
    if let Some(ref mode) = patch.tls_trust_mode {
        log_info!("[Config] tls_trust_mode = {}", mode);
        config.tls.trust_mode = mode.clone();
        *tls_pending = Some(true);
    }
}
