//! 库候选链测试：声明来源优先、镜像兜底，「无 sha1 也保留镜像」是既定取舍

use super::build_download_urls;
use super::types::LibEntry;

fn lib(url: Option<&str>, sha1: Option<&str>) -> LibEntry {
    LibEntry {
        original_name: Some("org.example:demo:1.0".to_string()),
        local_path: "/games/demo/libraries/org/example/demo/1.0/demo-1.0.jar".to_string(),
        size: 123,
        is_natives: false,
        sha1: sha1.map(str::to_string),
        url: url.map(str::to_string),
    }
}

#[test]
fn test_build_download_urls_声明来源排在最前() {
    let urls = build_download_urls(
        &lib(Some("https://libraries.minecraft.net/a.jar"), None),
        None,
    );
    assert_eq!(
        urls.first().map(String::as_str),
        Some("https://libraries.minecraft.net/a.jar")
    );
}

#[test]
fn test_build_download_urls_无哈希仍保留镜像候选() {
    // 没有 sha1 时镜像仍在候选里（排在官方之后）：砍掉镜像会让老版本直接下不来，
    // 这是有意的取舍，见 download.rs 的函数注释
    let urls = build_download_urls(
        &lib(Some("https://libraries.minecraft.net/a.jar"), None),
        None,
    );
    assert!(
        urls.iter().any(|url| url.contains("bmclapi")),
        "应保留镜像兜底: {:?}",
        urls
    );
    assert!(
        urls.iter().any(|url| url.contains("repo1.maven.org")),
        "应保留 Maven Central 兜底: {:?}",
        urls
    );
}

#[test]
fn test_build_download_urls_用户镜像优先且不重复() {
    let urls = build_download_urls(
        &lib(
            Some("https://libraries.minecraft.net/a.jar"),
            Some("deadbeef"),
        ),
        Some("https://mirror.example.com/maven"),
    );
    assert!(urls[0].starts_with("https://mirror.example.com/maven/"));
    let mut unique = urls.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), urls.len(), "候选里不应有重复: {:?}", urls);
}
