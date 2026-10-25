//! 模组中文名：分层决策（内嵌中文 → AI 推荐 → 已知表 → 直译 → 文件名 → 显示名 → modId）
//!
//! 词表数据外置在 resources/mod_translation/mod-name-dictionary.txt（数据与代码分离），
//! 决策链是一串候选生成器，按顺序取第一个可用结果。

use std::path::Path;
use std::sync::OnceLock;

use crate::mod_translation::types::{has_chinese, ModNameResult};

/// 名称长度上限（字符）
const NAME_LIMIT: usize = 64;
/// 词表资源路径
const DICTIONARY_RESOURCE: &str = "mod_translation/mod-name-dictionary.txt";

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

/// 词表：known（归一化名称 → 中文）与 word（英文词 → 中文）
struct Dictionary {
    known: Vec<(String, String)>,
    words: Vec<(String, String)>,
}

impl Dictionary {
    fn known_name(&self, key: &str) -> Option<&str> {
        self.known
            .iter()
            .find(|(alias, _)| alias == key)
            .map(|(_, chinese)| chinese.as_str())
    }

    fn word(&self, word: &str) -> Option<&str> {
        self.words
            .iter()
            .find(|(english, _)| english == word)
            .map(|(_, chinese)| chinese.as_str())
    }
}

/// 解析词表资源（进程内只解析一次）
fn dictionary() -> &'static Dictionary {
    static CELL: OnceLock<Dictionary> = OnceLock::new();
    CELL.get_or_init(|| {
        let raw = crate::resources::read_resource(DICTIONARY_RESOURCE).unwrap_or_default();
        let mut known: Vec<(String, String)> = Vec::new();
        let mut words: Vec<(String, String)> = Vec::new();
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut fields = trimmed.split('\t');
            let (Some(kind), Some(key), Some(value)) =
                (fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            let entry = (key.trim().to_string(), value.trim().to_string());
            match kind {
                "known" => known.push(entry),
                "word" => words.push(entry),
                _ => {}
            }
        }
        Dictionary { known, words }
    })
}

/// 归一化：小写 + 去空白
fn fold_key(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
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
    let dictionary = dictionary();
    mod_ids
        .iter()
        .chain(project_names.iter())
        .find_map(|value| dictionary.known_name(&fold_key(value)).map(str::to_string))
}

/// 逐词直译；一个词都没命中时返回 None
fn render_words(words: &[&str]) -> Option<String> {
    let mut hit = false;
    let mut rendered = String::new();
    for word in words {
        let lowered = word.to_ascii_lowercase();
        if NOISE_WORDS.contains(&lowered.as_str()) {
            continue;
        }
        match dictionary().word(&lowered) {
            Some(chinese) => {
                hit = true;
                rendered.push_str(chinese);
            }
            None => rendered.push_str(word),
        }
    }
    hit.then_some(rendered)
}

/// 拆分所有格：X's Y → (X, Y)
fn split_possessive(joined: &str) -> Option<(&str, &str)> {
    let (position, width) = joined
        .find("'s")
        .map(|index| (index, 2))
        .or_else(|| joined.find('’').map(|index| (index, '’'.len_utf8() + 1)))?;
    let head = joined[..position].trim();
    let tail = joined[position + width..].trim();
    (!head.is_empty() && !tail.is_empty()).then_some((head, tail))
}

/// 直译英文标签：所有格形式译为「X 的Y」，否则逐词翻译
pub fn translate_english_label(value: &str) -> Option<String> {
    let cleaned = value.replace(['_', '-'], " ");
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    if let Some((head, tail)) = split_possessive(&words.join(" ")) {
        let translated: String = tail
            .split_whitespace()
            .map(|word| {
                dictionary()
                    .word(&word.to_ascii_lowercase())
                    .unwrap_or(word)
            })
            .collect();
        if has_chinese(&translated) {
            return Some(format!("{head} 的{translated}"));
        }
    }
    render_words(&words)
}

/// 去掉版本号后缀（-1.20.1 / _mc1.20.1 等）
fn drop_version_suffix(value: &str) -> String {
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
fn label_from_filename(original_name: &str) -> Option<String> {
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

    let label = drop_version_suffix(&stem)
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!label.is_empty()).then_some(label)
}

fn clipped(value: &str) -> String {
    value.trim().chars().take(NAME_LIMIT).collect()
}

/// 决策链输入
struct Chain<'a> {
    project_names: &'a [String],
    mod_ids: &'a [String],
    original_name: &'a str,
    recommended_name: Option<&'a str>,
}

/// 决策链的一步：返回名称，来源标签由表里的字符串给出
type StepFn = fn(&Chain<'_>) -> Option<String>;

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
    let label = label_from_filename(chain.original_name)?;
    let translated = translate_english_label(&label)?;
    usable_chinese_mod_name(&translated).then_some(translated)
}

fn original_filename(chain: &Chain<'_>) -> Option<String> {
    label_from_filename(chain.original_name)
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
