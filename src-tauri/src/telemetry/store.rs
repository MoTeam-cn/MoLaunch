//! 本地遥测状态：设备 ID、开关、去重表、待发送队列（JSONL）

use std::collections::HashMap;
use std::path::PathBuf;

use super::config::{MAX_EVENTS_PER_DAY, MAX_QUEUE_LINES};
use super::types::TelemetryEvent;

/// 本地状态文件结构
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct PersistedState {
    /// 是否允许上报（默认开启）
    enabled: bool,
    /// 当日计数所属日期（YYYY-MM-DD）
    daily_date: String,
    /// 当日已上报条数
    daily_count: u32,
    /// 签名 -> 最近上报时间戳
    sent: HashMap<String, i64>,
}

/// 遥测本地存储
pub struct TelemetryStore {
    dir: PathBuf,
}

impl TelemetryStore {
    /// 打开存储（目录不可用时返回 None，遥测整体降级为不工作）
    pub fn open() -> Option<Self> {
        // 注意：这里不能调用会写日志的 ensure_appdata_subdir —— record_error 由 logger
        // 持锁调用，任何 log_* 都会重入 LOGGER 造成死锁，故直接建目录。
        let dir = crate::storage::appdata::appdata_subdir("telemetry").ok()?;
        std::fs::create_dir_all(&dir).ok()?;
        Some(Self { dir })
    }

    fn state_path(&self) -> PathBuf {
        self.dir.join("state.json")
    }

    fn queue_path(&self) -> PathBuf {
        self.dir.join("pending.jsonl")
    }

    fn device_path(&self) -> PathBuf {
        self.dir.join("device_id")
    }

    /// 读持久化状态
    ///
    /// 文件缺失或解析失败时一律视为「未同意」：遥测必须由用户自己打开。
    /// 因为读不到设置就替他做决定，等于把默认值当成了同意。
    fn read_state(&self) -> PersistedState {
        std::fs::read_to_string(self.state_path())
            .ok()
            .and_then(|text| serde_json::from_str::<PersistedState>(&text).ok())
            .unwrap_or_default()
    }

    fn write_state(&self, state: &PersistedState) {
        if let Ok(text) = serde_json::to_string(state) {
            let _ = std::fs::write(self.state_path(), text);
        }
    }

    /// 匿名设备 ID（随机生成后持久化，不采集硬件信息）
    pub fn device_id(&self) -> String {
        if let Ok(existing) = std::fs::read_to_string(self.device_path()) {
            let trimmed = existing.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let id: String = (0..32)
            .map(|_| {
                let value: u8 = rng.gen_range(0..16);
                char::from_digit(u32::from(value), 16).unwrap_or('0')
            })
            .collect();
        let _ = std::fs::write(self.device_path(), &id);
        id
    }

    /// 是否允许上报
    pub fn is_enabled(&self) -> bool {
        self.read_state().enabled
    }

    /// 设置是否允许上报
    pub fn set_enabled(&self, enabled: bool) {
        let mut state = self.read_state();
        state.enabled = enabled;
        self.write_state(&state);
    }

    /// 判断是否应当上报（去重窗口 + 每日上限）
    pub fn should_send(&self, signature: &str, now: i64, today: &str, window_secs: i64) -> bool {
        let state = self.read_state();
        if !state.enabled {
            return false;
        }
        if let Some(last) = state.sent.get(signature) {
            if now - *last < window_secs {
                return false;
            }
        }
        let count = if state.daily_date == today {
            state.daily_count
        } else {
            0
        };
        count < MAX_EVENTS_PER_DAY
    }

    /// 记录一次上报（更新去重表与当日计数）
    pub fn mark_sent(&self, signature: &str, now: i64, today: &str) {
        let mut state = self.read_state();
        if state.daily_date != today {
            state.daily_date = today.to_string();
            state.daily_count = 0;
        }
        state.daily_count = state.daily_count.saturating_add(1);
        state.sent.insert(signature.to_string(), now);
        self.write_state(&state);
    }

    /// 事件入队（超出上限丢弃最旧）
    pub fn enqueue(&self, event: &TelemetryEvent) {
        let Ok(line) = serde_json::to_string(event) else {
            return;
        };
        let mut lines = self.read_queue();
        lines.push(line);
        if lines.len() > MAX_QUEUE_LINES {
            let overflow = lines.len() - MAX_QUEUE_LINES;
            lines.drain(0..overflow);
        }
        self.write_queue(&lines);
    }

    /// 读取队列原始行
    pub fn read_queue(&self) -> Vec<String> {
        std::fs::read_to_string(self.queue_path())
            .map(|text| {
                text.lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 重写队列（保留未发送部分）
    pub fn write_queue(&self, lines: &[String]) {
        let mut text = String::new();
        for line in lines {
            text.push_str(line);
            text.push('\n');
        }
        let _ = std::fs::write(self.queue_path(), text);
    }

    /// 队列长度
    pub fn pending_count(&self) -> usize {
        self.read_queue().len()
    }
}

#[cfg(test)]
#[path = "store_test.rs"]
mod store_test;
