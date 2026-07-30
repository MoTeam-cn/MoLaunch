//! sources 单元测试

use super::*;

#[test]
fn test_is_mirror_url_bmclapi() {
    assert!(is_mirror_url("https://bmclapi2.bangbang93.com/version/1.20.1/json"));
    assert!(is_mirror_url("https://bmclapi-mirror.example.com/file.jar"));
}

#[test]
fn test_is_mirror_url_mocdn() {
    assert!(is_mirror_url("https://mocdn.net/file.jar"));
    assert!(is_mirror_url("https://meta.mocdn.net/version.json"));
}

#[test]
fn test_is_mirror_url_mcimirror() {
    assert!(is_mirror_url("https://mod.mcimirror.top/file.jar"));
    assert!(is_mirror_url("https://mcimirror.top/asset.zip"));
}

#[test]
fn test_is_mirror_url_official() {
    assert!(!is_mirror_url("https://piston-meta.mojang.com/v1/packages/abc.json"));
    assert!(!is_mirror_url("https://resources.download.minecraft.net/abc/def"));
    assert!(!is_mirror_url("https://meta.fabricmc.net/v2/versions/loader"));
}

#[test]
fn test_is_mirror_url_empty_and_edge() {
    assert!(!is_mirror_url(""));
    assert!(!is_mirror_url("https://example.com"));
    // 包含 bmclapi 子串但不是域名
    assert!(is_mirror_url("https://example.com/bmclapi/path"));
}

#[test]
fn test_download_source_mode_from_str() {
    assert_eq!(DownloadSourceMode::from_str("official"), DownloadSourceMode::Official);
    assert_eq!(DownloadSourceMode::from_str("mirror"), DownloadSourceMode::Mirror);
    assert_eq!(DownloadSourceMode::from_str("smart"), DownloadSourceMode::Smart);
    // 未知值默认 Smart
    assert_eq!(DownloadSourceMode::from_str("unknown"), DownloadSourceMode::Smart);
    assert_eq!(DownloadSourceMode::from_str(""), DownloadSourceMode::Smart);
}
