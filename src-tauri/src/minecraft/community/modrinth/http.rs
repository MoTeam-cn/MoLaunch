//! Modrinth HTTP 请求层
//!
//! 包含：
//! - pick_base：根据 source 策略选择官方/镜像源
//! - mr_get / mr_post：通用 GET/POST 请求封装
//!   - source=1 时官方失败自动回退镜像（404 不重试，因为镜像也是 404）
//!   - source=0 强制镜像，source=2 强制官方

use std::time::Instant;

use super::super::common::fmt_elapsed;
use super::types::{MR_MIRROR_BASE, MR_OFFICIAL_BASE};

/// 根据 source 策略选择 Modrinth 基地址
///
/// source 策略：
/// - 0=尽量镜像：强制走镜像
/// - 1=缓慢时换镜像：优先官方，失败后由调用方回退镜像
/// - 2=尽量官方：强制走官方
pub(crate) fn pick_base() -> (&'static str, u8) {
    let source = super::super::get_source_pref();
    match source {
        0 => (MR_MIRROR_BASE, source),
        2 => (MR_OFFICIAL_BASE, source),
        // 1=缓慢时换镜像：优先官方
        _ => (MR_OFFICIAL_BASE, source),
    }
}

/// 发送 GET 请求
///
/// source=1（缓慢时换镜像）时，若官方请求失败/超时，自动回退到镜像源重试
/// source=2 时官方请求不设超时（让其自然完成），失败直接报错
/// source=0 时直接走镜像
pub(crate) async fn mr_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let (base, source) = pick_base();
    let url = format!("{}{}", base, path);

    // 官方请求超时（Modrinth 官方默认 20s）
    const MR_OFFICIAL_TIMEOUT_SECS: u64 = 20;

    let is_official = base == MR_OFFICIAL_BASE;
    let start = Instant::now();

    /// 解析响应：404 单独处理（避免空 body 触发 "EOF while parsing" 警告混淆）
    async fn parse_resp<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
        url: &str,
    ) -> Result<T, String> {
        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            // 404 视为正常 "未找到"，记 INFO 不报警告（用户查不到 mod 不算异常）
            crate::log_info!("[Community] MR 资源不存在 (404): {}", url);
            return Err(format!("Modrinth 资源不存在: {}", url));
        }
        if !status.is_success() {
            let code = status.as_u16();
            crate::log_warn!("[Community] MR 响应非 2xx: {} ({})", url, code);
            return Err(format!("Modrinth 响应异常: HTTP {}", code));
        }
        resp.json::<T>().await.map_err(|e| {
            crate::log_warn!("[Community] MR 响应解析失败: {} ({})", url, e);
            format!("Modrinth 响应解析失败: {}", e)
        })
    }

    let result = if is_official {
        // source=1 时官方请求加超时，超时触发回退；source=2 时不加超时
        if source == 1 {
            match tokio::time::timeout(
                std::time::Duration::from_secs(MR_OFFICIAL_TIMEOUT_SECS),
                crate::http::get_client().get(&url).send(),
            )
            .await
            {
                Ok(Ok(resp)) => parse_resp::<T>(resp, &url).await,
                Ok(Err(e)) => {
                    crate::log_warn!("[Community] MR 请求失败: {} ({:?})", url, e);
                    Err(format!("Modrinth 请求失败: {}", e))
                }
                Err(_) => {
                    crate::log_warn!(
                        "[Community] MR 官方请求超时（{}s），回退镜像",
                        MR_OFFICIAL_TIMEOUT_SECS
                    );
                    Err(format!("Modrinth 官方请求超时（{}s）", MR_OFFICIAL_TIMEOUT_SECS))
                }
            }
        } else {
            // source=2：官方请求不加超时
            let resp = crate::http::get_client()
                .get(&url)
                .send()
                .await
                .map_err(|e| {
                    crate::log_warn!("[Community] MR 请求失败: {} ({:?})", url, e);
                    format!("Modrinth 请求失败: {}", e)
                })?;
            parse_resp::<T>(resp, &url).await
        }
    } else {
        // 镜像请求不加超时
        let resp = crate::http::get_client()
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] MR 请求失败: {} ({:?})", url, e);
                format!("Modrinth 请求失败: {}", e)
            })?;
        parse_resp::<T>(resp, &url).await
    };

    match result {
        Ok(value) => {
            crate::log_info!("[Community] MR 请求成功: {} ({})", url, fmt_elapsed(start));
            Ok(value)
        }
        Err(e) => {
            // 策略 1（缓慢时换镜像）：官方失败时回退镜像
            // 但 404 表示资源真的不存在，重试镜像也无意义（镜像也是 404）
            let is_not_found = e.starts_with("Modrinth 资源不存在");
            if source == 1 && is_official && !is_not_found {
                crate::log_warn!("[Community] MR 官方请求失败，回退镜像: {}", e);
                let mirror_url = format!("{}{}", MR_MIRROR_BASE, path);
                let resp = crate::http::get_client()
                    .get(&mirror_url)
                    .send()
                    .await
                    .map_err(|e| {
                        crate::log_warn!("[Community] MR 镜像请求失败: {} ({:?})", mirror_url, e);
                        format!("Modrinth 镜像请求失败: {}", e)
                    })?;
                let value: T = parse_resp::<T>(resp, &mirror_url).await?;
                crate::log_info!("[Community] MR 镜像请求成功: {} ({})", mirror_url, fmt_elapsed(start));
                return Ok(value);
            }
            Err(e)
        }
    }
}

/// 发送 POST 请求
///
/// 与 `mr_get` 一致的 source 策略和 404 处理。
/// 用于 `/v2/version_files` 批量按 hash 查询本地 mod 对应的工程。
pub(crate) async fn mr_post<T: serde::de::DeserializeOwned>(
    path: &str,
    body: String,
) -> Result<T, String> {
    let (base, source) = pick_base();
    let url = format!("{}{}", base, path);

    const MR_OFFICIAL_TIMEOUT_SECS: u64 = 20;
    let is_official = base == MR_OFFICIAL_BASE;
    let start = Instant::now();

    /// 解析 POST 响应（复用 mr_get 的 404 优雅处理逻辑）
    async fn parse_post_resp<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
        url: &str,
    ) -> Result<T, String> {
        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            crate::log_info!("[Community] MR POST 资源不存在 (404): {}", url);
            return Err(format!("Modrinth 资源不存在: {}", url));
        }
        if !status.is_success() {
            let code = status.as_u16();
            crate::log_warn!("[Community] MR POST 响应非 2xx: {} ({})", url, code);
            return Err(format!("Modrinth 响应异常: HTTP {}", code));
        }
        resp.json::<T>().await.map_err(|e| {
            crate::log_warn!("[Community] MR POST 响应解析失败: {} ({})", url, e);
            format!("Modrinth 响应解析失败: {}", e)
        })
    }

    let result = if is_official && source == 1 {
        // source=1：官方请求加超时
        match tokio::time::timeout(
            std::time::Duration::from_secs(MR_OFFICIAL_TIMEOUT_SECS),
            crate::http::get_client()
                .post(&url)
                .header("Content-Type", "application/json")
                .body(body.clone())
                .send(),
        )
        .await
        {
            Ok(Ok(resp)) => parse_post_resp::<T>(resp, &url).await,
            Ok(Err(e)) => {
                crate::log_warn!("[Community] MR POST 请求失败: {} ({:?})", url, e);
                Err(format!("Modrinth 请求失败: {}", e))
            }
            Err(_) => {
                crate::log_warn!(
                    "[Community] MR POST 官方请求超时（{}s），回退镜像",
                    MR_OFFICIAL_TIMEOUT_SECS
                );
                Err(format!(
                    "Modrinth 官方请求超时（{}s）",
                    MR_OFFICIAL_TIMEOUT_SECS
                ))
            }
        }
    } else {
        // source=2 官方不加超时 / source=0 直接镜像
        let resp = crate::http::get_client()
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body.clone())
            .send()
            .await
            .map_err(|e| {
                crate::log_warn!("[Community] MR POST 请求失败: {} ({:?})", url, e);
                format!("Modrinth 请求失败: {}", e)
            })?;
        parse_post_resp::<T>(resp, &url).await
    };

    match result {
        Ok(value) => {
            crate::log_info!("[Community] MR POST 请求成功: {} ({})", url, fmt_elapsed(start));
            Ok(value)
        }
        Err(e) => {
            let is_not_found = e.starts_with("Modrinth 资源不存在");
            if source == 1 && is_official && !is_not_found {
                crate::log_warn!("[Community] MR POST 官方请求失败，回退镜像: {}", e);
                let mirror_url = format!("{}{}", MR_MIRROR_BASE, path);
                let resp = crate::http::get_client()
                    .post(&mirror_url)
                    .header("Content-Type", "application/json")
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| {
                        crate::log_warn!(
                            "[Community] MR POST 镜像请求失败: {} ({:?})",
                            mirror_url,
                            e
                        );
                        format!("Modrinth 镜像请求失败: {}", e)
                    })?;
                let value: T = parse_post_resp::<T>(resp, &mirror_url).await?;
                crate::log_info!(
                    "[Community] MR POST 镜像请求成功: {} ({})",
                    mirror_url,
                    fmt_elapsed(start)
                );
                return Ok(value);
            }
            Err(e)
        }
    }
}
