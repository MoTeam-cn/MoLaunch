//! MultiMC / Prism Launcher 探测与实例枚举
//!
//! 根目录存在 `multimc.cfg`（MultiMC）或 `prismlauncher.cfg`（Prism），
//! 其中 `InstanceDir` 指定实例目录（缺省 `instances`），
//! 实例为 `{instances_dir}/{name}/`，含 `instance.cfg` 与 `.minecraft/` 游戏数据。

use std::path::{Path, PathBuf};

use crate::log_debug;

use super::detect::{data_dir, home_dir, instance_from_dir, sorted_subdirs};
use crate::commands::tools::types::{ImportableInstance, LauncherKind, LauncherSource};

/// 探测 MultiMC
pub(super) fn detect_multimc() -> LauncherSource {
    detect(LauncherKind::MultiMc)
}

/// 探测 Prism Launcher
pub(super) fn detect_prism() -> LauncherSource {
    detect(LauncherKind::Prism)
}

fn detect(kind: LauncherKind) -> LauncherSource {
    let Some(base) = find_base(kind) else {
        return empty_source(kind, "未检测到安装目录".to_string());
    };

    let instances_dir = read_instances_dir(&base, &kind).unwrap_or_else(|| base.join("instances"));
    if !instances_dir.is_dir() {
        return empty_source(kind, format!("实例目录不存在: {}", instances_dir.display()));
    }

    let mut instances: Vec<ImportableInstance> = sorted_subdirs(&instances_dir)
        .iter()
        .filter(|d| d.join("instance.cfg").is_file())
        .map(|d| {
            let name = d.file_name().unwrap_or_default().to_string_lossy();
            instance_from_dir(&name, d)
        })
        .collect();
    instances.sort_by(|a, b| a.name.cmp(&b.name));

    LauncherSource {
        kind,
        label: kind.label().to_string(),
        base_path: base.to_string_lossy().to_string(),
        instances,
    }
}

/// 读取 cfg 中的 `InstanceDir`（相对根目录或绝对路径）
fn read_instances_dir(base: &Path, kind: &LauncherKind) -> Option<PathBuf> {
    let cfg_name = match kind {
        LauncherKind::MultiMc => "multimc.cfg",
        LauncherKind::Prism => "prismlauncher.cfg",
        _ => return None,
    };
    let cfg_path = base.join(cfg_name);
    let content = std::fs::read_to_string(cfg_path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("InstanceDir=") {
            let value = value.trim();
            if value.is_empty() {
                return None;
            }
            let path = PathBuf::from(value);
            return Some(if path.is_absolute() {
                path
            } else {
                base.join(path)
            });
        }
    }
    None
}

/// 探测启动器根目录
fn find_base(kind: LauncherKind) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    let home = home_dir();

    match kind {
        LauncherKind::MultiMc => {
            if let Some(home) = &home {
                candidates.push(home.join("MultiMC"));
                candidates.push(home.join("Desktop/MultiMC"));
                candidates.push(home.join("Downloads/MultiMC"));
            }
            if let Some(system) = program_files() {
                candidates.push(system.join("MultiMC"));
            }
        }
        LauncherKind::Prism => {
            if let Some(data) = data_dir() {
                candidates.push(data.join("PrismLauncher"));
            }
            if let Some(home) = &home {
                candidates.push(home.join(".local/share/PrismLauncher"));
            }
        }
        _ => return None,
    }

    let cfg_name = match kind {
        LauncherKind::MultiMc => "multimc.cfg",
        LauncherKind::Prism => "prismlauncher.cfg",
        _ => return None,
    };
    candidates.into_iter().find(|c| c.join(cfg_name).is_file())
}

/// Program Files 目录（Windows）
fn program_files() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("ProgramFiles").map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Path::new("");
        None
    }
}

fn empty_source(kind: LauncherKind, reason: String) -> LauncherSource {
    log_debug!("[LauncherImport] {}: {}", kind.label(), reason);
    LauncherSource {
        kind,
        label: kind.label().to_string(),
        base_path: String::new(),
        instances: Vec::new(),
    }
}
