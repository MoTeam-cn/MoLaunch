//! markdown_table 单元测试

use super::*;

#[test]
fn parses_simple_table() {
    let text = r#"
# 测试表格

| name | version |
| --- | --- |
| vue | 3.4 |
| vite | 5.0 |
"#;
    let rows = parse_markdown_table(text);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].get("name").map(|s| s.as_str()), Some("vue"));
    assert_eq!(rows[0].get("version").map(|s| s.as_str()), Some("3.4"));
    assert_eq!(rows[1].get("name").map(|s| s.as_str()), Some("vite"));
}

#[test]
fn handles_escaped_pipe_in_cell() {
    let text = "| name | desc |\n| --- | --- |\n| a\\|b | 含竖线 |\n";
    let rows = parse_markdown_table(text);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get("name").map(|s| s.as_str()), Some("a|b"));
    assert_eq!(rows[0].get("desc").map(|s| s.as_str()), Some("含竖线"));
}

#[test]
fn handles_missing_columns() {
    let text = "| a | b | c |\n| --- | --- | --- |\n| x | y |\n";
    let rows = parse_markdown_table(text);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get("a").map(|s| s.as_str()), Some("x"));
    assert_eq!(rows[0].get("b").map(|s| s.as_str()), Some("y"));
    assert_eq!(rows[0].get("c").map(|s| s.as_str()), Some(""));
}

#[test]
fn ignores_extra_columns() {
    let text = "| a | b |\n| --- | --- |\n| x | y | z |\n";
    let rows = parse_markdown_table(text);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get("a").map(|s| s.as_str()), Some("x"));
    assert_eq!(rows[0].get("b").map(|s| s.as_str()), Some("y"));
}

#[test]
fn returns_empty_when_no_table() {
    let text = "no table here\njust text";
    let rows = parse_markdown_table(text);
    assert!(rows.is_empty());
}

#[test]
fn stops_at_non_table_line() {
    let text = "| a | b |\n| --- | --- |\n| x | y |\n\nsome text\n| p | q |\n";
    let rows = parse_markdown_table(text);
    assert_eq!(rows.len(), 1);
}

#[test]
fn handles_alignment_separator() {
    let text = "| a | b |\n| :---: | ---: |\n| x | y |\n";
    let rows = parse_markdown_table(text);
    assert_eq!(rows.len(), 1);
}
