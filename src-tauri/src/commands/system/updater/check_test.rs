//! check 单元测试

use super::{is_version_newer, pick_github_asset};
use serde_json::json;

#[test]
fn rc_版本递增可识别() {
    assert!(is_version_newer("0.3.5-rc7", "0.3.5-rc6"));
    assert!(!is_version_newer("0.3.5-rc6", "0.3.5-rc7"));
    assert!(!is_version_newer("0.3.5-rc7", "0.3.5-rc7"));
}

#[test]
fn rc_两位数后缀按数值比较() {
    assert!(is_version_newer("0.3.5-rc10", "0.3.5-rc9"));
    assert!(is_version_newer("0.3.5-rc12", "0.3.5-rc11"));
}

#[test]
fn 正式版高于预发布版() {
    assert!(is_version_newer("0.3.5", "0.3.5-rc7"));
    assert!(is_version_newer("0.3.5", "0.3.5-beta.2"));
    assert!(!is_version_newer("0.3.5-rc7", "0.3.5"));
}

#[test]
fn 主版本号比较() {
    assert!(is_version_newer("0.4.0", "0.3.5-rc99"));
    assert!(is_version_newer("1.0.0", "0.9.9-rc1"));
    assert!(!is_version_newer("0.3.5", "0.4.0"));
}

#[test]
fn beta_alpha_先后顺序() {
    assert!(is_version_newer("0.3.5-beta.1", "0.3.5-alpha.2"));
    assert!(!is_version_newer("0.3.5-alpha.3", "0.3.5-beta.1"));
}

#[test]
fn v_前缀与多段pre() {
    assert!(is_version_newer("v0.3.5-rc7", "v0.3.5-rc6"));
    assert!(is_version_newer("0.3.5-rc7.1", "0.3.5-rc7"));
}

#[test]
fn 带点与不带点写法等价() {
    assert!(!is_version_newer("0.3.8-rc.4", "0.3.8-rc4"));
    assert!(!is_version_newer("0.3.8-rc4", "0.3.8-rc.4"));
    assert!(!is_version_newer("0.3.8-beta.2", "0.3.8-beta2"));
}

#[test]
fn 带点写法跨序号比较正确() {
    assert!(is_version_newer("0.3.8-rc.4", "0.3.8-rc2"));
    assert!(!is_version_newer("0.3.8-rc2", "0.3.8-rc.4"));
    assert!(is_version_newer("0.3.8-rc.4", "0.3.8-rc1"));
    assert!(is_version_newer("0.3.8-rc.4", "0.3.8-rc.3"));
    assert!(is_version_newer("0.3.8-rc.10", "0.3.8-rc.9"));
}

#[test]
fn beta_alpha_带点写法也按数值与先后判断() {
    assert!(is_version_newer("0.3.8-beta.2", "0.3.8-beta1"));
    assert!(is_version_newer("0.3.8-beta.2", "0.3.8-beta.1"));
    assert!(is_version_newer("0.3.8-beta.10", "0.3.8-beta.9"));
    assert!(is_version_newer("0.3.8-alpha.3", "0.3.8-alpha1"));
    assert!(is_version_newer("0.3.8-beta.1", "0.3.8-alpha.9"));
    assert!(!is_version_newer("0.3.8-alpha.3", "0.3.8-beta.1"));
    assert!(is_version_newer("0.3.8-rc.1", "0.3.8-beta.9"));
    assert!(is_version_newer("0.3.8", "0.3.8-rc.4"));
}

// ===== GitHub 备用更新源：资源挑选 =====

#[test]
fn github_资源优先匹配便携版而非安装版() {
    let release = json!({
        "assets": [
            { "name": "MoLaunch_1.0.0_x64-setup.exe", "browser_download_url": "https://github.com/a/setup.exe" },
            { "name": "MoLaunch_1.0.0_x64.exe", "browser_download_url": "https://github.com/a/portable.exe" }
        ]
    });
    let picked = pick_github_asset(&release);
    if cfg!(target_os = "windows") {
        // Windows 必须挑便携版（自更新需替换主程序本体）
        assert_eq!(picked.as_deref(), Some("https://github.com/a/portable.exe"));
    } else {
        // 非 Windows 平台不匹配 .exe 资源
        assert_eq!(picked, None);
    }
}

#[test]
fn github_无匹配资源返回_none() {
    let release = json!({
        "assets": [
            { "name": "notes.txt", "browser_download_url": "https://github.com/a/notes.txt" }
        ]
    });
    assert_eq!(pick_github_asset(&release), None);
}

#[test]
fn github_缺少_assets_字段返回_none() {
    let release = json!({ "tag_name": "v1.0.0" });
    assert_eq!(pick_github_asset(&release), None);
}
