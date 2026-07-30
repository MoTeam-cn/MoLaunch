//! 模糊匹配算法
//!
//! 移植自 PCL2 `ModBase.vb:818-946` 的 `SearchSimilarity` / `Search` 算法。
//! 用于中文搜索本地映射：在 moddata.txt 数据库中用中文关键词匹配对应的 CurseForge/Modrinth Slug。
//!
//! 算法核心：基于最长公共子串的相似度，考虑长度加成和位置加成。

/// 单个用于搜索的文本源
///
/// 每个文本源有多个别名，搜索时取最高相似度，按权重加权
#[derive(Clone)]
pub struct SearchSource {
    /// 别名列表（多个别名只取最高的一个相似度）
    pub aliases: Vec<String>,
    /// 权重
    pub weight: f64,
}

impl SearchSource {
    pub fn new(aliases: Vec<String>, weight: f64) -> Self {
        Self { aliases, weight }
    }

    pub fn from_text(text: String, weight: f64) -> Self {
        Self {
            aliases: vec![text],
            weight,
        }
    }
}

/// 用于搜索的项目
pub struct SearchEntry<T> {
    /// 源数据
    pub item: T,
    /// 文本源列表
    pub search_source: Vec<SearchSource>,
    /// 相似度（搜索后填充）
    pub similarity: f64,
    /// 是否完全匹配（搜索后填充）
    pub absolute_right: bool,
}

impl<T> SearchEntry<T> {
    pub fn new(item: T, search_source: Vec<SearchSource>) -> Self {
        Self {
            item,
            search_source,
            similarity: 0.0,
            absolute_right: false,
        }
    }
}

/// 计算源字符串与查询字符串的相似度（0-1）
///
/// 移植自 PCL2 `ModBase.vb:818` `SearchSimilarity`
///
/// 算法：基于最长公共子串，考虑长度加成（`1.4^(3+len) - 3.6`）和位置加成（`1 + 0.3 * max(0, 3-|qp-sp|)`）
/// 最终分数：`(lenSum / queryLength) * (3 / sqrt(sourceLength + 15))`
/// 短查询（≤2 字符）额外乘以 `3 - queryLength`
pub fn search_similarity(source: &str, query: &str) -> f64 {
    if source.is_empty() || query.is_empty() {
        return 0.0;
    }

    // 都做 lowercase + 去空格
    let source_lower: String = source
        .to_lowercase()
        .chars()
        .filter(|c| *c != ' ')
        .collect();
    let query_lower: String = query
        .to_lowercase()
        .chars()
        .filter(|c| *c != ' ')
        .collect();

    let source_chars: Vec<char> = source_lower.chars().collect();
    let query_chars: Vec<char> = query_lower.chars().collect();
    let source_len = source_chars.len();
    let query_len = query_chars.len();
    if query_len == 0 {
        return 0.0;
    }

    let mut work_source: Vec<char> = source_chars.clone();
    let mut len_sum = 0.0_f64;
    let mut qp = 0;

    while qp < query_len {
        let mut len_max = 0;
        let mut sp_max = 0;
        let current_len = work_source.len();
        let mut sp = 0;
        while sp < current_len {
            let mut len = 0;
            while qp + len < query_len
                && sp + len < current_len
                && work_source[sp + len] == query_chars[qp + len]
            {
                len += 1;
            }
            if len > len_max {
                len_max = len;
                sp_max = sp;
            }
            sp += if len > 0 { len } else { 1 };
        }

        if len_max > 0 {
            // 移除已匹配的部分，防止重复匹配
            work_source.drain(sp_max..sp_max + len_max);
            // 长度加成
            let inc_weight = 1.4_f64.powi(3 + len_max as i32) - 3.6;
            // 位置加成（位置越接近开头权重越高）
            let pos_diff = (qp as i32 - sp_max as i32).abs();
            let pos_bonus = 1.0 + 0.3 * (3 - pos_diff).max(0) as f64;
            len_sum += inc_weight * pos_bonus;
        }
        qp += if len_max > 0 { len_max } else { 1 };
    }

    let length_factor = 3.0 / (source_len as f64 + 15.0).sqrt();
    let short_query_factor = if query_len <= 2 {
        3.0 - query_len as f64
    } else {
        1.0
    };
    (len_sum / query_len as f64) * length_factor * short_query_factor
}

/// 多文本源加权相似度
///
/// 每个 SearchSource 内部多个别名只取最高相似度，各 SearchSource 之间按权重加权平均
pub fn search_similarity_weighted(sources: &[SearchSource], query: &str) -> f64 {
    let mut total_weight = 0.0_f64;
    let mut sum = 0.0_f64;
    for source in sources {
        if source.aliases.is_empty() {
            continue;
        }
        let max_sim = source
            .aliases
            .iter()
            .map(|a| search_similarity(a, query))
            .fold(0.0_f64, f64::max);
        sum += max_sim * source.weight;
        total_weight += source.weight;
    }
    if total_weight == 0.0 {
        0.0
    } else {
        sum / total_weight
    }
}

/// 多段文本加权搜索，返回相似度较高的前 N 项
///
/// 移植自 PCL2 `ModBase.vb:916` `Search`
///
/// 会修改 `entries` 中每项的 `similarity` 与 `absolute_right` 字段。
///
/// # 参数
/// - `entries`：搜索条目列表
/// - `query`：查询字符串
/// - `max_blur_count`：返回的最大模糊结果数（完全匹配不计入此限制）
/// - `min_blur_similarity`：模糊结果要求的最低相似度
pub fn search<'a, T>(
    entries: &'a mut [SearchEntry<T>],
    query: &str,
    max_blur_count: usize,
    min_blur_similarity: f64,
) -> Vec<&'a SearchEntry<T>> {
    if entries.is_empty() {
        return Vec::new();
    }

    let query_parts: Vec<String> = query.split_whitespace().map(|s| s.to_lowercase()).collect();
    let mut candidates: Vec<usize> = Vec::new();

    for (idx, entry) in entries.iter_mut().enumerate() {
        entry.similarity = search_similarity_weighted(&entry.search_source, query);
        // 完全匹配：查询按空格分割后，每一段都能在某个搜索源的别名中找到子串匹配
        entry.absolute_right = !query_parts.is_empty()
            && query_parts.iter().all(|qp| {
                entry.search_source.iter().any(|src| {
                    src.aliases.iter().any(|alias| {
                        alias
                            .to_lowercase()
                            .replace(' ', "")
                            .contains(&qp.replace(' ', ""))
                    })
                })
            });

        if entry.absolute_right || entry.similarity >= min_blur_similarity {
            candidates.push(idx);
        }
    }

    // 排序：完全匹配优先，其次相似度
    candidates.sort_by(|&a, &b| {
        let ea = &entries[a];
        let eb = &entries[b];
        match (ea.absolute_right, eb.absolute_right) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => eb.similarity
                .partial_cmp(&ea.similarity)
                .unwrap_or(std::cmp::Ordering::Equal),
        }
    });

    // 收集结果：完全匹配全部保留，模糊结果限制数量
    let mut result: Vec<&SearchEntry<T>> = Vec::new();
    let mut blur_count = 0;
    for idx in candidates {
        let entry = &entries[idx];
        if entry.absolute_right {
            result.append(&mut vec![&entries[idx]]);
        } else {
            if blur_count >= max_blur_count {
                break;
            }
            result.push(&entries[idx]);
            blur_count += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_similarity_exact_match() {
        let score = search_similarity("工业时代2", "工业时代");
        assert!(score > 0.5, "exact match should have high score, got {}", score);
    }

    #[test]
    fn test_search_similarity_no_match() {
        let score = search_similarity("完全不同的文本xyz", "工业时代");
        assert!(score < 0.1, "no match should have low score, got {}", score);
    }

    #[test]
    fn test_search_similarity_empty() {
        assert_eq!(search_similarity("", "test"), 0.0);
        assert_eq!(search_similarity("test", ""), 0.0);
    }

    #[test]
    fn test_search_similarity_weighted() {
        let sources = vec![
            SearchSource::new(vec!["工业时代2".to_string()], 1.0),
            SearchSource::new(vec!["industrial craft".to_string()], 0.5),
        ];
        let score = search_similarity_weighted(&sources, "工业时代");
        assert!(score > 0.3, "weighted match should have decent score, got {}", score);
    }
}
