//! Mod 索引测试

use super::{normalize, ModIndex};

#[test]
fn test_normalize_strips_separators_and_lowercases() {
    assert_eq!(normalize("Create-1.20.1-0.5.1.f"), "create1201051f");
    assert_eq!(normalize("JEI"), "jei");
}

#[test]
fn test_resolve_matches_jar_by_prefix() {
    let dir = std::env::temp_dir().join("molaunch-mod-index");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create dir");
    std::fs::write(dir.join("create-1.20.1-0.5.1.f.jar"), b"x").expect("write");
    std::fs::write(dir.join("jei-1.20.1-forge.jar.disabled"), b"x").expect("write");
    std::fs::write(dir.join("readme.txt"), b"x").expect("write");

    let index = ModIndex::load(&dir);
    assert_eq!(
        index.resolve("create").as_deref(),
        Some("create-1.20.1-0.5.1.f.jar")
    );
    assert_eq!(
        index.resolve("jei").as_deref(),
        Some("jei-1.20.1-forge.jar")
    );
    assert!(index.resolve("unknownmod").is_none());
    assert!(index.resolve("ab").is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_load_returns_empty_when_dir_missing() {
    let dir = std::env::temp_dir().join("molaunch-mod-index-missing");
    let _ = std::fs::remove_dir_all(&dir);
    let index = ModIndex::load(&dir);
    assert!(index.resolve("anything").is_none());
}
