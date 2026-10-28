//! DoH 解析（JSON 接口），带多提供方回落与熔断
//!
//! 用各家的 DNS-JSON 接口，不实现 RFC 8484 的 wire format：解析报文要自己编码/解码，
//! 出错面大又不带来额外能力。失败链为「主 → 备 → 系统解析」，DoH 全挂不会影响下载。

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::log_debug;

/// 单次 DoH 请求超时
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// 连续失败几次触发熔断
const BREAKER_THRESHOLD: u8 = 3;

/// 熔断持续时间
const BREAKER_OPEN: Duration = Duration::from_secs(300);

/// DoH 提供方（按顺序尝试）
struct Provider {
    name: &'static str,
    endpoint: &'static str,
}

const PROVIDERS: [Provider; 3] = [
    Provider {
        name: "腾讯",
        endpoint: "https://doh.pub/dns-query",
    },
    Provider {
        name: "阿里",
        endpoint: "https://dns.alidns.com/dns-query",
    },
    Provider {
        name: "Cloudflare",
        endpoint: "https://cloudflare-dns.com/dns-query",
    },
];

/// 开关（开发者选项 `download.doh`）
static ENABLED: AtomicBool = AtomicBool::new(false);

/// 连续失败计数
static FAILS: AtomicU8 = AtomicU8::new(0);

/// 熔断截止时间
static OPEN_UNTIL: Mutex<Option<Instant>> = Mutex::new(None);

/// 设置开关
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

/// 开关是否开启
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// DNS-JSON 响应
#[derive(serde::Deserialize)]
struct DohResponse {
    #[serde(rename = "Answer", default)]
    answer: Vec<DohAnswer>,
}

#[derive(serde::Deserialize)]
struct DohAnswer {
    #[serde(rename = "type")]
    record_type: u16,
    #[serde(default)]
    data: String,
}

/// 从 DNS-JSON 里挑出 A/AAAA 记录
///
/// CNAME（type=5）与其它记录一律丢弃：我们要的是最终地址，不是别名链。
fn parse_answers(body: &str) -> Result<Vec<IpAddr>, String> {
    let response: DohResponse =
        serde_json::from_str(body).map_err(|e| format!("DoH 响应解析失败: {}", e))?;
    let mut addrs: Vec<IpAddr> = Vec::new();
    for answer in response.answer {
        if answer.record_type != 1 && answer.record_type != 28 {
            continue;
        }
        let Ok(ip) = answer.data.parse::<IpAddr>() else {
            continue;
        };
        if super::dns::usable_address(ip) && !addrs.contains(&ip) {
            addrs.push(ip);
        }
    }
    Ok(addrs)
}

/// 熔断是否打开（打开期内不再尝试 DoH，直接用系统解析）
fn breaker_open(now: Instant) -> bool {
    let guard = OPEN_UNTIL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    match *guard {
        Some(until) => now < until,
        None => false,
    }
}

/// 记录一次失败：到达阈值就熔断一段时间
fn note_failure(now: Instant) {
    let fails = FAILS.fetch_add(1, Ordering::Relaxed).saturating_add(1);
    if fails >= BREAKER_THRESHOLD {
        let mut guard = OPEN_UNTIL
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Some(now + BREAKER_OPEN);
        FAILS.store(0, Ordering::Relaxed);
        log_debug!(
            "[DoH] 连续失败 {} 次，熔断 {}s",
            fails,
            BREAKER_OPEN.as_secs()
        );
    }
}

/// 记录一次成功：清空失败计数与熔断
fn note_success() {
    FAILS.store(0, Ordering::Relaxed);
    let mut guard = OPEN_UNTIL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = None;
}

/// 通过 DoH 解析域名，按提供方顺序回落
pub async fn resolve(host: &str) -> Result<Vec<IpAddr>, String> {
    if breaker_open(Instant::now()) {
        return Err("DoH 熔断中".to_string());
    }
    let mut last_error = String::from("没有可用的 DoH 提供方");
    for provider in PROVIDERS.iter() {
        match query_provider(provider, host).await {
            Ok(addrs) if !addrs.is_empty() => {
                note_success();
                return Ok(addrs);
            }
            Ok(_) => {
                last_error = format!("{} 无解析结果", provider.name);
            }
            Err(e) => {
                log_debug!("[DoH] {} 失败: {} ({})", provider.name, host, e);
                last_error = e;
            }
        }
    }
    note_failure(Instant::now());
    Err(last_error)
}

/// 向单个提供方发起查询（先 A，无结果再 AAAA）
async fn query_provider(provider: &Provider, host: &str) -> Result<Vec<IpAddr>, String> {
    for record_type in ["A", "AAAA"] {
        let url = format!("{}?name={}&type={}", provider.endpoint, host, record_type);
        let response = tokio::time::timeout(
            REQUEST_TIMEOUT,
            crate::http::get_client()
                .get(&url)
                .header("Accept", "application/dns-json")
                .send(),
        )
        .await
        .map_err(|_| format!("DoH 请求超时（{}s）", REQUEST_TIMEOUT.as_secs()))?
        .map_err(|e| format!("DoH 请求失败: {}", crate::http::request_error_msg(&e)))?;
        if !response.status().is_success() {
            return Err(format!("DoH 响应异常: HTTP {}", response.status().as_u16()));
        }
        let body = response
            .text()
            .await
            .map_err(|e| format!("DoH 响应读取失败: {}", e))?;
        let addrs = parse_answers(&body)?;
        if !addrs.is_empty() {
            return Ok(addrs);
        }
    }
    Ok(Vec::new())
}

#[cfg(test)]
#[path = "doh_test.rs"]
mod doh_test;
