//! 遥测事件结构与字段截断

use serde::{Deserialize, Serialize};

use super::config::{MAX_LOG_TAIL_CHARS, MAX_MESSAGE_CHARS, MAX_STACK_CHARS};
use super::signature::{sanitize_text, signature};

/// 事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// ERROR 级别日志
    Error,
    /// panic 崩溃
    Panic,
    /// 新版本首次启动（用于统计版本分布，每版本每设备一次）
    Startup,
}

impl EventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Panic => "panic",
            Self::Startup => "startup",
        }
    }
}

/// 一条待上报事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub kind: EventKind,
    /// 来源模块（相对路径:行号）
    pub module: String,
    /// 脱敏截断后的信息
    pub message: String,
    /// 聚合用签名（模块 + 归一化信息）
    pub signature: String,
    /// 最近日志尾部
    pub log_tail: String,
    /// panic 堆栈
    pub stack: String,
    /// 创建时间（Unix 秒）
    pub created_at: i64,
}

impl TelemetryEvent {
    /// 构造事件：自动脱敏、截断并计算签名
    pub fn new(kind: EventKind, module: &str, message: &str) -> Self {
        let clean = sanitize_text(message);
        Self {
            kind,
            module: truncate_chars(module, 120),
            signature: signature(module, &clean),
            message: truncate_chars(&clean, MAX_MESSAGE_CHARS),
            log_tail: String::new(),
            stack: String::new(),
            created_at: chrono::Local::now().timestamp(),
        }
    }

    /// 附加日志尾部（保留最后 N 个字符 —— 最近发生的才有诊断价值）
    pub fn with_log_tail(mut self, tail: &str) -> Self {
        self.log_tail = truncate_tail_chars(tail, MAX_LOG_TAIL_CHARS);
        self
    }

    /// 附加 panic 堆栈（自动截断）
    pub fn with_stack(mut self, stack: &str) -> Self {
        self.stack = truncate_chars(stack, MAX_STACK_CHARS);
        self
    }

    /// 转为 SLS WebTracking 请求体（保留字段 + 业务字段）
    pub fn to_payload(&self, device_id: &str) -> serde_json::Value {
        serde_json::json!({
            "__topic__": super::config::SLS_TOPIC,
            "__source__": device_id,
            "__time__": self.created_at,
            "kind": self.kind.as_str(),
            "device_id": device_id,
            "app_version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "module": self.module,
            "signature": self.signature,
            "message": self.message,
            "log_tail": self.log_tail,
            "stack": self.stack,
        })
    }
}

/// 按字符保留**末尾**（日志尾部用，最近几行才有诊断价值）
pub fn truncate_tail_chars(text: &str, max_chars: usize) -> String {
    let total = text.chars().count();
    if total <= max_chars {
        return text.to_string();
    }
    let skip = total - max_chars;
    let mut out = String::from("…");
    out.extend(text.chars().skip(skip));
    out
}

/// 按字符截断（避免切断 UTF-8）
pub fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max_chars).collect();
    out.push('…');
    out
}
