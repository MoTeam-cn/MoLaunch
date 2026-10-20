//! region 判定模块单元测试

use super::*;

#[test]
fn test_region_info_serializes_camel_case_fields() {
    let info = RegionInfo {
        is_china_mainland_machine: true,
        is_china_mainland_network: false,
        country_code: Some("CN".to_string()),
        is_restricted: false,
    };

    let value = serde_json::to_value(&info).expect("序列化 RegionInfo 失败");

    assert_eq!(value["isChinaMainlandMachine"], true);
    assert_eq!(value["isChinaMainlandNetwork"], false);
    assert_eq!(value["countryCode"], "CN");
    assert_eq!(value["isRestricted"], false);
}

#[test]
fn test_restricted_from_body_marks_cn_as_restricted() {
    assert!(restricted_from_body(Some("CN")));
    assert!(restricted_from_body(Some("cn")));
}

#[test]
fn test_restricted_from_body_keeps_other_countries_allowed() {
    assert!(!restricted_from_body(Some("HK")));
    assert!(!restricted_from_body(Some("TW")));
    assert!(!restricted_from_body(Some("JP")));
    assert!(!restricted_from_body(Some("US")));
}

#[test]
fn test_restricted_from_body_fails_closed_when_country_missing() {
    // ipinfo.io 未返回 country（限流 / 响应异常）：无法判定，按受限处理
    assert!(restricted_from_body(None));
}

#[test]
fn test_restricted_verdict_has_no_country_code() {
    let verdict = restricted_verdict();
    assert!(verdict.restricted);
    assert!(verdict.country_code.is_none());
}

#[test]
fn test_restricted_message_keeps_apology_copy() {
    assert!(RESTRICTED_MESSAGE.contains("暂不向中国用户提供"));
    assert!(RESTRICTED_MESSAGE.contains("恕我无能为力"));
}
