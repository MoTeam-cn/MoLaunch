//! 资源包编辑器 explore 模块单元测试

use std::fs;

use super::{build_tree, classify_file, pack_format_to_version, resolve_in_work_dir};

#[test]
fn test_classify_file() {
    assert_eq!(classify_file("pack.mcmeta"), "mcmeta");
    assert_eq!(classify_file("assets/minecraft/lang/zh_cn.json"), "lang");
    assert_eq!(
        classify_file("assets/minecraft/models/block/stone.json"),
        "model"
    );
    assert_eq!(
        classify_file("assets/minecraft/textures/block/stone.png"),
        "png"
    );
    assert_eq!(
        classify_file("assets/minecraft/sounds/step/wood.ogg"),
        "ogg"
    );
    assert_eq!(
        classify_file("assets/minecraft/blockstates/stone.json"),
        "json"
    );
    assert_eq!(classify_file("readme.txt"), "text");
    assert_eq!(classify_file("data/random.bin"), "other");
}

#[test]
fn test_pack_format_to_version() {
    assert_eq!(pack_format_to_version(1), "1.6–1.8");
    assert_eq!(pack_format_to_version(15), "1.19.5–1.20.1");
    assert_eq!(pack_format_to_version(34), "1.20.5–1.21.x");
    assert_eq!(pack_format_to_version(999), "未知版本");
}

#[test]
fn test_resolve_in_work_dir_rejects_parent_dir() {
    let dir = std::env::temp_dir().join(format!("rp-test-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.png"), b"x").unwrap();

    assert!(resolve_in_work_dir(&dir, "../a.png").is_err());
    assert!(resolve_in_work_dir(&dir, "sub/../../a.png").is_err());
    assert!(resolve_in_work_dir(&dir, "/a.png").is_err());
    assert!(resolve_in_work_dir(&dir, "a.png").is_ok());

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_build_tree_classifies_and_marks_animated() {
    let dir = std::env::temp_dir().join(format!("rp-tree-test-{}", std::process::id()));
    fs::create_dir_all(dir.join("assets/minecraft/textures/block")).unwrap();
    fs::write(dir.join("pack.mcmeta"), r#"{"pack":{"pack_format":15}}"#).unwrap();
    fs::write(
        dir.join("assets/minecraft/textures/block/stone.png"),
        b"png",
    )
    .unwrap();
    fs::write(
        dir.join("assets/minecraft/textures/block/stone.png.mcmeta"),
        r#"{"animation":{}}"#,
    )
    .unwrap();
    fs::write(dir.join("assets/minecraft/lang/zh_cn.json"), "{}").unwrap();

    let tree = build_tree(&dir, "");
    assert_eq!(tree.kind, "dir");
    let assets = tree
        .children
        .iter()
        .find(|n| n.name == "assets")
        .expect("assets 目录");
    let minecraft = assets
        .children
        .iter()
        .find(|n| n.name == "minecraft")
        .expect("minecraft 命名空间");
    let textures = minecraft
        .children
        .iter()
        .find(|n| n.name == "textures")
        .expect("textures 目录");
    let block = textures
        .children
        .iter()
        .find(|n| n.name == "block")
        .expect("block 目录");
    let stone = block
        .children
        .iter()
        .find(|n| n.name == "stone.png")
        .expect("stone.png");
    assert_eq!(stone.file_type, "png");
    assert!(stone.animated);

    let lang = minecraft
        .children
        .iter()
        .find(|n| n.name == "lang")
        .expect("lang 目录");
    let zh = lang
        .children
        .iter()
        .find(|n| n.name == "zh_cn.json")
        .expect("zh_cn.json");
    assert_eq!(zh.file_type, "lang");

    fs::remove_dir_all(&dir).unwrap();
}
