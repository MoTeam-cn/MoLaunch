//! MC 百科（mcmod.cn）数据库
//!
//! 加载内置 moddata.txt，通过工程 Slug 查找中文译名和 MC 百科 class id。
//! 关键设计：moddata.txt 第 N 行 → mcmod.cn class id = N（空行也占行号）
//!
//! 中文搜索：`search_by_chinese` 用本地模糊匹配把中文关键词映射到 CurseForge/Modrinth Slug，
//! 提取英文单词作为搜索关键词（参考 PCL2 `ResourceSearcher.vb` 189-290 行）。

use once_cell::sync::Lazy;
use std::collections::HashMap;

use super::fuzzy::{search, SearchEntry, SearchSource};

/// 全局数据库
static DATABASE: Lazy<Database> = Lazy::new(Database::load);

/// 单条查询结果：中文名 + MC 百科 class id + 人气值
#[derive(Clone)]
struct Entry {
    chinese_name: String,
    class_id: u32,
    /// MC 百科的浏览量逆序排行，1 代表浏览量最低（用于中文搜索排序权重）
    #[allow(dead_code)]
    popularity: u32,
}

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

/// 用于本地模糊匹配的内部条目
struct ChineseSearchEntry {
    cf_slug: Option<String>,
    mr_slug: Option<String>,
    /// 完整中文名，如 "工业时代2 (Industrial Craft 2)"
    chinese_name: String,
    /// MC 百科浏览量逆序排行
    popularity: u32,
}

struct Database {
    /// CurseForge Slug → 条目
    cf_map: HashMap<String, Entry>,
    /// Modrinth Slug → 条目
    mr_map: HashMap<String, Entry>,
    /// 用于中文反查的条目列表（含双平台 slug 信息）
    entries: Vec<ChineseSearchEntry>,
}

impl Database {
    fn load() -> Self {
        // 统一走 resources 模块（编译时 include_str! 嵌入二进制，运行时零 IO）
        let data = crate::resources::read_resource("moddata.txt")
            .expect("嵌入资源 moddata.txt 缺失，请检查 resources.rs 注册");
        let mut cf_map: HashMap<String, Entry> = HashMap::new();
        let mut mr_map: HashMap<String, Entry> = HashMap::new();
        let mut entries: Vec<ChineseSearchEntry> = Vec::new();

        let data_str: &str = &data;
        let lines: Vec<&str> = data_str.lines().collect();

        // 最后一行是 Popularity 排行数据（按行号对应，用 | 分隔）
        let (entry_lines, popularity_line) = if lines.len() > 1 {
            lines.split_at(lines.len() - 1)
        } else {
            (&lines[..], &[][..])
        };
        let ranks: Vec<u32> = popularity_line
            .first()
            .map(|s| s.split('|').map(|n| n.parse::<u32>().unwrap_or(0)).collect())
            .unwrap_or_default();

        // 行号即 class id
        // 关键：空行也计数，否则行号会错位
        for (idx, line) in entry_lines.iter().enumerate() {
            let class_id = (idx + 1) as u32; // 行号从 1 开始
            let popularity = ranks.get(idx).copied().unwrap_or(0);
            let line: &str = line;
            if line.is_empty() {
                continue;
            }
            // 同一行可能含多个条目，用 ¨ (U+00A8) 分隔
            for entry_data in line.split('¨') {
                let parts: Vec<&str> = entry_data.split('|').collect();
                if parts.is_empty() {
                    continue;
                }
                let slug_part = parts[0];
                // 中文名在最后（可能没有）
                let chinese_name = if parts.len() >= 2 {
                    let raw = parts.last().unwrap_or(&"");
                    process_wildcard(raw, slug_part)
                } else {
                    String::new()
                };
                if chinese_name.is_empty() {
                    continue;
                }

                // 解析 Slug 部分
                let (cf_slug, mr_slug) = parse_slug_part(slug_part);
                let entry = Entry {
                    chinese_name: chinese_name.clone(),
                    class_id,
                    popularity,
                };

                if let Some(ref cf) = cf_slug {
                    cf_map.insert(cf.clone(), entry.clone());
                }
                if let Some(ref mr) = mr_slug {
                    mr_map.insert(mr.clone(), entry);
                }

                // 收集到反查列表（至少有一个 slug 才加入）
                if cf_slug.is_some() || mr_slug.is_some() {
                    entries.push(ChineseSearchEntry {
                        cf_slug,
                        mr_slug,
                        chinese_name,
                        popularity,
                    });
                }
            }
        }

        crate::log_info!(
            "[Community] mcmod 数据库已加载: CF {} 条, MR {} 条, 反查条目 {} 条",
            cf_map.len(),
            mr_map.len(),
            entries.len()
        );

        Database {
            cf_map,
            mr_map,
            entries,
        }
    }

    /// 通过 CurseForge Slug 查找条目
    fn lookup_cf(&self, slug: &str) -> Option<&Entry> {
        self.cf_map.get(slug)
    }

    /// 通过 Modrinth Slug 查找条目
    fn lookup_mr(&self, slug: &str) -> Option<&Entry> {
        self.mr_map.get(slug)
    }
}

/// 解析 Slug 部分
/// 四种语法：
/// - `@slug` → CF=None, MR=slug
/// - `slug@` → CF=slug, MR=slug（相同）
/// - `cf_slug@mr_slug` → CF=cf_slug, MR=mr_slug
/// - `slug` → CF=slug, MR=None
fn parse_slug_part(s: &str) -> (Option<String>, Option<String>) {
    if s.is_empty() {
        return (None, None);
    }
    if s.starts_with('@') {
        // @slug → 仅 Modrinth
        let mr = s[1..].to_string();
        if mr.is_empty() {
            (None, None)
        } else {
            (None, Some(mr))
        }
    } else if s.ends_with('@') {
        // slug@ → CF=MR=slug
        let slug = s[..s.len() - 1].to_string();
        if slug.is_empty() {
            (None, None)
        } else {
            (Some(slug.clone()), Some(slug))
        }
    } else if let Some(idx) = s.find('@') {
        // cf@mr → 双平台不同 slug
        let cf = s[..idx].to_string();
        let mr = s[idx + 1..].to_string();
        if cf.is_empty() && mr.is_empty() {
            (None, None)
        } else {
            (
                if cf.is_empty() { None } else { Some(cf) },
                if mr.is_empty() { None } else { Some(mr) },
            )
        }
    } else {
        // 无 @ → 仅 CurseForge
        (Some(s.to_string()), None)
    }
}

/// 处理中文名中的 * 通配符
/// * 替换为 " (Slug 去横线并首字母大写)"
fn process_wildcard(name: &str, slug_part: &str) -> String {
    if !name.contains('*') {
        return name.to_string();
    }
    // 提取 slug 用于替换
    let slug = if let Some(idx) = slug_part.find('@') {
        let cf = &slug_part[..idx];
        let mr = &slug_part[idx + 1..];
        if !cf.is_empty() {
            cf
        } else {
            mr
        }
    } else if slug_part.starts_with('@') {
        &slug_part[1..]
    } else if slug_part.ends_with('@') {
        &slug_part[..slug_part.len() - 1]
    } else {
        slug_part
    };
    // 去掉横线，首字母大写
    let replacement = capitalize(slug.replace('-', " "));
    name.replace('*', &format!(" ({})", replacement))
}

/// 首字母大写
fn capitalize(s: String) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// 通过 CurseForge Slug 查找中文译名
pub fn lookup_cf(slug: &str) -> Option<&str> {
    DATABASE.lookup_cf(slug).map(|e| e.chinese_name.as_str())
}

/// 通过 Modrinth Slug 查找中文译名
pub fn lookup_mr(slug: &str) -> Option<&str> {
    DATABASE.lookup_mr(slug).map(|e| e.chinese_name.as_str())
}

/// 通过平台和 Slug 查找中文译名
pub fn translate(platform: super::types::Platform, slug: &str) -> Option<&str> {
    use super::types::Platform;
    match platform {
        Platform::CurseForge => lookup_cf(slug),
        Platform::Modrinth => lookup_mr(slug),
    }
}

/// 通过平台和 Slug 查找 MC 百科 class id（用于拼接详情页 URL）
pub fn lookup_class_id(platform: super::types::Platform, slug: &str) -> Option<u32> {
    use super::types::Platform;
    match platform {
        Platform::CurseForge => DATABASE.lookup_cf(slug).map(|e| e.class_id),
        Platform::Modrinth => DATABASE.lookup_mr(slug).map(|e| e.class_id),
    }
}

// ===== 中文搜索本地映射 =====
//
// 参考 PCL2 `ResourceSearcher.vb` 189-290 行：
// 1. 检测中文关键词 → 2. 本地数据库模糊匹配 → 3. 提取英文 Slug/英文名 → 4. 重写搜索关键词

/// 中文关键词本地搜索
///
/// 在 moddata.txt 数据库中用模糊匹配查找中文关键词对应的 Mod 条目，
/// 提取英文 Slug/英文名作为新的搜索关键词。
///
/// # 返回
/// - `cf_keyword`：CurseForge 重写后的英文关键词（CurseForge 要求所有词都匹配，只选 1 个最佳 Mod）
/// - `mr_keyword`：Modrinth 重写后的英文关键词（多匹配加权出最佳英文词）
/// - `mr_slugs`：Modrinth Slug 直查列表（最多 100 个，用于调 `/v2/projects` 批量拉取）
///
/// 若均返回 None / 空，则本地数据库未匹配上，调用方应回退到原样透传
pub fn search_by_chinese(query: &str) -> RewriteResult {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return RewriteResult::default();
    }

    // 构建模糊搜索条目（与 PCL2 一致：中文名主部分权重 1，英文名+Slug 权重 0.5）
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
                    b.item.popularity
                        .cmp(&a.item.popularity)
                        .then_with(|| {
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
            for word in extract_words(&r.item.chinese_name, r.item.cf_slug.as_deref(), Some(mr_slug))
            {
                let similarity = if r.absolute_right { 1000.0 } else { r.similarity };
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
/// 移植自 PCL2 `ResourceSearcher.vb:202-230` `ExtractWords`
///
/// 来源：Slug（`-` 替换为空格）+ 英文名（中文名括号内部分）
/// 清洗：过滤单字、常见词（the/of/mod/and/forge/fabric/for/quilt/neoforge）、纯数字
/// 去重 + 子串去重
fn extract_words(
    chinese_name: &str,
    cf_slug: Option<&str>,
    mr_slug: Option<&str>,
) -> Vec<String> {
    let mut candidates: Vec<String> = Vec::new();

    // 从 Slug 提取（- 替换为空格）
    if let Some(slug) = cf_slug {
        candidates.push(slug.replace('-', " ").replace('/', " "));
    }
    if let Some(slug) = mr_slug {
        candidates.push(slug.replace('-', " ").replace('/', " "));
    }

    // 从英文名提取（中文名括号内部分）
    if let Some(idx) = chinese_name.rfind(" (") {
        let en_part = &chinese_name[idx + 2..];
        let en_clean = en_part.trim_end_matches(')').split(" - ").next().unwrap_or("");
        candidates.push(
            en_clean
                .replace('-', " ")
                .replace('/', " ")
                .replace(':', " ")
                .replace('(', " ")
                .replace(')', ""),
        );
    }

    // 分词、清洗、去重
    let stop_words = [
        "the", "of", "mod", "and", "forge", "fabric", "for", "quilt", "neoforge",
    ];
    let mut words: Vec<String> = candidates
        .iter()
        .flat_map(|c| c.split_whitespace())
        .map(|w| {
            w.trim_matches(|ch| ch == '{' || ch == '[' || ch == '(' || ch == '}' || ch == ']' || ch == ')')
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
fn can_form(s: &str, words: &[String]) -> bool {
    if s.is_empty() {
        return true;
    }
    words
        .iter()
        .any(|c| s.starts_with(c) && can_form(&s[c.len()..], words))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slug_part() {
        // @slug → 仅 Modrinth
        let (cf, mr) = parse_slug_part("@redpower2-core");
        assert_eq!(cf, None);
        assert_eq!(mr, Some("redpower2-core".to_string()));

        // slug@ → CF=MR=slug
        let (cf, mr) = parse_slug_part("industrial-craft@");
        assert_eq!(cf, Some("industrial-craft".to_string()));
        assert_eq!(mr, Some("industrial-craft".to_string()));

        // cf@mr → 双平台不同 slug
        let (cf, mr) = parse_slug_part("railcraft@railcraft-reborn");
        assert_eq!(cf, Some("railcraft".to_string()));
        assert_eq!(mr, Some("railcraft-reborn".to_string()));

        // slug → 仅 CurseForge
        let (cf, mr) = parse_slug_part("buildcraft");
        assert_eq!(cf, Some("buildcraft".to_string()));
        assert_eq!(mr, None);
    }

    #[test]
    fn test_process_wildcard() {
        // * 替换为 (Slug 去横线首字母大写)
        let result = process_wildcard("林业*", "forestry@");
        assert!(result.contains("Forestry"), "wildcard should be replaced, got {}", result);
    }

    #[test]
    fn test_extract_words() {
        let words = extract_words(
            "工业时代2 (Industrial Craft 2)",
            Some("industrial-craft"),
            Some("industrial-craft"),
        );
        assert!(words.contains(&"industrial".to_string()), "should contain industrial, got {:?}", words);
        assert!(words.contains(&"craft".to_string()), "should contain craft, got {:?}", words);
    }
}
