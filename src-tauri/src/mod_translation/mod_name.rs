//! 模组中文名：分层决策（内嵌中文 → AI 推荐 → 已知表 → 直译 → 文件名 → 显示名 → modId）
//!
//! 决策链实现为一串「候选生成器」，按顺序取第一个可用结果，
//! 每个生成器只关心自己那一步，来源标签即步骤名。

use std::path::Path;

use crate::mod_translation::types::{has_chinese, ModNameResult};

/// 名称长度上限（字符）
const NAME_LIMIT: usize = 64;

/// 已知模组中文名表（归一化后匹配）
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

/// 通用下载名（不能当模组名）
const PLACEHOLDER_STEMS: &[&str] = &["download", "mod", "file", "unknown"];

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
        .find(|(english, _)| *english == word)
        .map(|(_, chinese)| *chinese)
}

/// 可用中文名：≥2 字符、含 CJK、非通用名
pub fn usable_chinese_mod_name(value: &str) -> bool {
    let normalized = value.trim();
    normalized.chars().count() >= 2
        && has_chinese(normalized)
        && !GENERIC_NAMES.contains(&normalized)
}

/// 已知表查询：modId 优先，其次项目显示名
pub fn known_chinese_mod_name(project_names: &[String], mod_ids: &[String]) -> Option<String> {
    mod_ids
        .iter()
        .chain(project_names.iter())
        .find_map(|value| {
            let key = identity_key(value);
            KNOWN_NAMES
                .iter()
                .find(|(known, _)| *known == key)
                .map(|(_, chinese)| chinese.to_string())
        })
}

/// 逐词直译；一个词都没命中时返回 None
fn translate_words(words: &[&str]) -> Option<String> {
    let mut hit = false;
    let translated: String = words
        .iter()
        .filter(|word| !NOISE_WORDS.contains(&word.to_ascii_lowercase().as_str()))
        .map(|word| match word_map(&word.to_ascii_lowercase()) {
            Some(chinese) => {
                hit = true;
                chinese.to_string()
            }
            None => (*word).to_string(),
        })
        .collect();
    hit.then_some(translated)
}

/// 直译英文标签：`X's Y` → 「X 的Y」；否则逐词翻译
pub fn translate_english_label(value: &str) -> Option<String> {
    let cleaned = value.replace(['_', '-'], " ");
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    let joined = words.join(" ");

    let possessive = joined
        .find("'s")
        .map(|position| (position, 2))
        .or_else(|| joined.find('’').map(|position| (position, 4)));
    if let Some((position, width)) = possessive {
        let head = joined[..position].trim();
        let tail = joined[position + width..].trim();
        if !head.is_empty() && !tail.is_empty() {
            let translated_tail: String = tail
                .split_whitespace()
                .map(|word| word_map(&word.to_ascii_lowercase()).unwrap_or(word))
                .collect();
            if has_chinese(&translated_tail) {
                return Some(format!("{head} 的{translated_tail}"));
            }
        }
    }

    translate_words(&words)
}

/// 去掉版本号后缀（`-1.20.1` / `_mc1.20.1` 等）
fn strip_version(value: &str) -> String {
    for (index, ch) in value.char_indices() {
        if !matches!(ch, '-' | '_' | ' ') {
            continue;
        }
        let rest = value[index + ch.len_utf8()..].trim_start_matches("mc");
        if rest.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return value[..index].to_string();
        }
    }
    value.to_string()
}

/// 从文件名提取可读标签（去版本号、过滤通用下载名）
fn original_project_label(original_name: &str) -> Option<String> {
    let stem = Path::new(original_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(original_name)
        .trim()
        .to_string();
    if stem.is_empty() {
        return None;
    }

    let letters: String = stem
        .chars()
        .filter(|c| !c.is_ascii_digit() && !matches!(c, '-' | '_' | ' '))
        .collect();
    if PLACEHOLDER_STEMS.contains(&letters.as_str()) {
        return None;
    }

    let label = strip_version(&stem)
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!label.is_empty()).then_some(label)
}

fn clipped(value: &str) -> String {
    value.trim().chars().take(NAME_LIMIT).collect()
}

/// 决策链的一步：返回名称，来源标签由表里的字符串给出
type StepFn = fn(&Chain<'_>) -> Option<String>;

/// 决策链输入
struct Chain<'a> {
    project_names: &'a [String],
    mod_ids: &'a [String],
    original_name: &'a str,
    recommended_name: Option<&'a str>,
}

fn embedded_chinese(chain: &Chain<'_>) -> Option<String> {
    chain
        .project_names
        .iter()
        .find(|name| usable_chinese_mod_name(name))
        .map(|name| name.to_string())
}

fn ai_recommended(chain: &Chain<'_>) -> Option<String> {
    chain
        .recommended_name
        .filter(|name| usable_chinese_mod_name(name))
        .map(str::to_string)
}

fn known_chinese(chain: &Chain<'_>) -> Option<String> {
    known_chinese_mod_name(chain.project_names, chain.mod_ids)
}

fn translated_display_name(chain: &Chain<'_>) -> Option<String> {
    chain.project_names.iter().find_map(|display| {
        let translated = translate_english_label(display)?;
        usable_chinese_mod_name(&translated).then_some(translated)
    })
}

fn translated_filename(chain: &Chain<'_>) -> Option<String> {
    let label = original_project_label(chain.original_name)?;
    let translated = translate_english_label(&label)?;
    usable_chinese_mod_name(&translated).then_some(translated)
}

fn original_filename(chain: &Chain<'_>) -> Option<String> {
    original_project_label(chain.original_name)
}

fn display_name(chain: &Chain<'_>) -> Option<String> {
    chain
        .project_names
        .iter()
        .map(|name| name.trim())
        .find(|name| !name.is_empty() && !GENERIC_NAMES.contains(name))
        .map(str::to_string)
}

fn mod_id(chain: &Chain<'_>) -> Option<String> {
    chain
        .mod_ids
        .iter()
        .map(|id| id.trim())
        .find(|id| !id.is_empty())
        .map(str::to_string)
}

/// 决策链：按顺序取第一个可用结果
const STEPS: &[(&str, StepFn)] = &[
    ("embedded_chinese", embedded_chinese),
    ("ai_recommended", ai_recommended),
    ("known_chinese", known_chinese),
    ("translated_display_name", translated_display_name),
    ("translated_filename", translated_filename),
    ("original_filename", original_filename),
    ("display_name", display_name),
    ("mod_id", mod_id),
];

/// 解析模组中文名
pub fn resolve_mod_name(
    project_names: &[String],
    mod_ids: &[String],
    original_name: &str,
    recommended_name: Option<String>,
) -> ModNameResult {
    let chain = Chain {
        project_names,
        mod_ids,
        original_name,
        recommended_name: recommended_name.as_deref(),
    };
    for (source, step) in STEPS {
        if let Some(name) = step(&chain) {
            return ModNameResult {
                name: clipped(&name),
                source: (*source).to_string(),
            };
        }
    }
    ModNameResult {
        name: clipped("mod"),
        source: "mod_id".to_string(),
    }
}

#[cfg(test)]
#[path = "mod_name_test.rs"]
mod tests;
