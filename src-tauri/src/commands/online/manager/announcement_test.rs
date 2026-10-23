//! 公告响应解析测试
//!
//! 覆盖服务端 data 包裹形态（items 数组）与旧形态（data 直接为数组），
//! 以及结构异常与单条字段缺失时的降级行为。

use super::extract_items;

fn sample(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "title": "标题",
        "content": "正文",
        "level": "info",
        "priority": 0,
        "starts_at": 0,
        "ends_at": 0,
        "link_url": "",
        "dismissible": true
    })
}

#[test]
fn test_extract_items_服务端_items_包裹时能取出全部公告() {
    let json = serde_json::json!({
        "code": 1,
        "data": { "items": [sample("a"), sample("b")] }
    });
    let list = extract_items(&json).expect("应能取出公告");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id, "a");
    assert_eq!(list[1].id, "b");
}

#[test]
fn test_extract_items_data_直接为数组时同样能取出() {
    let json = serde_json::json!({ "code": 1, "data": [sample("only")] });
    let list = extract_items(&json).expect("应能取出公告");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "only");
}

#[test]
fn test_extract_items_结构异常时返回_none() {
    let no_items = serde_json::json!({ "code": 1, "data": { "other": [] } });
    assert!(extract_items(&no_items).is_none());
    let no_data = serde_json::json!({ "code": 1 });
    assert!(extract_items(&no_data).is_none());
}

#[test]
fn test_extract_items_单条字段缺失时只跳过该条() {
    let broken = serde_json::json!({ "id": "broken" });
    let json = serde_json::json!({ "code": 1, "data": { "items": [sample("ok"), broken] } });
    let list = extract_items(&json).expect("应能取出公告");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "ok");
}

#[test]
fn test_extract_items_空列表时返回空数组() {
    let json = serde_json::json!({ "code": 1, "data": { "items": [] } });
    let list = extract_items(&json).expect("空列表也应成功");
    assert!(list.is_empty());
}
