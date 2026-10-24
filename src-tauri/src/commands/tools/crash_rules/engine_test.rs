//! 崩溃规则引擎测试

use super::{analyze, rules_count, rules_version};
use std::collections::HashMap;

fn has_rule(items: &[crate::commands::tools::types::CrashAnalysisItem], rule_id: &str) -> bool {
    items.iter().any(|item| item.rule_id == rule_id)
}

#[test]
fn test_rules_count_规则库非空() {
    assert!(
        rules_count() >= 30,
        "内置规则应不少于 30 条，实际 {}",
        rules_count()
    );
    assert!(!rules_version().is_empty());
}

#[test]
fn test_analyze_java_版本过低命中对应规则() {
    let log = "java.lang.UnsupportedClassVersionError: net/minecraft/client/main/Main has been compiled by a more recent version";
    let items = analyze(log, None);
    assert!(
        has_rule(&items, "java-unsupported-class-version"),
        "应命中 Java 版本过低，实际: {:?}",
        items.iter().map(|i| &i.rule_id).collect::<Vec<_>>()
    );
}

#[test]
fn test_analyze_内存不足命中对应规则() {
    let items = analyze("java.lang.OutOfMemoryError: Java heap space", None);
    assert!(has_rule(&items, "memory-oom"));
}

#[test]
fn test_analyze_intel_驱动规则需要厂商关键词() {
    let intel = "EXCEPTION_ACCESS_VIOLATION ... igdumd64.dll";
    assert!(has_rule(
        &analyze(intel, None),
        "driver-intel-access-violation"
    ));

    // 只有 ACCESS_VIOLATION 没有厂商信息时不应误报为 Intel 驱动问题
    let plain = "EXCEPTION_ACCESS_VIOLATION at net.minecraft.client.Minecraft.run";
    let items = analyze(plain, None);
    assert!(
        !has_rule(&items, "driver-intel-access-violation"),
        "无厂商信息时不应命中 Intel 规则"
    );
}

#[test]
fn test_analyze_错误分级_error_排在最前() {
    let log = "EXCEPTION_ACCESS_VIOLATION igdumd64.dll\njava.lang.OutOfMemoryError";
    let items = analyze(log, None);
    assert!(items.len() >= 2);
    assert_eq!(items[0].severity, "error");
}

#[test]
fn test_analyze_mod_索引用于反查可疑_mod() {
    let mut index = HashMap::new();
    index.insert("create".to_string(), "Create".to_string());
    index.insert("sodium".to_string(), "Sodium".to_string());
    let log = "java.lang.NoClassDefFoundError: com/simibubi/create/AllBlocks";
    let items = analyze(log, Some(&index));
    let hit = items
        .iter()
        .find(|item| item.rule_id == "mod-missing-dependency");
    let suspects = hit.map(|item| item.suspects.clone()).unwrap_or_default();
    assert!(
        suspects.iter().any(|name| name == "Create"),
        "应反查到 Create，实际: {:?}",
        suspects
    );
}

#[test]
fn test_analyze_空日志返回空() {
    assert!(analyze("   ", None).is_empty());
}

#[test]
fn test_analyze_无匹配时返回空交由兜底处理() {
    assert!(analyze("这是一段没有任何错误关键词的正常日志", None).is_empty());
}

#[test]
fn test_analyze_规则自带动作与参考链接被带出() {
    let items = analyze("EXCEPTION_ACCESS_VIOLATION igdumd64.dll", None);
    let hit = items
        .iter()
        .find(|item| item.rule_id == "driver-intel-access-violation")
        .expect("应命中 Intel 规则");
    assert_eq!(hit.refs.len(), 1);
    assert!(hit.refs[0].contains("MC-32606"));
}
