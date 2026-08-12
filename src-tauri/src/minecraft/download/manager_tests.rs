//! DownloadManager 单元测试

use super::*;

/// 构造指定 source_mode 的 DownloadManager（仅用于 reorder_urls 测试）
fn make_manager(mode: DownloadSourceMode) -> DownloadManager {
    DownloadManager::new(1, 0, 0, mode)
}

#[test]
fn test_reorder_urls_official_mode() {
    let manager = make_manager(DownloadSourceMode::Official);
    let urls = vec![
        "https://bmclapi2.bangbang93.com/file.jar".to_string(),
        "https://piston-meta.mojang.com/file.jar".to_string(),
        "https://mocdn.net/file.jar".to_string(),
    ];
    let result = manager.reorder_urls(&urls);
    // Official 模式只返回官方 URL
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "https://piston-meta.mojang.com/file.jar");
}

#[test]
fn test_reorder_urls_mirror_mode() {
    let manager = make_manager(DownloadSourceMode::Mirror);
    let urls = vec![
        "https://bmclapi2.bangbang93.com/file.jar".to_string(),
        "https://piston-meta.mojang.com/file.jar".to_string(),
        "https://mocdn.net/file.jar".to_string(),
    ];
    let result = manager.reorder_urls(&urls);
    // Mirror 模式只返回镜像 URL（bmclapi + mocdn）
    assert_eq!(result.len(), 2);
    assert!(result
        .iter()
        .all(|u| u.contains("bmclapi") || u.contains("mocdn")));
}

#[test]
fn test_reorder_urls_smart_mode() {
    let manager = make_manager(DownloadSourceMode::Smart);
    let urls = vec![
        "https://bmclapi2.bangbang93.com/file.jar".to_string(), // 镜像
        "https://piston-meta.mojang.com/file.jar".to_string(),  // 官方
        "https://mocdn.net/file.jar".to_string(),               // 镜像
    ];
    let result = manager.reorder_urls(&urls);
    // Smart 模式：官方优先，交替镜像（官方0, 镜像0, 镜像1）
    assert_eq!(result.len(), 3);
    // 第一个应是官方
    assert_eq!(result[0], "https://piston-meta.mojang.com/file.jar");
    // 后两个是镜像（顺序保留）
    assert!(result[1].contains("bmclapi") || result[1].contains("mocdn"));
    assert!(result[2].contains("bmclapi") || result[2].contains("mocdn"));
}

#[test]
fn test_reorder_urls_single_url() {
    let manager = make_manager(DownloadSourceMode::Smart);
    let urls = vec!["https://example.com/file.jar".to_string()];
    let result = manager.reorder_urls(&urls);
    // 单 URL 直接返回
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "https://example.com/file.jar");
}

#[test]
fn test_reorder_urls_empty() {
    let manager = make_manager(DownloadSourceMode::Smart);
    let urls: Vec<String> = vec![];
    let result = manager.reorder_urls(&urls);
    assert!(result.is_empty());
}

#[test]
fn test_reorder_urls_all_official_smart_mode() {
    let manager = make_manager(DownloadSourceMode::Smart);
    let urls = vec![
        "https://piston-meta.mojang.com/a.json".to_string(),
        "https://resources.download.minecraft.net/b/c".to_string(),
    ];
    let result = manager.reorder_urls(&urls);
    // 全是官方，Smart 模式返回全部（顺序保留）
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "https://piston-meta.mojang.com/a.json");
    assert_eq!(result[1], "https://resources.download.minecraft.net/b/c");
}

#[test]
fn test_reorder_urls_all_mirror_smart_mode() {
    let manager = make_manager(DownloadSourceMode::Smart);
    let urls = vec![
        "https://bmclapi2.bangbang93.com/a.json".to_string(),
        "https://mocdn.net/b/c".to_string(),
    ];
    let result = manager.reorder_urls(&urls);
    // 全是镜像，Smart 模式返回全部（顺序保留）
    assert_eq!(result.len(), 2);
    assert!(result[0].contains("bmclapi"));
    assert!(result[1].contains("mocdn"));
}

#[test]
fn test_from_config_preserves_fields() {
    let config = DownloadManagerConfig {
        max_threads: 8,
        chunk_count: 4,
        speed_limit: 1024,
        source_mode: DownloadSourceMode::Mirror,
        user_agent: None,
        app_handle: None,
        silent: false,
    };
    let manager = DownloadManager::from_config(&config);
    assert_eq!(manager.max_threads, 8);
    assert_eq!(manager.chunk_count, 4);
    assert_eq!(manager.speed_limit, 1024);
    assert_eq!(manager.source_mode, DownloadSourceMode::Mirror);
}
