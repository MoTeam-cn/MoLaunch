use super::*;

#[test]
fn extract_mc_version_from_name_finds_first_mc_like() {
    assert_eq!(extract_mc_version_from_name("RLCraft 1.12.2 - Beta v2.8.1.zip"), "1.12.2");
    assert_eq!(extract_mc_version_from_name("Pack 26.1.3 - v2.0"), "26.1.3");
    assert_eq!(extract_mc_version_from_name("NoMcVersion - v2.0"), "");
}

#[test]
fn extract_version_from_name_keeps_old_behavior() {
    assert_eq!(extract_version_from_name("create-1.20.1-6.0.4.jar"), "6.0.4");
    assert_eq!(extract_version_from_name("alltheleaks-1.1.1+1.20.1-forge.jar"), "1.1.1");
    assert_eq!(extract_version_from_name("RLCraft 1.12.2 - Beta v2.8.1.zip"), "2.8.1");
}
