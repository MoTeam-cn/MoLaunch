//! 从 mods 目录文件名建立 modid → 显示名索引

use std::collections::HashMap;
use std::path::Path;

/// 扫描 mods 目录，返回 modid（小写）→ 显示名
///
/// 目录不存在或不可读时返回空索引，不报错——崩溃分析不应因为读不到目录而失败。
pub fn load_mod_index(mods_dir: &Path) -> HashMap<String, String> {
    let mut index = HashMap::new();
    let Ok(entries) = std::fs::read_dir(mods_dir) else {
        return index;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_jar = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("jar"))
            .unwrap_or(false);
        if !is_jar {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let (modid, display) = split_jar_name(stem);
        if modid.is_empty() {
            continue;
        }
        index.insert(modid.to_lowercase(), display.to_string());
    }
    index
}

/// 从 jar 文件名拆出 (modid, 显示名)
///
/// 约定：`create-1.20.1-0.5.1.jar` → (`create`, `create-1.20.1-0.5.1`)；
/// 无连字符时整体作为 modid。
pub fn split_jar_name(stem: &str) -> (&str, &str) {
    let modid = stem.split('-').next().unwrap_or(stem).trim();
    (modid, stem)
}
