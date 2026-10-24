//! github 模块单元测试（纯函数：清单地址、平台键、通道与平台挑选）

use super::{manifest_url, pick_best_channel_entry, pick_platform, platform_key};
use serde_json::json;

#[test]
fn test_manifest_url_stable_uses_latest_path() {
    let url = manifest_url("stable");
    assert!(url.ends_with("/releases/latest/download/latest.json"));
}

#[test]
fn test_manifest_url_beta_and_alpha_use_rolling_tag() {
    let beta = manifest_url("beta");
    let alpha = manifest_url("alpha");
    assert!(beta.contains("/releases/download/channel-beta/latest.json"));
    assert!(alpha.contains("/releases/download/channel-alpha/latest.json"));
}

#[test]
fn test_platform_key_maps_os_and_arch() {
    assert_eq!(platform_key("windows", "x86_64"), "windows-x86_64");
    assert_eq!(platform_key("windows", "aarch64"), "windows-aarch64");
    assert_eq!(platform_key("macos", "aarch64"), "darwin-aarch64");
    assert_eq!(platform_key("macos", "arm64"), "darwin-aarch64");
    assert_eq!(platform_key("linux", "x86_64"), "linux-x86_64");
    assert_eq!(platform_key("linux", "unknown"), "linux-x86_64");
}

#[test]
fn test_pick_best_channel_entry_prefers_newer_channel() {
    let manifest = json!({
        "channels": {
            "stable": { "version": "0.4.0" },
            "beta": { "version": "0.4.1-beta.1" }
        }
    });
    let (entry, served) = pick_best_channel_entry(&manifest, "beta").expect("beta 通道应命中");
    assert_eq!(
        entry.get("version").and_then(|v| v.as_str()),
        Some("0.4.1-beta.1")
    );
    assert_eq!(served, "beta");
}

#[test]
fn test_pick_best_channel_entry_picks_stable_when_newer() {
    // 测试版用户遇到更高的正式版：必须拿到正式版
    let manifest = json!({
        "channels": {
            "stable": { "version": "0.4.0" },
            "beta": { "version": "0.3.9-rc.4" }
        }
    });
    let (entry, served) = pick_best_channel_entry(&manifest, "beta").expect("应命中 stable");
    assert_eq!(entry.get("version").and_then(|v| v.as_str()), Some("0.4.0"));
    assert_eq!(served, "stable");
}

#[test]
fn test_pick_best_channel_entry_falls_back_to_stable() {
    let manifest = json!({
        "channels": { "stable": { "version": "0.4.0" } }
    });
    let (entry, served) = pick_best_channel_entry(&manifest, "alpha").expect("缺通道应回退 stable");
    assert_eq!(entry.get("version").and_then(|v| v.as_str()), Some("0.4.0"));
    assert_eq!(served, "stable");
}

#[test]
fn test_pick_best_channel_entry_without_channels_returns_none() {
    let manifest = json!({ "version": "0.4.0" });
    assert!(pick_best_channel_entry(&manifest, "stable").is_none());
}

#[test]
fn test_pick_platform_returns_url_and_signature() {
    let entry = json!({
        "platforms": {
            "windows-x86_64": {
                "url": "https://github.com/MoTeam-cn/MoLaunch/releases/download/v0.4.0/MoLaunch.exe",
                "signature": "sig-abc"
            }
        }
    });
    let (url, sig) = pick_platform(&entry, "windows-x86_64").expect("平台应命中");
    assert!(url.ends_with("MoLaunch.exe"));
    assert_eq!(sig, "sig-abc");
}

#[test]
fn test_pick_platform_missing_or_empty_returns_none() {
    let entry = json!({ "platforms": { "linux-x86_64": { "url": "" } } });
    assert!(pick_platform(&entry, "linux-x86_64").is_none());
    assert!(pick_platform(&entry, "windows-x86_64").is_none());
    let no_platforms = json!({ "version": "0.4.0" });
    assert!(pick_platform(&no_platforms, "windows-x86_64").is_none());
}
