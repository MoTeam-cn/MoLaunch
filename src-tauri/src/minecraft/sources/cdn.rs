//! CDN 镜像处理 + Modrinth CDN 直连

use super::constants::{CF_MOCDN_MIRROR, MR_CDN_OFFICIAL, MR_CDN_RAW, MR_MOCDN_MIRROR};

/// authlib-injector 最新版元数据官方 URL
pub fn authlib_injector_meta_url_official() -> String {
    format!(
        "{}{}",
        super::constants::AUTHLIB_INJECTOR_OFFICIAL,
        super::constants::AUTHLIB_INJECTOR_LATEST_PATH
    )
}

/// authlib-injector 最新版元数据 BMCLAPI 镜像 URL
pub fn authlib_injector_meta_url_mirror() -> String {
    format!(
        "{}{}",
        super::constants::AUTHLIB_INJECTOR_BMCLAPI,
        super::constants::AUTHLIB_INJECTOR_LATEST_PATH
    )
}

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
        mirrors.push(url.replacen("https://edge.forgecdn.net", CF_MOCDN_MIRROR, 1));
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
        apply_cdn_mirrors(&url).into_iter().next().unwrap_or(url)
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
