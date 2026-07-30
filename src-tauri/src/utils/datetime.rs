//! 时间解析与格式化工具
//!
//! 提供统一的时间字符串解析函数，支持 RFC3339、naive datetime（T/空格分隔）、
//! 纯日期格式，naive 格式视为 UTC。

use chrono::{DateTime, Local, TimeZone, Utc};

/// 解析时间字符串为 UTC DateTime
///
/// 尝试依次按 RFC3339、naive datetime（T 分隔）、naive datetime（空格分隔）、
/// 纯日期格式解析。所有 naive 格式均视为 UTC。
///
/// 解析失败返回 `None`。
pub fn parse_utc(s: &str) -> Option<DateTime<Utc>> {
    // 1. 尝试 RFC3339（带时区）
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }

    // 2. 尝试 naive datetime（T 分隔，如 "2023-09-08T12:00:00"）
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Some(Utc.from_utc_datetime(&naive));
    }

    // 3. 尝试 naive datetime（空格分隔，如 "2023-09-08 12:00:00"）
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&naive));
    }

    // 4. 尝试纯日期格式（如 "2023-09-08"）
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        if let Some(naive_dt) = naive_date.and_hms_opt(0, 0, 0) {
            return Some(Utc.from_utc_datetime(&naive_dt));
        }
    }

    None
}

/// 将 UTC 时间字符串格式化为本地时间字符串
///
/// 格式：`YYYY/MM/DD HH:MM`（纯日期格式为 `YYYY/MM/DD`）
///
/// 解析失败返回 `None`。
pub fn format_utc_to_local(s: &str) -> Option<String> {
    let dt = parse_utc(s)?;
    let local = dt.with_timezone(&Local);
    Some(local.format("%Y/%m/%d %H:%M").to_string())
}
