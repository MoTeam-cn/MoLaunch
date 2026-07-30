//! 统一镜像源管理模块
//!
//! 所有涉及官方/镜像源请求的逻辑统一在此管理。
//! - 403 (Forbidden): 直接跳过，不重试
//! - 429 (Too Many Requests): 直接跳过，不重试
//! - 其他错误: 可重试
//!
//! ## URL 管理规范
//! 所有远程 URL 必须在此文件定义常量，禁止在其他文件硬编码。

use crate::minecraft::community::common::fmt_elapsed;
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
pub const NEOFORGE_API: &str =
    "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
pub const NEOFORGE_API_LEGACY: &str =
    "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/forge";
pub const FABRIC_META: &str = "https://meta.fabricmc.net/v2/versions/loader";
pub const LITELOADER_VERSIONS: &str = "https://dl.liteloader.com/versions/versions.json";

// ── BMCLAPI 路径 ──
pub const BMCLAPI_VERSION_MANIFEST: &str = "/mc/game/version_manifest.json";
pub const BMCLAPI_OPTIFINE: &str = "/optifine/versionList";
pub const BMCLAPI_FABRIC_META: &str = "/fabric-meta/v2/versions/loader";
pub const BMCLAPI_NEOFORGE: &str =
    "/neoforge/meta/api/maven/details/releases/net/neoforged/neoforge";
pub const BMCLAPI_NEOFORGE_LEGACY: &str =
    "/neoforge/meta/api/maven/details/releases/net/neoforged/forge";
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
pub const LIBRARY_REPLACEMENTS: &[(&str, &str)] = &[(
    MOJANG_LIBRARIES,
    "https://bmclapi2.bangbang93.com/libraries",
)];

// ── 社区资源 CDN 镜像（CurseForge + Modrinth 统一由 source 策略控制）──

/// Modrinth CDN 原始域名（官方 CDN，对中国大陆用户会跳转到慢速 cdn-alt）
const MR_CDN_OFFICIAL: &str = "https://cdn.modrinth.com";

/// Modrinth CDN 直连域名（绕过 cdn-alt 跳转，速度更快）
/// 路径结构与官方 CDN 完全一致，仅替换域名即可
/// 示例：https://cdn.modrinth.com/data/xxx/versions/yyy/zzz.mrpack
///    → https://cdn-raw.modrinth.com/data/xxx/versions/yyy/zzz.mrpack
const MR_CDN_RAW: &str = "https://cdn-raw.modrinth.com";

/// Modrinth 自建 CDN 镜像域名（mocdn.net）
/// 路径结构与官方 CDN 完全一致，仅替换域名即可
const MR_MOCDN_MIRROR: &str = "https://cdn-modrinth.mocdn.net";

/// CurseForge 自建 CDN 镜像域名（仅支持 edge.forgecdn.net 路径）
/// media.forgecdn.net 路径不支持，走官方源
const CF_MOCDN_MIRROR: &str = "https://cdn-curseforge.mocdn.net";

// ── authlib-injector（外置登录支持库）下载源 ──
/// authlib-injector 官方源（yushi.moe）
pub const AUTHLIB_INJECTOR_OFFICIAL: &str = "https://authlib-injector.yushi.moe";
/// authlib-injector 最新版元数据路径（拼接在官方源后）
pub const AUTHLIB_INJECTOR_LATEST_PATH: &str = "/artifact/latest.json";
/// authlib-injector BMCLAPI 镜像源
pub const AUTHLIB_INJECTOR_BMCLAPI: &str = "https://bmclapi2.bangbang93.com/mirrors/authlib-injector";

/// authlib-injector 最新版元数据官方 URL
pub fn authlib_injector_meta_url_official() -> String {
    format!("{}{}", AUTHLIB_INJECTOR_OFFICIAL, AUTHLIB_INJECTOR_LATEST_PATH)
}

/// authlib-injector 最新版元数据 BMCLAPI 镜像 URL
pub fn authlib_injector_meta_url_mirror() -> String {
    format!("{}{}", AUTHLIB_INJECTOR_BMCLAPI, AUTHLIB_INJECTOR_LATEST_PATH)
}

/// CurseForge CDN 官方域名列表
pub const CF_CDN_DOMAINS: &[&str] = &["https://edge.forgecdn.net", "https://media.forgecdn.net"];

/// Modrinth CDN 官方域名列表（含原始域名和直连域名，均需镜像替换）
pub const MR_CDN_DOMAINS: &[&str] = &[MR_CDN_OFFICIAL, MR_CDN_RAW];

/// 将 Modrinth CDN URL 的域名从 `cdn.modrinth.com` 替换为 `cdn-raw.modrinth.com`
///
/// 原始 `cdn.modrinth.com` 对中国大陆用户会 302 跳转到慢速 `cdn-alt.modrinth.com`，
/// 直接使用 `cdn-raw.modrinth.com` 绕过跳转，提升下载速度。
/// 非 Modrinth CDN URL 原样返回。
pub fn rewrite_mr_cdn(url: &str) -> String {
    url.replacen(MR_CDN_OFFICIAL, MR_CDN_RAW, 1)
}

/// 读取 Modrinth CDN 直连开关（从 INI 配置）
///
/// 与 `community::get_source_pref` 同模式，直接读 Storage 避免 mutex。
/// 默认 false（关闭），仅在开发者模式解锁后可在「设置 → 开发者模式」中开启。
fn get_modrinth_cdn_raw_enabled() -> bool {
    crate::storage::Storage::instance()
        .get_config("Download", "modrinth_cdn_raw_enabled")
        .as_deref()
        == Some("true")
}

/// 将 CDN URL 替换为可用镜像 URL（内部函数，不判断 source 策略）
///
/// 返回自建 CDN（mocdn.net）镜像 URL。非 CDN URL 或无可用镜像时返回空 Vec。
///
/// 域名替换规则：
/// - `cdn-raw.modrinth.com`（经 rewrite_mr_cdn 替换后）→ `cdn-modrinth.mocdn.net`
/// - `cdn.modrinth.com`（未经 rewrite，防御性兼容）→ `cdn-modrinth.mocdn.net`
/// - `edge.forgecdn.net` → `cdn-curseforge.mocdn.net`
/// - `media.forgecdn.net` → 无镜像（mocdn 不支持此域名，走官方）
fn apply_cdn_mirrors(url: &str) -> Vec<String> {
    let mut mirrors = Vec::new();

    // Modrinth CDN: cdn-raw.modrinth.com（rewrite 后）或 cdn.modrinth.com（防御性兼容）
    // → mocdn 镜像
    if url.starts_with(MR_CDN_RAW) {
        mirrors.push(url.replacen(MR_CDN_RAW, MR_MOCDN_MIRROR, 1));
    } else if url.starts_with(MR_CDN_OFFICIAL) {
        mirrors.push(url.replacen(MR_CDN_OFFICIAL, MR_MOCDN_MIRROR, 1));
    }
    // CurseForge edge.forgecdn.net → mocdn 镜像
    else if url.starts_with("https://edge.forgecdn.net") {
        mirrors.push(url.replacen(
            "https://edge.forgecdn.net",
            CF_MOCDN_MIRROR,
            1,
        ));
    }
    // media.forgecdn.net: mocdn 不支持此域名路径，无镜像，走官方

    mirrors
}

/// 根据 source 策略替换 CDN URL（单 URL，用于直连下载场景）
///
/// 当 `modrinth_cdn_raw_enabled` 开启时，入口处先将 `cdn.modrinth.com` 替换为
/// `cdn-raw.modrinth.com`（绕过中国大陆 cdn-alt 跳转），再按 source 策略决定是否使用镜像。
///
/// - source=0（尽量镜像）：返回优先镜像 URL（mocdn.net）
/// - source=1（缓慢时换镜像）：返回官方 URL（fallback 由 `cdn_urls` 处理）
/// - source=2（尽量官方）：返回官方 URL
pub fn replace_cdn(url: &str) -> String {
    let url = if get_modrinth_cdn_raw_enabled() {
        rewrite_mr_cdn(url)
    } else {
        url.to_string()
    };
    let source = crate::minecraft::community::get_source_pref();
    if source == 0 {
        apply_cdn_mirrors(&url)
            .into_iter()
            .next()
            .unwrap_or(url)
    } else {
        url
    }
}

/// 根据 source 策略构造 CDN URL 列表（多 URL，用于 DownloadManager fallback）
///
/// 当 `modrinth_cdn_raw_enabled` 开启时，入口处先将 `cdn.modrinth.com` 替换为
/// `cdn-raw.modrinth.com`（绕过中国大陆 cdn-alt 跳转），再按 source 策略构造候选 URL 列表。
///
/// - source=0（尽量镜像）：`[mocdn镜像URL]`（无镜像的 CDN 返回官方 URL）
/// - source=1（缓慢时换镜像）：`[官方URL, mocdn镜像URL]`（官方优先，失败自动 fallback）
/// - source=2（尽量官方）：`[官方URL]`
pub fn cdn_urls(url: &str) -> Vec<String> {
    let url = if get_modrinth_cdn_raw_enabled() {
        rewrite_mr_cdn(url)
    } else {
        url.to_string()
    };
    let source = crate::minecraft::community::get_source_pref();
    let mirrors = apply_cdn_mirrors(&url);
    let is_cdn = !mirrors.is_empty();

    match source {
        0 if is_cdn => mirrors,
        1 if is_cdn => {
            let mut urls = vec![url.clone()];
            urls.extend(mirrors);
            urls
        }
        _ => vec![url],
    }
}

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

// ═══════════════════════════════════════════════════════════
// HTTP 状态码处理
// ═══════════════════════════════════════════════════════════

/// 判断 HTTP 状态码是否应该直接跳过（不重试）
/// - 403 Forbidden: 服务器拒绝，重试无意义
/// - 429 Too Many Requests: 频率限制，重试会继续被拒
pub fn should_skip_status(status: reqwest::StatusCode) -> bool {
    status.as_u16() == 403 || status.as_u16() == 429
}

/// 判断 URL 是否为镜像源 URL
///
/// 用于 `DownloadManager::reorder_urls` 按源模式重排候选 URL。
/// 识别 BMCLAPI、mocdn.net、mcimirror.top 三类镜像域名。
pub fn is_mirror_url(url: &str) -> bool {
    url.contains("bmclapi") || url.contains("mocdn") || url.contains("mcimirror")
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
                            log_debug!(
                                "[Sources] 读取响应失败 {}: {} ({})",
                                url,
                                e,
                                fmt_elapsed(start)
                            );
                            last_err = format!("{}: 读取失败 - {}", url, e);
                        }
                    }
                } else if should_skip_status(status) {
                    log_warn!(
                        "[Sources] {} 返回 {}，跳过不重试 ({})",
                        url,
                        status,
                        fmt_elapsed(start)
                    );
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
