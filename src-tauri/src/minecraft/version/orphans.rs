//! 孤立版本目录探测
//!
//! `versions/` 下每个正常版本目录里都有一个同名 JSON。没有 JSON 的目录，基本是
//! 安装中断、或在磁盘上直接删过东西留下的残留：启动器不认它，用户也看不到它，
//! 但它照样占着空间。
//!
//! 这里**只报告、不删除**。删目录是不可逆操作，判断失误就把能玩的版本删了，
//! 所以先让问题可见，删除交给用户确认。

use std::path::{Path, PathBuf};

/// 一个疑似残留的版本目录
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OrphanDir {
    pub dir_name: String,
    /// 目录占用的字节数（递归求和）
    pub size_bytes: u64,
    /// 为什么判它是残留
    pub reason: String,
}

/// 扫描 `versions/` 下的孤立目录
pub(crate) fn scan_version_orphans(versions_dir: &Path) -> Vec<OrphanDir> {
    let Ok(entries) = std::fs::read_dir(versions_dir) else {
        return Vec::new();
    };
    let mut orphans = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) else {
            continue;
        };
        if has_version_json(&path, &name) {
            continue;
        }
        orphans.push(OrphanDir {
            dir_name: name,
            size_bytes: dir_size(&path),
            reason: "目录里没有同名 JSON，启动器不会识别它".to_string(),
        });
    }
    orphans.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));
    orphans
}

/// 版本目录里应当有的同名 JSON
fn has_version_json(dir: &Path, name: &str) -> bool {
    dir.join(format!("{}.json", name)).is_file()
}

/// 递归求和目录占用（读不到的子项按 0 计）
fn dir_size(dir: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut total = 0_u64;
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            total += dir_size(&entry.path());
        } else {
            total += meta.len();
        }
    }
    total
}

/// 把扫描结果的体量汇总成一句给人看的话
pub(crate) fn summarize(orphans: &[OrphanDir]) -> Option<String> {
    if orphans.is_empty() {
        return None;
    }
    let total: u64 = orphans.iter().map(|item| item.size_bytes).sum();
    Some(format!(
        "{} 个版本目录疑似安装残留，共 {} MB",
        orphans.len(),
        total / (1024 * 1024)
    ))
}

/// 版本目录根
pub(crate) fn versions_dir_of(game_dir: &Path) -> PathBuf {
    game_dir.join("versions")
}

#[cfg(test)]
#[path = "orphans_test.rs"]
mod orphans_test;
