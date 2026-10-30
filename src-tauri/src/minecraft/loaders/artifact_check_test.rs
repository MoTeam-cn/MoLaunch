//! 安装产物校验单测

use serde_json::json;

use super::{is_processor_artifact, required_library_files};

#[test]
fn test_is_processor_artifact_认出本地生成的那几类() {
    assert!(is_processor_artifact("net.minecraft:client:26.3-1:srg"));
    assert!(is_processor_artifact(
        "net.minecraft:client:1.20.1-20230612.114412:extra"
    ));
    assert!(is_processor_artifact(
        "net.minecraftforge:forge:1.20.1-47.4.0:patched"
    ));
    assert!(!is_processor_artifact("net.neoforged:neoforge:26.3.0.77"));
    assert!(!is_processor_artifact("com.google.guava:guava:31.1-jre"));
}

#[test]
fn test_required_library_files_优先用声明的下载路径() {
    let game_dir = std::path::Path::new("C:/mc");
    let doc = json!({
        "libraries": [
            {
                "name": "com.google.guava:guava:31.1-jre",
                "downloads": { "artifact": { "path": "com/google/guava/guava/31.1-jre/guava-31.1-jre.jar" } }
            }
        ]
    });
    let files = required_library_files(&doc, game_dir);
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0].1,
        game_dir.join("com/google/guava/guava/31.1-jre/guava-31.1-jre.jar")
    );
}

#[test]
fn test_required_library_files_没下载路径时按坐标推() {
    let game_dir = std::path::Path::new("C:/mc");
    let doc = json!({
        "libraries": [
            { "name": "net.minecraft:client:26.3-1:srg" }
        ]
    });
    let files = required_library_files(&doc, game_dir);
    assert_eq!(
        files.len(),
        1,
        "没有 downloads 的条目也要检查：processor 产物正是这种"
    );
    let path = files[0].1.to_string_lossy().replace('\\', "/");
    assert!(
        path.contains("net/minecraft/client/26.3-1"),
        "实际: {}",
        path
    );
}

#[test]
fn test_required_library_files_跳过_natives() {
    let game_dir = std::path::Path::new("C:/mc");
    let doc = json!({
        "libraries": [
            { "name": "org.lwjgl:lwjgl:3.3.1", "natives": { "windows": "natives-windows" } }
        ]
    });
    assert!(
        required_library_files(&doc, game_dir).is_empty(),
        "natives 走 classifier，另算"
    );
}

#[test]
fn test_required_library_files_空文档时返回空列表() {
    let doc = json!({});
    assert!(required_library_files(&doc, std::path::Path::new("C:/mc")).is_empty());
}
