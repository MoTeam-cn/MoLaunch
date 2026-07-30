//! frpc 二进制下载与管理
//!
//! 从 `install.rs` 拆分，职责：
//! - 系统默认厂商 frpc：从 GitHub Releases 下载 ZIP 并提取 frpc 二进制
//! - 外部厂商 frpc：按 `manifest.binary.distribution` 处理
//!   - bundled：仅校验文件存在（厂商包自带）
//!   - url：从配置 URL 下载（HTTPS + 域名白名单 + SHA256 校验 + 可选解压）
//!
//! 依赖 `provider.rs` 的路径函数、manifest 读取等（`super::provider::*`）。

use super::provider::{
    frpc_path, get_frpc_path_for_provider, is_external_frpc_ready, is_frpc_ready,
    read_provider_manifest, system_default_dir, FRPC_VERSION, SYSTEM_DEFAULT_ID,
};
use super::{ensure_dir, providers_root, ProviderManifest};
use crate::log_info;
use std::path::Path;

// ============================================================
// frpc 二进制管理
// ============================================================

/// 下载 frpc 二进制
///
/// `provider_id` 为 None 或 `system-default` 时走系统默认厂商下载逻辑。
/// 外部厂商根据 manifest.binary.distribution 处理：
/// - bundled: 仅校验文件存在（厂商包自带 frpc）
/// - url: 从配置的 URL 下载（HTTPS + 域名白名单 + SHA256 校验）
pub async fn ensure_frpc(provider_id: Option<String>) -> Result<String, String> {
    let pid = provider_id.unwrap_or_else(|| SYSTEM_DEFAULT_ID.to_string());
    if pid == SYSTEM_DEFAULT_ID {
        return ensure_system_default_frpc().await;
    }
    let manifest = read_provider_manifest(&pid)?;
    if is_external_frpc_ready(&pid, &manifest) {
        let path = get_frpc_path_for_provider(&pid)?;
        return Ok(format!("frpc 已就绪: {}", path.display()));
    }
    // frpc 未就绪：bundled 无法补下，仅 url 可下载
    match manifest.binary.distribution.as_str() {
        "bundled" => Err(format!(
            "厂商 {} 的 frpc 二进制缺失，请重新安装厂商包",
            pid
        )),
        "url" => ensure_external_frpc(&pid, &manifest).await,
        other => Err(format!(
            "厂商 {} 使用不支持的分发方式: {}",
            pid, other
        )),
    }
}

/// 系统默认厂商 frpc 下载（从 GitHub Releases 下载 ZIP 并提取 frpc 二进制）
async fn ensure_system_default_frpc() -> Result<String, String> {
    if is_frpc_ready() {
        return Ok(format!("frpc 已就绪: {}", frpc_path().display()));
    }

    let dir = system_default_dir();
    ensure_dir(&dir)?;

    let (zip_url, entry_name) = frpc_download_info()?;
    log_info!("[Frp] 开始下载 frpc: {}", zip_url);

    let client = crate::http::get_client();
    let response = client
        .get(&zip_url)
        .send()
        .await
        .map_err(|e| format!("下载 frpc 失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载 frpc 失败: HTTP {}", response.status()));
    }

    let zip_bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取 frpc 下载内容失败: {}", e))?;

    log_info!("[Frp] frpc ZIP 下载完成，大小: {} 字节", zip_bytes.len());

    let cursor = std::io::Cursor::new(&zip_bytes[..]);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("解析 frpc ZIP 失败: {}", e))?;

    let frpc_entry = format!("{}/{}", entry_name, frpc_filename());
    let target_path = frpc_path();

    let mut found = false;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let name = file.name().to_string();

        let entry_match = name.ends_with(frpc_filename().as_str())
            && name.split('/').last() == Some(frpc_filename().as_str());

        if entry_match {
            log_info!("[Frp] 从 ZIP 提取: {}", name);
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建 frpc 目录失败: {}", e))?;
            }
            let mut out = std::fs::File::create(&target_path)
                .map_err(|e| format!("创建 frpc 文件失败: {}", e))?;
            std::io::copy(&mut file, &mut out)
                .map_err(|e| format!("写入 frpc 文件失败: {}", e))?;
            found = true;
            break;
        }
    }

    if !found {
        return Err(format!(
            "ZIP 中未找到 frpc 二进制（期望条目 {}）",
            frpc_entry
        ));
    }

    let metadata = std::fs::metadata(&target_path)
        .map_err(|e| format!("frpc 文件元数据读取失败: {}", e))?;
    if metadata.len() < 1024 {
        std::fs::remove_file(&target_path).ok();
        return Err("frpc 下载文件过小，可能已损坏".to_string());
    }

    log_info!("[Frp] frpc 下载完成: {}", target_path.display());
    Ok(format!("frpc 下载完成: {}", target_path.display()))
}

/// 外部厂商 frpc 下载（distribution=url）
///
/// 校验 URL HTTPS + 域名白名单 + SHA256（如有）。
/// 下载完成后若 archive=true，则解压到厂商目录。
async fn ensure_external_frpc(
    provider_id: &str,
    manifest: &ProviderManifest,
) -> Result<String, String> {
    let dl = manifest
        .binary
        .download
        .as_ref()
        .ok_or_else(|| format!("厂商 {} 缺少 binary.download 配置", provider_id))?;

    validate_download_url(&dl.url, &dl.allowed_domains)?;

    let provider_dir = providers_root().join(provider_id);
    let target_path = provider_dir.join(&dl.target_path);
    if let Some(parent) = target_path.parent() {
        ensure_dir(parent)?;
    }

    log_info!("[Frp] 开始下载外部厂商 frpc: {} ({})", provider_id, dl.url);
    let client = crate::http::get_client();
    let response = client
        .get(&dl.url)
        .send()
        .await
        .map_err(|e| format!("下载失败: {}", e))?;
    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取下载内容失败: {}", e))?;

    if let Some(ref expected_sha) = dl.sha256 {
        let actual = compute_sha256(&bytes);
        if actual != *expected_sha {
            return Err(format!(
                "SHA256 校验失败：期望 {}，实际 {}",
                expected_sha, actual
            ));
        }
    }

    std::fs::write(&target_path, &bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    if dl.archive {
        extract_archive(&target_path, &provider_dir)?;
    }

    log_info!("[Frp] 外部厂商 frpc 下载完成: {}", target_path.display());
    Ok(format!("frpc 下载完成: {}", target_path.display()))
}

// ============================================================
// 下载辅助函数
// ============================================================

/// 校验下载 URL：必须 HTTPS + 域名在白名单中
fn validate_download_url(url: &str, allowed_domains: &[String]) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("下载 URL 必须使用 HTTPS".to_string());
    }
    let rest = &url[8..];
    let host_end = rest.find(|c| c == '/' || c == ':').unwrap_or(rest.len());
    let host = &rest[..host_end];
    if !allowed_domains.iter().any(|d| host == d.as_str()) {
        return Err(format!("下载域名 {} 不在白名单中", host));
    }
    Ok(())
}

/// 计算 SHA256（十六进制小写）
fn compute_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// 解压归档文件到目标目录（Zip Slip 防护）
fn extract_archive(archive_path: &Path, dst: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive_path)
        .map_err(|e| format!("打开归档失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解析归档失败: {}", e))?;
    let canonical_dst = dst.canonicalize().unwrap_or_else(|_| dst.to_path_buf());
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let name = entry.name().to_string();
        if name.ends_with('/') {
            std::fs::create_dir_all(dst.join(&name))
                .map_err(|e| format!("创建目录失败: {}", e))?;
            continue;
        }
        let file_path = dst.join(&name);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建父目录失败: {}", e))?;
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("canonicalize 失败: {}", e))?;
            if !canonical_parent.starts_with(&canonical_dst) {
                return Err(format!("Zip Slip 检测: {}", name));
            }
        }
        let mut out = std::fs::File::create(&file_path)
            .map_err(|e| format!("创建文件失败: {}", e))?;
        std::io::copy(&mut entry, &mut out)
            .map_err(|e| format!("写入文件失败: {}", e))?;
    }
    Ok(())
}

// ============================================================
// 系统默认 frpc 下载信息
// ============================================================

/// 获取 frpc 下载信息（URL + ZIP 内目录名）
fn frpc_download_info() -> Result<(String, String), String> {
    let version = FRPC_VERSION;
    let (platform, arch) = current_platform()?;
    let zip_name = format!("frp_{}_{}_{}.zip", version, platform, arch);
    let entry_name = format!("frp_{}_{}_{}", version, platform, arch);
    let url = format!(
        "https://github.com/fatedier/frp/releases/download/v{}/{}",
        version, zip_name
    );
    Ok((url, entry_name))
}

/// 当前平台的 frpc 标识
fn current_platform() -> Result<(&'static str, &'static str), String> {
    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err("不支持的操作系统".to_string());
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "x86") {
        "386"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        return Err("不支持的 CPU 架构".to_string());
    };

    Ok((platform, arch))
}

/// frpc 二进制文件名（含扩展名）
fn frpc_filename() -> String {
    #[cfg(target_os = "windows")]
    {
        "frpc.exe".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "frpc".to_string()
    }
}
