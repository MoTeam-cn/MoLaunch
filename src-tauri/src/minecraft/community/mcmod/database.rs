//! mcmod 数据库加载与查询（全局单例 + CF/MR Slug → Entry 索引）

use once_cell::sync::Lazy;
use std::collections::HashMap;

use super::parsers::process_wildcard;

/// 全局数据库
pub(super) static DATABASE: Lazy<Database> = Lazy::new(Database::load);

/// 单条查询结果：中文名 + MC 百科 class id + 人气值
#[derive(Clone)]
pub(super) struct Entry {
    pub(super) chinese_name: String,
    pub(super) class_id: u32,
    /// MC 百科的浏览量逆序排行，1 代表浏览量最低（用于中文搜索排序权重）
    #[allow(dead_code)]
    pub(super) popularity: u32,
}

/// 用于本地模糊匹配的内部条目
pub(super) struct ChineseSearchEntry {
    pub(super) cf_slug: Option<String>,
    pub(super) mr_slug: Option<String>,
    /// 完整中文名，如 "工业时代2 (Industrial Craft 2)"
    pub(super) chinese_name: String,
    /// MC 百科浏览量逆序排行
    pub(super) popularity: u32,
}

pub(super) struct Database {
    /// CurseForge Slug → 条目
    cf_map: HashMap<String, Entry>,
    /// Modrinth Slug → 条目
    mr_map: HashMap<String, Entry>,
    /// 用于中文反查的条目列表（含双平台 slug 信息）
    pub(super) entries: Vec<ChineseSearchEntry>,
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
            .map(|s| {
                s.split('|')
                    .map(|n| n.parse::<u32>().unwrap_or(0))
                    .collect()
            })
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
                let (cf_slug, mr_slug) = super::parsers::parse_slug_part(slug_part);
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
    pub(super) fn lookup_cf(&self, slug: &str) -> Option<&Entry> {
        self.cf_map.get(slug)
    }

    /// 通过 Modrinth Slug 查找条目
    pub(super) fn lookup_mr(&self, slug: &str) -> Option<&Entry> {
        self.mr_map.get(slug)
    }
}
