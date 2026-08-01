//! 中文搜索本地映射：模糊匹配中文关键词 → 提取英文 Slug/英文名 → 重写搜索关键词

use std::collections::HashMap;

use super::super::fuzzy::{search, SearchEntry, SearchSource};
use super::database::{ChineseSearchEntry, DATABASE};

/// 中文搜索重写结果
#[derive(Debug, Clone, Default)]
pub struct RewriteResult {
    /// CurseForge 重写后的英文关键词（None 表示未匹配）
    pub cf_keyword: Option<String>,
    /// Modrinth 重写后的英文关键词（None 表示未匹配）
    pub mr_keyword: Option<String>,
    /// Modrinth Slug 直查列表（最多 100 个）
    pub mr_slugs: Vec<String>,
}

// ===== 中文搜索本地映射 =====
//
// 实现思路：
// 1. 检测中文关键词 → 2. 本地数据库模糊匹配 → 3. 提取英文 Slug/英文名 → 4. 重写搜索关键词

/// 中文关键词本地搜索
///
/// 在 moddata.txt 中模糊匹配中文关键词对应的 Mod 条目，提取英文 Slug/英文名作为新关键词。
///
/// - `cf_keyword`：CurseForge 重写英文关键词（CF 要求全词匹配，只选 1 个最佳 Mod）
/// - `mr_keyword`：Modrinth 重写英文关键词（多匹配加权出最佳英文词）
/// - `mr_slugs`：Modrinth Slug 直查列表（最多 100，调 `/v2/projects` 批量拉取）
///   均为 None/空时本地未匹配，调用方应回退原样透传。
pub fn search_by_chinese(query: &str) -> RewriteResult {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return RewriteResult::default();
    }

    // 构建模糊搜索条目（中文名主部分权重 1，英文名+Slug 权重 0.5）
    let mut search_entries: Vec<SearchEntry<&ChineseSearchEntry>> = DATABASE
        .entries
        .iter()
        .map(|e| {
            let primary_aliases: Vec<String> = e
                .chinese_name
                .split(" (")
                .next()
                .unwrap_or("")
                .split('/')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let secondary_text = {
                let en_part = if let Some(idx) = e.chinese_name.find(" (") {
                    &e.chinese_name[idx + 2..]
                } else {
                    ""
                };
                let slug = e.cf_slug.as_deref().or(e.mr_slug.as_deref()).unwrap_or("");
                format!("{} {}", en_part, slug)
            };
            SearchEntry::new(
                e,
                vec![
                    SearchSource::new(primary_aliases, 1.0),
                    SearchSource::from_text(secondary_text, 0.5),
                ],
            )
        })
        .collect();

    let results = search(&mut search_entries, &query, 100, 0.25);
    if results.is_empty() {
        crate::log_info!("[Community] 中文搜索本地匹配未命中: {}", query);
        return RewriteResult::default();
    }

    // CurseForge：选 1 个最佳匹配的 Mod（CurseForge 要求所有词都匹配）
    let cf_target = {
        // 优先完全匹配，其次相似度最高，最后选最受欢迎的
        let absolute_right: Vec<&&SearchEntry<&ChineseSearchEntry>> =
            results.iter().filter(|r| r.absolute_right).collect();
        if !absolute_right.is_empty() {
            // 完全匹配中选最受欢迎的
            absolute_right
                .into_iter()
                .max_by_key(|r| r.item.popularity)
                .copied()
        } else {
            // 模糊匹配中相似度最高的
            results
                .iter()
                .max_by(|a, b| {
                    b.item.popularity.cmp(&a.item.popularity).then_with(|| {
                        b.similarity
                            .partial_cmp(&a.similarity)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                })
                .copied()
        }
    };

    let cf_keyword = cf_target.and_then(|r| {
        r.item.cf_slug.as_ref().map(|slug| {
            let words = extract_words(&r.item.chinese_name, Some(slug.as_str()), None);
            let kw = words.join(" ");
            if kw.is_empty() {
                slug.replace('-', " ")
            } else {
                kw
            }
        })
    });

    // Modrinth：多匹配加权出最佳英文词 + Slug 直查
    let mut word_weights: HashMap<String, f64> = HashMap::new();
    let mut mr_slugs: Vec<String> = Vec::new();
    for r in &results {
        if let Some(ref mr_slug) = r.item.mr_slug {
            // Slug 直查列表（最多 100 个）
            if mr_slugs.len() < 100 {
                mr_slugs.push(mr_slug.clone());
            }
            // 加权：相似度 × 受欢迎程度
            for word in extract_words(
                &r.item.chinese_name,
                r.item.cf_slug.as_deref(),
                Some(mr_slug),
            ) {
                let similarity = if r.absolute_right {
                    1000.0
                } else {
                    r.similarity
                };
                *word_weights.entry(word).or_insert(0.0) += similarity * r.item.popularity as f64;
            }
        }
    }

    let mr_keyword = word_weights
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(w, _)| w.clone());

    crate::log_info!(
        "[Community] 中文搜索重写: query={}, cf_keyword={:?}, mr_keyword={:?}, mr_slugs={}",
        query,
        cf_keyword,
        mr_keyword,
        mr_slugs.len()
    );

    RewriteResult {
        cf_keyword,
        mr_keyword,
        mr_slugs,
    }
}

/// 从匹配的条目中提取候选英文单词
///
/// 来源：Slug（`-` 替换为空格）+ 英文名（中文名括号内部分）
/// 清洗：过滤单字、常见词（the/of/mod/and/forge/fabric/for/quilt/neoforge）、纯数字
/// 去重 + 子串去重
pub(super) fn extract_words(
    chinese_name: &str,
    cf_slug: Option<&str>,
    mr_slug: Option<&str>,
) -> Vec<String> {
    let mut candidates: Vec<String> = Vec::new();

    // 从 Slug 提取（- 替换为空格）
    if let Some(slug) = cf_slug {
        candidates.push(slug.replace(['-', '/'], " "));
    }
    if let Some(slug) = mr_slug {
        candidates.push(slug.replace(['-', '/'], " "));
    }

    // 从英文名提取（中文名括号内部分）
    if let Some(idx) = chinese_name.rfind(" (") {
        let en_part = &chinese_name[idx + 2..];
        let en_clean = en_part
            .trim_end_matches(')')
            .split(" - ")
            .next()
            .unwrap_or("");
        candidates.push(en_clean.replace(['-', '/', ':', '('], " ").replace(')', ""));
    }

    // 分词、清洗、去重
    let stop_words = [
        "the", "of", "mod", "and", "forge", "fabric", "for", "quilt", "neoforge",
    ];
    let mut words: Vec<String> = candidates
        .iter()
        .flat_map(|c| c.split_whitespace())
        .map(|w| {
            w.trim_matches(|ch| {
                ch == '{' || ch == '[' || ch == '(' || ch == '}' || ch == ']' || ch == ')'
            })
            .to_lowercase()
        })
        .filter(|w| {
            if w.len() <= 1 {
                return false;
            }
            if stop_words.contains(&w.as_str()) {
                return false;
            }
            if w.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            true
        })
        .collect();
    words.sort();
    words.dedup();

    // 子串去重：如果一个词可以由其他词拼成，则去掉
    // 先克隆一份快照供闭包内只读访问，避免 retain 的可变借用与 iter 的不可变借用冲突
    let words_snapshot: Vec<String> = words.clone();
    words.retain(|w| {
        !words_snapshot.iter().any(|c| {
            c.len() < w.len() && w.starts_with(c) && can_form(&w[c.len()..], &words_snapshot)
        })
    });
    words
}

/// 递归判断字符串是否可以由 words 中的词拼接而成
pub(super) fn can_form(s: &str, words: &[String]) -> bool {
    if s.is_empty() {
        return true;
    }
    words
        .iter()
        .any(|c| s.starts_with(c) && can_form(&s[c.len()..], words))
}
