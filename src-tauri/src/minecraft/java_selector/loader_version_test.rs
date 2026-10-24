//! 加载器版本解析测试

use super::{parse_loader_version, parse_semver3};
use serde_json::json;

fn version_json(name: &str) -> serde_json::Value {
    json!({ "libraries": [{ "name": name }, { "name": "org.ow2.asm:asm:9.6" }] })
}

#[test]
fn test_parse_loader_version_fabric_能解析出版本号() {
    let json = version_json("net.fabricmc:fabric-loader:0.15.11");
    assert_eq!(
        parse_loader_version(&json, "fabric").as_deref(),
        Some("0.15.11")
    );
}

#[test]
fn test_parse_loader_version_quilt_能解析出版本号() {
    let json = version_json("org.quiltmc:quilt-loader:0.20.0");
    assert_eq!(
        parse_loader_version(&json, "quilt").as_deref(),
        Some("0.20.0")
    );
}

#[test]
fn test_parse_loader_version_forge_带_mc_前缀时原样返回() {
    let json = version_json("net.minecraftforge:forge:1.20.1-47.2.0");
    assert_eq!(
        parse_loader_version(&json, "forge").as_deref(),
        Some("1.20.1-47.2.0")
    );
}

#[test]
fn test_parse_loader_version_加载器不存在时返回_none() {
    let json = version_json("net.fabricmc:fabric-loader:0.15.11");
    assert!(parse_loader_version(&json, "forge").is_none());
}

#[test]
fn test_parse_loader_version_未知加载器返回_none() {
    let json = version_json("net.fabricmc:fabric-loader:0.15.11");
    assert!(parse_loader_version(&json, "unknown-loader").is_none());
}

#[test]
fn test_parse_loader_version_无_libraries_字段返回_none() {
    let json = json!({ "id": "1.20.1-fabric" });
    assert!(parse_loader_version(&json, "fabric").is_none());
}

#[test]
fn test_parse_semver3_常规版本() {
    assert_eq!(parse_semver3("0.15.11"), Some((0, 15, 11)));
}

#[test]
fn test_parse_semver3_带预发布后缀() {
    assert_eq!(parse_semver3("0.15.11-beta.1"), Some((0, 15, 11)));
}

#[test]
fn test_parse_semver3_段数不足返回_none() {
    assert!(parse_semver3("0.15").is_none());
}

#[test]
fn test_parse_semver3_非数字返回_none() {
    assert!(parse_semver3("abc.def.ghi").is_none());
}
