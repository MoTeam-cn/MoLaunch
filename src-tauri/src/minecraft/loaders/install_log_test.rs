use super::*;

#[test]
fn test_文件名过滤非法字符() {
    assert_eq!(
        sanitize("26.3-NeoForge_26.3.0.75-beta"),
        "26.3-NeoForge_26.3.0.75-beta"
    );
    assert_eq!(sanitize("a/b\\c:d"), "a_b_c_d");
    assert_eq!(sanitize("带中文"), "___");
}

#[test]
fn test_未超上限时全部保留() {
    let mut log = InstallLog::begin("NeoForge", "v", "26.3", "java", Path::new("C:/mc"));
    log.note("a");
    log.note("b");
    assert_eq!(log.entries.len(), 2);
    assert_eq!(log.dropped, 0);
}

#[test]
fn test_超出上限后计数递增() {
    let mut log = InstallLog::begin("NeoForge", "v", "26.3", "java", Path::new("C:/mc"));
    for i in 0..(MAX_ENTRIES + 5) {
        log.note(format!("第 {} 条", i));
    }
    assert_eq!(log.entries.len(), MAX_ENTRIES);
    assert_eq!(log.dropped, 5);
}

#[test]
fn test_problem_带前缀() {
    let mut log = InstallLog::begin("Forge", "v", "1.20.1", "java", Path::new("C:/mc"));
    log.problem("缺东西");
    assert!(log.entries[0].text.starts_with("[问题]"));
}

#[test]
fn test_落盘内容含摘要与阶段() {
    let dir = std::env::temp_dir().join("molaunch-install-log-test");
    let mut log = InstallLog::begin("NeoForge", "26.3-test", "26.3", "java.exe", &dir);
    log.stage(crate::minecraft::loaders::install_flow::InstallStage::Prepare);
    log.note("示例动作");
    let path = log.finish(false, "少了 srg jar").expect("记录应落盘");
    let text = std::fs::read_to_string(&path).expect("读回记录");
    assert!(text.contains("NeoForge"));
    assert!(text.contains("少了 srg jar"));
    assert!(text.contains("准备"));
    assert!(text.contains("示例动作"));
    let _ = std::fs::remove_file(&path);
}
