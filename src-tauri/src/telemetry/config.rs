//! 遥测上报配置：SLS WebTracking 端点与各项上限
//!
//! 免费额度按写入流量计（每月 500MB），因此所有上限都偏保守：只报错误与崩溃，
//! 同签名去重、每日限流、单条字段截断。

/// SLS 区域域名（阿里云日志服务）
pub const SLS_ENDPOINT: &str = "cn-chengdu.log.aliyuncs.com";

/// SLS Project 名（控制台可见，填好后即可上报）
pub const SLS_PROJECT: &str = "";

/// SLS Logstore 名（需在控制台开启 WebTracking）
pub const SLS_LOGSTORE: &str = "";

/// 日志主题（SLS 的 __topic__ 字段）
pub const SLS_TOPIC: &str = "launcher-error";

/// 单设备每日最多上报条数
pub const MAX_EVENTS_PER_DAY: u32 = 8;

/// 同签名去重窗口（秒）：6 小时内同一错误只报一次
pub const DEDUP_WINDOW_SECS: i64 = 6 * 3600;

/// 本地待发送队列上限（条），超出丢弃最旧
pub const MAX_QUEUE_LINES: usize = 60;

/// message 字段最大字符数
pub const MAX_MESSAGE_CHARS: usize = 400;

/// 日志尾部最大字符数
pub const MAX_LOG_TAIL_CHARS: usize = 1200;

/// panic 堆栈最大字符数
pub const MAX_STACK_CHARS: usize = 900;

/// 日志尾部最多保留行数（供后续事件复用）
pub const LOG_RING_LINES: usize = 25;

/// 上报请求超时（秒）
pub const SEND_TIMEOUT_SECS: u64 = 8;

/// 后台 flush 间隔（秒）
pub const FLUSH_INTERVAL_SECS: u64 = 90;

/// 是否已配置 Project 与 Logstore（未配置时只入队不上报）
pub fn is_configured() -> bool {
    !SLS_PROJECT.is_empty() && !SLS_LOGSTORE.is_empty()
}

/// WebTracking 写入地址（无需签名，只能写不能读）
pub fn track_url() -> String {
    format!(
        "https://{}.{}/logstores/{}/track?APIVersion=0.6.0",
        SLS_PROJECT, SLS_ENDPOINT, SLS_LOGSTORE
    )
}
