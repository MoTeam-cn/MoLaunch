//! HMCL 启动器探测与实例枚举
//!
//! 配置：`hmcl.json`（Windows 系统版位于 `{data_dir}/.hmcl/hmcl.json`，便携版位于启动器目录 `.hmcl/` 下，
//! 可用环境变量 `HMCL_DATA_DIR` 覆盖），`configurations` 字段为 `{名称: {gameDir}}`，
//! `gameDir` 为相对启动器根目录的路径，解析为实例目录。

use std::path::{Path, PathBuf};

use crate::log_debug;

use super::detect::{data_dir, home_dir, instance_from_dir};
use crate::commands::tools::types::{ImportableInstance, LauncherKind, LauncherSource};

/// 探测 HMCL
pub(super) fn detect_hmcl() -> LauncherSource {
    let Some((base, config_path)) = find_hmcl_config() else {
        return empty_source("未检测到 HMCL 配置文件".to_string());
    };

    let json = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
    let Some(json) = json else {
        log_debug!(
            "[LauncherImport] HMCL 配置文件解析失败: {}",
            config_path.display()
        );
        return empty_source("HMCL 配置文件解析失败".to_string());
    };

    let mut instances: Vec<ImportableInstance> = json
        .get("configurations")
        .and_then(|v| v.as_object())
        .map(|cfgs| {
            cfgs.iter()
                .filter_map(|(name, cfg)| {
                    let game_dir = cfg.get("gameDir").and_then(|v| v.as_str())?;
                    let resolved = resolve_game_dir(&base, game_dir);
                    if !resolved.is_dir() {
                        log_debug!(
                            "[LauncherImport] HMCL 实例目录不存在，跳过 {}: {}",
                            name,
                            resolved.display()
                        );
                        return None;
                    }
                    Some(instance_from_dir(name, &resolved))
                })
                .collect()
        })
        .unwrap_or_default();
    instances.sort_by(|a, b| a.name.cmp(&b.name));

    LauncherSource {
        kind: LauncherKind::Hmcl,
        label: LauncherKind::Hmcl.label().to_string(),
        base_path: base.to_string_lossy().to_string(),
        instances,
    }
}

/// 查找 hmcl.json 并返回（启动器根目录, 配置文件路径）
fn find_hmcl_config() -> Option<(PathBuf, PathBuf)> {
    // 环境变量优先
    if let Some(dir) = std::env::var_os("HMCL_DATA_DIR") {
        let dir = PathBuf::from(dir);
        let candidate = dir.join("hmcl.json");
        if candidate.is_file() {
            return Some((dir, candidate));
        }
    }

    // 常见系统安装位置
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(data) = data_dir() {
        candidates.push(data.join(".hmcl/hmcl.json"));
        candidates.push(data.join("hmcl/hmcl.json"));
    }
    if let Some(home) = home_dir() {
        candidates.push(home.join(".hmcl/hmcl.json"));
    }

    for candidate in candidates {
        if candidate.is_file() {
            // base = hmcl.json 所在目录的父目录（`.hmcl` 的上一级）
            if let Some(base) = candidate.parent().and_then(|p| p.parent()) {
                return Some((base.to_path_buf(), candidate));
            }
        }
    }
    None
}

/// 解析 gameDir：绝对路径直接用，相对路径拼启动器根目录
fn resolve_game_dir(base: &Path, game_dir: &str) -> PathBuf {
    let path = PathBuf::from(game_dir);
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn empty_source(reason: String) -> LauncherSource {
    log_debug!("[LauncherImport] HMCL: {}", reason);
    LauncherSource {
        kind: LauncherKind::Hmcl,
        label: LauncherKind::Hmcl.label().to_string(),
        base_path: String::new(),
        instances: Vec::new(),
    }
}
