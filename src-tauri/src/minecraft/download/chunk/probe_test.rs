//! Range 探测的纯逻辑测试（不依赖网络）

use super::parse_content_range_total;

#[test]
fn test_parse_content_range_取出总长度() {
    assert_eq!(parse_content_range_total(Some("bytes 0-0/1401")), 1401);
    assert_eq!(parse_content_range_total(Some("bytes 0-0/0")), 0);
}

#[test]
fn test_parse_content_range_无法解析时返回零() {
    assert_eq!(parse_content_range_total(Some("bytes 0-0/*")), 0);
    assert_eq!(parse_content_range_total(Some("garbage")), 0);
    assert_eq!(parse_content_range_total(None), 0);
}
