//! hongshic 内核输出解析单元测试（endpoint 提取与拆分）

use super::{is_version_rejection, parse_endpoint, split_endpoint};

/// 官方成功输出：INFO 行里带 endpoint= 字段
#[test]
fn test_parse_endpoint_官方成功输出() {
    let line = "2026-10-11T10:55:04.575424Z  INFO hongshi::client: tunnel created, \
                hand the endpoint above to the players uuid=58925466-3b77-4daf-8153-f450f12e782b \
                port=41269 endpoint=nj.hongshi.site:41269";
    assert_eq!(
        parse_endpoint(line).as_deref(),
        Some("nj.hongshi.site:41269")
    );
}

/// 提示语里的 endpoint 单词（无等号）不能被误当成地址
#[test]
fn test_parse_endpoint_忽略无等号的_endpoint_单词() {
    let line = "INFO hongshi::client: hand the endpoint above to the players";
    assert_eq!(parse_endpoint(line), None);
}

/// endpoint= 后面紧跟空白视为无值
#[test]
fn test_parse_endpoint_空值返回_none() {
    assert_eq!(parse_endpoint("info endpoint=   uuid=x"), None);
    assert_eq!(parse_endpoint("info endpoint="), None);
}

/// 错误行不产生地址
#[test]
fn test_parse_endpoint_错误行返回_none() {
    let line = "[ERROR] version check failed: 客户端版本过时，请更新到最新版本";
    assert_eq!(parse_endpoint(line), None);
}

/// 旧内核被服务端拒绝时的输出应被识别（触发自动重新下载）
#[test]
fn test_is_version_rejection_识别版本过期() {
    assert!(is_version_rejection(
        "2026/10/11 18:54:22 [ERROR] version check failed: 客户端版本过时，请更新到最新版本"
    ));
    assert!(is_version_rejection("client version is outdated"));
}

/// 正常输出与空行不应误判为版本过期
#[test]
fn test_is_version_rejection_正常输出不误判() {
    assert!(!is_version_rejection(
        "INFO hongshi::client: tunnel created, hand the endpoint above to the players port=41269 endpoint=nj.hongshi.site:41269"
    ));
    assert!(!is_version_rejection(""));
}

/// host:port 正常拆分
#[test]
fn test_split_endpoint_正常拆分() {
    let (host, port) = split_endpoint("nj.hongshi.site:41269");
    assert_eq!(host, "nj.hongshi.site");
    assert_eq!(port, Some(41269));
}

/// 无端口时整体作为 host
#[test]
fn test_split_endpoint_无端口() {
    let (host, port) = split_endpoint("nj.hongshi.site");
    assert_eq!(host, "nj.hongshi.site");
    assert_eq!(port, None);
}

/// 端口非数字时回退为整体 host，不 panic
#[test]
fn test_split_endpoint_端口非法() {
    let (host, port) = split_endpoint("nj.hongshi.site:abc");
    assert_eq!(host, "nj.hongshi.site:abc");
    assert_eq!(port, None);
}
