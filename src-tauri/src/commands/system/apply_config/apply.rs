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

/// 代理域：proxy_mode / proxy_type / proxy_url
fn apply_proxy(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
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
}

/// 下载域：download_source / meta_source / max_download_speed / max_download_threads / chunk_count / mirror_url
fn apply_download(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
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
}

/// 内存域：memory_mode（auto 联动清零）/ min_memory / max_memory
fn apply_memory(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
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

/// 社区资源域：community_source / community_filename_format / community_mod_local_name_style / community_ignore_quilt
fn apply_community(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
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
}

/// 启动高级选项域：launch_disable_jlw / launch_disable_lua / launch_use_dedicated_gpu
fn apply_launch_advanced(config: &mut crate::state::AppConfig, patch: &ConfigPatch) {
    if let Some(v) = patch.launch_disable_jlw {
        log_info!("[Config] launch_disable_jlw = {}", v);
        config.launch_disable_jlw = v;
    }
    if let Some(v) = patch.launch_disable_lua {
        log_info!("[Config] launch_disable_lua = {}", v);
        config.launch_disable_lua = v;
    }
    if let Some(v) = patch.launch_use_dedicated_gpu {
        log_info!("[Config] launch_use_dedicated_gpu = {}", v);
        config.launch_use_dedicated_gpu = v;
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
