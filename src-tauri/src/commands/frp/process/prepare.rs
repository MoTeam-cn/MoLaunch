//! frpc 启动配置准备。

use crate::commands::frp::provider;
use crate::commands::frp::tunnel;
use crate::commands::frp::{ensure_dir, Tunnel};
use crate::log_info;
use crate::state::AppState;

/// 生成启动用 frpc 配置文件（优先厂商原版，回退本地生成）。
pub(super) async fn prepare_config(
    state: &AppState,
    tunnel: &Tunnel,
) -> Result<std::path::PathBuf, String> {
    let config_dir = crate::commands::frp::frp_config_dir();
    ensure_dir(&config_dir)?;
    let config_path = config_dir.join(format!("{}.toml", tunnel.id));

    if let Some(raw) = tunnel
        .raw_config
        .as_deref()
        .filter(|v| !v.trim().is_empty())
    {
        std::fs::write(&config_path, raw)
            .map_err(|e| format!("写入厂商原版 frpc 配置失败: {}", e))?;
        log_info!(
            "[Frp] 直接复用已保存的厂商原版配置: {}",
            config_path.display()
        );
        return Ok(config_path);
    }

    if tunnel.provider_id == provider::SYSTEM_DEFAULT_ID {
        tunnel::generate_config(tunnel)?;
        return Ok(config_path);
    }

    let manifest = provider::read_provider_manifest(&tunnel.provider_id)?;
    let endpoints_file = manifest
        .api
        .as_ref()
        .map(|a| a.endpoints_file.as_str())
        .unwrap_or("api/endpoints.json");
    let spec = crate::commands::frp::api_spec::load_api_spec(&tunnel.provider_id, endpoints_file)?;
    let has_config_endpoint = spec
        .endpoints
        .as_ref()
        .and_then(|e| e.tunnels.as_ref())
        .and_then(|t| t.config.as_ref())
        .is_some();
    if !has_config_endpoint {
        tunnel::generate_config(tunnel)?;
        return Ok(config_path);
    }

    let remote_name = tunnel.remote_tunnel_name.as_deref().unwrap_or(&tunnel.name);
    let raw = crate::commands::frp::api_spec::fetch_raw_tunnel_config(
        state,
        &tunnel.provider_id,
        &tunnel.id,
        remote_name,
    )
    .await
    .map_err(|e| {
        format!(
            "厂商 config 接口获取失败，已停止启动（不会回退本地配置）: {}",
            e
        )
    })?;

    std::fs::write(&config_path, raw).map_err(|e| format!("写入厂商原版 frpc 配置失败: {}", e))?;
    log_info!(
        "[Frp] 使用厂商 config 接口原样配置启动: {}",
        config_path.display()
    );
    Ok(config_path)
}
