//! MC 百科（mcmod.cn）数据库
//!
//! 参考 PCL2 WikiEntry.vb
//! 加载内置 moddata.txt，通过工程 Slug 查找中文译名和 MC 百科 class id
//! 仅对 Mod / 数据包类型生效
//!
//! 关键设计（与 PCL2 对齐）：moddata.txt 第 N 行 → mcmod.cn class id = N
//! 空行也占用行号（PCL2 WikiEntry.vb: i += 1 在 Continue For 之前）

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// 全局数据库
static DATABASE: Lazy<Database> = Lazy::new(Database::load);

/// 单条查询结果：中文名 + MC 百科 class id
#[derive(Clone)]
struct Entry {
    chinese_name: String,
    class_id: u32,
}

struct Database {
    /// CurseForge Slug → 条目
    cf_map: HashMap<String, Entry>,
    /// Modrinth Slug → 条目
    mr_map: HashMap<String, Entry>,
}

impl Database {
    fn load() -> Self {
        // 统一走 resources 模块（编译时 include_str! 嵌入二进制，运行时零 IO）
        let data = crate::resources::read_resource("moddata.txt")
            .expect("嵌入资源 moddata.txt 缺失，请检查 resources.rs 注册");
        let mut cf_map: HashMap<String, Entry> = HashMap::new();
        let mut mr_map: HashMap<String, Entry> = HashMap::new();

        // 最后一行是 Popularity 排行数据，跳过
        let data_str: &str = &data;
        let lines: Vec<&str> = data_str.lines().collect();
        let entry_lines = if lines.len() > 1 { &lines[..lines.len() - 1] } else { &lines[..] };

        // 行号即 class id（参考 PCL2 WikiEntry.vb:39-82）
        // 关键：空行也计数（i += 1 在 Continue For 之前），否则行号会错位
        for (idx, line) in entry_lines.iter().enumerate() {
            let class_id = (idx + 1) as u32; // 行号从 1 开始
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

                // 解析 Slug 部分（参考 PCL2 WikiEntry.vb:55-67）
                let (cf_slug, mr_slug) = parse_slug_part(slug_part);
                let entry = Entry {
                    chinese_name: chinese_name.clone(),
                    class_id,
                };

                if let Some(cf) = cf_slug {
                    cf_map.insert(cf, entry.clone());
                }
                if let Some(mr) = mr_slug {
                    mr_map.insert(mr, entry);
                }
            }
        }

        crate::log_info!(
            "[Community] mcmod 数据库已加载: CF {} 条, MR {} 条",
            cf_map.len(),
            mr_map.len()
        );

        Database { cf_map, mr_map }
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

/// 解析 Slug 部分（参考 PCL2 WikiEntry.vb:55-67）
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
            (if cf.is_empty() { None } else { Some(cf) },
             if mr.is_empty() { None } else { Some(mr) })
        }
    } else {
        // 无 @ → 仅 CurseForge
        (Some(s.to_string()), None)
    }
}

/// 处理中文名中的 * 通配符（参考 PCL2 WikiEntry.vb:72-75）
/// * 替换为 " (Slug 去横线并首字母大写)"
fn process_wildcard(name: &str, slug_part: &str) -> String {
    if !name.contains('*') {
        return name.to_string();
    }
    // 提取 slug 用于替换
    let slug = if let Some(idx) = slug_part.find('@') {
        let cf = &slug_part[..idx];
        let mr = &slug_part[idx + 1..];
        if !cf.is_empty() { cf } else { mr }
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
