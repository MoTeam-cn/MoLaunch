//! 库候选链的纯逻辑测试

use super::{
    fallback_urls, legacy_repositories, push_unique, relative_artifact_path, ALIYUN_MAVEN_PUBLIC,
    MAVEN_CENTRAL, MAVEN_FORGE, MAVEN_NEOFORGE, MOJANG_LIBRARIES,
};

const JINPUT: &str =
    "net/java/jinput/jinput-platform/2.0.5/jinput-platform-2.0.5-natives-windows.jar";

#[test]
fn test_relative_path_优先用坐标并保留classifier() {
    let path = relative_artifact_path(
        Some("net.java.jinput:jinput-platform:2.0.5:natives-windows"),
        "",
    );
    assert_eq!(path, JINPUT);
}

#[test]
fn test_relative_path_坐标缺失时从本地路径推() {
    let path = relative_artifact_path(
        None,
        "C:\\game\\libraries\\com\\google\\guava\\guava\\32.1.2-jre\\guava-32.1.2-jre.jar",
    );
    assert_eq!(
        path,
        "com/google/guava/guava/32.1.2-jre/guava-32.1.2-jre.jar"
    );
}

#[test]
fn test_relative_path_推不出来时为空() {
    assert_eq!(relative_artifact_path(None, "unknown/path.jar"), "");
    // 坐标段数不足也算不出来
    assert_eq!(relative_artifact_path(Some("only:two"), ""), "");
}

#[test]
fn test_legacy_repositories_central系老构件() {
    for prefix in [
        "org/ow2/asm/asm/9.6/asm-9.6.jar",
        "org/scala-lang/scala-library/2.11.1/scala-library-2.11.1.jar",
        "org/jline/jline/3.21.0/jline-3.21.0.jar",
        "net/java/dev/jna/jna/5.13.0/jna-5.13.0.jar",
        "com/typesafe/akka/akka-actor_2.11/2.3.3/akka-actor_2.11-2.3.3.jar",
    ] {
        assert_eq!(legacy_repositories(prefix), &[MAVEN_CENTRAL], "{prefix}");
    }
}

#[test]
fn test_legacy_repositories_加载器自家仓库() {
    assert_eq!(
        legacy_repositories("net/minecraftforge/forge/1.20.1-47.2.0/forge-1.20.1-47.2.0.jar"),
        &[MAVEN_FORGE]
    );
    assert_eq!(
        legacy_repositories("cpw/mods/modlauncher/10.0.9/modlauncher-10.0.9.jar"),
        &[MAVEN_FORGE]
    );
    assert_eq!(
        legacy_repositories("net/neoforged/neoforge/21.1.0/neoforge-21.1.0.jar"),
        &[MAVEN_NEOFORGE]
    );
}

#[test]
fn test_legacy_repositories_普通构件猜不出() {
    assert!(legacy_repositories("com/mojang/brigadier/1.0.18/brigadier-1.0.18.jar").is_empty());
}

#[test]
fn test_fallback_urls_顺序与阿里云在最后() {
    let urls = fallback_urls(JINPUT);
    assert!(urls[0].contains("bmclapi") && urls[0].contains("/maven/"));
    assert!(urls[1].contains("bmclapi") && urls[1].contains("/libraries/"));
    assert_eq!(urls[2], format!("{MOJANG_LIBRARIES}/{JINPUT}"));
    // jinput 属于 Central 系老构件，兜底里应有 Central，随后才是阿里云
    assert!(urls.iter().any(|u| u.starts_with(MAVEN_CENTRAL)));
    assert_eq!(
        urls.last().expect("非空"),
        &format!("{ALIYUN_MAVEN_PUBLIC}/{JINPUT}")
    );
}

#[test]
fn test_fallback_urls_空路径返回空() {
    assert!(fallback_urls("").is_empty());
}

#[test]
fn test_push_unique_保序去重() {
    let mut urls = vec!["a".to_string()];
    push_unique(&mut urls, "a".to_string());
    push_unique(&mut urls, "b".to_string());
    assert_eq!(urls, vec!["a".to_string(), "b".to_string()]);
}
