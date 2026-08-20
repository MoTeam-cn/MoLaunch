//! 资源包目录遍历：结构树构建 / 目录大小 / 会话目录 / 路径安全校验

use std::path::{Component, Path, PathBuf};

use super::parse::classify_file;
use super::RpTreeNode;

/// 递归构建结构树（目录在前，文件按类型分类；动画纹理标记同名 .png.mcmeta 存在）
pub(crate) fn build_tree(dir: &Path, rel: &str) -> RpTreeNode {
    let name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    let mut node = RpTreeNode {
        name,
        rel_path: rel.to_string(),
        kind: "dir".to_string(),
        file_type: String::new(),
        size: 0,
        animated: false,
        children: Vec::new(),
    };
    let mut entries = std::fs::read_dir(dir)
        .map(|r| r.flatten().collect::<Vec<_>>())
        .unwrap_or_default();
    entries.sort_by_key(|e| (e.path().is_file(), e.file_name()));
    for entry in entries {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        let child_rel = if rel.is_empty() {
            fname.clone()
        } else {
            format!("{}/{}", rel, fname)
        };
        if path.is_dir() {
            let mut child = build_tree(&path, &child_rel);
            node.size += child.size;
            child.name = fname;
            node.children.push(child);
        } else if path.is_file() {
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            node.size += size;
            let is_png = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("png"))
                .unwrap_or(false);
            let animated =
                is_png && Path::new(&format!("{}.mcmeta", path.to_string_lossy())).exists();
            let file_type = classify_file(&child_rel).to_string();
            node.children.push(RpTreeNode {
                name: fname,
                rel_path: child_rel,
                kind: "file".to_string(),
                file_type,
                size,
                animated,
                children: Vec::new(),
            });
        }
    }
    node
}

/// 递归统计目录总大小
pub(crate) fn dir_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += dir_size(&path);
            } else if let Ok(meta) = std::fs::metadata(&path) {
                total += meta.len();
            }
        }
    }
    total
}

/// 打开失败时的空结构树
pub(crate) fn empty_tree() -> RpTreeNode {
    RpTreeNode {
        name: String::new(),
        rel_path: String::new(),
        kind: "dir".to_string(),
        file_type: String::new(),
        size: 0,
        animated: false,
        children: Vec::new(),
    }
}

/// 创建 zip 会话的临时工作目录（temp_dir/molaunch-rp/rp-{pid}-{纳秒时间戳}）
pub(crate) fn create_session_dir() -> Result<PathBuf, String> {
    let root = std::env::temp_dir().join("molaunch-rp");
    std::fs::create_dir_all(&root).map_err(|e| format!("创建临时目录失败: {}", e))?;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = root.join(format!("rp-{}-{}", std::process::id(), nanos));
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建临时目录失败: {}", e))?;
    Ok(dir)
}

/// 清理 zip 会话临时工作目录（仅允许删除 molaunch-rp 根下的会话目录）
pub(crate) fn cleanup_work_dir(dir: &str) {
    let root = std::env::temp_dir().join("molaunch-rp");
    let root_str = root.to_string_lossy().replace('\\', "/");
    let dir_str = dir.replace('\\', "/");
    if dir_str.starts_with(&root_str) && dir_str.len() > root_str.len() {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// 路径安全：拦截绝对路径与 `..` 跳转，canonicalize 后必须位于工作目录内
pub(crate) fn resolve_in_work_dir(work_dir: &Path, rel_path: &str) -> Result<PathBuf, String> {
    let rel = Path::new(rel_path);
    if rel.is_absolute() {
        return Err("路径必须为包内相对路径".to_string());
    }
    if rel.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("路径包含非法跳转 (..)".to_string());
    }
    let work_canon = work_dir
        .canonicalize()
        .map_err(|e| format!("工作目录不可用: {}", e))?;
    let target = work_dir
        .join(rel)
        .canonicalize()
        .map_err(|e| format!("文件不存在: {}", e))?;
    if !target.starts_with(&work_canon) {
        return Err("路径超出资源包范围".to_string());
    }
    if !target.is_file() {
        return Err("目标不是文件".to_string());
    }
    Ok(target)
}
