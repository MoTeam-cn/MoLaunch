//! 下载专用 DNS 解析缓存
//!
//! CDN 域名在一次批量下载里会被反复解析（每个分片、每次换源各一次），
//! 这里在进程内做短 TTL 缓存，把重复解析压成一次。

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use crate::log_debug;

/// 解析结果缓存时长
///
/// 取 60s 而非更长：CDN 边缘随时会变，缓存太久等于把客户端钉死在一个可能失效的
/// 节点上；60s 已足够覆盖一次批量下载里的成百上千次重复解析。
const CACHE_TTL: Duration = Duration::from_secs(60);

/// 单次系统解析的超时
///
/// 解析卡住时宁可快速失败交给上层换源，也不要让整个下载停在握手阶段。
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(6);

/// 缓存条目
struct CacheEntry {
    addrs: Vec<SocketAddr>,
    at: Instant,
}

/// 缓存本体（用 Arc 共享，解析 future 才能是 'static）
type Cache = Mutex<HashMap<String, CacheEntry>>;

/// 带短 TTL 缓存的解析器（仅供下载客户端使用）
pub struct CacheResolver {
    cache: Arc<Cache>,
}

impl CacheResolver {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for CacheResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// 读缓存：未命中或已过期返回 None
fn read_cache(cache: &Cache, host: &str, now: Instant) -> Option<Vec<SocketAddr>> {
    let guard = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let entry = guard.get(host)?;
    if is_fresh(entry.at, now) {
        Some(entry.addrs.clone())
    } else {
        None
    }
}

/// 写缓存：同一 host 直接覆盖
fn write_cache(cache: &Cache, host: &str, addrs: &[SocketAddr], now: Instant) {
    let mut guard = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.insert(
        host.to_string(),
        CacheEntry {
            addrs: addrs.to_vec(),
            at: now,
        },
    );
}

/// 缓存是否仍在有效期内
///
/// 时间顺序异常（now 早于 at）时按「已过期」处理：最多多解析一次，不会 panic。
fn is_fresh(at: Instant, now: Instant) -> bool {
    match now.checked_duration_since(at) {
        Some(elapsed) => elapsed < CACHE_TTL,
        None => false,
    }
}

/// 解析地址：开了 DoH 就先走 DoH，失败一律回落系统解析
async fn resolve_addrs(
    host: &str,
) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error + Send + Sync>> {
    if super::doh::enabled() {
        match super::doh::resolve(host).await {
            Ok(addrs) if !addrs.is_empty() => {
                return Ok(addrs
                    .into_iter()
                    .map(|ip| SocketAddr::new(ip, 443))
                    .collect());
            }
            Ok(_) => {}
            Err(e) => log_debug!("[DNS] DoH 未取到结果，回落系统解析: {} ({})", host, e),
        }
    }
    lookup_system(host).await
}

/// 系统解析（含私有/保留地址过滤与超时）
async fn lookup_system(
    host: &str,
) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error + Send + Sync>> {
    let looked_up =
        tokio::time::timeout(LOOKUP_TIMEOUT, tokio::net::lookup_host((host, 443))).await;
    let mut addrs: Vec<SocketAddr> = Vec::new();
    match looked_up {
        Ok(Ok(iter)) => {
            for addr in iter {
                if usable_address(addr.ip()) && !addrs.contains(&addr) {
                    addrs.push(addr);
                }
            }
        }
        Ok(Err(e)) => {
            let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
            return Err(boxed);
        }
        Err(_) => {
            return Err(format!("DNS 解析超时（{}s）: {}", LOOKUP_TIMEOUT.as_secs(), host).into());
        }
    }
    // 空结果不进缓存：下一次应当重新问一次，而不是拿到一个空集合
    if addrs.is_empty() {
        return Err(format!("DNS 解析无结果: {}", host).into());
    }
    Ok(addrs)
}

/// 地址是否可用（排除内网、保留段与代理伪造的假地址）
///
/// DoH、系统解析与边缘优选共用这一份判断，避免三处各自实现出现缺口。
pub(crate) fn usable_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            if v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_multicast()
                || v4.is_broadcast()
                || v4.is_unspecified()
            {
                return false;
            }
            let octets = v4.octets();
            // 198.18.0.0/15：TUN/代理常用的假地址段，测速结果全是本机回环
            if octets[0] == 198 && (octets[1] == 18 || octets[1] == 19) {
                return false;
            }
            // 100.64.0.0/10：运营商级 NAT
            if octets[0] == 100 && (64..=127).contains(&octets[1]) {
                return false;
            }
            true
        }
        IpAddr::V6(v6) => {
            if v6.is_loopback() || v6.is_multicast() || v6.is_unspecified() {
                return false;
            }
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return usable_address(IpAddr::V4(mapped));
            }
            let segments = v6.segments();
            // fc00::/7 唯一本地地址
            if (segments[0] & 0xfe00) == 0xfc00 {
                return false;
            }
            // fe80::/10 链路本地地址
            if (segments[0] & 0xffc0) == 0xfe80 {
                return false;
            }
            true
        }
    }
}

impl Resolve for CacheResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_string();
        // 只把 Arc 带进 future：Resolving 要求 'static，不能借用 self
        let cache = Arc::clone(&self.cache);
        Box::pin(async move {
            let now = Instant::now();
            if let Some(addrs) = read_cache(&cache, &host, now) {
                let boxed: Addrs = Box::new(addrs.into_iter());
                return Ok(boxed);
            }
            let addrs = resolve_addrs(&host).await?;
            write_cache(&cache, &host, &addrs, now);
            let boxed: Addrs = Box::new(addrs.into_iter());
            Ok(boxed)
        })
    }
}

#[cfg(test)]
#[path = "dns_test.rs"]
mod dns_test;
