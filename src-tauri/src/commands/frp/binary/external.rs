//! 外部厂商 frpc 下载（distribution=url）：安全校验与解压留在这里，字节搬运交给统一下载栈。
//!
//! 下载本身走 `DownloadManager`（分片、多源重试、限速、SHA256 校验），
//! 但白名单这一层不能交出去：重定向由这里逐跳校验，且下载用的 client 不跟随 3xx，
//! 任何「服务器临时把你转到别处」都在越过白名单之前失败。

use super::super::{
    ensure_dir, frpc_platform_skip, providers_root, resolve_download_config, ProviderManifest,
};
use super::archive;
use crate::log_info;
use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::minecraft::sources::DownloadSourceMode;
use crate::state::AppState;
use std::path::Path;
use std::time::Duration;

/// 外部厂商 frpc 下载（distribution=url）
///
/// 校验 URL HTTPS + 域名白名单 + SHA256（如有）。
/// 下载完成后若 archive=true，则解压到厂商目录。
/// 按当前平台选择下载 URL 与目标路径（urls/target_paths 平台映射，回退 url/target_path）。
pub(crate) async fn ensure_external_frpc(
    state: &AppState,
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

    // 目标路径来自云端下发的厂商 manifest，不能被用来往厂商目录外写文件
    validate_target_path(&target_rel)?;
    let provider_dir = providers_root().join(provider_id);
    let target_path = provider_dir.join(&target_rel);
    if let Some(parent) = target_path.parent() {
        ensure_dir(parent)?;
    }

    // 重定向在这里走完并逐跳校验白名单，拿到最终地址再交给统一下载栈
    let final_url = resolve_redirect_target(&download_url, &dl.allowed_domains).await?;
    log_info!(
        "[Frp] 开始下载外部厂商 frpc: {} ({})",
        provider_id,
        final_url
    );

    // 统一栈：分片、重试阶梯、限速、哈希校验都用同一套；frpc 无独立进度 UI，静默下载
    let config = DownloadManagerConfig::from_state(state)
        .await
        .with_silent(true);
    let mut manager = DownloadManager::from_config(&config);
    // 不允许镜像替换：这条 URL 是厂商在自己的白名单域名上声明的，换成任何镜像都算越界
    manager.source_mode = DownloadSourceMode::Official;
    manager.preserve_order = true;
    manager.client = crate::http::no_redirect_client();

    let task = DownloadTask {
        id: format!("frpc_binary_{}", provider_id),
        urls: vec![final_url],
        local_path: target_path.to_string_lossy().to_string(),
        // manifest 未声明大小，交给下载栈自己探测（探测不到就退化为单流）
        expected_size: 0,
        expected_hash: dl.sha256.clone(),
    };
    let results = manager.download_batch(vec![task], None).await;
    match results.into_iter().next() {
        Some(result) if result.status == DownloadStatus::Completed => {}
        Some(result) => {
            return Err(format!(
                "frpc 下载失败: {}",
                result.error.unwrap_or_else(|| "未知错误".to_string())
            ))
        }
        None => return Err("frpc 下载未执行".to_string()),
    }

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

/// 手动跟随重定向并逐跳校验域名白名单，返回最终 URL
///
/// 用不跟随 3xx 的 client 发一次不带 body 的请求，把链路走完：
/// 任何一跳落在白名单外都直接失败，不把「跳过白名单」的机会留给下载阶段。
async fn resolve_redirect_target(url: &str, allowed_domains: &[String]) -> Result<String, String> {
    const MAX_REDIRECTS: u32 = 5;
    let client = crate::http::no_redirect_client();
    let mut current_url = url.to_string();
    let mut redirects = 0_u32;
    loop {
        let resp = client
            .get(&current_url)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| format!("下载失败: {}", crate::http::request_error_msg(&e)))?;

        if resp.status().is_redirection() {
            redirects += 1;
            if redirects > MAX_REDIRECTS {
                return Err("重定向次数超过限制".to_string());
            }
            let location = resp
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| "重定向响应缺少 Location 头".to_string())?;
            let base =
                reqwest::Url::parse(&current_url).map_err(|e| format!("解析 URL 失败: {}", e))?;
            let next_url = base
                .join(location)
                .map_err(|e| format!("解析重定向 URL 失败: {}", e))?;
            let host = next_url.host_str().unwrap_or("");
            if !allowed_domains.iter().any(|d| host_matches(host, d)) {
                return Err(format!("重定向域名 {} 不在白名单中", host));
            }
            log_info!("[Frp] 重定向到白名单域名: {}", next_url);
            current_url = next_url.to_string();
            continue;
        }

        if !resp.status().is_success() {
            return Err(format!("下载失败: HTTP {}", resp.status()));
        }
        return Ok(current_url);
    }
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

/// 校验下载目标：必须是厂商目录内的相对路径
///
/// manifest 是云端下发的，`target_path` 里出现 `..`、绝对路径或盘符就相当于
/// 允许它往任意位置写文件，这里一律拒绝。
fn validate_target_path(target_rel: &str) -> Result<(), String> {
    if target_rel.trim().is_empty() {
        return Err("下载目标路径为空".to_string());
    }
    let path = Path::new(target_rel);
    if path.is_absolute() {
        return Err(format!("下载目标路径不能是绝对路径: {}", target_rel));
    }
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!("下载目标路径不能包含上级目录: {}", target_rel));
    }
    // Windows 盘符 / UNC / 备用数据流都靠这两个字符，一律拒绝
    if target_rel.contains(':') || target_rel.contains('\\') {
        return Err(format!("下载目标路径含非法字符: {}", target_rel));
    }
    Ok(())
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
/// 白名单同样需要通配符，如某平台动态节点 `*.qwq.fan`）。
pub(crate) fn host_matches(host: &str, pattern: &str) -> bool {
    if let Some(rest) = pattern.strip_prefix("*.") {
        // 必须落在点边界上：`*.example.com` 不能匹配 `evilexample.com`，
        // 否则攻击者注册一个「后缀相同」的域名就能混进白名单
        return host
            .strip_suffix(rest)
            .is_some_and(|prefix| prefix.ends_with('.'));
    }
    host == pattern
}

#[cfg(test)]
#[path = "external_test.rs"]
mod external_test;
