//! 版本 Setup 单元测试

use super::helpers::{extract_maven_version, parse_ini};

#[test]
fn test_parse_ini() {
    let content = "[info]\nOriginalVersion=1.20.1\nType=forge\nForgeVersion=47.2.0\n";
    let ini = parse_ini(content);
    assert_eq!(ini.get("OriginalVersion").unwrap(), "1.20.1");
    assert_eq!(ini.get("Type").unwrap(), "forge");
    assert_eq!(ini.get("ForgeVersion").unwrap(), "47.2.0");
}

#[test]
fn test_extract_maven_version() {
    assert_eq!(
        extract_maven_version(
            "net.minecraftforge:forge:1.20.1-47.2.0",
            "net.minecraftforge:forge:"
        ),
        Some("1.20.1-47.2.0".to_string())
    );
    assert_eq!(
        extract_maven_version(
            "net.fabricmc:fabric-loader:0.16.0",
            "net.fabricmc:fabric-loader:"
        ),
        Some("0.16.0".to_string())
    );
    assert_eq!(
        extract_maven_version("other:lib:1.0", "net.minecraftforge:forge:"),
        None
    );
}
