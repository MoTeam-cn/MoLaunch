//! CurseForge 启动器探测与实例枚举
//!
//! 实例位于 `{base}/Instances/{name}/`，以 `minecraftinstance.json` 为标志，
//! 游戏数据为实例目录内的 `.minecraft` 或目录本身。

use std::path::PathBuf;

use crate::log_debug;

use super::detect::{home_dir, instance_from_dir, sorted_subdirs};
use crate::commands::tools::types::{ImportableInstance, LauncherKind, LauncherSource};

/// 探测 CurseForge
pub(super) fn detect_curseforge() -> LauncherSource {
    let Some(base) = find_base() else {
        return empty_source("未检测到安装目录".to_string());
    };

    let instances_dir = base.join("Instances");
    if !instances_dir.is_dir() {
        return empty_source(format!("实例目录不存在: {}", instances_dir.display()));
    }

    let mut instances: Vec<ImportableInstance> = sorted_subdirs(&instances_dir)
        .iter()
        .filter(|d| d.join("minecraftinstance.json").is_file())
        .map(|d| {
            let name = d.file_name().unwrap_or_default().to_string_lossy();
            instance_from_dir(&name, d)
        })
        .collect();
    instances.sort_by(|a, b| a.name.cmp(&b.name));

    LauncherSource {
        kind: LauncherKind::Curseforge,
        label: LauncherKind::Curseforge.label().to_string(),
        base_path: base.to_string_lossy().to_string(),
        instances,
    }
}

/// 探测 CurseForge 根目录
fn find_base() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(home) = home_dir() {
        candidates.push(home.join("curseforge/minecraft"));
        candidates.push(home.join("Documents/curseforge/minecraft"));
    }
    candidates
        .into_iter()
        .find(|c| c.join("Instances").is_dir())
}

fn empty_source(reason: String) -> LauncherSource {
    log_debug!("[LauncherImport] CurseForge: {}", reason);
    LauncherSource {
        kind: LauncherKind::Curseforge,
        label: LauncherKind::Curseforge.label().to_string(),
        base_path: String::new(),
        instances: Vec::new(),
    }
}
