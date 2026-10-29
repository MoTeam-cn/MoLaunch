//! 版本目录的文件事实表
//!
//! 记下版本目录里每个文件的名字、大小、修改时间，下次安装/更新前拿来比对。
//! 有了这份事实，「这个文件是安装带来的」还是「用户自己放进来的」才分得清——
//! 否则更新只能二选一：覆盖（吃掉用户改动）或跳过（装不上新版本）。

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

const STORE_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileFact {
    /// 相对版本目录的路径（统一用 / 分隔）
    pub path: String,
    pub size: u64,
    pub mtime_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Store {
    version: u32,
    version_id: String,
    files: Vec<FileFact>,
}

/// 两次快照的差异
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct FileDiff {
    /// 现在有、之前没有：用户新增或新版本带来的
    pub added: Vec<String>,
    /// 之前有、现在没了：被删除
    pub removed: Vec<String>,
    /// 同名但大小或时间变了：被改写
    pub changed: Vec<String>,
}

/// 比对两份快照（纯函数）
pub(crate) fn diff(prev: &[FileFact], now: &[FileFact]) -> FileDiff {
    let mut result = FileDiff::default();
    for current in now {
        match prev.iter().find(|old| old.path == current.path) {
            None => result.added.push(current.path.clone()),
            Some(old) if old.size != current.size || old.mtime_ms != current.mtime_ms => {
                result.changed.push(current.path.clone());
            }
            Some(_) => {}
        }
    }
    for old in prev {
        if !now.iter().any(|current| current.path == old.path) {
            result.removed.push(old.path.clone());
        }
    }
    result
}

/// 给一个目录拍快照（只记元数据，读内容太贵）
pub(crate) fn snapshot(root: &Path) -> Vec<FileFact> {
    let mut facts = Vec::new();
    collect(root, root, &mut facts, 0);
    facts.sort_by(|a, b| a.path.cmp(&b.path));
    facts
}

fn collect(root: &Path, dir: &Path, facts: &mut Vec<FileFact>, depth: usize) {
    // 防御性深度上限：符号链接成环时不至于把栈耗光
    if depth > 16 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            collect(root, &path, facts, depth + 1);
            continue;
        }
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        facts.push(FileFact {
            path: relative.to_string_lossy().replace('\\', "/"),
            size: meta.len(),
            mtime_ms: meta
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        });
    }
}

/// 记录一次安装结果，并把「安装之外发生的改动」报出来
pub(crate) fn record(game_dir: &Path, version_id: &str) {
    let version_dir = game_dir.join("versions").join(version_id);
    if !version_dir.is_dir() {
        return;
    }
    let now = snapshot(&version_dir);
    let store_path = store_path(game_dir, version_id);

    if let Ok(text) = std::fs::read_to_string(&store_path) {
        if let Ok(prev) = serde_json::from_str::<Store>(&text) {
            if prev.version == STORE_VERSION {
                let drift = diff(&prev.files, &now);
                if !drift.added.is_empty() || !drift.changed.is_empty() {
                    crate::log_info!(
                        "[Install] 版本 {} 有 {} 个安装之外的文件、{} 个被改动过，更新时不会覆盖",
                        version_id,
                        drift.added.len(),
                        drift.changed.len()
                    );
                }
            }
        }
    }

    let store = Store {
        version: STORE_VERSION,
        version_id: version_id.to_string(),
        files: now,
    };
    let Ok(text) = serde_json::to_string(&store) else {
        return;
    };
    if let Some(parent) = store_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = crate::utils::fs::write_atomic(&store_path, text.as_bytes()) {
        crate::log_warn!("[Install] 文件事实表落盘失败: {}", e);
    }
}

fn store_path(game_dir: &Path, version_id: &str) -> PathBuf {
    game_dir
        .join(".molaunch")
        .join("instance_files")
        .join(format!("{}.json", sanitize(version_id)))
}

/// 版本名可能带路径分隔符（用户自定义名），落盘前换成下划线
fn sanitize(version_id: &str) -> String {
    version_id
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect()
}

#[cfg(test)]
#[path = "instance_files_test.rs"]
mod instance_files_test;
