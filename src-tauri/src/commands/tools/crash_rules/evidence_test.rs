//! 日志行索引测试

use super::LogIndex;

const LOG: &str = "第一行普通日志\n第二行有 error 关键字\n\n第四行 no space left on device";

#[test]
fn test_line_count_空行也计数() {
    let index = LogIndex::new(LOG);
    assert_eq!(index.line_count(), 4);
}

#[test]
fn test_first_line_返回从1开始的行号() {
    let index = LogIndex::new(LOG);
    assert_eq!(index.first_line("error"), Some(2));
    assert_eq!(index.first_line("NO SPACE"), Some(4));
    assert_eq!(index.first_line("不存在"), None);
}

#[test]
fn test_evidence_保留原文并去掉首尾空白() {
    let index = LogIndex::new("  hello world  ");
    let evidence = index.evidence(1).expect("应有证据");
    assert_eq!(evidence.line_no, 1);
    assert_eq!(evidence.raw_line, "hello world");
}

#[test]
fn test_evidence_空行不产出证据() {
    let index = LogIndex::new(LOG);
    assert!(index.evidence(3).is_none());
}

#[test]
fn test_collect_按行号升序且去重() {
    let index = LogIndex::new(LOG);
    let needles = vec!["no space left".to_string(), "error".to_string()];
    let evidence = index.collect(&needles, &[], 4);
    assert_eq!(evidence.len(), 2);
    assert_eq!(evidence[0].line_no, 2);
    assert_eq!(evidence[1].line_no, 4);
}

#[test]
fn test_window_在中心行附近命中() {
    let index = LogIndex::new(LOG);
    let needles = vec!["no space left".to_string()];
    let found = index.window(2, 2, &needles).expect("窗口内应命中");
    assert_eq!(found.line_no, 4);
    assert!(index.window(1, 1, &needles).is_none());
}

#[test]
fn test_count_统计出现次数() {
    let index = LogIndex::new("error a\nerror b\nerror c");
    assert_eq!(index.count("error"), 3);
    assert_eq!(index.count(""), 0);
}
