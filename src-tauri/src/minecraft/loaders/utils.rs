//! Loader 工具函数模块

/// 解析版本号为可比较的元组
///
/// 例如 "1.20.1" -> [1, 20, 1]
pub fn parse_version_number(version: &str) -> Vec<u32> {
    version.split('.').filter_map(|s| s.parse().ok()).collect()
}

/// 将 UTC 时间字符串转换为本地时间格式
///
/// 支持格式：
/// - RFC3339/ISO8601: "2023-09-08T12:00:00+08:00" 或 "2023-09-08T12:00:00Z"
/// - Naive datetime (视为 UTC): "2023-09-08 12:00:00"
/// - Date only (视为 UTC 00:00): "2023-09-08"
pub fn parse_utc_to_local(s: &str) -> Option<String> {
    use chrono::{Local, TimeZone};

    // 尝试 RFC3339（带时区）
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        let local = dt.with_timezone(&Local);
        return Some(local.format("%Y/%m/%d %H:%M").to_string());
    }

    // 尝试 naive datetime（视为 UTC）
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        let utc_dt = chrono::Utc.from_utc_datetime(&naive);
        let local = utc_dt.with_timezone(&Local);
        return Some(local.format("%Y/%m/%d %H:%M").to_string());
    }

    // 尝试日期格式 "2023-09-08"
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let naive_dt = naive_date.and_hms_opt(0, 0, 0)?;
        let utc_dt = chrono::Utc.from_utc_datetime(&naive_dt);
        let local = utc_dt.with_timezone(&Local);
        return Some(local.format("%Y/%m/%d").to_string());
    }

    None
}
