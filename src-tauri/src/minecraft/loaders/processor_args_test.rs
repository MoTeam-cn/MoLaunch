//! processor 参数解析单测

use std::collections::HashMap;
use std::path::Path;

use super::{resolve_argument, resolve_arguments, ProcessorContext};

fn context_with(data: HashMap<String, String>) -> ProcessorContext<'static> {
    ProcessorContext {
        game_dir: Path::new("C:/mc"),
        minecraft_jar: Path::new("C:/mc/versions/26.3/26.3.jar"),
        installer: Path::new("C:/tmp/neoforge-installer.jar"),
        binpatch: Some(Path::new("C:/tmp/binpatches.lzma")),
        version: "26.3",
        data: Box::leak(Box::new(data)),
    }
}

#[test]
fn test_resolve_argument_全局占位符() {
    let ctx = context_with(HashMap::new());
    assert_eq!(
        resolve_argument("{MINECRAFT_JAR}", &ctx).replace('\\', "/"),
        "C:/mc/versions/26.3/26.3.jar"
    );
    assert_eq!(resolve_argument("{MINECRAFT_VERSION}", &ctx), "26.3");
    assert_eq!(
        resolve_argument("{LIBRARY_DIR}", &ctx).replace('\\', "/"),
        "C:/mc/libraries"
    );
    assert_eq!(resolve_argument("{SIDE}", &ctx), "client");
    assert!(resolve_argument("{INSTALLER}", &ctx).contains("neoforge-installer.jar"));
    assert!(resolve_argument("{BINPATCH}", &ctx).contains("binpatches"));
}

#[test]
fn test_resolve_argument_没有_binpatch_时是空串() {
    let mut ctx = context_with(HashMap::new());
    ctx.binpatch = None;
    assert_eq!(resolve_argument("{BINPATCH}", &ctx), "");
}

#[test]
fn test_resolve_argument_字面量去掉引号() {
    let mut data = HashMap::new();
    data.insert("MC_VERSION".to_string(), "'1.21.1'".to_string());
    let ctx = context_with(data);
    assert_eq!(resolve_argument("[MC_VERSION]", &ctx), "1.21.1");
}

#[test]
fn test_resolve_argument_绝对路径原样() {
    let mut data = HashMap::new();
    data.insert(
        "CLIENT_JAR".to_string(),
        "/home/user/mc/client.jar".to_string(),
    );
    let ctx = context_with(data);
    assert_eq!(
        resolve_argument("[CLIENT_JAR]", &ctx),
        "/home/user/mc/client.jar"
    );
}

#[test]
fn test_resolve_argument_坐标转本地路径() {
    let mut data = HashMap::new();
    data.insert(
        "MC_SRG".to_string(),
        "net.minecraft:client:26.3-1:srg".to_string(),
    );
    let ctx = context_with(data);
    let resolved = resolve_argument("[MC_SRG]", &ctx).replace('\\', "/");
    assert!(
        resolved.contains("net/minecraft/client/26.3-1"),
        "坐标必须落到 libraries 下，实际: {}",
        resolved
    );
}

#[test]
fn test_resolve_argument_未知键是空串() {
    let ctx = context_with(HashMap::new());
    assert_eq!(resolve_argument("[NOT_THERE]", &ctx), "");
}

#[test]
fn test_resolve_argument_普通参数原样返回() {
    let ctx = context_with(HashMap::new());
    assert_eq!(resolve_argument("--task", &ctx), "--task");
    assert_eq!(resolve_argument("decompile", &ctx), "decompile");
}

#[test]
fn test_resolve_arguments_整条命令行() {
    let mut data = HashMap::new();
    data.insert("MC_SRG".to_string(), "'srg-value'".to_string());
    let ctx = context_with(data);
    let args = vec![
        "--task".to_string(),
        "decompile".to_string(),
        "--input".to_string(),
        "{MINECRAFT_JAR}".to_string(),
        "--output".to_string(),
        "[MC_SRG]".to_string(),
    ];
    let resolved = resolve_arguments(&args, &ctx);
    assert_eq!(resolved.len(), 6);
    assert_eq!(resolved[0], "--task");
    assert_eq!(resolved[5], "srg-value");
    assert!(resolved[3].contains("26.3.jar"));
}
