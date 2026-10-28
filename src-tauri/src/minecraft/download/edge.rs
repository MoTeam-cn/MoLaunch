//! 边缘节点优选：绕开 DNS 的选择权，自己测速选点
//!
//! 对同一域名解析出的多个边缘 IP 各拉一小段真实文件测吞吐，挑最快的那个并把
//! 后续请求钉在这个 IP 上（证书/Host 仍用原域名）。只在允许的几个 CDN 域名上生效，
//! 且受探测流量预算约束。

use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::{log_debug, log_warn};

/// 单个候选至少要跑出这么多字节才参与比较（太少说明根本没跑起来）
const MIN_PROBE_BYTES: u64 = 256 * 1024;

/// 单次探测最多拉多少字节
///
/// 4 MiB 足以区分「快」与「绕路」，再大就只是白耗流量了。
const PROBE_RANGE_BYTES: u64 = 4 * 1024 * 1024;

/// 单次探测（建连 + 拉流）的超时
const PROBE_TIMEOUT: Duration = Duration::from_secs(8);

/// 解析候选的超时
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(6);

/// 优选结果的有效期
const CHOICE_TTL: Duration = Duration::from_secs(300);

/// 每小时探测流量预算
const PROBE_BUDGET_BYTES: u64 = 64 * 1024 * 1024;

/// 预算窗口长度
const BUDGET_WINDOW: Duration = Duration::from_secs(3600);

/// 最多探测几个候选（控制流量与耗时）
const MAX_PROBE_CANDIDATES: usize = 6;

/// 少于这么多个候选就没有优选的意义（DNS 只给了一个，测了也没得选）
const MIN_CANDIDATES: usize = 2;

/// 新节点要比当前节点快这么多才切换：避免噪声导致的反复横跳
const SWITCH_GAIN: f64 = 0.25;

/// 连续失败几次就作废该节点
const MAX_EDGE_FAILURES: u32 = 2;

/// 启用优选的域名（含子域）
///
/// 只放「多边缘、DNS 就近可能选错」的 CDN 域名。自家 CDN 与公共镜像不加：
/// 它们本来就是直连或按量调度，钉死 IP 只会更慢。
const EDGE_HOSTS: [&str; 4] = [
    "cdn.modrinth.com",
    "cdn.curseforge.com",
    "edge.forgecdn.net",
    "mediafilez.forgecdn.net",
];

/// 开关（开发者选项 `download.edge_select`）
static ENABLED: AtomicBool = AtomicBool::new(false);

/// 选中的边缘节点
struct Choice {
    addr: SocketAddr,
    throughput: u64,
    at: Instant,
    fails: u32,
}

/// 探测流量预算（滚动小时窗口）
struct ProbeBudget {
    window_start: Instant,
    used: u64,
}

impl ProbeBudget {
    fn new(now: Instant) -> Self {
        Self {
            window_start: now,
            used: 0,
        }
    }

    /// 窗口滚动后重置用量
    fn roll(&mut self, now: Instant) {
        if now.saturating_duration_since(self.window_start) >= BUDGET_WINDOW {
            self.window_start = now;
            self.used = 0;
        }
    }

    /// 还能不能再花 `want` 字节
    fn allow(&mut self, now: Instant, want: u64) -> bool {
        self.roll(now);
        self.used.saturating_add(want) <= PROBE_BUDGET_BYTES
    }

    /// 记一次实际花费
    fn charge(&mut self, now: Instant, bytes: u64) {
        self.roll(now);
        self.used = self.used.saturating_add(bytes);
    }
}

#[derive(Default)]
struct State {
    choices: HashMap<String, Choice>,
    clients: HashMap<(String, IpAddr), reqwest::Client>,
    probing: HashSet<String>,
    budget: Option<ProbeBudget>,
}

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

fn state() -> &'static Mutex<State> {
    STATE.get_or_init(|| Mutex::new(State::default()))
}

fn lock() -> std::sync::MutexGuard<'static, State> {
    state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 设置开关（启动加载配置与保存配置时各写一次）
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

/// 开关是否开启
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// 该 host 是否在优选名单内
fn eligible_host(host: &str) -> bool {
    EDGE_HOSTS
        .iter()
        .any(|allowed| host == *allowed || host.ends_with(&format!(".{}", allowed)))
}

/// 本 URL 是否值得优选（返回 host）
fn eligible(url: &str) -> Option<String> {
    if !enabled() || crate::http::proxy_configured() {
        return None;
    }
    // 关了证书校验时「TLS 失败即作废候选」这条防线失效，宁可不用
    if crate::commands::system::developer::is_ignore_tls() {
        return None;
    }
    let host = reqwest::Url::parse(url)
        .ok()?
        .host_str()?
        .to_ascii_lowercase();
    if !eligible_host(&host) {
        return None;
    }
    Some(host)
}

/// 为本 URL 取下载客户端：命中优选时返回钉住该 IP 的客户端，否则返回兜底客户端
pub fn client_for(url: &str, fallback: &reqwest::Client) -> reqwest::Client {
    let Some(host) = eligible(url) else {
        return fallback.clone();
    };
    let now = Instant::now();
    let mut cached = None;
    let mut need_probe = false;
    {
        let mut guard = lock();
        let ip = guard
            .choices
            .get(&host)
            .filter(|choice| is_fresh(choice.at, now) && choice.fails < MAX_EDGE_FAILURES)
            .map(|choice| choice.addr.ip());
        match ip {
            Some(ip) => cached = guard.clients.get(&(host.clone(), ip)).cloned(),
            None => {
                guard.choices.remove(&host);
            }
        }
        if cached.is_none() {
            need_probe = true;
        }
    }
    if need_probe {
        spawn_probe(host, url.to_string());
    }
    cached.unwrap_or_else(|| fallback.clone())
}

/// 请求失败上报：连续失败到阈值就作废该节点，下次重新优选
pub fn note_failure(url: &str) {
    let Some(host) = eligible(url) else {
        return;
    };
    let dead_ip = {
        let mut guard = lock();
        match guard.choices.get_mut(&host) {
            Some(choice) => {
                choice.fails = choice.fails.saturating_add(1);
                if choice.fails >= MAX_EDGE_FAILURES {
                    Some(choice.addr.ip())
                } else {
                    None
                }
            }
            None => None,
        }
    };
    if let Some(ip) = dead_ip {
        {
            let mut guard = lock();
            guard.choices.remove(&host);
            guard.clients.remove(&(host.clone(), ip));
        }
        log_warn!("[Edge] 节点连续失败，已作废并重新优选: {} ({})", host, ip);
    }
}

/// 后台跑一轮优选
fn spawn_probe(host: String, url: String) {
    {
        let mut guard = lock();
        if !guard.probing.insert(host.clone()) {
            return;
        }
    }
    tokio::spawn(async move {
        let outcome = probe_host(&host, &url).await;
        let mut guard = lock();
        guard.probing.remove(&host);
        let Some((addr, throughput)) = outcome else {
            return;
        };
        let current = guard.choices.get(&host).map(|choice| choice.throughput);
        if !should_switch(throughput, current) {
            log_debug!(
                "[Edge] 新候选没有明显更快，保持现状: {} ({} B/s)",
                host,
                throughput
            );
            return;
        }
        drop(guard);
        let Some(client) = crate::http::build_pinned_client(&host, addr) else {
            log_warn!("[Edge] 构建钉 IP 客户端失败: {} ({})", host, addr.ip());
            return;
        };
        let mut guard = lock();
        guard.clients.insert((host.clone(), addr.ip()), client);
        guard.choices.insert(
            host.clone(),
            Choice {
                addr,
                throughput,
                at: Instant::now(),
                fails: 0,
            },
        );
        log_debug!(
            "[Edge] 选定边缘节点: {} -> {} ({} B/s)",
            host,
            addr.ip(),
            throughput
        );
    });
}

/// 解析候选地址（过滤掉私有/保留/TUN 假地址）
async fn candidates_for(host: &str) -> Vec<IpAddr> {
    let resolved =
        tokio::time::timeout(RESOLVE_TIMEOUT, tokio::net::lookup_host((host, 443))).await;
    let Ok(Ok(iter)) = resolved else {
        return Vec::new();
    };
    let mut seen: Vec<IpAddr> = Vec::new();
    for addr in iter {
        let ip = addr.ip();
        if usable_candidate(ip) && !seen.contains(&ip) {
            seen.push(ip);
        }
    }
    seen
}

/// 跑一轮探测，返回最快的候选
async fn probe_host(host: &str, url: &str) -> Option<(SocketAddr, u64)> {
    {
        let mut guard = lock();
        let budget = guard
            .budget
            .get_or_insert_with(|| ProbeBudget::new(Instant::now()));
        if !budget.allow(Instant::now(), MIN_PROBE_BYTES) {
            log_debug!("[Edge] 探测流量预算已用完，本轮跳过: {}", host);
            return None;
        }
    }
    let candidates = candidates_for(host).await;
    if candidates.len() < MIN_CANDIDATES {
        log_debug!(
            "[Edge] 候选不足 {} 个，跳过优选: {} ({})",
            MIN_CANDIDATES,
            host,
            candidates.len()
        );
        return None;
    }
    let mut best: Option<(SocketAddr, u64)> = None;
    for ip in candidates.into_iter().take(MAX_PROBE_CANDIDATES) {
        let addr = SocketAddr::new(ip, 443);
        let Some(throughput) = probe_one(host, url, addr).await else {
            continue;
        };
        {
            let mut guard = lock();
            let budget = guard
                .budget
                .get_or_insert_with(|| ProbeBudget::new(Instant::now()));
            budget.charge(Instant::now(), PROBE_RANGE_BYTES);
        }
        if best.is_none_or(|(_, current)| throughput > current) {
            best = Some((addr, throughput));
        }
        let exhausted = {
            let mut guard = lock();
            let budget = guard
                .budget
                .get_or_insert_with(|| ProbeBudget::new(Instant::now()));
            !budget.allow(Instant::now(), MIN_PROBE_BYTES)
        };
        if exhausted {
            break;
        }
    }
    best
}

/// 探测单个候选：拉一小段真实文件，返回吞吐（B/s）
///
/// 任何失败（含 TLS 校验失败）都返回 None，把该候选直接排除——宁可不用优选，
/// 也不能被中间人伪造的响应污染优选表。
async fn probe_one(host: &str, url: &str, addr: SocketAddr) -> Option<u64> {
    let client = crate::http::build_pinned_client(host, addr)?;
    let started = Instant::now();
    let request = client
        .get(url)
        .header(
            reqwest::header::RANGE,
            format!("bytes=0-{}", PROBE_RANGE_BYTES - 1),
        )
        .header(reqwest::header::ACCEPT_ENCODING, "identity");
    let response = tokio::time::timeout(PROBE_TIMEOUT, request.send())
        .await
        .ok()?
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = tokio::time::timeout(PROBE_TIMEOUT, response.bytes())
        .await
        .ok()?
        .ok()?;
    let received = body.len() as u64;
    if received < MIN_PROBE_BYTES {
        return None;
    }
    let elapsed = started.elapsed().as_secs_f64().max(0.001);
    Some((received as f64 / elapsed) as u64)
}

/// 优选结果是否仍在有效期内
fn is_fresh(at: Instant, now: Instant) -> bool {
    match now.checked_duration_since(at) {
        Some(elapsed) => elapsed < CHOICE_TTL,
        None => false,
    }
}

/// 是否值得从当前节点切到新节点
fn should_switch(best: u64, current: Option<u64>) -> bool {
    match current {
        None => true,
        Some(current) => (best as f64) > (current as f64) * (1.0 + SWITCH_GAIN),
    }
}

/// 候选地址是否可用（排除内网、保留段与代理伪造的假地址）
fn usable_candidate(ip: IpAddr) -> bool {
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
                return usable_candidate(IpAddr::V4(mapped));
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

#[cfg(test)]
#[path = "edge_test.rs"]
mod edge_test;
