//! 外部厂商：manifest / auth.json 解析、图标、启用状态持久化
//! 外部厂商存放于 `<base_dir>/providers/<provider_id>/`，manifest.json 描述厂商元信息、
//! frpc 分发方式（bundled/url）和认证配置。安装/卸载见 [`super::super::install`]，
//! frpc 下载见 [`super::super::binary`]。

use super::provider_system::resolve_bundled_path;
use super::super::{
    ensure_dir, providers_root, providers_state_path, validate_provider_id, AuthFile,
    AuthFileDeviceCode, AuthFileOAuth2, ProviderManifest,
};
use std::collections::HashMap;

/// 读取外部厂商的 auth.json
///
/// 按 manifest.authFile 指定的相对路径读取厂商认证交互层配置。
/// 文件缺失、不可读或解析失败均返回 None（调用方按需报错）。
pub(super) fn read_auth_file(provider_id: &str, manifest: &ProviderManifest) -> Option<AuthFile> {
    let auth_file_name = manifest.auth_file.as_ref()?;
    let auth_path = providers_root().join(provider_id).join(auth_file_name);
    let content = std::fs::read_to_string(&auth_path).ok()?;
    serde_json::from_str::<AuthFile>(&content).ok()
}

/// 解析厂商认证类型
///
/// 优先使用 manifest.json 的 auth.type；
/// 若为 "none" 且 manifest 声明了 authFile，则回退从 auth.json 的 type 读取。
/// 这样厂商只需在 auth.json 中声明 type，无需在 manifest.json 中重复声明。
pub(crate) fn resolve_auth_type(provider_id: &str, manifest: &ProviderManifest) -> String {
    if manifest.auth.auth_type != "none" {
        return manifest.auth.auth_type.clone();
    }
    if let Some(auth_file) = read_auth_file(provider_id, manifest) {
        if auth_file.auth_type != "none" {
            return auth_file.auth_type;
        }
    }
    "none".to_string()
}

/// 解析厂商的 OAuth2 交互配置（从 auth.json 读取）
///
/// 新设计中 OAuth2 配置（authorizeUrl / clientId / clientSecret / scopes / redirectPort）
/// 存放在 auth.json 中，token 交换的请求/响应规范见 endpoints.json 的 authFlows.oauth2。
pub(crate) fn resolve_oauth2_config(
    provider_id: &str,
    manifest: &ProviderManifest,
) -> Result<AuthFileOAuth2, String> {
    let auth_file = read_auth_file(provider_id, manifest).ok_or_else(|| {
        format!(
            "厂商 {} 的 manifest 未声明 authFile，无法读取 OAuth2 配置",
            provider_id
        )
    })?;
    auth_file
        .oauth2
        .ok_or_else(|| format!("厂商 {} 的 auth.json 缺少 oauth2 配置", provider_id))
}

/// 解析厂商的 Device Code 交互配置（从 auth.json 读取）
pub(crate) fn resolve_device_code_config(
    provider_id: &str,
    manifest: &ProviderManifest,
) -> Result<AuthFileDeviceCode, String> {
    let auth_file = read_auth_file(provider_id, manifest).ok_or_else(|| {
        format!(
            "厂商 {} 的 manifest 未声明 authFile，无法读取 Device Code 配置",
            provider_id
        )
    })?;
    auth_file
        .device_code
        .ok_or_else(|| format!("厂商 {} 的 auth.json 缺少 device_code 配置", provider_id))
}

/// 读取图标文件并转为 base64 data URL
///
/// 支持常见图片格式（png/jpg/jpeg/ico/svg/webp/gif），文件不存在或读取失败返回 None。
pub(crate) fn read_icon_as_data_url(provider_id: &str, icon_rel: &str) -> Option<String> {
    let icon_path = providers_root().join(provider_id).join(icon_rel);
    if !icon_path.exists() {
        return None;
    }
    let data = std::fs::read(&icon_path).ok()?;
    let mime = match icon_path.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "image/png",
    };
    use base64::{engine::general_purpose::STANDARD, Engine};
    let b64 = STANDARD.encode(&data);
    Some(format!("data:{};base64,{}", mime, b64))
}

/// 判断外部厂商的 frpc 是否就绪
pub(crate) fn is_external_frpc_ready(provider_id: &str, manifest: &ProviderManifest) -> bool {
    let dir = providers_root().join(provider_id);
    match manifest.binary.distribution.as_str() {
        "bundled" => {
            // 优先按平台映射 paths 查找，回退到 path
            if let Some(rel) = resolve_bundled_path(&manifest.binary) {
                dir.join(rel).exists()
            } else {
                false
            }
        }
        "url" => {
            if let Some(ref dl) = manifest.binary.download {
                dir.join(&dl.target_path).exists()
            } else {
                false
            }
        }
        _ => false,
    }
}

// 厂商启用状态持久化
/// 读取厂商启用状态（`<base_dir>/frp/providers.json`）
///
/// 文件不存在或损坏时返回空 HashMap（所有外部厂商默认启用）。
pub(crate) fn read_providers_state() -> HashMap<String, bool> {
    let path = providers_state_path();
    if !path.exists() {
        return HashMap::new();
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    if content.trim().is_empty() {
        return HashMap::new();
    }
    serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new())
}

/// 写入厂商启用状态
pub(crate) fn write_providers_state(state: &HashMap<String, bool>) -> Result<(), String> {
    let path = providers_state_path();
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let content =
        serde_json::to_string_pretty(state).map_err(|e| format!("序列化厂商状态失败: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("写入厂商状态失败: {}", e))
}

// 厂商清单读取
/// 读取外部厂商的 manifest.json
pub(crate) fn read_provider_manifest(provider_id: &str) -> Result<ProviderManifest, String> {
    validate_provider_id(provider_id)?;
    let manifest_path = providers_root().join(provider_id).join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!("厂商 manifest 不存在: {}", manifest_path.display()));
    }
    let content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取 manifest 失败: {}", e))?;
    let manifest: ProviderManifest =
        serde_json::from_str(&content).map_err(|e| format!("解析 manifest 失败: {}", e))?;
    if manifest.id != provider_id {
        return Err(format!(
            "manifest.id ({}) 与目录名 ({}) 不一致",
            manifest.id, provider_id
        ));
    }
    Ok(manifest)
}
