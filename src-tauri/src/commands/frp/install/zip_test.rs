//! ZIP 安全解压单元测试
//!
//! 背景：目录条目与文件条目是两条分支，早期只有文件分支做 Zip Slip 校验，
//! 目录分支直接 create_dir_all 并 continue，可用 "../x/" 在目标目录之外建目录。

use super::extract_zip_safely;
use std::io::{Cursor, Write};
use std::path::Path;

/// 构造只含目录/文件条目的内存 ZIP
fn build_zip(entries: &[&str]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default();
        for name in entries {
            writer.start_file(*name, options).expect("写入条目");
            writer.write_all(b"").expect("写入内容");
        }
        writer.finish().expect("收尾");
    }
    buf
}

/// 准备一个独立的临时工作目录
fn temp_base(tag: &str) -> std::path::PathBuf {
    let base = std::env::temp_dir().join(format!("molaunch_zip_test_{}", tag));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("建临时目录");
    base
}

/// 目录条目与文件条目都不得逃出目标目录
#[test]
fn test_extract_zip_safely_拒绝目录与文件条目穿越() {
    let base = temp_base("traversal");
    let dst = base.join("dst");
    std::fs::create_dir_all(&dst).expect("建 dst");

    for entry in ["../escaped_dir/", "../evil/deep/", "../escaped_file.txt"] {
        let bytes = build_zip(&[entry]);
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("读 ZIP");
        let result = extract_zip_safely(&mut archive, "", &dst);
        assert!(result.is_err(), "应拒绝穿越条目: {:?}", entry);
        assert!(!base.join("escaped_dir").exists(), "不得在 dst 外建目录");
        assert!(!base.join("evil").exists(), "不得在 dst 外建目录");
        assert!(
            !base.join("escaped_file.txt").exists(),
            "不得在 dst 外建文件"
        );
    }
    let _ = std::fs::remove_dir_all(&base);
}

/// 合法嵌套目录与文件必须仍然正常解压
#[test]
fn test_extract_zip_safely_接受合法嵌套条目() {
    let base = temp_base("normal");
    let dst = base.join("dst");
    std::fs::create_dir_all(&dst).expect("建 dst");

    let bytes = build_zip(&["assets/lang/", "assets/lang/zh_cn.json"]);
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("读 ZIP");
    let result = extract_zip_safely(&mut archive, "", &dst);
    assert!(result.is_ok(), "合法条目不应被拒绝: {:?}", result);
    assert!(Path::new(&dst).join("assets/lang/zh_cn.json").exists());
    let _ = std::fs::remove_dir_all(&base);
}
