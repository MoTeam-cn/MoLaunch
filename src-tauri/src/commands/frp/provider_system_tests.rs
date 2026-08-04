use super::*;
use std::collections::HashMap;

fn bundled() -> BinaryConfig {
    BinaryConfig {
        distribution: "bundled".to_string(),
        frpc_version: Some("0.51.3".to_string()),
        path: None,
        paths: Some(HashMap::from([
            (
                "windows_amd64".to_string(),
                "bin/LoliaFrp-windows-amd64.exe".to_string(),
            ),
            (
                "windows_arm64".to_string(),
                "bin/LoliaFrp-windows-arm64.exe".to_string(),
            ),
            (
                "linux_amd64".to_string(),
                "bin/LoliaFrp-linux-amd64".to_string(),
            ),
        ])),
        download: None,
        launch: None,
    }
}

#[test]
fn test_bundled_skip_other_platforms() {
    let (skip, current) = frpc_platform_skip(&bundled());
    let current_name = current.as_deref().expect("当前平台应有路径映射");
    assert_eq!(
        current_name,
        resolve_bundled_path(&bundled())
            .as_deref()
            .expect("当前平台应有路径映射")
    );
    assert_eq!(skip.len(), 2);
    // 当前平台的路径不在跳过集中
    assert!(!skip.contains(current_name));
}

#[test]
fn test_bundled_single_path_no_filter() {
    let binary = BinaryConfig {
        distribution: "bundled".to_string(),
        frpc_version: None,
        path: Some("frpc.exe".to_string()),
        paths: None,
        download: None,
        launch: None,
    };
    let (skip, current) = frpc_platform_skip(&binary);
    assert!(skip.is_empty());
    assert_eq!(current.as_deref(), Some("frpc.exe"));
}
