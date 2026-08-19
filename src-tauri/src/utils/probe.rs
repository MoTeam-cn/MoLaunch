//! 通用测速组件：并发 HTTP 测速（HEAD + Range 0-1，禁止重定向）
//! 供 GitHub 镜像竞速、外部二进制下载等场景复用。

use std::sync::Arc;
use std::time::Duration;

/// 并发测速 URL 列表（HEAD + Range 0-1，单请求 10s 超时，禁止重定向），
/// 返回按耗时升序排列的可用列表；`cancel_flag` 置位时返回「下载已取消」。
///
/// 使用共享无重定向单例 `no_redirect_client()`：会跳转的地址直接失败剔除
/// （重定向引入额外跳转，慢且不稳定）。
///
/// 逐候选 DEBUG 日志（发送的原始 URL + 状态/错误 + 耗时）：
/// 排查 URL 形态问题，定位竞速为何全灭。
pub async fn probe_urls(
    urls: &[String],
    cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
) -> Result<Vec<(Duration, String)>, String> {
    let client = crate::http::no_redirect_client();
    let mut handles = Vec::with_capacity(urls.len());
    for url in urls {
        let client = client.clone();
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            let start = std::time::Instant::now();
            let resp = client
                .get(&url)
                .header("Range", "bytes=0-1")
                .timeout(Duration::from_secs(10))
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() || r.status().as_u16() == 206 => {
                    crate::log_debug!(
                        "[Probe] 测速可用: {}ms status={} url={}",
                        start.elapsed().as_millis(),
                        r.status(),
                        url
                    );
                    Some((start.elapsed(), url))
                }
                Ok(r) => {
                    crate::log_debug!(
                        "[Probe] 测速被拒: {}ms status={} url={}",
                        start.elapsed().as_millis(),
                        r.status(),
                        url
                    );
                    None
                }
                Err(e) => {
                    crate::log_debug!(
                        "[Probe] 测速失败: {}ms err={} url={}",
                        start.elapsed().as_millis(),
                        crate::http::request_error_msg(&e),
                        url
                    );
                    None
                }
            }
        }));
    }
    let mut results = Vec::new();
    for mut h in handles {
        // 每 200ms 轮询取消信号，同时等待测速完成（&mut 借用避免 select 丢弃已完成结果）
        loop {
            if let Some(ref flag) = cancel_flag {
                if flag.load(std::sync::atomic::Ordering::Relaxed) {
                    return Err("下载已取消".to_string());
                }
            }
            match tokio::time::timeout(Duration::from_millis(200), &mut h).await {
                Ok(r) => {
                    if let Ok(Some((elapsed, url))) = r {
                        results.push((elapsed, url));
                    }
                    break;
                }
                Err(_) => continue,
            }
        }
    }
    results.sort_by_key(|(t, _)| *t);
    Ok(results)
}

/// 竞速：并发测速取响应最快者（`probe_urls` 的取首封装）
pub async fn pick_fastest(
    candidates: &[String],
    cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
) -> Result<String, String> {
    let results = probe_urls(candidates, cancel_flag).await?;
    match results.into_iter().next() {
        Some((_, url)) => {
            crate::log_debug!("[Probe] 竞速胜者: {url}");
            Ok(url)
        }
        None => Err("所有镜像源均不可用".to_string()),
    }
}
