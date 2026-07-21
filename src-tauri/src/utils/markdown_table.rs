//! Markdown 表格解析工具
//!
//! 解析形如下的 markdown 表格文本（可包含 `#` 开头的注释行与空行）：
//!
//! ```text
//! # 注释行
//!
//! | name | version | desc |
//! | --- | --- | --- |
//! | vue | 3.4 | 渐进式框架 |
//! | vite | 5.0 | 构建工具 |
//! ```
//!
//! 解析结果为 `Vec<HashMap<String, String>>`，每行一个 HashMap，
//! key 来自表头单元格（去除两端空白），value 来自数据行对应列单元格。
//!
//! 规则：
//! - 仅识别第一个表格，遇到表头分隔行（`| --- | --- |`）后开始读取数据行
//! - 数据行以 `|` 起始判定，遇到非 `|` 起始的行或空行即结束
//! - 单元格内 ` \| ` 转义为 `|`，避免与分隔符冲突
//! - 单元格两端空白被 trim
//! - 单元格数少于表头数时，缺失列填充为空字符串
//! - 单元格数多于表头数时，多余列被忽略

use std::collections::HashMap;

/// 解析 markdown 表格文本为行数据列表
///
/// 每行数据为 `HashMap<String, String>`，key 为表头列名（小写化处理由调用方决定，
/// 本函数保持原样），value 为单元格内容（已 trim、已反转义 `\|`）。
///
/// 若文本中找不到合法的 markdown 表格，返回空 Vec。
pub fn parse_markdown_table(text: &str) -> Vec<HashMap<String, String>> {
    let lines: Vec<&str> = text.lines().collect();
    let mut rows: Vec<HashMap<String, String>> = Vec::new();

    let mut idx = 0;
    // 1. 找到表头行（首个以 | 起始且含至少一个 | 的非空行）
    while idx < lines.len() {
        let trimmed = lines[idx].trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.matches('|').count() >= 2 {
            break;
        }
        idx += 1;
    }
    if idx >= lines.len() {
        return rows;
    }

    // 2. 解析表头
    let headers = split_row(lines[idx]);
    idx += 1;

    // 3. 跳过分隔行（| --- | --- |）
    if idx < lines.len() {
        let trimmed = lines[idx].trim();
        if trimmed.starts_with('|') && is_separator_row(trimmed) {
            idx += 1;
        }
    }

    // 4. 读取数据行（连续以 | 起始的行；遇到非表格行即停止）
    while idx < lines.len() {
        let trimmed = lines[idx].trim();
        if !trimmed.starts_with('|') {
            break;
        }
        let cells = split_row(lines[idx]);
        let mut row: HashMap<String, String> = HashMap::new();
        for (i, header) in headers.iter().enumerate() {
            let value = cells.get(i).map(|s| s.as_str()).unwrap_or("");
            row.insert(header.clone(), value.to_string());
        }
        rows.push(row);
        idx += 1;
    }

    rows
}

/// 拆分单行表格单元格
///
/// 输入示例：`| vue | 3.4 | 渐进式框架 |`
/// 输出：`["vue", "3.4", "渐进式框架"]`
///
/// 处理细节：
/// - 移除行首行尾的 `|` 后再按 `|` 拆分
/// - 支持 `\|` 转义（不作为分隔符）
/// - 每个单元格 trim 两端空白
fn split_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    // 移除首尾的 | 字符
    let inner = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed);

    // 按 | 拆分，但 \| 视为转义不拆分
    let mut cells: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next) = chars.peek() {
                if next == '|' {
                    current.push('|');
                    chars.next();
                    continue;
                }
            }
            current.push(ch);
        } else if ch == '|' {
            cells.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(ch);
        }
    }
    cells.push(current.trim().to_string());
    cells
}

/// 判断是否为 markdown 表格分隔行
///
/// 分隔行形如：`| --- | --- |` 或 `| :---: | ---: |`
fn is_separator_row(line: &str) -> bool {
    let cells = split_row(line);
    if cells.is_empty() {
        return false;
    }
    cells.iter().all(|c| {
        let c = c.trim();
        !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')) && c.contains('-')
    })
}

#[cfg(test)]
mod tests {
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
}
