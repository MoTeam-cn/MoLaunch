//! 模组中文名：分层决策（内嵌中文 → AI 推荐 → 已知表 → 直译 → 文件名 → 显示名 → modId）。

use std::path::Path;

use crate::mod_translation::types::{has_chinese, ModNameResult};

/// 已知模组中文名表（identity_key 归一化后匹配）
const KNOWN_NAMES: &[(&str, &str)] = &[
    ("macaw'swindows", "Macaw 的窗户"),
    ("mcwwindows", "Macaw 的窗户"),
    ("iron'sspells'nspellbooks", "铁魔法与法术书"),
    ("irons_spellbooks", "铁魔法与法术书"),
    ("farmer'sdelight", "农夫乐事"),
    ("farmersdelight", "农夫乐事"),
    ("alex'scaves", "Alex 的洞穴"),
    ("alexscaves", "Alex 的洞穴"),
    ("mekanism", "通用机械"),
    ("jei", "JEI 物品管理器"),
];

/// 英文词 → 中文（直译用）
const WORD_TRANSLATIONS: &[(&str, &str)] = &[
    ("window", "窗户"),
    ("windows", "窗户"),
    ("cave", "洞穴"),
    ("caves", "洞穴"),
    ("spell", "法术"),
    ("spells", "法术"),
    ("spellbook", "法术书"),
    ("spellbooks", "法术书"),
    ("loot", "战利品"),
    ("mate", "助手"),
    ("farmer", "农夫"),
    ("farmers", "农夫"),
    ("delight", "乐事"),
    ("magic", "魔法"),
    ("iron", "铁"),
    ("tool", "工具"),
    ("tools", "工具"),
    ("door", "门"),
    ("doors", "门"),
];

/// 通用名（不可作为模组名）
const GENERIC_NAMES: &[&str] = &[
    "未命名",
    "未知",
    "模组",
    "中文模组",
    "魔法模组",
    "冒险模组",
    "科技模组",
    "装饰模组",
    "工具模组",
    "有趣的冒险",
    "中文扩展",
    "未命名扩展",
];

/// 直译时忽略的噪声词
const NOISE_WORDS: &[&str] = &["mc", "forge", "fabric", "neoforge", "mod"];

/// 归一化：小写 + 去空白
fn identity_key(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn word_map(word: &str) -> Option<&'static str> {
    WORD_TRANSLATIONS
        .iter()
        .find(|(k, _)| *k == word)
        .map(|(_, v)| *v)
}

/// 是否为可用中文名：≥2 字符、含 CJK、非通用名
pub fn usable_chinese_mod_name(value: &str) -> bool {
    let normalized = value.trim();
    normalized.chars().count() >= 2
        && has_chinese(normalized)
        && !GENERIC_NAMES.contains(&normalized)
}

/// 已知表查询：project_names 与 mod_ids 归一化后匹配
pub fn known_chinese_mod_name(project_names: &[String], mod_ids: &[String]) -> Option<String> {
    mod_ids
        .iter()
        .chain(project_names.iter())
        .find_map(|value| {
            KNOWN_NAMES
                .iter()
                .find(|(k, _)| *k == identity_key(value))
                .map(|(_, n)| n.to_string())
        })
}

/// 直译英文标签：`X's Y` → "X 的Y"；否则逐词翻译，至少命中一词才返回
pub fn translate_english_label(value: &str) -> Option<String> {
    let cleaned = value.replace(['_', '-'], " ");
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    let joined = words.join(" ");
    let possessive = joined
        .find("'s")
        .map(|p| (p, 2))
        .or_else(|| joined.find("’s").map(|p| (p, 4)));
    if let Some((pos, len)) = possessive {
        let head = joined[..pos].trim();
        let tail = joined[pos + len..].trim();
        if !head.is_empty() && !tail.is_empty() {
            let translated_tail: String = tail
                .split_whitespace()
                .map(|w| word_map(&w.to_ascii_lowercase()).unwrap_or(w))
                .collect();
            if has_chinese(&translated_tail) {
                return Some(format!("{head} 的{translated_tail}"));
            }
        }
    }
    let mut translated = false;
    let parts: String = words
        .iter()
        .filter(|w| !NOISE_WORDS.contains(&w.to_ascii_lowercase().as_str()))
        .map(|w| match word_map(&w.to_ascii_lowercase()) {
            Some(t) => {
                translated = true;
                t.to_string()
            }
            None => (*w).to_string(),
        })
        .collect();
    if translated {
        Some(parts)
    } else {
        None
    }
}

/// 去掉版本号后缀（`-1.20.1` / `_mc1.20.1` 等）
fn strip_version(value: &str) -> String {
    for (i, c) in value.char_indices() {
        if matches!(c, '-' | '_' | ' ') {
            let rest = value[i + c.len_utf8()..].trim_start_matches("mc");
            if rest.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
                return value[..i].to_string();
            }
        }
    }
    value.to_string()
}

/// 从文件名提取可读标签（去版本号、过滤通用下载名）
fn original_project_label(original_name: &str) -> Option<String> {
    let stem = Path::new(original_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(original_name)
        .trim()
        .to_string();
    if stem.is_empty() {
        return None;
    }
    let stripped: String = stem
        .chars()
        .filter(|c| !c.is_ascii_digit() && !matches!(c, '-' | '_' | ' '))
        .collect();
    if matches!(stripped.as_str(), "download" | "mod" | "file" | "unknown") {
        return None;
    }
    let label = strip_version(&stem)
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if label.is_empty() {
        None
    } else {
        Some(label)
    }
}

fn truncated(value: &str) -> String {
    value.trim().chars().take(64).collect()
}

/// 决策链主入口
pub fn resolve_mod_name(
    project_names: &[String],
    mod_ids: &[String],
    original_name: &str,
    recommended_name: Option<String>,
) -> ModNameResult {
    if let Some(embedded) = project_names.iter().find(|n| usable_chinese_mod_name(n)) {
        return ModNameResult {
            name: truncated(embedded),
            source: "embedded_chinese".to_string(),
        };
    }
    if let Some(name) = recommended_name.filter(|n| usable_chinese_mod_name(n)) {
        return ModNameResult {
            name: truncated(&name),
            source: "ai_recommended".to_string(),
        };
    }
    if let Some(known) = known_chinese_mod_name(project_names, mod_ids) {
        return ModNameResult {
            name: truncated(&known),
            source: "known_chinese".to_string(),
        };
    }
    for display in project_names {
        if let Some(translated) = translate_english_label(display) {
            if usable_chinese_mod_name(&translated) {
                return ModNameResult {
                    name: truncated(&translated),
                    source: "translated_display_name".to_string(),
                };
            }
        }
    }
    if let Some(original) = original_project_label(original_name) {
        if let Some(translated) = translate_english_label(&original) {
            if usable_chinese_mod_name(&translated) {
                return ModNameResult {
                    name: truncated(&translated),
                    source: "translated_filename".to_string(),
                };
            }
        }
        return ModNameResult {
            name: truncated(&original),
            source: "original_filename".to_string(),
        };
    }
    if let Some(display) = project_names
        .iter()
        .map(|n| n.trim())
        .find(|n| !n.is_empty() && !GENERIC_NAMES.contains(n))
    {
        return ModNameResult {
            name: truncated(display),
            source: "display_name".to_string(),
        };
    }
    let mod_id = mod_ids.iter().map(|id| id.trim()).find(|id| !id.is_empty());
    ModNameResult {
        name: truncated(mod_id.unwrap_or("mod")),
        source: "mod_id".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_chinese_wins() {
        let r = resolve_mod_name(
            &["Macaw 的窗户".to_string()],
            &["mcwwindows".to_string()],
            "mcwwindows-1.0.jar",
            None,
        );
        assert_eq!(
            (r.source.as_str(), r.name.as_str()),
            ("embedded_chinese", "Macaw 的窗户")
        );
    }

    #[test]
    fn known_names_are_used() {
        let r = resolve_mod_name(&[], &["mekanism".to_string()], "mekanism-1.20.1.jar", None);
        assert_eq!(
            (r.source.as_str(), r.name.as_str()),
            ("known_chinese", "通用机械")
        );
    }

    #[test]
    fn possessive_names_translate() {
        let r = resolve_mod_name(&[], &[], "Alex's Caves-1.2.jar", None);
        assert_eq!(
            (r.source.as_str(), r.name.as_str()),
            ("translated_filename", "Alex 的洞穴")
        );
    }

    #[test]
    fn generic_names_fall_through_to_mod_id() {
        let r = resolve_mod_name(&[], &["weird_mod_id".to_string()], "download.jar", None);
        assert_eq!(
            (r.source.as_str(), r.name.as_str()),
            ("mod_id", "weird_mod_id")
        );
    }
}
