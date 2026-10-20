//! download_policy 解析与判定单元测试

use super::*;

/// 官网下发的样例清单（字段齐全）
const SAMPLE: &str = r#"{
  "enabled": true,
  "version": 2,
  "updatedAt": "2026-10-20",
  "regionCheck": { "enabled": true, "blockedRegions": ["CN"] },
  "download": { "disabledForBlockedRegions": true, "allowDeveloperModeBypass": true },
  "notice": { "title": "下载功能说明", "text": "暂不提供下载", "confirmText": "我知道了" }
}"#;

#[test]
fn test_parse_full_policy() {
    let policy: DownloadPolicy = serde_json::from_str(SAMPLE).expect("解析样例清单失败");
    assert!(policy.enabled);
    assert_eq!(policy.version, 2);
    assert_eq!(policy.updated_at, "2026-10-20");
    assert!(policy.region_check.enabled);
    assert_eq!(policy.blocked_regions(), vec!["CN".to_string()]);
    assert!(policy.download.disabled_for_blocked_regions);
    assert!(policy.download.allow_developer_mode_bypass);
    assert_eq!(policy.notice.title, "下载功能说明");
    assert_eq!(policy.notice.text, "暂不提供下载");
    assert_eq!(policy.notice.confirm_text, "我知道了");
}

#[test]
fn test_default_policy_means_no_restriction() {
    let policy = DownloadPolicy::default();
    assert!(!policy.enabled);
    assert!(!policy.region_check_active());
    assert!(!policy.blocks_region("CN"));
}

#[test]
fn test_missing_fields_fall_back_to_default() {
    // 只给 enabled：其余字段缺失按默认值处理，不报错
    let policy: DownloadPolicy =
        serde_json::from_str(r#"{ "enabled": true }"#).expect("缺失字段应可解析");
    assert!(policy.enabled);
    assert_eq!(policy.version, 0);
    assert!(policy.region_check.blocked_regions.is_empty());
    assert!(!policy.region_check_active());
    assert!(!policy.download.allow_developer_mode_bypass);
    assert!(policy.notice.text.is_empty());
}

#[test]
fn test_unknown_fields_are_ignored() {
    let json = r#"{
      "enabled": true,
      "futureField": { "nested": [1, 2, 3] },
      "regionCheck": { "enabled": true, "blockedRegions": ["CN"], "anotherNew": true }
    }"#;
    let policy: DownloadPolicy = serde_json::from_str(json).expect("未知字段不应导致解析失败");
    assert!(policy.region_check_active());
}

#[test]
fn test_blocked_regions_are_normalized() {
    let json = r#"{
      "enabled": true,
      "regionCheck": { "enabled": true, "blockedRegions": [" cn ", "CN", "hk", "", "C", "CHN", "1x"] }
    }"#;
    let policy: DownloadPolicy = serde_json::from_str(json).expect("解析失败");
    // 大写归一 + 去重 + 丢弃非法值（空串 / 单字符 / 三字符 / 含数字）
    assert_eq!(
        policy.blocked_regions(),
        vec!["CN".to_string(), "HK".to_string()]
    );
}

/// 构造只开启地区检测开关的策略（受限列表为空）
fn policy_with_region_check() -> DownloadPolicy {
    DownloadPolicy {
        enabled: true,
        region_check: RegionCheckPolicy {
            enabled: true,
            blocked_regions: Vec::new(),
        },
        ..DownloadPolicy::default()
    }
}

#[test]
fn test_region_check_active_requires_all_switches() {
    // 受限列表为空 → 不检测
    let mut policy = policy_with_region_check();
    assert!(!policy.region_check_active());

    // 三项齐备 → 检测
    policy.region_check.blocked_regions = vec!["CN".to_string()];
    assert!(policy.region_check_active());

    // 总开关关闭 → 不检测
    policy.enabled = false;
    assert!(!policy.region_check_active());

    // 地区检测关闭 → 不检测
    policy.enabled = true;
    policy.region_check.enabled = false;
    assert!(!policy.region_check_active());
}

#[test]
fn test_blocks_region_respects_download_switch() {
    // disabledForBlockedRegions 默认 false → 不禁用
    let mut policy = policy_with_region_check();
    policy.region_check.blocked_regions = vec!["CN".to_string()];
    assert!(!policy.blocks_region("CN"));

    policy.download.disabled_for_blocked_regions = true;
    assert!(policy.blocks_region("CN"));
    assert!(policy.blocks_region("cn"));
    assert!(!policy.blocks_region("HK"));

    // 空列表 → 永不命中
    policy.region_check.blocked_regions.clear();
    assert!(!policy.blocks_region("CN"));
}
