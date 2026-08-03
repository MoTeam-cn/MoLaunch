//! 配置更新编排逻辑实现（apply_config_inner + apply_java，原聚合入口 mod.rs 中的实现）
//!
//! `apply_config_inner` 三段式分流：1.校验（validate：mirror_url SSRF、download_source/
//! meta_source 枚举）→ 2.加密字段分流（secure：CurseForge/开发者模式/IgnoreTls，不进
//! AppConfig）→ 3.普通字段统一更新（`update_config` 闭包内按域调用 `fields` 子函数 + 副作用）。

use super::super::secure;
use super::super::types::ConfigPatch;
use super::super::validate;
use super::fields;
use crate::log_info;
use crate::state::AppState;

/// 配置更新核心逻辑（从扁平参数构建 `ConfigPatch` 后调用）
pub(crate) async fn apply_config_inner(state: &AppState, patch: ConfigPatch) -> Result<(), String> {
    validate::validate_patch(&patch)?;

    // 2. 加密字段分流（CurseForge API Key）
    secure::apply_curseforge(state, &patch).await?;

    // 2b. 开发者模式分流（注册表，不进 AppConfig）
    secure::apply_developer_mode(&patch)?;

    // 2c. IgnoreTls 分流（注册表，仅开发者模式可开启）
    secure::apply_ignore_tls(&patch)?;
    // 2d. Java path 分流（INI [Java] path 独立存储，不进 AppConfig）
    apply_java(&patch)?;
    // log_level 变更需闭包外立即生效，用 Option 收集待应用的值（避免跨 await 持有锁）
    let mut log_level_pending: Option<u32> = None;
    // 代理变更需闭包外重建 HTTP 客户端（同 log_level 模式，避免跨 await 持有锁）
    // 四元组：(mode, kind, url, ip_version)
    let mut proxy_pending: Option<(String, String, String, String)> = None;
    // TLS 变更需闭包外重建 HTTP 客户端（trust_mode + ignore_tls）
    let mut tls_pending: Option<bool> = None;

    super::super::super::update_config(state, |config| {
        fields::apply_proxy(config, &patch, &mut proxy_pending);
        fields::apply_download(config, &patch);
        fields::apply_memory(config, &patch);
        fields::apply_launcher(config, &patch, &mut log_level_pending);
        fields::apply_community(config, &patch);
        fields::apply_launch_advanced(config, &patch);
        fields::apply_external_download(config, &patch);
        fields::apply_online(config, &patch);
        fields::apply_tls(config, &patch, &mut tls_pending);
    })
    .await?;

    // 副作用阶段（闭包外执行）
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

    // 代理或 TLS 变更需重建 HTTP 客户端（热更新，无需重启应用）
    if proxy_pending.is_some() || tls_pending.is_some() {
        // 锁定读取最新配置（包含刚更新的 proxy + tls 字段）
        let config = state.config.lock().await;
        let (mode, kind, url, ip_version, trust_mode) = (
            config.proxy.mode.clone(),
            config.proxy.kind.clone(),
            config.proxy.url.clone(),
            config.proxy.ip_version.clone(),
            config.tls.trust_mode.clone(),
        );
        drop(config);
        // ignore_tls 走注册表，开发者模式关闭时自动为 false
        let ignore_tls = crate::commands::system::developer::is_ignore_tls();
        crate::http::init_client(&mode, &kind, &url, &ip_version, &trust_mode, ignore_tls);
        log_info!(
            "[Config] HTTP client rebuilt (proxy: {}, ip_version: {}, trust_mode: {}, ignore_tls: {})",
            mode,
            ip_version,
            trust_mode,
            ignore_tls
        );
    }

    Ok(())
}

/// Java 路径域：写 INI [Java] path（不进 AppConfig，保留独立存储设计）
///
/// 与 `secure::apply_*` 同属"非 AppConfig 分流"，不进 AppConfig 内存态，故不在 `update_config` 闭包内。
fn apply_java(patch: &ConfigPatch) -> Result<(), String> {
    if let Some(ref path) = patch.java_path {
        let storage = crate::storage::Storage::instance();
        storage
            .set_config("Java", "path", path)
            .map_err(|e| format!("写入 Java path 失败: {}", e))?;
        log_info!("[Config] java_path = {}", path);
    }
    Ok(())
}
