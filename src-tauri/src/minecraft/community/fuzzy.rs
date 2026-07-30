//! 模糊匹配算法
//!
//! 用于中文搜索本地映射：在 moddata.txt 数据库中用中文关键词匹配对应的
//! CurseForge / Modrinth Slug，把中文重写为英文关键词后再调平台 API。
//!
//! 算法核心：基于最长公共子串的相似度，结合长度增益与位置邻近度加权，
//! 归一化后得到 0~1 区间的分数。设计目标是对短查询（如中文 Mod 名）有较高
//! 容错性，同时避免长字符串误匹配。

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

/// 长度增益系数：匹配子串越长，增益增长越快（指数底数）
const LEN_GAIN_BASE: f64 = 1.42;
/// 长度增益偏移：使得 len=1 时增益为正、len=0 时增益为负（不触发）
const LEN_GAIN_OFFSET: f64 = 2.8;
/// 位置邻近度影响范围：|qp - sp| 不超过此值时才有加成
const POS_BONUS_RANGE: i32 = 4;
/// 位置邻近度最大加成倍数
const POS_BONUS_STEP: f64 = 0.25;
/// 归一化分子的调节常数（影响短源字符串的最终得分上限）
const NORM_NUMERATOR: f64 = 2.5;
/// 归一化分母的长度补偿（避免过短源字符串得分虚高）
const NORM_LEN_PAD: f64 = 12.0;

/// 计算源字符串与查询字符串的相似度（0-1）
///
/// 算法：基于最长公共子串的累积匹配长度，结合：
/// - 长度增益：`LEN_GAIN_BASE^(2+len) - LEN_GAIN_OFFSET`，子串越长权重越高
/// - 位置邻近度：查询位置与源位置越接近，权重越高
///   `1 + POS_BONUS_STEP * max(0, POS_BONUS_RANGE - |qp - sp|)`
/// - 归一化：`(累积增益 / 查询长度) * (NORM_NUMERATOR / sqrt(源长度 + NORM_LEN_PAD))`
/// - 短查询补偿：查询长度 ≤ 2 时额外乘以 `2.5 - 0.5 * 查询长度`
///
/// # 参数
/// - `source`：被匹配的源字符串（如数据库名条目）
/// - `query`：用户输入的查询字符串
pub fn search_similarity(source: &str, query: &str) -> f64 {
    if source.is_empty() || query.is_empty() {
        return 0.0;
    }

    // 统一小写并去除空格，降低噪声
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

    // 工作副本：匹配过的片段会被移除，避免同一片段被重复计入
    let mut work_source: Vec<char> = source_chars.clone();
    let mut len_sum = 0.0_f64;
    let mut qp = 0;

    while qp < query_len {
        let mut len_max = 0;
        let mut sp_max = 0;
        let current_len = work_source.len();
        let mut sp = 0;
        // 在工作副本中扫描，找出从 qp 开始能与 query 匹配的最长子串
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
            // 跳过已匹配片段，避免重复扫描
            sp += if len > 0 { len } else { 1 };
        }

        if len_max > 0 {
            // 移除已匹配片段，防止后续重复计入
            work_source.drain(sp_max..sp_max + len_max);
            // 长度增益：子串越长增益越高（指数增长）
            let inc_weight = LEN_GAIN_BASE.powi(2 + len_max as i32) - LEN_GAIN_OFFSET;
            // 位置邻近度：query 位置与 source 位置越接近，加成越高
            let pos_diff = (qp as i32 - sp_max as i32).abs();
            let pos_bonus = 1.0 + POS_BONUS_STEP * (POS_BONUS_RANGE - pos_diff).max(0) as f64;
            len_sum += inc_weight * pos_bonus;
        }
        // 推进 query 指针：匹配了就跳过已匹配长度，没匹配就前进 1
        qp += if len_max > 0 { len_max } else { 1 };
    }

    // 归一化：用源字符串长度做分母调节，避免长字符串得分虚高
    let length_factor = NORM_NUMERATOR / (source_len as f64 + NORM_LEN_PAD).sqrt();
    // 短查询补偿：1~2 字符的查询额外加权，提升短中文词的召回率
    let short_query_factor = if query_len <= 2 {
        2.5 - 0.5 * query_len as f64
    } else {
        1.0
    };
    (len_sum / query_len as f64) * length_factor * short_query_factor
}

/// 多文本源加权相似度
///
/// 每个 `SearchSource` 内部多个别名只取最高相似度，各 `SearchSource` 之间按权重加权平均
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
            result.push(&entries[idx]);
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
#[path = "fuzzy_tests.rs"]
mod tests;
