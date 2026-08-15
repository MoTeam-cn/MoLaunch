//! 资源包编辑器 explore 模块单元测试

use std::fs;
use std::io::Write;

use base64::Engine;

use super::{
    build_tree, classify_file, export_inner, pack_format_to_version, resolve_in_work_dir,
    write_inner, RpExportParams, RpWriteParams,
};

/// 同步运行异步逻辑（测试用）
fn block_on<T>(fut: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(fut)
}

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

#[test]
fn test_write_text_and_base64() {
    let dir = std::env::temp_dir().join(format!("rp-write-test-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.json"), "{}").unwrap();
    fs::write(dir.join("tex.png"), b"old").unwrap();
    let work = dir.to_string_lossy().to_string();

    let r = write_inner(&RpWriteParams {
        work_dir: work.clone(),
        rel_path: "a.json".to_string(),
        kind: "text".to_string(),
        content: r#"{"pack":{"pack_format":15}}"#.to_string(),
    })
    .unwrap();
    assert!(r.success);
    assert_eq!(
        fs::read_to_string(dir.join("a.json")).unwrap(),
        r#"{"pack":{"pack_format":15}}"#
    );

    let img = base64::engine::general_purpose::STANDARD.encode(b"new-png-bytes");
    let r = write_inner(&RpWriteParams {
        work_dir: work.clone(),
        rel_path: "tex.png".to_string(),
        kind: "base64".to_string(),
        content: format!("data:image/png;base64,{}", img),
    })
    .unwrap();
    assert!(r.success);
    assert_eq!(fs::read(dir.join("tex.png")).unwrap(), b"new-png-bytes");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_write_rejects_parent_dir() {
    let dir = std::env::temp_dir().join(format!("rp-write-safe-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.json"), "{}").unwrap();

    let r = write_inner(&RpWriteParams {
        work_dir: dir.to_string_lossy().to_string(),
        rel_path: "../a.json".to_string(),
        kind: "text".to_string(),
        content: "x".to_string(),
    });
    assert!(r.is_err());

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_export_zip_roundtrip_preserves_comment() {
    let dir = std::env::temp_dir().join(format!("rp-export-test-{}", std::process::id()));
    fs::create_dir_all(dir.join("assets/minecraft/lang")).unwrap();
    fs::write(dir.join("pack.mcmeta"), r#"{"pack":{"pack_format":15}}"#).unwrap();
    fs::write(
        dir.join("assets/minecraft/lang/zh_cn.json"),
        r#"{"key":"值"}"#,
    )
    .unwrap();

    // 构造带注释的源 zip（模拟 loader 附加属性）
    let src_zip = dir.join("src.zip");
    {
        let f = fs::File::create(&src_zip).unwrap();
        let mut w = zip::ZipWriter::new(f);
        let opts = zip::write::SimpleFileOptions::default();
        w.start_file("pack.mcmeta", opts).unwrap();
        w.write_all(b"{}").unwrap();
        w.set_comment("loader-comment");
        w.finish().unwrap();
    }

    let out_zip = dir.join("out.zip");
    let r = block_on(export_inner(&RpExportParams {
        work_dir: dir.to_string_lossy().to_string(),
        path: out_zip.to_string_lossy().to_string(),
        format: "zip".to_string(),
        src_path: Some(src_zip.to_string_lossy().to_string()),
    }))
    .unwrap();
    assert!(r.success);

    let f = fs::File::open(&out_zip).unwrap();
    let mut archive = zip::ZipArchive::new(f).unwrap();
    assert_eq!(archive.comment(), b"loader-comment");
    assert_eq!(archive.len(), 2);
    assert!(archive.by_name("assets/minecraft/lang/zh_cn.json").is_ok());

    fs::remove_dir_all(&dir).unwrap();
}
