//! 配置更新核心逻辑
//!
//! `apply_config_inner` 三段式分流：
//! 1. 校验（validate 模块：mirror_url SSRF、download_source / meta_source 枚举）
//! 2. 加密字段分流（secure 模块：CurseForge / 开发者模式，不进 AppConfig）
//! 3. 普通字段统一更新（`update_config` 闭包内按域调用 6 个子函数 + 副作用）
//!
//! 6 个域子函数：代理 / 下载 / 内存 / 启动器 / 社区 / 启动高级。
//! CurseForge 与开发者模式不在闭包内（分别走 secure_storage 与注册表）。

use super::secure;
use super::types::ConfigPatch;
use super::validate;
use crate::log_info;
use crate::state::AppState;
use tauri::State;

/// 配置更新核心逻辑（从扁平参数构建 `ConfigPatch` 后调用）
pub(crate) async fn apply_config_inner(
    state: State<'_, AppState>,
    patch: ConfigPatch,
) -> Result<(), String> {
    // ===== 1. 校验阶段 =====
    validate::validate_patch(&patch)?;

    // ===== 2. 加密字段分流（CurseForge API Key）=====
    secure::apply_curseforge(&state, &patch).await?;

    // ===== 2b. 开发者模式分流（注册表，不进 AppConfig）=====
    secure::apply_developer_mode(&patch)?;

    // ===== 3. 普通字段统一更新 =====
    // log_level 变更需闭包外立即生效，用 Option 收集待应用的值（避免跨 await 持有锁）
    let mut log_level_pending: Option<u32> = None;

    super::super::update_config(&state, |config| {
        apply_proxy(config, &patch);
        apply_download(config, &patch);
        apply_memory(config, &patch);
        apply_launcher(config, &patch, &mut log_level_pending);
        apply_community(config, &patch);
        apply_launch_advanced(config, &patch);
        apply_external_download(config, &patch);
    })
    .await?;

    // ===== 副作用阶段（闭包外执行）=====
    // log_level 变更需要立即生效（参考此前 set_config_value 的特例补丁）
    if let Some(level) = log_level_pending {
        let log_level = match level {
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

// ============================================================
// 域子函数（均在 update_config 闭包内调用，操作 &mut AppConfig）
// ============================================================

/// 代理域：proxy.mode / proxy.kind / proxy.url
fn apply_proxy(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
    if let Some(ref mode) = patch.proxy.mode {
        log_info!("[Config] proxy_mode = {}", mode);
        config.proxy.mode = mode.clone();
    }
    if let Some(ref t) = patch.proxy.kind {
        log_info!("[Config] proxy_type = {}", t);
        config.proxy.kind = t.clone();
    }
    if let Some(ref url) = patch.proxy.url {
        log_info!("[Config] proxy_url = {}", url);
        config.proxy.url = url.clone();
    }
}

/// 下载域：download.source / meta_source / max_speed / max_threads / chunk_count / mirror_url
fn apply_download(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
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
}

/// 内存域：memory.mode（auto 联动清零）/ memory.min / memory.max
fn apply_memory(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
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
fn apply_launcher(
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
}

/// 社区资源域：community.source / filename_format / mod_local_name_style / ignore_quilt
fn apply_community(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
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
fn apply_launch_advanced(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
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
fn apply_external_download(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
    if let Some(ref dir_opt) = patch.external_download_dir {
        log_info!("[Config] external_download_dir = {:?}", dir_opt);
        config.external_download_dir = dir_opt.clone();
    }
}
