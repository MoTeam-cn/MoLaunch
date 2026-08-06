//! 版本 Setup 单元测试

use super::helpers::{extract_maven_version, parse_ini};
use super::VersionSetup;

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

#[test]
fn test_load_or_create_backfills_missing_loader_version() {
    use std::fs;

    let base = std::env::temp_dir().join(format!(
        "mol_setup_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let dir = base.join("TestPack");
    fs::create_dir_all(&dir).unwrap();

    // 旧格式 setup.ini：只有 Type/OriginalVersion，无 ForgeVersion
    fs::write(
        dir.join("setup.ini"),
        "[info]\nOriginalVersion=1.12.2\nType=forge\n",
    )
    .unwrap();
    // 版本 JSON：libraries 含 forge maven 坐标
    fs::write(
        dir.join("TestPack.json"),
        r#"{"id":"TestPack","type":"release","libraries":[{"name":"net.minecraftforge:forge:1.12.2-14.23.5.2860"}]}"#,
    )
    .unwrap();

    let setup = VersionSetup::load_or_create(&dir, "TestPack");
    assert_eq!(
        setup.loader.forge_version.as_deref(),
        Some("1.12.2-14.23.5.2860")
    );
    assert_eq!(setup.loader.original_version, "1.12.2");

    // 已持久化：setup.ini 应写入 ForgeVersion
    let content = fs::read_to_string(dir.join("setup.ini")).unwrap();
    assert!(content.contains("ForgeVersion=1.12.2-14.23.5.2860"));

    let _ = fs::remove_dir_all(&base);
}
