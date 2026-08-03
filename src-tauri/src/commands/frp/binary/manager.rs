//! frpc 二进制入口编排（ensure_frpc）：按厂商分发方式路由到 system_default / external

use super::super::provider::{
    get_frpc_path_for_provider, is_external_frpc_ready, read_provider_manifest, SYSTEM_DEFAULT_ID,
};
use super::{external, system_default};
use crate::state::AppState;

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
