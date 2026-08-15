//! 存档目录 NBT 文件递归收集与分类（level / player / region / other）

use std::path::Path;

use super::super::types::NbtSaveFileItem;

/// 递归收集存档内 NBT 文件（仅递归 playerdata / region 目录，避免噪音）
pub(super) fn collect_save_files(dir: &Path, rel: &str, out: &mut Vec<NbtSaveFileItem>) {
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for entry in read.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let rel_path = if rel.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", rel, name)
        };
        if path.is_file() {
            let lower = name.to_lowercase();
            let parent_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("");
            let kind = if lower == "level.dat" {
                "level"
            } else if parent_name == "playerdata" && lower.ends_with(".dat") {
                "player"
            } else if lower.ends_with(".mca") {
                "region"
            } else if lower.ends_with(".dat") || lower.ends_with(".nbt") {
                "other"
            } else {
                continue;
            };
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            out.push(NbtSaveFileItem {
                rel_path,
                name,
                size,
                kind: kind.to_string(),
                path: path.to_str().unwrap_or("").to_string(),
            });
        } else if path.is_dir() && (name == "playerdata" || name == "region") {
            collect_save_files(&path, &rel_path, out);
        }
    }
}
