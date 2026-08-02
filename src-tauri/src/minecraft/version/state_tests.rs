//! state 单元测试

use super::*;

#[test]
fn test_version_type_str_roundtrip() {
    let types = vec![
        VersionType::Release,
        VersionType::Snapshot,
        VersionType::Fool,
        VersionType::Old,
        VersionType::Forge,
        VersionType::NeoForge,
        VersionType::Fabric,
        VersionType::Quilt,
        VersionType::OptiFine,
        VersionType::LiteLoader,
    ];
    for vt in types {
        assert_eq!(VersionType::from_str(vt.as_str()), vt);
    }
}

#[test]
fn test_version_type_old_aliases() {
    assert_eq!(VersionType::from_str("old_alpha"), VersionType::Old);
    assert_eq!(VersionType::from_str("old_beta"), VersionType::Old);
    assert_eq!(VersionType::from_str("OLD_ALPHA"), VersionType::Old);
}

#[test]
fn test_is_release() {
    assert!(VersionType::Release.is_release());
    assert!(VersionType::Forge.is_release());
    assert!(!VersionType::Snapshot.is_release());
    assert!(!VersionType::Fool.is_release());
    assert!(!VersionType::Old.is_release());
}

#[test]
fn test_is_modded() {
    assert!(VersionType::Forge.is_modded());
    assert!(VersionType::Fabric.is_modded());
    assert!(VersionType::OptiFine.is_modded());
    assert!(!VersionType::Release.is_modded());
    assert!(!VersionType::Snapshot.is_modded());
}

#[test]
fn test_infer_from_loader() {
    assert_eq!(
        infer_from_loader(Some("47.2.0"), None, None, None, None, None),
        VersionType::Forge
    );
    assert_eq!(
        infer_from_loader(None, Some("20.4.0"), None, None, None, None),
        VersionType::NeoForge
    );
    assert_eq!(
        infer_from_loader(None, None, Some("0.16.0"), None, None, None),
        VersionType::Fabric
    );
    assert_eq!(
        infer_from_loader(None, None, None, None, Some("HD_U_I7"), None),
        VersionType::OptiFine
    );
    assert_eq!(
        infer_from_loader(None, None, None, None, None, None),
        VersionType::Release
    );
}
