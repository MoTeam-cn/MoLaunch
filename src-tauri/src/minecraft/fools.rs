//! 愚人节版本检测模块
//!
//! 通过硬编码列表 + 日期兜底识别 Minecraft 愚人节版本。

/// 已知的愚人节版本映射表
/// (版本ID, 基于的MC版本, 年份, 描述)
const KNOWN_FOOLS: &[(&str, &str, u16, &str)] = &[
    ("15w14a", "1.8.3", 2015, "和平、爱与拥抱"),
    ("1.rv-pre1", "1.9.2", 2016, "现代科技"),
    ("3D Shareware v1.34", "1.13.2", 2019, "1994年的杰作"),
    ("20w14infinite", "1.15.2", 2020, "20亿个新维度"),
    ("22w13oneblockatatime", "1.18.2", 2022, "一次一个方块"),
    ("23w13a_or_b", "1.19.4", 2023, "越多越好的选择"),
    ("24w14potato", "1.20.4", 2024, "毒马铃薯超级加强"),
    ("25w14craftmine", "1.21.4", 2025, "合成任何东西"),
    ("26w14a", "26.1.1", 2026, "方块跟着你走"),
];

/// 愚人节版本信息
pub struct FoolInfo {
    /// 基于的 MC 版本
    pub base_version: String,
    /// 年份
    pub year: u16,
    /// 趣味描述
    pub description: String,
}

/// 判断版本是否为愚人节版本
///
/// 三重判定：
/// 1. 版本名在硬编码列表中
/// 2. type 为 snapshot 且发布日期为 4月1日（UTC+2 瑞典时间）
/// 3. 版本名包含已知愚人节关键词
pub fn detect_fool(id: &str, version_type: &str, release_time: &str) -> Option<FoolInfo> {
    // 1. 硬编码列表匹配
    for &(fool_id, base, year, desc) in KNOWN_FOOLS {
        if id == fool_id || id.eq_ignore_ascii_case(fool_id) {
            return Some(FoolInfo {
                base_version: base.to_string(),
                year,
                description: desc.to_string(),
            });
        }
    }

    // 2. 日期兜底：snapshot 类型 + 4月1日（UTC+2）
    if version_type == "snapshot" {
        if let Some(dt) = parse_april_fools_date(release_time) {
            use chrono::Datelike;
            return Some(FoolInfo {
                base_version: String::new(),
                year: dt.year() as u16,
                description: "愚人节版本".to_string(),
            });
        }
    }

    None
}

/// 解析时间为 UTC+2，返回 NaiveDateTime（仅当日期为 4月1日）
fn parse_april_fools_date(time_str: &str) -> Option<chrono::NaiveDateTime> {
    use chrono::{TimeZone, Datelike};

    let utc_plus_2 = chrono::FixedOffset::east_opt(2 * 3600)?;

    let local = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(time_str) {
        dt.with_timezone(&utc_plus_2)
    } else if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S") {
        chrono::Utc.from_utc_datetime(&naive).with_timezone(&utc_plus_2)
    } else {
        return None;
    };

    if local.month() == 4 && local.day() == 1 {
        Some(local.naive_local())
    } else {
        None
    }
}

/// 获取愚人节版本的显示描述（格式：年份 | 描述）
pub fn get_fool_description(id: &str) -> Option<String> {
    for &(fool_id, _base, year, desc) in KNOWN_FOOLS {
        if id == fool_id || id.eq_ignore_ascii_case(fool_id) {
            return Some(format!("{} | {}", year, desc));
        }
    }
    None
}
