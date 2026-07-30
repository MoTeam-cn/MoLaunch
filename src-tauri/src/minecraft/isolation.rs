//! 版本隔离模块
//!
//! 管理版本隔离策略（5 种模式：关闭/仅Mod/仅非正式/两者/全部），决定使用独立或共享目录。

use super::version::state::VersionType;
use std::path::{Path, PathBuf};

/// 版本隔离模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationMode {
    /// 关闭 - 所有版本共享
    Disabled = 0,
    /// 隔离可安装 Mod 的版本
    ModdedOnly = 1,
    /// 隔离非正式版
    NonReleaseOnly = 2,
    /// 隔离非正式版 + Mod 版本
    NonReleaseAndModded = 3,
    /// 隔离所有版本
    All = 4,
}

impl IsolationMode {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::Disabled,
            1 => Self::ModdedOnly,
            2 => Self::NonReleaseOnly,
            3 => Self::NonReleaseAndModded,
            _ => Self::All,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Disabled => "关闭 — 所有版本共享目录",
            Self::ModdedOnly => "隔离 Mod 版本 — Forge/Fabric 等独立，原版共享",
            Self::NonReleaseOnly => "隔离非正式版 — 快照/远古/愚人节版本独立",
            Self::NonReleaseAndModded => "隔离非正式版 + Mod 版本",
            Self::All => "隔离所有版本 — 每个版本完全独立",
        }
    }
}

/// 判断某个版本是否应该被隔离
pub fn should_isolate(mode: IsolationMode, version_type: VersionType) -> bool {
    match mode {
        IsolationMode::Disabled => false,
        IsolationMode::ModdedOnly => version_type.is_modded(),
        IsolationMode::NonReleaseOnly => !version_type.is_release(),
        IsolationMode::NonReleaseAndModded => {
            !version_type.is_release() || version_type.is_modded()
        }
        IsolationMode::All => true,
    }
}

/// 计算版本的有效游戏目录
///
/// - 隔离时返回 `{game_dir}/versions/{version_id}/`
/// - 非隔离时返回 `{game_dir}/`
pub fn get_effective_game_dir(
    game_dir: &Path,
    version_id: &str,
    mode: IsolationMode,
    version_type: VersionType,
) -> PathBuf {
    if should_isolate(mode, version_type) {
        game_dir.join("versions").join(version_id)
    } else {
        game_dir.to_path_buf()
    }
}

/// 创建隔离版本所需的基本目录结构（原版）
pub fn ensure_isolated_dirs(version_dir: &Path) -> std::io::Result<()> {
    for dir in &["saves", "config", "crash-reports", "logs"] {
        std::fs::create_dir_all(version_dir.join(dir))?;
    }
    Ok(())
}

/// 创建隔离版本所需的完整目录结构（Mod 版本）
pub fn ensure_modded_dirs(version_dir: &Path) -> std::io::Result<()> {
    // 先创建基本目录
    ensure_isolated_dirs(version_dir)?;
    // 再创建 Mod 相关目录
    for dir in &["mods", "resourcepacks", "shaderpacks"] {
        std::fs::create_dir_all(version_dir.join(dir))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "isolation_tests.rs"]
mod tests;
