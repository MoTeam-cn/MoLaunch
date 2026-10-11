//! 整合包实例名 / 文件名校验单元测试
//!
//! 背景：实例名与压缩包文件名都会 join 进 <game_dir>/versions/<实例名>/，
//! 两者都是路径穿越 sink，必须各自有独立的校验（扩展名校验不承担路径安全）。

use super::{validate_instance_name, validate_modpack_extension};

/// 路径分隔符与相对路径段必须被拒绝（否则可逃出 versions/ 目录）
#[test]
fn test_validate_instance_name_拒绝路径穿越() {
    for name in [
        "../../evil",
        "..",
        ".",
        "../x",
        "a/b",
        "/abs/path",
        "versions/../../evil",
    ] {
        assert!(
            validate_instance_name(name).is_err(),
            "应拒绝含路径分隔符或相对路径段的实例名: {:?}",
            name
        );
    }
}

/// 正常实例名必须通过（含中文、空格、连字符、点号）
#[test]
fn test_validate_instance_name_接受正常名称() {
    for name in [
        "1.20.1-forge-36.2.39",
        "我的整合包",
        "Create Above and Beyond",
        "pack_v2",
    ] {
        assert!(
            validate_instance_name(name).is_ok(),
            "应接受正常实例名: {:?}",
            name
        );
    }
}

/// 扩展名校验对路径穿越无感知，必须由 sanitize_file_name 兜底
#[test]
fn test_validate_modpack_extension_不承担路径安全() {
    assert!(validate_modpack_extension("../../../evil.zip").is_ok());
    assert!(crate::utils::path::sanitize_file_name("../../../evil.zip").is_err());
}
