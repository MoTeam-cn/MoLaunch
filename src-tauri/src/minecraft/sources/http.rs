//! HTTP 状态码处理 + 统一请求函数
//!
//! 三个层次：
//! - `fetch_text`：单次请求，5 秒超时，记录源健康度
//! - `fetch_with_fallback`：按顺序试候选（官方优先）
//! - `fetch_with_race`：并发试候选，谁先回用谁——元数据专用

use crate::minecraft::community::common::fmt_elapsed;
use crate::{log_debug, log_info, log_warn};
use std::time::{Duration, Instant};

/// 元数据请求的单次超时
///
/// 官方源不可用时不该让用户干等全局 30s 超时——5s 没响应就换镜像。
/// 文件下载不走这里，那条链路由 DownloadManager 处理。
const META_REQUEST_TIMEOUT_SECS: u64 = 5;

/// 判断 HTTP 状态码是否应该直接跳过（不重试）
/// - 403 Forbidden: 服务器拒绝，重试无意义
/// - 429 Too Many Requests: 频率限制，重试会继续被拒
pub fn should_skip_status(status: reqwest::StatusCode) -> bool {
    status.as_u16() == 403 || status.as_u16() == 429
}

/// 判断 URL 是否为镜像源 URL
///
/// 用于 `DownloadManager::reorder_urls` 按源模式重排候选 URL。
/// 识别 BMCLAPI、mocdn.net、mcimirror.top、阿里云公共仓四类镜像域名。
pub fn is_mirror_url(url: &str) -> bool {
    url.contains("bmclapi")
        || url.contains("mocdn")
        || url.contains("mcimirror")
        || url.contains("maven.aliyun.com")
}

/// 单次请求：5 秒超时 + 状态判断 + 源健康度记录
async fn fetch_text(url: &str) -> anyhow::Result<String> {
    let client = crate::http::get_client();
    let start = Instant::now();
    let mirror = is_mirror_url(url);

    let sent = match tokio::time::timeout(
        Duration::from_secs(META_REQUEST_TIMEOUT_SECS),
        client.get(url).send(),
    )
    .await
    {
        Ok(result) => {
            result.map_err(|e| anyhow::anyhow!("{}", crate::http::request_error_msg(&e)))?
        }
        Err(_) => {
            // 官方超时就把整段时间切成镜像，避免下一个请求又干等一次
            if mirror {
                log_warn!(
                    "[Sources] {} 超过 {}s 未响应 ({})",
                    url,
                    META_REQUEST_TIMEOUT_SECS,
                    fmt_elapsed(start)
                );
            } else {
                super::health::mark_official_slow();
                log_warn!(
                    "[Sources] {} 超过 {}s 未响应，改用镜像 ({})",
                    url,
                    META_REQUEST_TIMEOUT_SECS,
                    fmt_elapsed(start)
                );
            }
            return Err(anyhow::anyhow!(
                "超过 {}s 未响应",
                META_REQUEST_TIMEOUT_SECS
            ));
        }
    };

    let status = sent.status();
    if !status.is_success() {
        if should_skip_status(status) {
            log_warn!(
                "[Sources] {} 返回 {}，跳过不重试 ({})",
                url,
                status,
                fmt_elapsed(start)
            );
        } else {
            log_debug!("[Sources] {} 返回 {} ({})", url, status, fmt_elapsed(start));
        }
        return Err(anyhow::anyhow!("HTTP {}", status));
    }

    let text = sent
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("读取响应失败: {}", e))?;
    if !mirror {
        super::health::mark_official_ok();
    }
    log_info!("[Sources] 请求成功: {} ({})", url, fmt_elapsed(start));
    Ok(text)
}

/// 依次尝试 URLs，遇到 403/429 直接跳过，其他错误继续试下一个
pub async fn fetch_with_fallback(urls: &[String]) -> anyhow::Result<String> {
    let mut last_err = String::new();
    for url in urls {
        match fetch_text(url).await {
            Ok(text) => return Ok(text),
            Err(e) => last_err = format!("{}: {}", url, e),
        }
    }
    Err(anyhow::anyhow!("所有源均失败: {}", last_err))
}

/// 并发请求所有候选，取第一个成功的结果
///
/// 只用于元数据（版本列表、加载器版本列表这类几 KB 的 JSON）：
/// 双发一次的成本远低于让用户干等 5 秒。文件下载绝不能走这里——
/// 几百 MB 的东西发两份是纯浪费，那条链路用超时切换 + 健康度记忆。
pub async fn fetch_with_race(urls: &[String]) -> anyhow::Result<String> {
    if urls.len() <= 1 {
        return fetch_with_fallback(urls).await;
    }

    let mut set = tokio::task::JoinSet::new();
    for url in urls {
        let owned = url.clone();
        set.spawn(async move {
            let result = fetch_text(&owned).await;
            (owned, result)
        });
    }

    let mut errors: Vec<String> = Vec::new();
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok((url, Ok(text))) => {
                log_info!("[Sources] 竞速胜出: {}", url);
                // 取消还在跑的另一个请求，别让它白下载
                set.shutdown().await;
                return Ok(text);
            }
            Ok((url, Err(e))) => errors.push(format!("{}: {}", url, e)),
            Err(e) => errors.push(format!("请求任务异常: {}", e)),
        }
    }

    Err(anyhow::anyhow!("所有源均失败: {}", errors.join("; ")))
}
