use super::*;

#[test]
fn test_frp_manifest() {
    let r = classify_manifest(
        r#"{"id":"lolia-frp","name":"LoliaFrp","binary":{"distribution":"bundled"},"api":{"endpointsFile":"api/endpoints.json"}}"#,
    );
    assert_eq!(r.package_type, PackageType::FrpProvider);
    assert_eq!(r.provider_id.as_deref(), Some("lolia-frp"));
    assert_eq!(r.provider_name.as_deref(), Some("LoliaFrp"));
}

#[test]
fn test_curseforge_manifest() {
    let r = classify_manifest(
        r#"{"formatVersion":1,"game":"minecraft","versionId":"1.20.1","name":"test","files":[],"minecraft":{"version":"1.20.1","modLoaders":[]}}"#,
    );
    assert_eq!(r.package_type, PackageType::Modpack);
}

#[test]
fn test_mcbbs_manifest() {
    let r = classify_manifest(r#"{"id":1,"name":"x","addons":[],"files":[]}"#);
    assert_eq!(r.package_type, PackageType::Modpack);
}

#[test]
fn test_unknown_manifest() {
    let r = classify_manifest(r#"{"foo":"bar"}"#);
    assert_eq!(r.package_type, PackageType::Unknown);
}

#[test]
fn test_invalid_json() {
    let r = classify_manifest("not json");
    assert_eq!(r.package_type, PackageType::Unknown);
}

#[test]
fn test_frp_manifest_requires_binary_or_api() {
    // 只有 id 没有 binary/api：不应判为 frp 厂商包
    let r = classify_manifest(r#"{"id":"only-id"}"#);
    assert_eq!(r.package_type, PackageType::Unknown);
}
