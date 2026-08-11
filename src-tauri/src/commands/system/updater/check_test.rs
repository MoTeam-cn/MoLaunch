//! check 单元测试

use super::is_version_newer;

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
