//! 厂商管理 action 注册：列表/启禁/安装/卸载、frpc 二进制就绪、
//! 厂商 API 引擎（fetch_vendor_config）、认证适配器沙箱（run_auth_adapter）。

use crate::commands::frp;
use crate::handler;
use crate::utils::dispatcher::Dispatcher;

use super::{EnsureFrpcParams, InstallProviderParams, ProviderIdParams, RunAuthAdapterParams};

/// 注册厂商管理相关 action
pub fn register(d: &mut Dispatcher) {
    d.register("list_providers", handler!(state, _app, _params, {
        let r = frp::provider::list_providers(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("ensure_frpc", handler!(state, _app, params, {
        // 兼容空 params（{} 或 null）：unwrap_or_default 返回 provider_id=None
        let p: EnsureFrpcParams = serde_json::from_value(params).unwrap_or_default();
        let r = frp::binary::ensure_frpc(&state, p.provider_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("install_provider_from_dir", handler!(_state, _app, params, {
        let p: InstallProviderParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::install::install_provider_from_dir(p.source_dir).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("install_provider_from_zip", handler!(_state, _app, params, {
        let p: InstallProviderParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::install::install_provider_from_zip(p.source_dir).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("uninstall_provider", handler!(_state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::install::uninstall_provider(p.provider_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("enable_provider", handler!(_state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::provider::enable_provider(p.provider_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("disable_provider", handler!(_state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        frp::provider::disable_provider(p.provider_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    // 厂商 API 引擎（阶段三：api-schema.json 解析 + 配置拉取）
    d.register("fetch_vendor_config", handler!(state, _app, params, {
        let p: ProviderIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::api_schema::fetch_vendor_config(&state, &p.provider_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // 认证适配器脚本沙箱（阶段四 §7.5）
    d.register("run_auth_adapter", handler!(_state, _app, params, {
        let p: RunAuthAdapterParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = frp::sandbox::run_auth_adapter(&p.provider_id, p.command, p.args).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
}
