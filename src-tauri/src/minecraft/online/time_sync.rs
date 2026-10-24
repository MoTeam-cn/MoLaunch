//! 服务端时间对齐（MoSign-v1 时间戳基准）
//!
//! 注册/登录/刷新/登出的 `timestamp` 参与签名，服务端按 `auth.timestamp_skew`
//! 校验容差；客户端时钟漂移（CMOS 掉电、手动改时区/夏令时、虚拟机挂起、
//! 系统时间被同步工具回拨）会直接被拒成「时间戳超出容差范围」。
//! 这里用 `GET /v3/time` 把时间基准对齐到服务端，本地时钟只负责推进秒数。

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::OnceLock;
use tokio::sync::Mutex;

use super::client::OnlineClient;
use super::client_types::ClientError;

/// 对齐结果复用时长（秒）：TTL 内不再重复请求 /v3/time
const SYNC_TTL_SECS: u64 = 600;

/// 偏移告警阈值（秒）：超过该值打 warn，便于排查客户端时钟异常
const OFFSET_WARN_SECS: i64 = 30;

/// 服务端时间 - 本机时间（秒）
static OFFSET_SECS: AtomicI64 = AtomicI64::new(0);
/// 上次对齐成功时的服务端时间戳（0 = 从未对齐）
static LAST_SYNC_AT: AtomicU64 = AtomicU64::new(0);

/// 串行化对齐请求，避免并发触发多次 /v3/time
fn sync_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// 本机 Unix 时间戳（秒）
pub fn local_now() -> u64 {
    chrono::Utc::now().timestamp().max(0) as u64
}

/// 偏移计算（纯函数，便于单测）：服务端时间 - 本机时间
pub fn compute_offset(server_time: u64, local_time: u64) -> i64 {
    server_time as i64 - local_time as i64
}

/// 是否已对齐过服务端时间
pub fn is_synced() -> bool {
    LAST_SYNC_AT.load(Ordering::Relaxed) != 0
}

/// 当前偏移（秒），正数表示本机慢于服务端
pub fn offset_secs() -> i64 {
    OFFSET_SECS.load(Ordering::Relaxed)
}

/// 对齐后的当前时间戳（秒）；未对齐时等价于本机时钟
pub fn server_now() -> u64 {
    let local = local_now() as i64;
    (local + offset_secs()).max(0) as u64
}

/// 对齐结果是否仍在 TTL 内
fn is_fresh() -> bool {
    let last = LAST_SYNC_AT.load(Ordering::Relaxed);
    last != 0 && server_now().saturating_sub(last) < SYNC_TTL_SECS
}

/// 立即对齐一次（GET /v3/time），返回偏移（秒）
pub async fn sync_once(client: &OnlineClient) -> Result<i64, ClientError> {
    let data = client.get_server_time().await?;
    let offset = compute_offset(data.server_time, local_now());
    OFFSET_SECS.store(offset, Ordering::Relaxed);
    LAST_SYNC_AT.store(data.server_time, Ordering::Relaxed);
    if offset.abs() >= OFFSET_WARN_SECS {
        crate::log_warn!(
            "[Online] 本机时钟与服务端偏差 {} 秒，认证时间戳已按服务端对齐（timezone={}）",
            offset,
            data.timezone
        );
    } else {
        crate::log_debug!("[Online] 时间基准已对齐服务端，偏移 {} 秒", offset);
    }
    Ok(offset)
}

/// 保证时间基准新鲜：未对齐或超过 TTL 时请求一次 /v3/time
///
/// 失败只告警并回落本机时钟——时间对齐是增强项，不能阻断认证流程。
pub async fn ensure_fresh(client: &OnlineClient) {
    if is_fresh() {
        return;
    }
    let _guard = sync_lock().lock().await;
    // 双检：并发调用时只有第一个真正发请求
    if is_fresh() {
        return;
    }
    if let Err(e) = sync_once(client).await {
        crate::log_warn!("[Online] 时间对齐失败，本次沿用本机时钟: {}", e);
    }
}

#[cfg(test)]
#[path = "time_sync_test.rs"]
mod time_sync_test;
