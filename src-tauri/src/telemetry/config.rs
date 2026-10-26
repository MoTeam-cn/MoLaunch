//! 遥测上报配置：SLS 接入参数（编译期注入）与各项上限
//!
//! 三个接入参数（endpoint / project / logstore）来自 CI secrets，编译期通过
//! `option_env!` 注入，仓库内不出现任何真实值；本地构建为空即视为未配置。
//! 总开关 `TELEMETRY_ENABLED` 暂为 false（功能暂缓上线），需要启用时改 true 并配置 secrets。

/// 上报总开关（功能暂缓上线；启用时改为 true）
pub const TELEMETRY_ENABLED: bool = false;

const BUILD_ENDPOINT: Option<&str> = option_env!("MOLAUNCH_SLS_ENDPOINT");
const BUILD_PROJECT: Option<&str> = option_env!("MOLAUNCH_SLS_PROJECT");
const BUILD_LOGSTORE: Option<&str> = option_env!("MOLAUNCH_SLS_LOGSTORE");

/// SLS 区域域名（编译期注入）
pub fn endpoint() -> &'static str {
    BUILD_ENDPOINT.unwrap_or("")
}

/// SLS Project 名（编译期注入）
pub fn project() -> &'static str {
    BUILD_PROJECT.unwrap_or("")
}

/// SLS Logstore 名（编译期注入，需在控制台开启 WebTracking）
pub fn logstore() -> &'static str {
    BUILD_LOGSTORE.unwrap_or("")
}

/// 日志主题（SLS 的 __topic__ 字段）
pub const SLS_TOPIC: &str = "launcher-error";

/// 单设备每日最多上报条数
pub const MAX_EVENTS_PER_DAY: u32 = 5;

/// 同签名去重窗口（秒）：6 小时内同一错误只报一次
pub const DEDUP_WINDOW_SECS: i64 = 6 * 3600;

/// 本地待发送队列上限（条），超出丢弃最旧
pub const MAX_QUEUE_LINES: usize = 60;

/// message 字段最大字符数
pub const MAX_MESSAGE_CHARS: usize = 300;

/// 日志尾部最大字符数（保留最近若干行）
pub const MAX_LOG_TAIL_CHARS: usize = 800;

/// panic 堆栈最大字符数
pub const MAX_STACK_CHARS: usize = 700;

/// 日志环形缓冲保留行数（供后续事件复用）
pub const LOG_RING_LINES: usize = 25;

/// 上报请求超时（秒）
pub const SEND_TIMEOUT_SECS: u64 = 8;

/// 后台 flush 间隔（秒）
pub const FLUSH_INTERVAL_SECS: u64 = 90;

/// 是否已具备上报条件（总开关打开 + 三个参数齐全）
pub fn is_configured() -> bool {
    TELEMETRY_ENABLED && !endpoint().is_empty() && !project().is_empty() && !logstore().is_empty()
}

/// WebTracking 写入地址（无需签名，只能写不能读）
pub fn track_url() -> String {
    format!(
        "https://{}.{}/logstores/{}/track?APIVersion=0.6.0",
        project(),
        endpoint(),
        logstore()
    )
}
