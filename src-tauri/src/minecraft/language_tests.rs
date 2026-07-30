//! language 单元测试

use super::*;

#[test]
fn test_adjust_lang_case() {
    // MC 1.0 ~ 1.10：后缀大写
    assert_eq!(adjust_lang_case("zh_cn", "1.0"), "zh_CN");
    assert_eq!(adjust_lang_case("zh_cn", "1.5.2"), "zh_CN");
    assert_eq!(adjust_lang_case("zh_cn", "1.10.2"), "zh_CN");
    assert_eq!(adjust_lang_case("en_us", "1.8.9"), "en_US");

    // MC 1.11+：小写
    assert_eq!(adjust_lang_case("zh_cn", "1.11.2"), "zh_cn");
    assert_eq!(adjust_lang_case("zh_cn", "1.12.2"), "zh_cn");
    assert_eq!(adjust_lang_case("zh_cn", "1.13.2"), "zh_cn");
    assert_eq!(adjust_lang_case("zh_cn", "1.20.1"), "zh_cn");
    assert_eq!(adjust_lang_case("zh_CN", "1.20.1"), "zh_cn");

    // MC 26+：小写
    assert_eq!(adjust_lang_case("zh_cn", "26.2"), "zh_cn");
    assert_eq!(adjust_lang_case("zh_cn", "27.1"), "zh_cn");

    // 无下划线的代码原样返回
    assert_eq!(adjust_lang_case("none", "1.20.1"), "none");
    assert_eq!(adjust_lang_case("auto", "1.20.1"), "auto");
}

#[test]
fn test_to_upper_suffix() {
    assert_eq!(to_upper_suffix("zh_cn"), "zh_CN");
    assert_eq!(to_upper_suffix("en_us"), "en_US");
    assert_eq!(to_upper_suffix("ja_jp"), "ja_JP");
    assert_eq!(to_upper_suffix("ko_kr"), "ko_KR");
    // 无下划线的原样返回
    assert_eq!(to_upper_suffix("none"), "none");
}
