//! 统一镜像源管理模块
//!
//! 所有涉及官方/镜像源请求的逻辑统一在此管理。
//! - 403 (Forbidden): 直接跳过，不重试
//! - 429 (Too Many Requests): 直接跳过，不重试
//! - 其他错误: 可重试
//!
//! ## URL 管理规范
//! 所有远程 URL 必须在此文件定义常量，禁止在其他文件硬编码。

use crate::{log_debug, log_info, log_warn};
use std::time::Instant;

// ═══════════════════════════════════════════════════════════
// 基础 URL 常量
// ═══════════════════════════════════════════════════════════

/// BMCLAPI 基础 URL
pub const BMCLAPI_BASE: &str = "https://bmclapi2.bangbang93.com";

// ── Mojang 官方 ──
pub const MOJANG_PISTON_DATA: &str = "https://piston-data.mojang.com";
pub const MOJANG_PISTON_META: &str = "https://piston-meta.mojang.com";
pub const MOJANG_LAUNCHER: &str = "https://launcher.mojang.com";
pub const MOJANG_LAUNCHERMETA: &str = "https://launchermeta.mojang.com";
pub const MOJANG_LIBRARIES: &str = "https://libraries.minecraft.net";
pub const MOJANG_RESOURCES: &str = "https://resources.download.minecraft.net";

// ── Maven 仓库 ──
pub const MAVEN_FORGE: &str = "https://maven.minecraftforge.net";
pub const MAVEN_NEOFORGE: &str = "https://maven.neoforged.net/releases";
pub const MAVEN_FABRIC: &str = "https://maven.fabricmc.net";

// ── Mod Loader 官方 ──
pub const FORGE_VERSIONS_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge";
pub const NEOFORGE_API: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
pub const NEOFORGE_API_LEGACY: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/forge";
pub const FABRIC_META: &str = "https://meta.fabricmc.net/v2/versions/loader";
pub const LITELOADER_VERSIONS: &str = "https://dl.liteloader.com/versions/versions.json";

// ── BMCLAPI 路径 ──
pub const BMCLAPI_VERSION_MANIFEST: &str = "/mc/game/version_manifest.json";
pub const BMCLAPI_OPTIFINE: &str = "/optifine/versionList";
pub const BMCLAPI_FABRIC_META: &str = "/fabric-meta/v2/versions/loader";
pub const BMCLAPI_NEOFORGE: &str = "/neoforge/meta/api/maven/details/releases/net/neoforged/neoforge";
pub const BMCLAPI_NEOFORGE_LEGACY: &str = "/neoforge/meta/api/maven/details/releases/net/neoforged/forge";
pub const BMCLAPI_LITELOADER: &str = "/maven/com/mumfrey/liteloader/versions.json";

// ═══════════════════════════════════════════════════════════
// 域名替换规则
// ═══════════════════════════════════════════════════════════

/// Mojang 域名 -> BMCLAPI（直传）
pub const MOJANG_REPLACEMENTS: &[(&str, &str)] = &[
    (MOJANG_PISTON_DATA, BMCLAPI_BASE),
    (MOJANG_PISTON_META, BMCLAPI_BASE),
    (MOJANG_LAUNCHER, BMCLAPI_BASE),
    (MOJANG_LAUNCHERMETA, BMCLAPI_BASE),
];

/// Maven 仓库 -> BMCLAPI/maven
pub const MAVEN_REPLACEMENTS: &[(&str, &str)] = &[
    (MAVEN_FORGE, "https://bmclapi2.bangbang93.com/maven"),
    (MAVEN_NEOFORGE, "https://bmclapi2.bangbang93.com/maven"),
    (MAVEN_FABRIC, "https://bmclapi2.bangbang93.com/maven"),
];

/// Minecraft 库 -> BMCLAPI/libraries
pub const LIBRARY_REPLACEMENTS: &[(&str, &str)] = &[
    (MOJANG_LIBRARIES, "https://bmclapi2.bangbang93.com/libraries"),
];

// ═══════════════════════════════════════════════════════════
// 动态路径构建
// ═══════════════════════════════════════════════════════════

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
    format!("{}/{}/{}/profile/json", FABRIC_META, major_version, loader_version)
}

/// LiteLoader 版本 JSON 官方 URL
pub fn liteloader_json_url(mc_version: &str, loader_version: &str) -> String {
    format!(
        "{}/versions/com/mumfrey/liteloader/{}/liteloader-{}-{}.json",
        LITELOADER_VERSIONS, mc_version, mc_version, loader_version
    )
}

// ═══════════════════════════════════════════════════════════
// HTTP 状态码处理
// ═══════════════════════════════════════════════════════════

/// 判断 HTTP 状态码是否应该直接跳过（不重试）
/// - 403 Forbidden: 服务器拒绝，重试无意义
/// - 429 Too Many Requests: 频率限制，重试会继续被拒
pub fn should_skip_status(status: reqwest::StatusCode) -> bool {
    status.as_u16() == 403 || status.as_u16() == 429
}

// ═══════════════════════════════════════════════════════════
// 下载源模式
// ═══════════════════════════════════════════════════════════

/// 下载源模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DownloadSourceMode {
    Official,
    Mirror,
    Smart,
}

impl DownloadSourceMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "official" => Self::Official,
            "mirror" => Self::Mirror,
            "smart" => Self::Smart,
            _ => Self::Smart,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// URL 构建函数
// ═══════════════════════════════════════════════════════════

/// 构建候选 URL 列表（根据下载源模式排序）
///
/// - mirror: 只用镜像源，失败直接报错
/// - official: 只用官方源，失败直接报错
/// - smart: 官方优先，失败回退 BMCLAPI
pub fn build_urls(
    mirror_url: Option<&str>,
    official_url: &str,
    bmclapi_path: &str,
    mode: DownloadSourceMode,
) -> Vec<String> {
    let bmclapi_url = format!("{}{}", BMCLAPI_BASE, bmclapi_path);

    match mode {
        DownloadSourceMode::Mirror => {
            // 只用镜像源：自定义镜像 -> BMCLAPI（无官方回退）
            let mut urls = Vec::new();
            if let Some(mirror) = mirror_url.filter(|m| !m.is_empty()) {
                urls.push(format!("{}{}", mirror.trim_end_matches('/'), bmclapi_path));
            }
            urls.push(bmclapi_url);
            urls
        }
        DownloadSourceMode::Official => {
            // 只用官方源，不回退
            vec![official_url.to_string()]
        }
        DownloadSourceMode::Smart => {
            // 官方优先，失败回退 BMCLAPI
            vec![official_url.to_string(), bmclapi_url]
        }
    }
}

/// 对 URL 应用替换规则
pub fn apply_replacements(url: &str, replacements: &[(&str, &str)]) -> String {
    let mut result = url.to_string();
    for (from, to) in replacements {
        result = result.replace(from, to);
    }
    result
}

/// 对已有 URL 做域名替换，生成候选列表
///
/// 根据模式决定是否包含替换版本：
/// - mirror: 只返回替换后的镜像 URL
/// - official: 只返回原始官方 URL
/// - smart: 官方原始 + BMCLAPI 替换（官方优先）
pub fn build_replace_urls(
    official_url: &str,
    mirror_url: Option<&str>,
    replacements: &[(&str, &str)],
    mode: DownloadSourceMode,
) -> Vec<String> {
    match mode {
        DownloadSourceMode::Mirror => {
            // 只用镜像源
            let mut urls = Vec::new();
            if let Some(mirror) = mirror_url.filter(|m| !m.is_empty()) {
                if let Ok(parsed) = reqwest::Url::parse(official_url) {
                    if let Some(path) = parsed.path().strip_prefix('/') {
                        urls.push(format!("{}/{}", mirror.trim_end_matches('/'), path));
                    }
                }
            }
            let bmclapi_url = apply_replacements(official_url, replacements);
            if !urls.contains(&bmclapi_url) {
                urls.push(bmclapi_url);
            }
            urls
        }
        DownloadSourceMode::Official => {
            // 只用官方源
            vec![official_url.to_string()]
        }
        DownloadSourceMode::Smart => {
            // 官方优先，失败回退 BMCLAPI
            let bmclapi_url = apply_replacements(official_url, replacements);
            vec![official_url.to_string(), bmclapi_url]
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 统一请求函数
// ═══════════════════════════════════════════════════════════

/// 统一的带回退的 HTTP GET 请求
///
/// 依次尝试 URLs，遇到 403/429 直接跳过，其他错误可重试。
pub async fn fetch_with_fallback(urls: &[String]) -> anyhow::Result<String> {
    let client = crate::http::get_client();
    let mut last_err = String::new();

    for url in urls {
        let start = Instant::now();
        match client.get(url).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    match resp.text().await {
                        Ok(text) => {
                            log_info!("[Sources] 请求成功: {} ({})", url, fmt_elapsed(start));
                            return Ok(text);
                        }
                        Err(e) => {
                            log_debug!("[Sources] 读取响应失败 {}: {} ({})", url, e, fmt_elapsed(start));
                            last_err = format!("{}: 读取失败 - {}", url, e);
                        }
                    }
                } else if should_skip_status(status) {
                    log_warn!("[Sources] {} 返回 {}，跳过不重试 ({})", url, status, fmt_elapsed(start));
                    last_err = format!("{}: HTTP {}", url, status);
                    continue;
                } else {
                    log_debug!("[Sources] {} 返回 {} ({})", url, status, fmt_elapsed(start));
                    last_err = format!("{}: HTTP {}", url, status);
                }
            }
            Err(e) => {
                log_debug!("[Sources] 请求失败 {}: {} ({})", url, e, fmt_elapsed(start));
                last_err = format!("{}: {}", url, e);
            }
        }
    }

    Err(anyhow::anyhow!("所有源均失败: {}", last_err))
}

/// 格式化耗时：< 1000ms 显示 ms，>= 1000ms 显示 s
fn fmt_elapsed(start: Instant) -> String {
    let ms = start.elapsed().as_millis();
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}
