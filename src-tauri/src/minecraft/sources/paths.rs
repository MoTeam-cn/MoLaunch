//! 动态路径构建函数

use super::constants::{
    FABRIC_META, FORGE_VERSIONS_URL, LITELOADER_VERSIONS, MAVEN_FORGE, MAVEN_NEOFORGE,
};

/// BMCLAPI Forge 版本列表路径
pub fn bmclapi_forge_path(mc_version: &str) -> String {
    format!("/forge/minecraft/{}", mc_version)
}

/// Forge 版本列表官方 URL
pub fn forge_versions_url(mc_version: &str) -> String {
    format!("{}/index_{}.html", FORGE_VERSIONS_URL, mc_version)
}

/// Forge 安装器下载 URL（官方 Maven）
pub fn forge_installer_url(mc_version: &str, forge_version: &str) -> String {
    format!(
        "{}/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
        MAVEN_FORGE, mc_version, forge_version, mc_version, forge_version
    )
}

/// NeoForge 安装器下载 URL（官方 Maven）
pub fn neoforge_installer_url(neoforge_version: &str) -> String {
    format!(
        "{}/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
        MAVEN_NEOFORGE, neoforge_version, neoforge_version
    )
}

/// Fabric profile JSON 官方 URL
pub fn fabric_profile_url(major_version: &str, loader_version: &str) -> String {
    format!(
        "{}/{}/{}/profile/json",
        FABRIC_META, major_version, loader_version
    )
}

/// LiteLoader 版本 JSON 官方 URL
pub fn liteloader_json_url(mc_version: &str, loader_version: &str) -> String {
    format!(
        "{}/versions/com/mumfrey/liteloader/{}/liteloader-{}-{}.json",
        LITELOADER_VERSIONS, mc_version, mc_version, loader_version
    )
}
