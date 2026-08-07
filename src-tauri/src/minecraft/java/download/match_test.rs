//! Java Runtime 组件匹配回归测试。
//! 覆盖组件映射、平台匹配及缺失数据时的错误处理。

use serde_json::{json, Value};

use super::r#match::{component_key_for_major, match_component, platform_key};

fn entry_json(name: &str) -> Value {
    json!({
        "manifest": { "url": format!("https://example.com/{}.json", name) },
        "version": { "name": name }
    })
}

/// 构造包含全部组件的最小 all.json（平台 key 取当前编译平台，保证跨平台可测）
fn sample_index() -> Value {
    let platform = platform_key().expect("当前平台应支持 Java Runtime");
    json!({
        (platform): {
            "jre-legacy": [entry_json("8u51-b16")],
            "java-runtime-alpha": [entry_json("16.0.1.9.1")],
            "java-runtime-beta": [entry_json("17.0.15")],
            "java-runtime-gamma": [entry_json("17.0.15")],
            "java-runtime-delta": [entry_json("21.0.7")],
            "java-runtime-epsilon": [entry_json("25.0.1")]
        }
    })
}

#[test]
fn component_key_mapping_matches_official() {
    assert_eq!(component_key_for_major(8), Some("jre-legacy"));
    assert_eq!(component_key_for_major(16), Some("java-runtime-alpha"));
    assert_eq!(component_key_for_major(17), Some("java-runtime-gamma"));
    assert_eq!(component_key_for_major(21), Some("java-runtime-delta"));
    assert_eq!(component_key_for_major(25), Some("java-runtime-epsilon"));
}

#[test]
fn component_key_unknown_major_is_none() {
    for m in [7u32, 9, 11, 15, 18, 22, 24, 26] {
        assert_eq!(component_key_for_major(m), None, "major={}", m);
    }
}

#[test]
fn platform_key_matches_known_platforms() {
    let p = platform_key().expect("当前平台应支持 Java Runtime");
    assert!(
        p == "linux"
            || p.starts_with("windows-")
            || p.starts_with("mac-")
            || p.starts_with("linux-"),
        "unexpected platform key: {}",
        p
    );
}

#[test]
fn match_component_returns_expected_components() {
    let index = sample_index();
    assert_eq!(match_component(&index, 8).unwrap().0, "jre-legacy");
    assert_eq!(match_component(&index, 16).unwrap().0, "java-runtime-alpha");
    assert_eq!(match_component(&index, 17).unwrap().0, "java-runtime-gamma");
    assert_eq!(match_component(&index, 21).unwrap().0, "java-runtime-delta");
    assert_eq!(
        match_component(&index, 25).unwrap().0,
        "java-runtime-epsilon"
    );
}

#[test]
fn match_component_returns_clear_errors() {
    let index = sample_index();

    // 未知 target：官方不提供下载
    let err = match_component(&index, 7).unwrap_err();
    assert!(err.contains("官方 Runtime 不提供 Java 7"), "err: {}", err);

    // 索引缺少平台节点
    let err = match_component(&json!({}), 8).unwrap_err();
    let platform = platform_key().unwrap();
    assert!(
        err.contains(&format!("不存在平台 {}", platform)),
        "err: {}",
        err
    );

    // 平台存在但缺少组件
    let mut index_missing = sample_index();
    if let Some(obj) = index_missing[platform_key().unwrap()].as_object_mut() {
        obj.remove("java-runtime-delta");
    }
    let err = match_component(&index_missing, 21).unwrap_err();
    assert!(
        err.contains("未提供组件 java-runtime-delta"),
        "err: {}",
        err
    );
}
