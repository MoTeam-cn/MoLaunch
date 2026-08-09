//! PoW Challenge 客户端求解
//!
//! 兼容 apiServer `pow_guard` 中间件：`401 + code:1007` 下发 challenge，
//! 客户端求解 `SHA256(salt || nonce_le_bytes)` 前导零 ≥ difficulty 后带
//! `{challenge_id}:{nonce}` 头重试。求解用 `std::thread` + `mpsc::recv_timeout`
//! 实现 3 秒超时，整体在 `spawn_blocking` 中执行，不阻塞 runtime。

use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// PoW 请求头字段名默认值（与服务端 `pow_guard` 一致）
pub const POW_HEADER: &str = "x-molaunch-pow";

/// PoW 业务码（`401 + code:1007`，与服务端 `pow_guard` 一致）
pub const POW_CHALLENGE_CODE: u32 = 1007;

/// 求解超时（秒）：超过即放弃，返回 None，调用方按原错误处理
const SOLVE_TIMEOUT: Duration = Duration::from_secs(3);

/// 求解线程数上限（避免把小机拖垮）
const MAX_SOLVE_THREADS: u64 = 8;

/// 求解难度上限：防恶意服务端下发超高难度放大客户端 DoS
const MAX_DIFFICULTY: u32 = 32;

/// 服务端下发的 challenge（对应 `pow_guard` 响应体 `data` 字段）
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PowChallenge {
    pub challenge_id: String,
    /// 16 字节随机盐（hex 编码）
    pub salt: String,
    pub difficulty: u32,
    pub ttl: u64,
    /// 服务端生成时固化的目标路径（校验一致性用，防误用错接口）
    pub path: String,
    /// 重试时需携带的请求头字段名（服务端下发，兼容其改名）
    #[serde(default = "default_header_name")]
    pub header_name: String,
}

fn default_header_name() -> String {
    POW_HEADER.to_string()
}

impl PowChallenge {
    /// 解码 salt（hex → 原始字节，用于求解）
    pub fn salt_bytes(&self) -> Option<Vec<u8>> {
        hex::decode(&self.salt).ok()
    }
}

/// 从 `401 + code:1007` 响应中解析 challenge
///
/// 返回 `Some` 表示服务端要求先完成 PoW，`None` 表示非 PoW 响应
/// （如 token 过期、参数错误等，按原逻辑处理）。
pub fn parse_challenge(body: &str) -> Option<PowChallenge> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    if v.get("code").and_then(|c| c.as_u64())? != POW_CHALLENGE_CODE as u64 {
        return None;
    }
    let data = v.get("data")?;
    serde_json::from_value(data.clone()).ok()
}

/// 计算字节切片的前导零比特数
fn leading_zero_bits(bytes: &[u8]) -> u32 {
    let mut bits = 0;
    for &b in bytes {
        if b == 0 {
            bits += 8;
        } else {
            bits += b.leading_zeros();
            break;
        }
    }
    bits
}

/// 求解 challenge：找到 nonce 使 `SHA256(salt || nonce_le_bytes)` 前导零 ≥ difficulty
///
/// - 多线程分片并行搜索（共享原子计数器分配 nonce 空间）
/// - `recv_timeout` 施加 3 秒硬超时，超时放弃并标记所有线程停止
/// - 整体放入 `spawn_blocking`，不阻塞异步 runtime
pub async fn solve_challenge(salt: &[u8], difficulty: u32) -> Option<u64> {
    if difficulty == 0 {
        return Some(0);
    }
    // 难度钳制上限，防恶意服务端放大 DoS
    let difficulty = difficulty.min(MAX_DIFFICULTY);
    let salt = salt.to_vec();
    tokio::task::spawn_blocking(move || solve_sync(&salt, difficulty))
        .await
        .ok()
        .flatten()
}

/// 同步求解（在后台线程池中执行）
fn solve_sync(salt: &[u8], difficulty: u32) -> Option<u64> {
    let thread_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4) as u64;
    let thread_count = thread_count.clamp(1, MAX_SOLVE_THREADS);

    let stop = AtomicBool::new(false);
    let counter = AtomicU64::new(0);
    let (tx, rx) = mpsc::channel::<u64>();

    std::thread::scope(|scope| {
        for _ in 0..thread_count {
            let tx = tx.clone();
            let stop = &stop;
            let counter = &counter;
            let salt = salt.to_vec();
            scope.spawn(move || loop {
                if stop.load(Ordering::Relaxed) {
                    return;
                }
                let nonce = counter.fetch_add(1, Ordering::Relaxed);
                let mut hasher = Sha256::new();
                hasher.update(&salt);
                hasher.update(nonce.to_le_bytes());
                let digest = hasher.finalize();
                if leading_zero_bits(&digest) >= difficulty {
                    stop.store(true, Ordering::Relaxed);
                    let _ = tx.send(nonce);
                    return;
                }
            });
        }

        // 等待解出或超时
        let deadline = Instant::now() + SOLVE_TIMEOUT;
        let wait = deadline.saturating_duration_since(Instant::now());
        let result = rx.recv_timeout(wait).ok();
        stop.store(true, Ordering::Relaxed);
        result
    })
}

#[cfg(test)]
mod pow_test;
