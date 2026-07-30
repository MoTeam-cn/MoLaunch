//! isolation 单元测试

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
    let result = get_effective_game_dir(
        game_dir,
        "1.20.1-forge-47.2.0",
        IsolationMode::All,
        VersionType::Forge,
    );
    assert_eq!(
        result,
        PathBuf::from("/home/user/.minecraft/versions/1.20.1-forge-47.2.0")
    );

    // 非隔离模式下，使用根目录
    let result = get_effective_game_dir(
        game_dir,
        "1.20.1-forge-47.2.0",
        IsolationMode::Disabled,
        VersionType::Forge,
    );
    assert_eq!(result, PathBuf::from("/home/user/.minecraft"));

    // ModdedOnly 模式下，原版不隔离
    let result = get_effective_game_dir(
        game_dir,
        "1.20.1",
        IsolationMode::ModdedOnly,
        VersionType::Release,
    );
    assert_eq!(result, PathBuf::from("/home/user/.minecraft"));

    // ModdedOnly 模式下，Mod 版本隔离
    let result = get_effective_game_dir(
        game_dir,
        "1.20.1-forge-47.2.0",
        IsolationMode::ModdedOnly,
        VersionType::Forge,
    );
    assert_eq!(
        result,
        PathBuf::from("/home/user/.minecraft/versions/1.20.1-forge-47.2.0")
    );
}
