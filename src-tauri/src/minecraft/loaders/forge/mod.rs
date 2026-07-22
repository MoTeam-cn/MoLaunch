//! Forge loader module
//!
//! 模块结构：
//! - mod.rs: list_versions（Forge 版本列表查询，BMCLAPI JSON / 官方 HTML 双格式）+ 模块入口
//! - install.rs: install 调度器 + install_modern（1.13+ 安装，injector 方式）
//! - legacy.rs: install_legacy（1.7.10 ~ 1.12.2 旧版安装，解析 install_profile.json）

mod install;
mod legacy;

pub use install::install;

use super::LoaderVersion;
use crate::minecraft::sources::{self, DownloadSourceMode};

/// List Forge versions
pub async fn list_versions(
    mc_version: &str,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    let urls = sources::build_urls(
        mirror_url,
        &sources::forge_versions_url(mc_version),
        &format!("/forge/minecraft/{}", mc_version),
        source_mode,
    );

    let content = sources::fetch_with_fallback(&urls).await?;

    // 尝试 BMCLAPI JSON 格式
    if let Ok(json_array) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
        let mut versions: Vec<LoaderVersion> = json_array
            .iter()
            .filter_map(|v| {
                let version = v["version"].as_str()?;
                let modified = v["modified"].as_str();
                let release_time = modified.and_then(|s| crate::utils::datetime::format_utc_to_local(s));
                Some(LoaderVersion {
                    version: version.to_string(),
                    is_recommended: v["category"].as_str() == Some("recommended"),
                    release_time,
                })
            })
            .collect();

        versions.sort_by(|a, b| {
            let v_a = crate::utils::version::parse_number(&a.version);
            let v_b = crate::utils::version::parse_number(&b.version);
            v_b.cmp(&v_a)
        });

        return Ok(versions);
    }

    // 官方源 HTML 格式解析
    super::forge_html::parse_forge_version_html(&content)
}
