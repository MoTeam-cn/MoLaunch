//! skin_resourcepack 单元测试

use super::*;

#[test]
fn test_pack_format() {
    assert_eq!(get_pack_format("1.12.2"), 4);
    assert_eq!(get_pack_format("1.16.5"), 7);
    assert_eq!(get_pack_format("1.19.2"), 9);
    assert_eq!(get_pack_format("1.19.3"), 12);
    assert_eq!(get_pack_format("1.20.1"), 15);
}

#[test]
fn test_is_1193_plus() {
    assert!(!is_version_1193_plus("1.19.2"));
    assert!(is_version_1193_plus("1.19.3"));
    assert!(is_version_1193_plus("1.20.1"));
}

#[test]
fn test_texture_paths_1192() {
    let paths = get_texture_paths("1.19.2", true);
    assert_eq!(paths.len(), 1);
    assert!(paths[0].contains("alex.png"));
}

#[test]
fn test_texture_paths_1193() {
    let paths = get_texture_paths("1.19.3", false);
    assert_eq!(paths.len(), 9);
    // DEFAULT_SKINS_1193 按字母序排列，第一个是 alex
    assert!(paths[0].contains("player/wide/alex.png"));
    assert!(paths[6].contains("player/wide/steve.png"));
}
