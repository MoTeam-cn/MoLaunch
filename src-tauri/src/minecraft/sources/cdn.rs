//! CDN 镜像处理

use super::constants::{CF_MOCDN_MIRROR, MR_CDN_OFFICIAL, MR_MOCDN_MIRROR};

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

/// 将 CDN URL 替换为可用镜像 URL（内部函数，不判断 source 策略）
///
/// 返回自建 CDN（mocdn.net）镜像 URL。非 CDN URL 或无可用镜像时返回空 Vec。
///
/// 域名替换规则：
/// - `cdn.modrinth.com` → `cdn-modrinth.mocdn.net`
/// - `edge.forgecdn.net` → `cdn-curseforge.mocdn.net`
/// - `media.forgecdn.net` → 无镜像（mocdn 不支持此域名，走官方）
fn apply_cdn_mirrors(url: &str) -> Vec<String> {
    let mut mirrors = Vec::new();

    // Modrinth CDN → mocdn 镜像
    if url.starts_with(MR_CDN_OFFICIAL) {
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
/// - source=0（尽量镜像）：返回优先镜像 URL（mocdn.net）
/// - source=1（缓慢时换镜像）：返回官方 URL（fallback 由 `cdn_urls` 处理）
/// - source=2（尽量官方）：返回官方 URL
pub fn replace_cdn(url: &str) -> String {
    let source = crate::minecraft::community::get_source_pref();
    if source == 0 {
        apply_cdn_mirrors(url)
            .into_iter()
            .next()
            .unwrap_or_else(|| url.to_string())
    } else {
        url.to_string()
    }
}

/// 根据 source 策略构造 CDN URL 列表（多 URL，用于 DownloadManager fallback）
///
/// - source=0（尽量镜像）：`[mocdn镜像URL]`（无镜像的 CDN 返回官方 URL）
/// - source=1（缓慢时换镜像）：`[官方URL, mocdn镜像URL]`（官方优先，失败自动 fallback）
/// - source=2（尽量官方）：`[官方URL]`
pub fn cdn_urls(url: &str) -> Vec<String> {
    let source = crate::minecraft::community::get_source_pref();
    let mirrors = apply_cdn_mirrors(url);
    let is_cdn = !mirrors.is_empty();

    match source {
        0 if is_cdn => mirrors,
        1 if is_cdn => {
            let mut urls = vec![url.to_string()];
            urls.extend(mirrors);
            urls
        }
        _ => vec![url.to_string()],
    }
}
