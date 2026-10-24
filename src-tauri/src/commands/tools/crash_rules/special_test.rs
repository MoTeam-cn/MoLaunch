//! 特殊取证规则测试

use super::run;
use crate::commands::tools::crash_rules::evidence::LogIndex;

#[test]
fn test_missing_dependency_证据取依赖声明行() {
    let log = "[main/WARN]: Mod 'create' (0.5.1) is missing dependency 'flywheel' (>=0.6.9)\n\
               [main/ERROR]: Failed to load mod 'create'\n\
               [main/ERROR]: java.lang.NoClassDefFoundError: com/simibubi/create/ClientEvents";
    let index = LogIndex::new(log);
    let (confirmed, _pending) = run(&index, None);
    let hit = confirmed
        .iter()
        .find(|item| item.cause == "missing_dependency")
        .expect("应产出缺前置");
    assert_eq!(hit.evidence[0].line_no, 1);
    assert!(hit.title.contains("create"));
    assert!(hit.title.contains("flywheel"));
}

#[test]
fn test_missing_dependency_无依赖声明时降级为低置信类缺失() {
    let log = "java.lang.NoClassDefFoundError: com/example/Foo";
    let index = LogIndex::new(log);
    let (confirmed, _pending) = run(&index, None);
    let hit = confirmed
        .iter()
        .find(|item| item.cause == "class_missing")
        .expect("应产出类缺失");
    assert_eq!(hit.confidence, "low");
    assert!(!hit.missing_evidence.is_empty());
}

#[test]
fn test_duplicate_mods_需要两条不同版本才输出() {
    let duplicated = "Loading 3 mods:\n\t- sodium 0.5.8\n\t- sodium 0.5.9\n";
    let index = LogIndex::new(duplicated);
    let (confirmed, _) = run(&index, None);
    assert!(confirmed.iter().any(|item| item.cause == "duplicate_mod"));

    let single = "Loading 2 mods:\n\t- sodium 0.5.8\n\t- lithium 0.11.3\n";
    let index = LogIndex::new(single);
    let (confirmed, _) = run(&index, None);
    assert!(!confirmed.iter().any(|item| item.cause == "duplicate_mod"));
}

#[test]
fn test_plugin_load_failures_按三种子因拆分() {
    let log = "[ERROR]: Failed to load plugin 'A'\n\
               [ERROR]: java.lang.NoClassDefFoundError: org/bukkit/Foo\n\
               [ERROR]: Failed to load plugin 'B'\n\
               [ERROR]: java.lang.UnsupportedClassVersionError: B has been compiled by a more recent version\n\
               [ERROR]: Failed to load plugin 'C'\n\
               [ERROR]: java.lang.ClassNotFoundException: net/example/C";
    let index = LogIndex::new(log);
    let (confirmed, _) = run(&index, None);
    let causes: Vec<&str> = confirmed.iter().map(|i| i.cause.as_str()).collect();
    assert!(
        causes.contains(&"plugin_missing_class"),
        "实际: {:?}",
        causes
    );
    assert!(
        causes.contains(&"plugin_java_version"),
        "实际: {:?}",
        causes
    );
    assert!(
        causes.contains(&"plugin_class_missing"),
        "实际: {:?}",
        causes
    );
    // 每个插件的原因行必须紧跟自己的插件行，不能串到下一个插件
    let b = confirmed
        .iter()
        .find(|i| i.cause == "plugin_java_version")
        .expect("应有 Java 版本子因");
    assert!(b.title.contains('B'));
    assert_eq!(b.evidence[1].line_no, 4);
}
