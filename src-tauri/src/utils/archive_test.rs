//! ZIP 解压共享工具单元测试

use super::{determine_zip_prefix, extract_zip_safely};
use std::io::{Cursor, Write};
use std::path::Path;

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

fn temp_base(tag: &str) -> std::path::PathBuf {
    let base = std::env::temp_dir().join(format!("molaunch_archive_test_{}", tag));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("建临时目录");
    base
}

#[test]
fn test_determine_zip_prefix_单根目录返回前缀() {
    let names = vec!["pack/a.txt".to_string(), "pack/b/c.txt".to_string()];
    assert_eq!(determine_zip_prefix(&names), "pack/");
}

#[test]
fn test_determine_zip_prefix_扁平结构返回空() {
    let names = vec!["a.txt".to_string(), "pack/b.txt".to_string()];
    assert_eq!(determine_zip_prefix(&names), "");
}

#[test]
fn test_determine_zip_prefix_多根目录返回空() {
    let names = vec!["one/a.txt".to_string(), "two/b.txt".to_string()];
    assert_eq!(determine_zip_prefix(&names), "");
}

#[test]
fn test_determine_zip_prefix_空列表返回空() {
    assert_eq!(determine_zip_prefix(&[]), "");
}

#[test]
fn test_extract_zip_safely_拒绝目录条目穿越() {
    let base = temp_base("traversal");
    let dst = base.join("dst");
    std::fs::create_dir_all(&dst).expect("建 dst");
    for entry in ["../escaped_dir/", "../evil/deep/", "../escaped.txt"] {
        let bytes = build_zip(&[entry]);
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("读 ZIP");
        let result = extract_zip_safely(&mut archive, "", &dst, |_| true);
        assert!(result.is_err(), "应拒绝穿越条目: {:?}", entry);
        assert!(!base.join("escaped_dir").exists(), "不得在 dst 外建目录");
        assert!(!base.join("evil").exists(), "不得在 dst 外建目录");
        assert!(!base.join("escaped.txt").exists(), "不得在 dst 外建文件");
    }
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_extract_zip_safely_接受合法嵌套条目并应用过滤器() {
    let base = temp_base("normal");
    let dst = base.join("dst");
    std::fs::create_dir_all(&dst).expect("建 dst");
    let bytes = build_zip(&["assets/lang/", "assets/lang/zh_cn.json", "assets/skip.txt"]);
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("读 ZIP");
    let result = extract_zip_safely(&mut archive, "", &dst, |rel| !rel.ends_with("skip.txt"));
    assert!(result.is_ok(), "合法条目不应被拒绝: {:?}", result);
    assert!(Path::new(&dst).join("assets/lang/zh_cn.json").exists());
    assert!(
        !Path::new(&dst).join("assets/skip.txt").exists(),
        "过滤器应跳过该条目"
    );
    let _ = std::fs::remove_dir_all(&base);
}
