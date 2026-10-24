//! 版本 JSON 的 JVM 参数过滤单元测试

use super::add_json_jvm_args;
use serde_json::json;
use std::path::Path;

fn build(jvm: serde_json::Value) -> Vec<String> {
    let value = json!({ "arguments": { "jvm": jvm } });
    let mut args = Vec::new();
    add_json_jvm_args(&mut args, &value, Path::new("C:/game"), "26.3");
    args
}

#[test]
fn test_add_json_jvm_args_丢弃_classpath_选项与占位值() {
    let args = build(json!(["-cp", "${classpath}"]));
    assert!(args.is_empty(), "-cp 与占位值都应被丢弃，实际: {:?}", args);
}

#[test]
fn test_add_json_jvm_args_丢弃独立占位值() {
    let args = build(json!(["${classpath}"]));
    assert!(args.is_empty());
}

#[test]
fn test_add_json_jvm_args_保留普通选项() {
    let args = build(json!([
        "-XX:StackShadowPages=32",
        "--enable-native-access=ALL-UNNAMED"
    ]));
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], "-XX:StackShadowPages=32");
}

#[test]
fn test_add_json_jvm_args_替换启动器占位符() {
    let args = build(json!([
        "-Dminecraft.launcher.brand=${launcher_name}",
        "-Dminecraft.launcher.version=${launcher_version}"
    ]));
    assert!(args[0].ends_with("=molaunch"), "实际: {}", args[0]);
    assert!(
        !args[1].contains("${"),
        "版本占位符应被替换，实际: {}",
        args[1]
    );
}

#[test]
fn test_add_json_jvm_args_丢弃_natives_与库路径() {
    let args = build(json!([
        "-Djava.library.path=${natives_directory}/java",
        "-Djna.tmpdir=${natives_directory}/jna"
    ]));
    assert!(
        args.is_empty(),
        "natives 相关参数应由调用方统一注入，实际: {:?}",
        args
    );
}

#[test]
fn test_add_json_jvm_args_规则不匹配时跳过() {
    let args = build(json!([
        { "rules": [{ "action": "allow", "os": { "name": "osx" } }], "value": "-XstartOnFirstThread" },
        "-Xss1M"
    ]));
    assert_eq!(args, vec!["-Xss1M".to_string()]);
}

// ===== 内存池参数（永久代 / 元空间） =====

use super::super::rules::add_memory_pool_args;

fn memory_pool_args(java_major: Option<u32>, max_memory: u32) -> Vec<String> {
    let mut args = Vec::new();
    add_memory_pool_args(&mut args, java_major, max_memory);
    args
}

#[test]
fn test_add_memory_pool_args_java7_注入永久代() {
    let args = memory_pool_args(Some(7), 2048);
    assert_eq!(args, vec!["-XX:MaxPermSize=1024M"]);
}

#[test]
fn test_add_memory_pool_args_小内存时永久代保底128m() {
    let args = memory_pool_args(Some(7), 256);
    assert_eq!(args, vec!["-XX:MaxPermSize=128M"]);
}

#[test]
fn test_add_memory_pool_args_java8_起不再注入永久代() {
    assert!(memory_pool_args(Some(8), 2048).is_empty());
    assert!(memory_pool_args(Some(17), 4096).is_empty());
}

#[test]
fn test_add_memory_pool_args_版本未知时不注入() {
    assert!(memory_pool_args(None, 2048).is_empty());
}
