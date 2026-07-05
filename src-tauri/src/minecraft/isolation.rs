//! 版本隔离模块
//!
//! 管理版本隔离策略，决定每个版本使用独立目录还是共享目录。
//!
//! 5 种隔离模式：
//! - 0: 关闭 — 所有版本共享 mods、config、saves 等目录
//! - 1: 隔离可安装 Mod 的版本 — Forge/Fabric/NeoForge 等使用独立目录，原版共享
//! - 2: 隔离非正式版 — snapshot/愚人节/远古版使用独立目录，正式版共享
//! - 3: 隔离非正式版 + Mod 版本 — 以上两种的组合
//! - 4: 隔离所有版本 — 每个版本完全独立

use std::path::{Path, PathBuf};
use super::version::state::VersionType;

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
        IsolationMode::NonReleaseAndModded => !version_type.is_release() || version_type.is_modded(),
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

/// 创建隔离版本所需的目录结构
pub fn ensure_isolated_dirs(version_dir: &Path) -> std::io::Result<()> {
    for dir in &["mods", "config", "saves", "resourcepacks", "shaderpacks", "crash-reports", "logs"] {
        std::fs::create_dir_all(version_dir.join(dir))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_isolate_disabled() {
        let mode = IsolationMode::Disabled;
        assert!(!should_isolate(mode, VersionType::Release));
        assert!(!should_isolate(mode, VersionType::Forge));
        assert!(!should_isolate(mode, VersionType::Snapshot));
    }

    #[test]
    fn test_should_isolate_modded_only() {
        let mode = IsolationMode::ModdedOnly;
        assert!(!should_isolate(mode, VersionType::Release));
        assert!(should_isolate(mode, VersionType::Forge));
        assert!(should_isolate(mode, VersionType::Fabric));
        assert!(should_isolate(mode, VersionType::OptiFine));
        assert!(!should_isolate(mode, VersionType::Snapshot));
    }

    #[test]
    fn test_should_isolate_non_release_only() {
        let mode = IsolationMode::NonReleaseOnly;
        assert!(!should_isolate(mode, VersionType::Release));
        assert!(!should_isolate(mode, VersionType::Forge)); // Forge is release
        assert!(should_isolate(mode, VersionType::Snapshot));
        assert!(should_isolate(mode, VersionType::Fool));
        assert!(should_isolate(mode, VersionType::Old));
    }

    #[test]
    fn test_should_isolate_all() {
        let mode = IsolationMode::All;
        assert!(should_isolate(mode, VersionType::Release));
        assert!(should_isolate(mode, VersionType::Forge));
        assert!(should_isolate(mode, VersionType::Snapshot));
    }

    #[test]
    fn test_get_effective_game_dir() {
        let game_dir = Path::new("/home/user/.minecraft");

        // 隔离模式下，Mod 版本使用版本目录
        let result = get_effective_game_dir(game_dir, "1.20.1-forge-47.2.0", IsolationMode::All, VersionType::Forge);
        assert_eq!(result, PathBuf::from("/home/user/.minecraft/versions/1.20.1-forge-47.2.0"));

        // 非隔离模式下，使用根目录
        let result = get_effective_game_dir(game_dir, "1.20.1-forge-47.2.0", IsolationMode::Disabled, VersionType::Forge);
        assert_eq!(result, PathBuf::from("/home/user/.minecraft"));

        // ModdedOnly 模式下，原版不隔离
        let result = get_effective_game_dir(game_dir, "1.20.1", IsolationMode::ModdedOnly, VersionType::Release);
        assert_eq!(result, PathBuf::from("/home/user/.minecraft"));

        // ModdedOnly 模式下，Mod 版本隔离
        let result = get_effective_game_dir(game_dir, "1.20.1-forge-47.2.0", IsolationMode::ModdedOnly, VersionType::Forge);
        assert_eq!(result, PathBuf::from("/home/user/.minecraft/versions/1.20.1-forge-47.2.0"));
    }
}
