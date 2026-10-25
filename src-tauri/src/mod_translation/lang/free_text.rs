//! 自由文本布局快照与按行对齐写回（规格 §2）

use std::collections::BTreeMap;
use std::path::Path;

const BOM: char = '\u{feff}';
const LINE_KEY_PREFIX: &str = "/lines/";

/// 布局快照：BOM / 换行符 / 尾换行 / 源行 / 已翻译行
#[derive(Debug, Clone)]
pub struct FreeTextSnapshot {
    pub has_bom: bool,
    pub eol: String,
    pub trailing_newline: bool,
    pub source_lines: Vec<String>,
    pub target_lines: Vec<String>,
}

/// 解析自由文本：检测 BOM、换行风格（优先 CRLF）、尾换行
pub fn snapshot_free_text(content: &str) -> FreeTextSnapshot {
    let body = content.strip_prefix(BOM).unwrap_or(content);
    let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let trailing_newline = body.ends_with('\n');
    let mut source_lines: Vec<String> = body
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
        .collect();
    if trailing_newline {
        source_lines.pop();
    }
    FreeTextSnapshot {
        has_bom: body.len() != content.len(),
        eol: eol.to_string(),
        trailing_newline,
        source_lines,
        target_lines: Vec::new(),
    }
}

/// 按布局重建文本：已翻译行优先，空行回退原文
pub fn render_localized_text(snapshot: &FreeTextSnapshot) -> String {
    let mut out = String::new();
    if snapshot.has_bom {
        out.push(BOM);
    }
    let total = snapshot.source_lines.len().max(snapshot.target_lines.len());
    for index in 0..total {
        if index > 0 {
            out.push_str(&snapshot.eol);
        }
        let translated = snapshot
            .target_lines
            .get(index)
            .filter(|line| !line.trim().is_empty());
        let text = translated
            .or_else(|| snapshot.source_lines.get(index))
            .map(String::as_str)
            .unwrap_or("");
        out.push_str(text);
    }
    if snapshot.trailing_newline && total > 0 {
        out.push_str(&snapshot.eol);
    }
    out
}

fn line_key(index: usize) -> String {
    format!("{LINE_KEY_PREFIX}{index:06}")
}

fn line_index(key: &str) -> Option<usize> {
    key.strip_prefix(LINE_KEY_PREFIX)
        .unwrap_or(key)
        .parse::<usize>()
        .ok()
}

/// 读取已有 zh_cn 自由文本，按行号给出 `/lines/NNNNNN` 键（续传合并用）
pub fn read_localized_target(workspace: &Path, target_path: &Path) -> BTreeMap<String, String> {
    let mut entries = BTreeMap::new();
    let Ok(content) = std::fs::read_to_string(workspace.join(target_path)) else {
        return entries;
    };
    for (index, line) in snapshot_free_text(&content).source_lines.iter().enumerate() {
        if !line.trim().is_empty() {
            entries.insert(line_key(index), line.clone());
        }
    }
    entries
}

/// 按行对齐：只替换非空行，行数保持不变
pub fn align_free_text(lines: &[String], translations: &BTreeMap<String, String>) -> Vec<String> {
    let mut out = lines.to_vec();
    for (key, text) in translations {
        let Some(index) = line_index(key) else {
            continue;
        };
        let Some(slot) = out.get_mut(index) else {
            continue;
        };
        if text.trim().is_empty() || slot.trim().is_empty() {
            continue;
        }
        *slot = text.clone();
    }
    out
}
