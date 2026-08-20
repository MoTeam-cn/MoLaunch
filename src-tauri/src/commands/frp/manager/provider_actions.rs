//! 厂商管理 action 注册：列表/启禁/安装/卸载、frpc 二进制就绪、
//! 厂商 API 引擎（fetch_tunnels）、认证适配器沙箱（run_auth_adapter）。

use crate::commands::frp;
use crate::handler;
use crate::utils::dispatcher::Dispatcher;

use super::{
    DetectPackageParams, EnsureFrpcParams, InstallProviderFromUrlParams, InstallProviderParams,
    ProviderIdParams, RunAuthAdapterParams,
};

/// 注册厂商管理相关 action
pub fn register(d: &mut Dispatcher) {
    d.register(
        "list_providers",
        handler!(_state, _app, _params, {
            let r = frp::provider::list_providers().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "ensure_frpc",
        handler!(state, _app, params, {
            // 兼容空 params（{} 或 null）：unwrap_or_default 返回 provider_id=None
            let p: EnsureFrpcParams = serde_json::from_value(params).unwrap_or_default();
            let r = frp::binary::ensure_frpc(&state, p.provider_id).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "install_provider_from_dir",
        handler!(_state, _app, params, {
            let p: InstallProviderParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = frp::install::install_provider_from_dir(p.source_dir).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "install_provider_from_zip",
        handler!(_state, _app, params, {
            let p: InstallProviderParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = frp::install::install_provider_from_zip(p.source_dir).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "detect_package_type",
        handler!(_state, _app, params, {
            let p: DetectPackageParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = frp::detect::detect_package_type(&p.path)?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "install_provider_from_url",
        handler!(state, _app, params, {
            let p: InstallProviderFromUrlParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = frp::install::install_provider_from_url(&state, p.url).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "uninstall_provider",
        handler!(_state, _app, params, {
            let p: ProviderIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            frp::install::uninstall_provider(p.provider_id).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "enable_provider",
        handler!(_state, _app, params, {
            let p: ProviderIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            frp::provider::enable_provider(p.provider_id).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "disable_provider",
        handler!(_state, _app, params, {
            let p: ProviderIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            frp::provider::disable_provider(p.provider_id).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    // 厂商 API 引擎（按 endpoints.json 配置拉取隧道列表 + 账号信息）
    d.register(
        "fetch_tunnels",
        handler!(state, _app, params, {
            let p: ProviderIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let (tunnels, account) = frp::api_spec::fetch_tunnels(&state, &p.provider_id).await?;
            // 包装为对象返回，前端按 {tunnels, account} 取值
            #[derive(serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            struct FetchTunnelsResult<'a> {
                tunnels: &'a [frp::api_spec::TunnelInfo],
                account: &'a frp::api_spec::AccountInfo,
            }
            let r = FetchTunnelsResult {
                tunnels: &tunnels,
                account: &account,
            };
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    // 认证适配器脚本沙箱（阶段四 §7.5）
    d.register(
        "run_auth_adapter",
        handler!(_state, _app, params, {
            let p: RunAuthAdapterParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = frp::sandbox::run_auth_adapter(&p.provider_id, p.command, p.args).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
}
