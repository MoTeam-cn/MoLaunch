//! 实例快照：记录 mods 目录清单，支持差异对比与一键回滚
//!
//! 快照只存元数据（文件名 + 大小 + 启用状态），不复制 jar 本体；
//! 回滚时按「缺的回补、多的禁用」处理，无法找回的文件单独报出。

use crate::log_info;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::sanitize_version_id;
use super::helpers::get_mods_dir;

/// 快照目录（位于版本目录下）
const SNAPSHOT_DIR: &str = ".molaunch-snapshots";
/// 最多保留的快照数量（超出时删最旧的）
const MAX_SNAPSHOTS: usize = 10;
/// 禁用态后缀（与 toggle_mod 保持一致）
const DISABLED_SUFFIX: &str = ".disabled";

/// 快照里的单个文件条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotFile {
    /// 文件显示名（不含 .disabled 后缀）
    pub name: String,
    /// 字节大小
    pub size: u64,
    /// 是否启用
    pub enabled: bool,
}

/// 快照元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMeta {
    /// 快照 ID（创建时间戳）
    pub id: String,
    /// 触发方式：manual / auto-before-install / auto-before-update / auto-before-delete
    pub kind: String,
    /// 创建时间（Unix 毫秒）
    pub created_at: u64,
    /// 文件清单
    pub files: Vec<SnapshotFile>,
}

/// 快照差异
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotDiff {
    /// 快照 ID
    pub id: String,
    /// 快照里有、当前没有（回滚会补回或重新启用）
    pub missing: Vec<String>,
    /// 当前有、快照里没有（回滚会禁用）
    pub added: Vec<String>,
    /// 两侧都有但启用状态不同
    pub toggled: Vec<String>,
}

/// 回滚结果
#[derive(Debug, Clone, Serialize)]
pub struct RestoreResult {
    /// 重新启用的文件
    pub restored: Vec<String>,
    /// 被禁用的文件
    pub disabled: Vec<String>,
    /// 无法恢复的文件（已删除且没有禁用副本）
    pub unresolved: Vec<String>,
}

/// 当前 Unix 毫秒时间戳
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 扫描 mods 目录，产出文件清单（按名称排序）
pub(crate) fn scan_mods(mods_dir: &Path) -> Result<Vec<SnapshotFile>, String> {
    let entries = match std::fs::read_dir(mods_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(format!("读取 mods 目录失败: {}", err)),
    };

    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(raw_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let (name, enabled) = match raw_name.strip_suffix(DISABLED_SUFFIX) {
            Some(stripped) => (stripped.to_string(), false),
            None => (raw_name.to_string(), true),
        };
        if !is_mod_file(&name) {
            continue;
        }
        let size = entry.metadata().map(|meta| meta.len()).unwrap_or(0);
        files.push(SnapshotFile {
            name,
            size,
            enabled,
        });
    }
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

/// 是否 mod 文件（jar / litemod）
fn is_mod_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".jar") || lower.ends_with(".litemod")
}

/// 计算快照与当前状态的差异
pub(crate) fn diff_files(snapshot: &[SnapshotFile], current: &[SnapshotFile]) -> SnapshotDiff {
    let mut missing = Vec::new();
    let mut toggled = Vec::new();
    for item in snapshot {
        match current.iter().find(|cur| cur.name == item.name) {
            Some(cur) => {
                if cur.enabled != item.enabled {
                    toggled.push(item.name.clone());
                }
            }
            None => missing.push(item.name.clone()),
        }
    }
    let added = current
        .iter()
        .filter(|cur| !snapshot.iter().any(|item| item.name == cur.name))
        .map(|cur| cur.name.clone())
        .collect();
    SnapshotDiff {
        id: String::new(),
        missing,
        added,
        toggled,
    }
}

/// 解析版本目录（mods 目录的父目录）
async fn resolve_version_dir(state: &AppState, version_id: &str) -> Result<PathBuf, String> {
    let mods_dir = get_mods_dir(state, version_id).await?;
    mods_dir
        .parent()
        .map(|dir| dir.to_path_buf())
        .ok_or_else(|| "无法定位版本目录".to_string())
}

/// 快照目录路径（不存在则创建）
fn ensure_snapshot_dir(version_dir: &Path) -> Result<PathBuf, String> {
    let dir = version_dir.join(SNAPSHOT_DIR);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|err| format!("创建快照目录失败: {}", err))?;
    }
    Ok(dir)
}

/// 创建快照
pub async fn create_snapshot(
    state: &AppState,
    version_id: String,
    kind: String,
    note: Option<String>,
) -> Result<SnapshotMeta, String> {
    sanitize_version_id(&version_id)?;
    let version_dir = resolve_version_dir(state, &version_id).await?;
    let mods_dir = get_mods_dir(state, &version_id).await?;
    let files = scan_mods(&mods_dir)?;
    let dir = ensure_snapshot_dir(&version_dir)?;

    let id = format!("{}", now_ms());
    let meta = SnapshotMeta {
        id: id.clone(),
        kind: if kind.trim().is_empty() {
            "manual".to_string()
        } else {
            kind
        },
        created_at: now_ms(),
        files,
    };
    let payload = serde_json::to_string_pretty(&meta).map_err(|err| err.to_string())?;
    std::fs::write(dir.join(format!("{}.json", id)), payload)
        .map_err(|err| format!("写入快照失败: {}", err))?;
    prune_snapshots(&dir);
    match note.as_deref() {
        Some(text) => log_info!("[Snapshot] 已创建 {}（{}）：{}", id, meta.kind, text),
        None => log_info!("[Snapshot] 已创建 {}（{}）", id, meta.kind),
    }
    Ok(meta)
}

/// 只保留最近 MAX_SNAPSHOTS 个快照
fn prune_snapshots(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut ids: Vec<String> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                return None;
            }
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(|stem| stem.to_string())
        })
        .collect();
    ids.sort();
    while ids.len() > MAX_SNAPSHOTS {
        let oldest = ids.remove(0);
        let _ = std::fs::remove_file(dir.join(format!("{}.json", oldest)));
    }
}

/// 读取单个快照
fn read_snapshot(dir: &Path, id: &str) -> Result<SnapshotMeta, String> {
    let path = dir.join(format!("{}.json", id));
    let raw = std::fs::read_to_string(&path).map_err(|err| format!("读取快照失败: {}", err))?;
    serde_json::from_str(&raw).map_err(|err| format!("解析快照失败: {}", err))
}

/// 列出快照（新的在前）
pub async fn list_snapshots(
    state: &AppState,
    version_id: String,
) -> Result<Vec<SnapshotMeta>, String> {
    sanitize_version_id(&version_id)?;
    let version_dir = resolve_version_dir(state, &version_id).await?;
    let dir = version_dir.join(SNAPSHOT_DIR);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let entries = std::fs::read_dir(&dir).map_err(|err| format!("读取快照目录失败: {}", err))?;
    let mut list = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if let Ok(meta) = read_snapshot(&dir, id) {
            list.push(meta);
        }
    }
    list.sort_by_key(|item| std::cmp::Reverse(item.created_at));
    Ok(list)
}

/// 对比快照与当前状态
pub async fn snapshot_diff(
    state: &AppState,
    version_id: String,
    id: String,
) -> Result<SnapshotDiff, String> {
    sanitize_version_id(&version_id)?;
    let version_dir = resolve_version_dir(state, &version_id).await?;
    let dir = version_dir.join(SNAPSHOT_DIR);
    let meta = read_snapshot(&dir, &id)?;
    let mods_dir = get_mods_dir(state, &version_id).await?;
    let current = scan_mods(&mods_dir)?;
    let mut diff = diff_files(&meta.files, &current);
    diff.id = id;
    Ok(diff)
}

/// 回滚到指定快照
pub async fn restore_snapshot(
    state: &AppState,
    version_id: String,
    id: String,
) -> Result<RestoreResult, String> {
    sanitize_version_id(&version_id)?;
    let version_dir = resolve_version_dir(state, &version_id).await?;
    let dir = version_dir.join(SNAPSHOT_DIR);
    let meta = read_snapshot(&dir, &id)?;
    let mods_dir = get_mods_dir(state, &version_id).await?;
    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir).map_err(|err| format!("创建 mods 目录失败: {}", err))?;
    }
    let current = scan_mods(&mods_dir)?;

    let mut restored = Vec::new();
    let mut disabled = Vec::new();
    let mut unresolved = Vec::new();

    // 1) 快照里有、当前缺失或状态不同的文件
    for item in &meta.files {
        let enabled_path = mods_dir.join(&item.name);
        let disabled_path = mods_dir.join(format!("{}{}", item.name, DISABLED_SUFFIX));
        let current_entry = current.iter().find(|cur| cur.name == item.name);
        match current_entry {
            None => {
                if disabled_path.exists() {
                    std::fs::rename(&disabled_path, &enabled_path)
                        .map_err(|err| format!("恢复 {} 失败: {}", item.name, err))?;
                    if item.enabled {
                        restored.push(item.name.clone());
                    }
                } else {
                    unresolved.push(item.name.clone());
                }
            }
            Some(cur) => {
                if cur.enabled == item.enabled {
                    continue;
                }
                if item.enabled {
                    std::fs::rename(&disabled_path, &enabled_path)
                        .map_err(|err| format!("恢复 {} 失败: {}", item.name, err))?;
                    restored.push(item.name.clone());
                } else {
                    std::fs::rename(&enabled_path, &disabled_path)
                        .map_err(|err| format!("禁用 {} 失败: {}", item.name, err))?;
                    disabled.push(item.name.clone());
                }
            }
        }
    }

    // 2) 当前有、快照里没有的文件：禁用（不删除，随时可恢复）
    for cur in &current {
        if meta.files.iter().any(|item| item.name == cur.name) || !cur.enabled {
            continue;
        }
        let enabled_path = mods_dir.join(&cur.name);
        let disabled_path = mods_dir.join(format!("{}{}", cur.name, DISABLED_SUFFIX));
        std::fs::rename(&enabled_path, &disabled_path)
            .map_err(|err| format!("禁用 {} 失败: {}", cur.name, err))?;
        disabled.push(cur.name.clone());
    }

    log_info!(
        "[Snapshot] 回滚到 {}：恢复 {} 个，禁用 {} 个，未找回 {} 个",
        id,
        restored.len(),
        disabled.len(),
        unresolved.len()
    );

    Ok(RestoreResult {
        restored,
        disabled,
        unresolved,
    })
}

/// 删除快照
pub async fn delete_snapshot(
    state: &AppState,
    version_id: String,
    id: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    let version_dir = resolve_version_dir(state, &version_id).await?;
    let path = version_dir.join(SNAPSHOT_DIR).join(format!("{}.json", id));
    if path.exists() {
        std::fs::remove_file(&path).map_err(|err| format!("删除快照失败: {}", err))?;
    }
    Ok(())
}
