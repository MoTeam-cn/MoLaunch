//! 遥测上报调度：日志环形缓冲、事件构造、去重入队、后台 flush
//!
//! 重要约束：`record_error` / `push_log_line` 由 logger 持锁调用，内部**不得**使用
//! 任何 log_* 宏（会重入 LOGGER 死锁），只能做内存操作与静默文件 IO。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use once_cell::sync::Lazy;

use super::config::{DEDUP_WINDOW_SECS, FLUSH_INTERVAL_SECS, LOG_RING_LINES};
use super::sink::SlsSink;
use super::store::TelemetryStore;
use super::types::{EventKind, TelemetryEvent};

/// 启动事件去重窗口：同一版本每设备只报一次（30 天）
const STARTUP_WINDOW_SECS: i64 = 30 * 24 * 3600;

static STORE: OnceLock<Option<TelemetryStore>> = OnceLock::new();
static LOG_RING: Lazy<Mutex<VecDeque<String>>> = Lazy::new(|| Mutex::new(VecDeque::new()));
static LAST_SEND_ERROR: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
/// 运行期是否真正启用（总开关 + 配置齐全 + 用户未关闭）；关闭时所有入口零开销直接返回
static ACTIVE: AtomicBool = AtomicBool::new(false);

fn store() -> Option<&'static TelemetryStore> {
    STORE.get_or_init(TelemetryStore::open).as_ref()
}

/// 重新计算运行期启用状态（未配置时直接短路，不触碰磁盘）
fn refresh_active() {
    if !super::config::is_configured() {
        ACTIVE.store(false, Ordering::Relaxed);
        return;
    }
    let active = store().map(|s| s.is_enabled()).unwrap_or(false);
    ACTIVE.store(active, Ordering::Relaxed);
}

fn is_active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

/// 记录一行日志（供日志尾部上下文使用）
pub fn push_log_line(line: &str) {
    if !is_active() {
        return;
    }
    if let Ok(mut ring) = LOG_RING.lock() {
        ring.push_back(line.trim_end().to_string());
        while ring.len() > LOG_RING_LINES {
            ring.pop_front();
        }
    }
}

fn log_tail() -> String {
    LOG_RING
        .lock()
        .map(|ring| ring.iter().cloned().collect::<Vec<_>>().join("\n"))
        .unwrap_or_default()
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn remember_send_error(message: &str) {
    if let Ok(mut slot) = LAST_SEND_ERROR.lock() {
        *slot = Some(message.to_string());
    }
}

fn clear_send_error() {
    if let Ok(mut slot) = LAST_SEND_ERROR.lock() {
        *slot = None;
    }
}

/// 入队（去重 + 限流后）
fn enqueue(event: TelemetryEvent, window_secs: i64) {
    let Some(store) = store() else {
        return;
    };
    let now = chrono::Local::now().timestamp();
    let date = today();
    if !store.should_send(&event.signature, now, &date, window_secs) {
        return;
    }
    store.enqueue(&event);
    store.mark_sent(&event.signature, now, &date);
}

/// 记录一条 ERROR 日志事件
pub fn record_error(module: &str, message: &str) {
    if !is_active() {
        return;
    }
    let event = TelemetryEvent::new(EventKind::Error, module, message).with_log_tail(&log_tail());
    enqueue(event, DEDUP_WINDOW_SECS);
}

/// 记录一次 panic
pub fn record_panic(message: &str, location: &str, stack: &str) {
    if !is_active() {
        return;
    }
    let event = TelemetryEvent::new(EventKind::Panic, location, message)
        .with_log_tail(&log_tail())
        .with_stack(stack);
    enqueue(event, DEDUP_WINDOW_SECS);
}

/// 记录一次启动（同一版本每设备只报一次，用于版本分布统计）
pub fn record_startup() {
    if !is_active() {
        return;
    }
    let module = format!("startup:{}", env!("CARGO_PKG_VERSION"));
    let event = TelemetryEvent::new(EventKind::Startup, &module, "launcher started");
    enqueue(event, STARTUP_WINDOW_SECS);
}

/// 是否允许上报
pub fn is_enabled() -> bool {
    store().map(|s| s.is_enabled()).unwrap_or(false)
}

/// 设置是否允许上报
pub fn set_enabled(enabled: bool) {
    if let Some(store) = store() {
        store.set_enabled(enabled);
    }
    refresh_active();
}

/// 上报队列中的事件，返回成功条数
pub async fn flush() -> usize {
    if !is_active() {
        return 0;
    }
    let Some(store) = store() else {
        return 0;
    };
    if !store.is_enabled() {
        return 0;
    }
    let Some(sink) = SlsSink::new() else {
        return 0;
    };
    let lines = store.read_queue();
    if lines.is_empty() {
        return 0;
    }

    let device_id = store.device_id();
    let mut remaining: Vec<String> = Vec::new();
    let mut sent = 0usize;
    for line in lines {
        let Ok(event) = serde_json::from_str::<TelemetryEvent>(&line) else {
            // 损坏行直接丢弃，避免卡住队列
            continue;
        };
        match sink.send(&event.to_payload(&device_id)).await {
            Ok(()) => sent += 1,
            Err(e) => {
                remember_send_error(&e);
                remaining.push(line);
            }
        }
    }
    if sent > 0 {
        clear_send_error();
    }
    store.write_queue(&remaining);
    sent
}

/// 遥测状态（供设置页展示）
pub fn status() -> serde_json::Value {
    let configured = super::config::is_configured();
    let pending = if configured {
        store().map(|s| s.pending_count()).unwrap_or(0)
    } else {
        0
    };
    let device_id = if configured {
        store().map(|s| s.device_id()).unwrap_or_default()
    } else {
        String::new()
    };
    let last_error = LAST_SEND_ERROR.lock().ok().and_then(|slot| slot.clone());
    serde_json::json!({
        "enabled": is_enabled(),
        "active": is_active(),
        "configured": configured,
        "pending": pending,
        "deviceId": device_id,
        "lastError": last_error,
    })
}

/// 启动后台上报任务（每 FLUSH_INTERVAL_SECS 秒尝试发送一次）
pub fn spawn_background() {
    refresh_active();
    if !is_active() {
        // 未启用：不建目录、不入队、不起定时任务
        return;
    }
    // 预热存储与设备 ID，避免首次 record 时才建目录
    let _ = store().map(|s| s.device_id());
    record_startup();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(FLUSH_INTERVAL_SECS)).await;
            let _ = flush().await;
        }
    });
}
