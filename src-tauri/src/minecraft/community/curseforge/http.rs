//! CurseForge HTTP 请求层
//!
//! 包含：
//! - 配置读取（get_cf_config）：根据 source 策略选择官方/镜像源
//! - cf_get / cf_post：通用 GET/POST 请求封装
//!   - source=1 时官方失败自动回退镜像
//!   - source=0 强制镜像，source=2 强制官方

use std::time::Instant;

use super::super::common::fmt_elapsed;

/// CurseForge 官方 API 基地址
pub(crate) const CF_OFFICIAL_BASE: &str = "https://api.curseforge.com/v1";

/// CurseForge 镜像 API 基地址（MCIM 镜像源）
pub(crate) const CF_MIRROR_BASE: &str = "https://mod.mcimirror.top/curseforge/v1";

/// 读取 CurseForge 配置，返回 (base_url, api_key, source_pref)
///
/// source 策略（参考 PCL2 ToolDownloadMod）：
/// - 0=尽量镜像：强制走镜像（即使配置了 API Key 也不用）
/// - 1=缓慢时换镜像：优先官方（若有 API Key），失败后由 cf_get 回退镜像
/// - 2=尽量官方：优先官方（若有 API Key），否则镜像
///
/// 异步：首次调用会触发 SDK DES 解密 api_key 并缓存，后续直接读缓存
pub(crate) async fn get_cf_config() -> (String, Option<String>, u8) {
    let (enabled, api_key) = super::super::secure_storage::get_config_async().await;
    let source = super::super::get_source_pref();

    // 0=尽量镜像：强制走镜像（忽略 API Key 配置）
    if source == 0 {
        crate::log_debug!("[Community] CF 走镜像源（source=0 强制镜像）");
        return (CF_MIRROR_BASE.to_string(), None, source);
    }

    // 1=缓慢时换镜像 / 2=尽量官方：有 API Key 则走官方
    if enabled {
        if let Some(ref key) = api_key {
            if !key.is_empty() {
                crate::log_debug!("[Community] CF 走官方 API（source={}, API Key 已配置）", source);
                return (CF_OFFICIAL_BASE.to_string(), api_key, source);
            }
        }
        crate::log_warn!("[Community] CF 已启用 API Key 但未配置 key，回退到镜像");
    }
    (CF_MIRROR_BASE.to_string(), None, source)
}

/// 发送 GET 请求并附加 API Key header（如果配置了）
///
/// source=1（缓慢时换镜像）时，若官方请求失败/超时，自动回退到镜像源重试
pub(crate) async fn cf_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let (base, api_key, source) = get_cf_config().await;
    let url = format!("{}{}", base, path);

    // 官方请求超时（参考 PCL2 DlModRequest：CF 官方默认 10s）
    const CF_OFFICIAL_TIMEOUT_SECS: u64 = 10;

    let is_official = base == CF_OFFICIAL_BASE;
    let start = Instant::now();

    let result = if is_official {
        // 官方请求加超时，超时视为"缓慢"，触发回退
        let req = build_cf_request(&url, api_key.as_deref());
        match tokio::time::timeout(
            std::time::Duration::from_secs(CF_OFFICIAL_TIMEOUT_SECS),
            req.send(),
        )
        .await
        {
            Ok(Ok(resp)) => resp.json::<T>().await.map_err(|e| {
                crate::log_warn!("[Community] CF 响应解析失败: {} ({})", url, e);
                format!("CurseForge 响应解析失败: {}", e)
            }),
            Ok(Err(e)) => {
                crate::log_warn!("[Community] CF 请求失败: {} ({:?})", url, e);
                Err(format!("CurseForge 请求失败: {}", e))
            }
            Err(_) => {
                crate::log_warn!(
                    "[Community] CF 官方请求超时（{}s），{}",
                    CF_OFFICIAL_TIMEOUT_SECS,
                    if source == 1 { "回退镜像" } else { "报错" }
                );
                Err(format!("CurseForge 官方请求超时（{}s）", CF_OFFICIAL_TIMEOUT_SECS))
            }
        }
    } else {
        // 镜像请求不加超时（镜像本身可能较慢，让其自然完成）
        let req = build_cf_request(&url, api_key.as_deref());
        req.send().await
            .map_err(|e| {
                crate::log_warn!("[Community] CF 请求失败: {} ({:?})", url, e);
                format!("CurseForge 请求失败: {}", e)
            })?
            .json::<T>()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] CF 响应解析失败: {} ({})", url, e);
                format!("CurseForge 响应解析失败: {}", e)
            })
    };

    match result {
        Ok(value) => {
            crate::log_info!("[Community] CF 请求成功: {} ({})", url, fmt_elapsed(start));
            Ok(value)
        }
        Err(e) => {
            // 策略 1（缓慢时换镜像）：官方失败时回退镜像
            if source == 1 && is_official {
                crate::log_warn!("[Community] CF 官方请求失败，回退镜像: {}", e);
                let mirror_url = format!("{}{}", CF_MIRROR_BASE, path);
                let req = build_cf_request(&mirror_url, None);
                let resp = req.send().await
                    .map_err(|e| {
                        crate::log_warn!("[Community] CF 镜像请求失败: {} ({:?})", mirror_url, e);
                        format!("CurseForge 镜像请求失败: {}", e)
                    })?;
                let value: T = resp.json().await
                    .map_err(|e| {
                        crate::log_warn!("[Community] CF 镜像响应解析失败: {} ({})", mirror_url, e);
                        format!("CurseForge 镜像响应解析失败: {}", e)
                    })?;
                crate::log_info!("[Community] CF 镜像请求成功: {} ({})", mirror_url, fmt_elapsed(start));
                return Ok(value);
            }
            Err(e)
        }
    }
}

/// 构造 CF GET 请求（附加 API Key header）
fn build_cf_request(url: &str, api_key: Option<&str>) -> reqwest::RequestBuilder {
    let mut req = crate::http::get_client().get(url);
    if let Some(key) = api_key {
        req = req.header("x-api-key", key);
        req = req.header("Accept", "application/json");
    }
    req
}

/// 构造 CF POST 请求（附加 API Key header + JSON body）
fn build_cf_post_request(
    url: &str,
    api_key: Option<&str>,
    body: String,
) -> reqwest::RequestBuilder {
    let mut req = crate::http::get_client()
        .post(url)
        .header("Content-Type", "application/json")
        .body(body);
    if let Some(key) = api_key {
        req = req.header("x-api-key", key);
        req = req.header("Accept", "application/json");
    }
    req
}

/// 发送 POST 请求（参考 PCL2 DlModRequest 对 CF POST 接口的处理）
///
/// 与 `cf_get` 一致的 source 策略：
/// - source=1 时官方失败回退镜像
/// - source=0 强制镜像
/// - source=2 强制官方
pub(crate) async fn cf_post<T: serde::de::DeserializeOwned>(
    path: &str,
    body: String,
) -> Result<T, String> {
    let (base, api_key, source) = get_cf_config().await;
    let url = format!("{}{}", base, path);

    const CF_OFFICIAL_TIMEOUT_SECS: u64 = 15;
    let is_official = base == CF_OFFICIAL_BASE;
    let start = Instant::now();

    let result = if is_official {
        let req = build_cf_post_request(&url, api_key.as_deref(), body.clone());
        match tokio::time::timeout(
            std::time::Duration::from_secs(CF_OFFICIAL_TIMEOUT_SECS),
            req.send(),
        )
        .await
        {
            Ok(Ok(resp)) => resp.json::<T>().await.map_err(|e| {
                crate::log_warn!("[Community] CF POST 响应解析失败: {} ({})", url, e);
                format!("CurseForge 响应解析失败: {}", e)
            }),
            Ok(Err(e)) => {
                crate::log_warn!("[Community] CF POST 请求失败: {} ({:?})", url, e);
                Err(format!("CurseForge 请求失败: {}", e))
            }
            Err(_) => {
                crate::log_warn!(
                    "[Community] CF POST 官方请求超时（{}s），{}",
                    CF_OFFICIAL_TIMEOUT_SECS,
                    if source == 1 { "回退镜像" } else { "报错" }
                );
                Err(format!(
                    "CurseForge 官方请求超时（{}s）",
                    CF_OFFICIAL_TIMEOUT_SECS
                ))
            }
        }
    } else {
        let req = build_cf_post_request(&url, api_key.as_deref(), body.clone());
        req.send()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] CF POST 请求失败: {} ({:?})", url, e);
                format!("CurseForge 请求失败: {}", e)
            })?
            .json::<T>()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] CF POST 响应解析失败: {} ({})", url, e);
                format!("CurseForge 响应解析失败: {}", e)
            })
    };

    match result {
        Ok(value) => {
            crate::log_info!("[Community] CF POST 请求成功: {} ({})", url, fmt_elapsed(start));
            Ok(value)
        }
        Err(e) => {
            // 策略 1：官方失败时回退镜像
            if source == 1 && is_official {
                crate::log_warn!("[Community] CF POST 官方请求失败，回退镜像: {}", e);
                let mirror_url = format!("{}{}", CF_MIRROR_BASE, path);
                let req = build_cf_post_request(&mirror_url, None, body);
                let resp = req.send().await.map_err(|e| {
                    crate::log_warn!(
                        "[Community] CF POST 镜像请求失败: {} ({:?})",
                        mirror_url,
                        e
                    );
                    format!("CurseForge 镜像请求失败: {}", e)
                })?;
                let value: T = resp.json().await.map_err(|e| {
                    crate::log_warn!(
                        "[Community] CF POST 镜像响应解析失败: {} ({})",
                        mirror_url,
                        e
                    );
                    format!("CurseForge 镜像响应解析失败: {}", e)
                })?;
                crate::log_info!(
                    "[Community] CF POST 镜像请求成功: {} ({})",
                    mirror_url,
                    fmt_elapsed(start)
                );
                return Ok(value);
            }
            Err(e)
        }
    }
}
