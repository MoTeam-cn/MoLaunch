//! 自由文本布局快照与按行对齐写回

use std::collections::BTreeMap;
use std::path::Path;

/// 自由文本文件布局快照：BOM / EOL / 尾换行 / 源行 / 已有目标行
#[derive(Debug, Clone)]
pub struct FreeTextSnapshot {
    pub has_bom: bool,
    pub eol: String,
    pub trailing_newline: bool,
    pub source_lines: Vec<String>,
    pub target_lines: Vec<String>,
}

/// 解析自由文本：检测 BOM、EOL（优先 \r\n）、尾换行，按 EOL 分行
pub fn snapshot_free_text(content: &str) -> FreeTextSnapshot {
    let body = content.strip_prefix('\u{feff}').unwrap_or(content);
    let has_bom = body.len() != content.len();
    let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let trailing_newline = body.ends_with("\r\n") || body.ends_with('\n');
    let mut source_lines: Vec<String> = body
        .split("\r\n")
        .flat_map(|part| part.split('\n'))
        .map(str::to_string)
        .collect();
    if trailing_newline {
        source_lines.pop();
    }
    FreeTextSnapshot {
        has_bom,
        eol: eol.to_string(),
        trailing_newline,
        source_lines,
        target_lines: Vec::new(),
    }
}

/// 按布局重建文本：target_lines 为基底，空（未翻译）行回退 source_lines
pub fn render_localized_text(snapshot: &FreeTextSnapshot) -> String {
    let mut out = String::new();
    if snapshot.has_bom {
        out.push('\u{feff}');
    }
    let count = snapshot.source_lines.len().max(snapshot.target_lines.len());
    for i in 0..count {
        if i > 0 {
            out.push_str(&snapshot.eol);
        }
        let text = snapshot
            .target_lines
            .get(i)
            .filter(|l| !l.trim().is_empty())
            .or_else(|| snapshot.source_lines.get(i))
            .map(String::as_str)
            .unwrap_or("");
        out.push_str(text);
    }
    if snapshot.trailing_newline && count > 0 {
        out.push_str(&snapshot.eol);
    }
    out
}

/// 行号 -> /lines/000000 形式键
fn line_key(index: usize) -> String {
    format!("/lines/{index:06}")
}

/// 解析 /lines/NNNNNN 形式键（兼容纯行号字符串）
fn parse_line_key(key: &str) -> Option<usize> {
    key.strip_prefix("/lines/")
        .or(Some(key))
        .and_then(|s| s.parse::<usize>().ok())
}

/// 读取已有 zh_cn 自由文本（存在时）按行号解析为 /lines/NNNNNN 键，供断点续传合并
pub fn read_localized_target(workspace: &Path, target_path: &Path) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let Ok(content) = std::fs::read_to_string(workspace.join(target_path)) else {
        return map;
    };
    for (i, line) in snapshot_free_text(&content).source_lines.iter().enumerate() {
        if !line.trim().is_empty() {
            map.insert(line_key(i), line.clone());
        }
    }
    map
}

/// 自由文本按行对齐：仅替换非空行（跳过空翻译与空源行），保持行数
pub fn align_free_text(lines: &[String], translations: &BTreeMap<String, String>) -> Vec<String> {
    let mut out: Vec<String> = lines.to_vec();
    for (key, text) in translations {
        let Some(index) = parse_line_key(key) else {
            continue;
        };
        if text.trim().is_empty() || index >= out.len() || out[index].trim().is_empty() {
            continue;
        }
        out[index] = text.clone();
    }
    out
}
