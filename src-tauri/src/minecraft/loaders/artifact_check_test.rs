//! 安装产物校验单测

use serde_json::json;

use super::{is_locally_generated, required_library_files};

#[test]
fn test_is_locally_generated_看有没有下载地址() {
    let downloadable = json!({
        "name": "com.google.guava:guava:31.1-jre",
        "downloads": { "artifact": { "path": "a/b.jar", "url": "https://example.com/b.jar" } }
    });
    assert!(!is_locally_generated(&downloadable));

    let generated = json!({
        "name": "net.minecraft:client:26.3-1:srg",
        "downloadable": false
    });
    assert!(is_locally_generated(&generated));
}

#[test]
fn test_is_locally_generated_不看名字只看地址() {
    let renamed = json!({
        "name": "net.neoforged:something:1.0:whatever-weird-suffix",
        "downloads": { "artifact": { "path": "x.jar", "url": "https://example.com/x.jar" } }
    });
    assert!(
        !is_locally_generated(&renamed),
        "有下载地址就是可下载的，命名变了也不该误判成生成物"
    );
}

#[test]
fn test_required_library_files_优先用声明的下载路径() {
    let game_dir = std::path::Path::new("C:/mc");
    let doc = json!({
        "libraries": [
            {
                "name": "com.google.guava:guava:31.1-jre",
                "downloads": { "artifact": { "path": "com/google/guava/guava/31.1-jre/guava-31.1-jre.jar", "url": "https://example.com/g.jar" } }
            }
        ]
    });
    let files = required_library_files(&doc, game_dir);
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0].path,
        game_dir.join("com/google/guava/guava/31.1-jre/guava-31.1-jre.jar")
    );
    assert!(!files[0].generated);
}

#[test]
fn test_required_library_files_没下载地址时按坐标推且标记生成物() {
    let game_dir = std::path::Path::new("C:/mc");
    let doc = json!({ "libraries": [{ "name": "net.minecraft:client:26.3-1:srg" }] });
    let files = required_library_files(&doc, game_dir);
    assert_eq!(files.len(), 1);
    assert!(files[0].generated, "没有下载地址的条目必须是本地生成物");
    let path = files[0].path.to_string_lossy().replace('\\', "/");
    assert!(
        path.contains("net/minecraft/client/26.3-1"),
        "实际: {}",
        path
    );
}

#[test]
fn test_required_library_files_跳过_natives() {
    let doc = json!({
        "libraries": [
            { "name": "org.lwjgl:lwjgl:3.3.1", "natives": { "windows": "natives-windows" } }
        ]
    });
    assert!(required_library_files(&doc, std::path::Path::new("C:/mc")).is_empty());
}

#[test]
fn test_required_library_files_空文档时返回空列表() {
    let doc = json!({});
    assert!(required_library_files(&doc, std::path::Path::new("C:/mc")).is_empty());
}
