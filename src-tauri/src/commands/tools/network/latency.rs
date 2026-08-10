use std::time::{Duration, Instant};

use futures_util::future::join_all;

use crate::log_info;
use crate::state::AppState;

use super::super::types::{LatencyItem, NetworkLatencyResult, NetworkLatencyTestParams};

/// 并发测试多个 URL 的 HTTP 延迟
///
/// 通过 `crate::http::get_client()` 复用全局 HTTP 客户端（自动应用用户配置的代理、
/// User-Agent、连接池），在每个请求上附加 10 秒超时（测速场景的合理上限）。
/// 用 `join_all` 并发请求所有 URL，失败时 `latency_ms=None`、`status_code=0`、
/// `error` 填失败原因。
pub async fn latency_test(
    state: &AppState,
    params: NetworkLatencyTestParams,
) -> Result<serde_json::Value, String> {
    let _game_dir = {
        let config = state.config.lock().await;
        crate::state::resolve_game_dir(&config.game_dir)
    };

    log_info!("[NetworkLatency] 测试 {} 个 URL", params.urls.len());

    // 复用全局客户端（统一代理 / User-Agent / 连接池），请求级别覆盖超时为 10s
    let client = crate::http::get_client();

    let futures: Vec<_> = params
        .urls
        .into_iter()
        .map(|url| {
            let client = client.clone();
            async move {
                let start = Instant::now();
                match client
                    .get(&url)
                    .timeout(Duration::from_secs(10))
                    .send()
                    .await
                {
                    Ok(resp) => {
                        let latency = start.elapsed().as_millis() as u64;
                        let status = resp.status().as_u16();
                        LatencyItem {
                            url,
                            latency_ms: Some(latency),
                            status_code: status,
                            error: String::new(),
                        }
                    }
                    Err(e) => LatencyItem {
                        url,
                        latency_ms: None,
                        status_code: 0,
                        error: crate::http::request_error_msg(&e),
                    },
                }
            }
        })
        .collect();

    let results = join_all(futures).await;

    log_info!("[NetworkLatency] 完成 {} 个 URL 测试", results.len());

    let result = NetworkLatencyResult { results };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}
