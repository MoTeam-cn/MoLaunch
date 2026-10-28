//! 统一镜像源管理模块
//!
//! 管理官方/镜像源 URL 常量与请求逻辑：403/429 直接跳过，其他错误可重试。

mod cdn;
mod constants;
mod http;
mod library_chain;
mod mode;
mod paths;
mod retired;

// 公开 API re-export（外部调用方通过 sources::xxx 访问）
pub use cdn::{
    authlib_injector_meta_url_mirror, authlib_injector_meta_url_official, cdn_urls, replace_cdn,
};
pub use constants::{
    ALIYUN_MAVEN_PUBLIC, AUTHLIB_INJECTOR_BMCLAPI, AUTHLIB_INJECTOR_LATEST_PATH,
    AUTHLIB_INJECTOR_OFFICIAL, BMCLAPI_BASE, BMCLAPI_FABRIC_META, BMCLAPI_LITELOADER,
    BMCLAPI_NEOFORGE, BMCLAPI_NEOFORGE_LEGACY, BMCLAPI_OPTIFINE, BMCLAPI_VERSION_MANIFEST,
    CF_CDN_DOMAINS, FABRIC_META, FORGE_VERSIONS_URL, LIBRARY_REPLACEMENTS, LITELOADER_VERSIONS,
    MAVEN_CENTRAL, MAVEN_FABRIC, MAVEN_FORGE, MAVEN_NEOFORGE, MAVEN_REPLACEMENTS, MAVEN_SPONGE,
    MOJANG_LAUNCHER, MOJANG_LAUNCHERMETA, MOJANG_LIBRARIES, MOJANG_PISTON_DATA, MOJANG_PISTON_META,
    MOJANG_REPLACEMENTS, MOJANG_RESOURCES, MR_CDN_DOMAINS, NEOFORGE_API, NEOFORGE_API_LEGACY,
};
pub use http::{fetch_with_fallback, is_mirror_url, should_skip_status};
pub use library_chain::{
    fallback_urls as library_fallbacks, legacy_repositories, push_unique, relative_artifact_path,
};
pub use mode::{apply_replacements, build_replace_urls, build_urls, DownloadSourceMode};
pub use paths::{
    bmclapi_forge_path, fabric_profile_url, forge_installer_url, forge_versions_url,
    liteloader_json_url, neoforge_installer_url,
};
pub use retired::normalize as normalize_retired;

#[cfg(test)]
#[path = "../sources_tests.rs"]
mod tests;
