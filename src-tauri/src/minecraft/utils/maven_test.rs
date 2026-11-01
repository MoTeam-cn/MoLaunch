use super::*;

#[test]
fn test_maven_普通坐标_默认jar() {
    assert_eq!(
        maven_to_relative_path("net.neoforged:srgutils:1.0.10"),
        "net/neoforged/srgutils/1.0.10/srgutils-1.0.10.jar"
    );
}

#[test]
fn test_maven_带classifier_默认jar() {
    assert_eq!(
        maven_to_relative_path("net.minecraft:client:26.3-1:srg"),
        "net/minecraft/client/26.3-1/client-26.3-1-srg.jar"
    );
}

#[test]
fn test_maven_扩展名后缀替换默认jar() {
    assert_eq!(
        maven_to_relative_path("net.minecraft:client_mappings:26.3-1:client@txt"),
        "net/minecraft/client_mappings/26.3-1/client_mappings-26.3-1-client.txt"
    );
}

#[test]
fn test_maven_扩展名后缀无classifier() {
    assert_eq!(
        maven_to_relative_path("net.neoforged:mergetool:1.1.5@zip"),
        "net/neoforged/mergetool/1.1.5/mergetool-1.1.5.zip"
    );
}

#[test]
fn test_maven_非法坐标原样返回() {
    assert_eq!(
        maven_to_relative_path("not-a-coordinate"),
        "not-a-coordinate"
    );
}

#[test]
fn test_maven_转绝对路径() {
    let path = maven_to_local_path("a.b:c:1.0", Path::new("C:/mc"));
    assert!(path.ends_with("c-1.0.jar"));
}
