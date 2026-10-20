//! region 判定模块单元测试
//!
//! 只测纯逻辑（策略判定 / 序列化 / 兜底文案）；网络与机器层依赖运行环境，不在单测覆盖。

use super::*;
use crate::utils::download_policy::{
    DownloadPolicy, DownloadRulesPolicy, PolicyNotice, RegionCheckPolicy,
};

fn sample_info() -> RegionInfo {
    RegionInfo {
        is_china_mainland_machine: true,
        is_china_mainland_network: true,
        country_code: Some("CN".to_string()),
        is_restricted: true,
        blocked_regions: vec!["CN".to_string()],
        notice: PolicyNotice {
            title: "下载功能说明".to_string(),
            text: "暂不提供下载".to_string(),
            confirm_text: "我知道了".to_string(),
        },
        allow_developer_mode_bypass: true,
        developer_mode: false,
        bypassed: false,
    }
}

#[test]
fn test_region_info_serializes_camel_case_fields() {
    let value = serde_json::to_value(sample_info()).expect("序列化 RegionInfo 失败");

    assert_eq!(value["isChinaMainlandMachine"], true);
    assert_eq!(value["isChinaMainlandNetwork"], true);
    assert_eq!(value["countryCode"], "CN");
    assert_eq!(value["isRestricted"], true);
    assert_eq!(value["blockedRegions"][0], "CN");
    assert_eq!(value["notice"]["title"], "下载功能说明");
    assert_eq!(value["notice"]["text"], "暂不提供下载");
    assert_eq!(value["notice"]["confirmText"], "我知道了");
    assert_eq!(value["allowDeveloperModeBypass"], true);
    assert_eq!(value["developerMode"], false);
    assert_eq!(value["bypassed"], false);
}

#[test]
fn test_only_china_blocked_matches_single_cn_entry() {
    assert!(only_china_blocked(&["CN".to_string()]));
    assert!(only_china_blocked(&["cn".to_string()]));
}

#[test]
fn test_only_china_blocked_rejects_other_lists() {
    assert!(!only_china_blocked(&[]));
    assert!(!only_china_blocked(&["CN".to_string(), "HK".to_string()]));
    assert!(!only_china_blocked(&["HK".to_string()]));
}

/// 受限地区为 CN + HK 的策略（用于国家判定测试）
fn blocked_policy() -> DownloadPolicy {
    DownloadPolicy {
        enabled: true,
        region_check: RegionCheckPolicy {
            enabled: true,
            blocked_regions: vec!["CN".to_string(), "HK".to_string()],
        },
        download: DownloadRulesPolicy {
            disabled_for_blocked_regions: true,
            allow_developer_mode_bypass: false,
        },
        ..DownloadPolicy::default()
    }
}

#[test]
fn test_restricted_by_country_matches_configured_regions() {
    let policy = blocked_policy();
    assert!(restricted_by_country(&policy, Some("CN")));
    assert!(restricted_by_country(&policy, Some("cn")));
    assert!(restricted_by_country(&policy, Some("HK")));
    assert!(!restricted_by_country(&policy, Some("JP")));
    assert!(!restricted_by_country(&policy, Some("US")));
}

#[test]
fn test_restricted_by_country_fails_closed_when_country_missing() {
    // ipinfo.io 未返回 country（限流 / 响应异常）：无法判定，按受限处理
    assert!(restricted_by_country(&blocked_policy(), None));
}

#[test]
fn test_restricted_by_country_respects_download_switch() {
    // 策略关闭「命中受限地区禁用下载」：即便国家在列表内也放行
    let policy = DownloadPolicy {
        download: DownloadRulesPolicy {
            disabled_for_blocked_regions: false,
            allow_developer_mode_bypass: false,
        },
        ..blocked_policy()
    };
    assert!(!restricted_by_country(&policy, Some("CN")));
}

#[test]
fn test_fallback_notice_text_keeps_apology_copy() {
    assert!(FALLBACK_NOTICE_TEXT.contains("暂不向中国用户提供"));
    assert!(FALLBACK_NOTICE_TEXT.contains("恕我无能为力"));
}

#[test]
fn test_default_policy_keeps_download_allowed() {
    // 无缓存 / 拉取失败时策略为默认值：不检测地区、不禁用下载
    let policy = DownloadPolicy::default();
    assert!(!policy.region_check_active());
    assert!(!policy.blocks_region("CN"));
}
