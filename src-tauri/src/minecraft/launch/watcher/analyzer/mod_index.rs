//! Mod 归属索引：把堆栈里出现的名字映射到 mods 目录中的 jar 文件
//!
//! 崩溃堆栈里只有类名与包名，用户看不懂；本索引把 `create`、`jei` 这类
//! 名字对上 `create-1.20.1-0.5.1.f.jar`，弹窗才能给出「禁用哪个文件」。

use std::path::Path;

/// 一个 mod 文件条目
struct ModEntry {
    /// 归一化后的 jar 名（去掉版本号与分隔符，全小写）
    key: String,
    /// mods 目录下的文件名（含扩展名，不含 .disabled）
    file_name: String,
}

/// Mod 索引（mods 目录文件名到归一化名字的映射）
pub(super) struct ModIndex {
    entries: Vec<ModEntry>,
}

impl ModIndex {
    /// 空索引（未安装 Mod 或目录不存在时使用）
    pub(super) fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 扫描 mods 目录建立索引
    pub(super) fn load(mods_dir: &Path) -> Self {
        let Ok(entries) = std::fs::read_dir(mods_dir) else {
            return Self::empty();
        };
        let mut list = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(raw_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let file_name = raw_name.trim_end_matches(".disabled").to_string();
            let lower = file_name.to_lowercase();
            if !lower.ends_with(".jar") && !lower.ends_with(".litemod") {
                continue;
            }
            let stem = file_name
                .rsplit_once('.')
                .map(|(head, _)| head)
                .unwrap_or(&file_name);
            let key = normalize(stem);
            if key.is_empty() {
                continue;
            }
            list.push(ModEntry { key, file_name });
        }
        // 长名字优先匹配，避免 create 抢走 createaddition
        list.sort_by_key(|entry| std::cmp::Reverse(entry.key.len()));
        Self { entries: list }
    }

    /// 把关键字解析为 mods 目录下的文件名
    pub(super) fn resolve(&self, keyword: &str) -> Option<String> {
        let target = normalize(keyword);
        if target.len() < 3 {
            return None;
        }
        self.entries
            .iter()
            .find(|entry| entry.key.starts_with(&target) || target.starts_with(&entry.key))
            .map(|entry| entry.file_name.clone())
    }
}

/// 归一化：只保留 ASCII 字母数字并转小写
pub(super) fn normalize(text: &str) -> String {
    text.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}
