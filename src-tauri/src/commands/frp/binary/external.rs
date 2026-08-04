//! 外部厂商 frpc 下载（distribution=url）：HTTPS + 域名白名单 + SHA256 校验 + 可选解压。
//! 重定向手动校验域名（防重定向到非白名单域名）；解压走 `archive::extract_archive`（Zip Slip 防护）。

use super::super::{
    ensure_dir, frpc_platform_skip, providers_root, resolve_download_config, ProviderManifest,
};
use super::archive;
use crate::log_info;
use std::path::Path;

/// 外部厂商 frpc 下载（distribution=url）
///
/// 校验 URL HTTPS + 域名白名单 + SHA256（如有）。
/// 下载完成后若 archive=true，则解压到厂商目录。
/// 按当前平台选择下载 URL 与目标路径（urls/target_paths 平台映射，回退 url/target_path）。
pub(crate) async fn ensure_external_frpc(
    provider_id: &str,
    manifest: &ProviderManifest,
) -> Result<String, String> {
    let dl = manifest
        .binary
        .download
        .as_ref()
        .ok_or_else(|| format!("厂商 {} 缺少 binary.download 配置", provider_id))?;

    // 解析当前平台的下载 URL 与目标相对路径
    let (download_url, target_rel) = resolve_download_config(dl);
    validate_download_url(&download_url, &dl.allowed_domains)?;

    let provider_dir = providers_root().join(provider_id);
    let target_path = provider_dir.join(&target_rel);
    if let Some(parent) = target_path.parent() {
        ensure_dir(parent)?;
    }

    log_info!(
        "[Frp] 开始下载外部厂商 frpc: {} ({})",
        provider_id,
        download_url
    );

    // 构造禁止自动重定向的 client，手动校验重定向域名（防止重定向到非白名单域名）
    // 对应设计文档 §7.7 frpc 二进制下载安全。
    // 复用 crate::http 统一管线（代理 / IP 版本 / TLS 信任源 / User-Agent 与全局一致）。
    let client =
        crate::http::build_client_with_redirect(reqwest::redirect::Policy::none(), Some(60_000));

    let mut current_url = download_url.clone();
    let mut redirects = 0u32;
    const MAX_REDIRECTS: u32 = 5;
    let response = loop {
        let resp = client
            .get(&current_url)
            .send()
            .await
            .map_err(|e| format!("下载失败: {}", e))?;

        // 3xx 重定向：手动校验 Location 域名是否在白名单内
        if resp.status().is_redirection() {
            redirects += 1;
            if redirects > MAX_REDIRECTS {
                return Err("重定向次数超过限制".to_string());
            }
            let location = resp
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| "重定向响应缺少 Location 头".to_string())?;
            let base =
                reqwest::Url::parse(&current_url).map_err(|e| format!("解析 URL 失败: {}", e))?;
            let next_url = base
                .join(location)
                .map_err(|e| format!("解析重定向 URL 失败: {}", e))?;
            let host = next_url.host_str().unwrap_or("");
            if !dl.allowed_domains.iter().any(|d| host_matches(host, d)) {
                return Err(format!("重定向域名 {} 不在白名单中", host));
            }
            log_info!("[Frp] 重定向到白名单域名: {}", next_url);
            current_url = next_url.to_string();
            continue;
        }
        break resp;
    };

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取下载内容失败: {}", e))?;

    if let Some(ref expected_sha) = dl.sha256 {
        let actual = crate::utils::hash::sha256_hex(&bytes);
        if actual != *expected_sha {
            return Err(format!(
                "SHA256 校验失败：期望 {}，实际 {}",
                expected_sha, actual
            ));
        }
    }

    std::fs::write(&target_path, &bytes).map_err(|e| format!("写入文件失败: {}", e))?;

    let result_msg = if dl.archive {
        archive::extract_archive(&target_path, &provider_dir)?;
        // 解压成功后删除原始 archive 文件，避免 providers 目录残留冗余 zip
        let _ = std::fs::remove_file(&target_path);
        // 平台过滤：删除其他平台的 frpc，只保留当前平台对应的二进制
        let (skip, _) = frpc_platform_skip(&manifest.binary);
        if !skip.is_empty() {
            let removed = remove_skipped_frpc(&provider_dir, &skip);
            log_info!(
                "[Frp] 外部厂商 frpc 平台过滤：清理 {} 个其他平台二进制",
                removed
            );
        }
        log_info!(
            "[Frp] 外部厂商 frpc 解压完成，已清理 archive: {}",
            target_path.display()
        );
        format!("frpc 下载并解压完成: {}", provider_dir.display())
    } else {
        log_info!("[Frp] 外部厂商 frpc 下载完成: {}", target_path.display());
        format!("frpc 下载完成: {}", target_path.display())
    };

    Ok(result_msg)
}

/// 删除厂商目录下其他平台的 frpc 文件（相对路径命中跳过集），返回删除数量
fn remove_skipped_frpc(provider_dir: &Path, skip: &std::collections::HashSet<String>) -> u32 {
    let mut removed = 0u32;
    for rel in skip {
        let p = provider_dir.join(rel);
        if p.is_file() && std::fs::remove_file(&p).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// 校验下载 URL：必须 HTTPS + 域名在白名单中
fn validate_download_url(url: &str, allowed_domains: &[String]) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("下载 URL 必须使用 HTTPS".to_string());
    }
    let rest = &url[8..];
    let host_end = rest.find(['/', ':']).unwrap_or(rest.len());
    let host = &rest[..host_end];
    if !allowed_domains.iter().any(|d| host_matches(host, d)) {
        return Err(format!("下载域名 {} 不在白名单中", host));
    }
    Ok(())
}

/// 域名白名单匹配，支持 `*.example.com` 一级通配符
///
/// `pub(crate)`：供 `sandbox::validate_network_permissions` 复用（隧道服务器地址
/// 白名单同样需要通配符，如 LoliaFrp 平台动态节点 `*.qwq.fan`）。
pub(crate) fn host_matches(host: &str, pattern: &str) -> bool {
    if let Some(rest) = pattern.strip_prefix("*.") {
        host.ends_with(rest) && host.len() > rest.len()
    } else {
        host == pattern
    }
}
