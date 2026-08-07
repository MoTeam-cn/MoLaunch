//! 厂商安装/卸载命令编排。
//!
//! 具体文件合并、ZIP 安全解压和卸载职责分别位于子模块；公开入口保持不变。

mod files;
mod uninstall;
mod zip;

use super::provider::{frpc_platform_skip, write_provider_frpc_version};
use super::{ensure_dir, providers_root, validate_provider_id, ProviderInfo, ProviderManifest};
use crate::log_info;
use std::path::Path;

pub use files::install_provider_from_dir;
pub use uninstall::uninstall_provider;
pub use zip::install_provider_from_zip;

/// 从 URL 下载并安装外部厂商。
///
/// 下载 ZIP 到临时文件，复用 `install_provider_from_zip` 安装逻辑。
/// 仅允许 HTTPS URL（用户主动提供，无域名白名单限制）。
/// 无论安装成功或失败，临时文件都会被清理。
pub async fn install_provider_from_url(url: String) -> Result<ProviderInfo, String> {
    if !url.starts_with("https://") {
        return Err("URL 必须使用 HTTPS".to_string());
    }

    log_info!("[Frp] 开始从 URL 下载厂商包: {}", url);

    let client = crate::http::build_client_with_redirect(
        reqwest::redirect::Policy::limited(5),
        Some(60_000),
    );
    let response = client
        .get(&url)
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
    let temp_zip =
        std::env::temp_dir().join(format!("molaunch-provider-{}.zip", std::process::id()));
    std::fs::write(&temp_zip, &bytes).map_err(|e| format!("写入临时文件失败: {}", e))?;
    log_info!("[Frp] 厂商包下载完成，大小: {} 字节", bytes.len());

    let result = install_provider_from_zip(temp_zip.to_string_lossy().to_string()).await;
    let _ = std::fs::remove_file(&temp_zip);
    result
}

/// 从 manifest + 启用状态构建 ProviderInfo。
pub(super) fn build_provider_info(manifest: &ProviderManifest) -> ProviderInfo {
    let state = super::provider::read_providers_state();
    let frpc_ready = super::provider::is_external_frpc_ready(&manifest.id, manifest);
    let enabled = state.get(&manifest.id).copied().unwrap_or(true);
    let auth_type = super::provider::resolve_auth_type(&manifest.id, manifest);
    let icon = manifest
        .icon
        .as_ref()
        .and_then(|icon_rel| super::provider::read_icon_as_data_url(&manifest.id, icon_rel));
    ProviderInfo {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        description: manifest.description.clone(),
        version: manifest.version.clone(),
        author: manifest.author.clone(),
        builtin: false,
        auth_type,
        frpc_ready,
        enabled,
        distribution: manifest.binary.distribution.clone(),
        homepage: manifest.homepage.clone(),
        icon,
    }
}

pub(super) fn prepare_install_target(
    manifest: &ProviderManifest,
) -> Result<(std::path::PathBuf, std::collections::HashSet<String>), String> {
    validate_provider_id(&manifest.id)?;
    let target_dir = providers_root().join(&manifest.id);
    let (skip, _) = frpc_platform_skip(&manifest.binary);
    Ok((target_dir, skip))
}

pub(super) fn finalize_install(
    target_dir: &Path,
    manifest: &ProviderManifest,
    is_install: bool,
    added: u32,
    zip: bool,
) -> Result<ProviderInfo, String> {
    let installed_manifest_path = target_dir.join("manifest.json");
    if !installed_manifest_path.exists() {
        if added == 0 {
            let _ = std::fs::remove_dir_all(target_dir);
        }
        return Err("安装校验失败：manifest.json 不存在".to_string());
    }
    if !is_install {
        log_info!(
            "[Frp] 厂商{}更新: {} ({}), 变更 {} 个文件",
            if zip { "已从 ZIP " } else { "已" },
            manifest.name,
            manifest.id,
            added
        );
    } else {
        log_info!(
            "[Frp] 厂商{}安装: {} ({}), {} 个文件",
            if zip { "已从 ZIP " } else { "已" },
            manifest.name,
            manifest.id,
            added
        );
    }
    if let Some(version) = manifest.binary.frpc_version.as_deref() {
        write_provider_frpc_version(&manifest.id, version);
    }
    Ok(build_provider_info(manifest))
}

pub(super) fn ensure_provider_root() -> Result<(), String> {
    ensure_dir(&providers_root())
}
