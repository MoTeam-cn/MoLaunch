//! frpc 二进制下载与管理（从原 `binary.rs` 拆分）
//! 系统默认厂商 frpc：从 apiServer `/v1/frp/manifest` 获取最新版本下载 URL；
//! 外部厂商 frpc：按 `manifest.binary.distribution` 处理（bundled 仅校验，url 下载）。
//! 子模块：system_default（系统默认 frpc）/external（外部厂商 frpc）/archive（ZIP 提取）。

mod archive;
mod external;
mod system_default;

use super::provider::{
    get_frpc_path_for_provider, is_external_frpc_ready, read_provider_manifest, SYSTEM_DEFAULT_ID,
};
use crate::state::AppState;

pub(crate) use external::host_matches;
pub use system_default::fetch_latest_frpc_version;

/// 下载 frpc 二进制
///
/// `provider_id` 为 None 或 `system-default` 时走系统默认厂商下载逻辑
/// （从 apiServer `/v1/frp/manifest` 获取 URL）。
/// 外部厂商根据 manifest.binary.distribution 处理：
/// - bundled: 仅校验文件存在（厂商包自带 frpc）
/// - url: 从配置的 URL 下载（HTTPS + 域名白名单 + SHA256 校验）
pub async fn ensure_frpc(state: &AppState, provider_id: Option<String>) -> Result<String, String> {
    let pid = provider_id.unwrap_or_else(|| SYSTEM_DEFAULT_ID.to_string());
    if pid == SYSTEM_DEFAULT_ID {
        return system_default::ensure_system_default_frpc(state).await;
    }
    let manifest = read_provider_manifest(&pid)?;
    if is_external_frpc_ready(&pid, &manifest) {
        let path = get_frpc_path_for_provider(&pid)?;
        return Ok(format!("frpc 已就绪: {}", path.display()));
    }
    // frpc 未就绪：bundled 无法补下，仅 url 可下载
    match manifest.binary.distribution.as_str() {
        "bundled" => Err(format!("厂商 {} 的 frpc 二进制缺失，请重新安装厂商包", pid)),
        "url" => external::ensure_external_frpc(&pid, &manifest).await,
        other => Err(format!("厂商 {} 使用不支持的分发方式: {}", pid, other)),
    }
}

/// apiServer 端期望的平台/架构字符串（用于 `/v1/frp/manifest` 查询参数）
///
/// - 平台：`windows` / `macos` / `linux`
/// - 架构：`x86_64` / `aarch64` / `i686` / `armv7`
pub(super) fn api_server_platform_arch() -> Result<(&'static str, &'static str), String> {
    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err("不支持的操作系统".to_string());
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "x86") {
        "i686"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "arm") {
        "armv7"
    } else {
        return Err("不支持的 CPU 架构".to_string());
    };

    Ok((platform, arch))
}

/// frpc 二进制文件名（含扩展名）
pub(super) fn frpc_filename() -> String {
    #[cfg(target_os = "windows")]
    {
        "frpc.exe".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "frpc".to_string()
    }
}
