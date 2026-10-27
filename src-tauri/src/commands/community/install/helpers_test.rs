//! 整合包实例名 / 文件名校验单元测试
//!
//! 背景：实例名与压缩包文件名都会 join 进 <game_dir>/versions/<实例名>/，
//! 两者都是路径穿越 sink，必须各自有独立的校验（扩展名校验不承担路径安全）。

use super::{
    construct_cf_edge_url, resolve_cf_download_url, validate_instance_name,
    validate_modpack_extension,
};

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

/// 受限 CF 文件地址推导：fileId 按 1000 拆两段，文件名必须百分号编码
#[test]
fn test_construct_cf_edge_url_百分号编码文件名() {
    assert_eq!(
        construct_cf_edge_url(6_767_951, "jei 1.21.5+neoforge.jar"),
        "https://edge.forgecdn.net/files/6767/951/jei%201.21.5%2Bneoforge.jar"
    );
    assert_eq!(
        construct_cf_edge_url(1_234_005, "example.jar"),
        "https://edge.forgecdn.net/files/1234/5/example.jar"
    );
}

/// 布局恒为 id/1000 与 id%1000 两段：小 fileId 落在 0/ 组，不做特殊分支
#[test]
fn test_construct_cf_edge_url_短fileid同布局() {
    assert_eq!(
        construct_cf_edge_url(123, "a.jar"),
        "https://edge.forgecdn.net/files/0/123/a.jar"
    );
    // 六位 fileId 同样按 /1000 拆段（此前的「取前四位」实现在这里会算错）
    assert_eq!(
        construct_cf_edge_url(123_005, "b.jar"),
        "https://edge.forgecdn.net/files/123/5/b.jar"
    );
}

/// 文件名里的路径分隔符必须被编码，不能拼出穿越路径
#[test]
fn test_construct_cf_edge_url_路径穿越被编码() {
    let url = construct_cf_edge_url(2_725_062, "../../evil.jar");
    assert!(url.starts_with("https://edge.forgecdn.net/files/2725/62/"));
    assert!(
        !url.contains("../"),
        "反斜杠或斜杠不得以路径形式出现: {url}"
    );
    assert!(url.contains("%2F"), "斜杠必须编码为 %2F: {url}");
}

/// 官方地址优先；地址为空时由开关决定是否推导
#[test]
fn test_resolve_cf_download_url_开关语义() {
    let official = "https://edge.forgecdn.net/files/1/2/x.jar";
    assert_eq!(
        resolve_cf_download_url(Some(official), 1_234_005, "x.jar", true).as_deref(),
        Some(official)
    );
    // 空串等同于未下发地址，且关闭推导时不产生任何地址
    assert_eq!(
        resolve_cf_download_url(Some(""), 1_234_005, "x.jar", false),
        None
    );
    assert_eq!(
        resolve_cf_download_url(None, 1_234_005, "x.jar", true).as_deref(),
        Some("https://edge.forgecdn.net/files/1234/5/x.jar")
    );
    assert_eq!(
        resolve_cf_download_url(None, 1_234_005, "x.jar", false),
        None
    );
}
