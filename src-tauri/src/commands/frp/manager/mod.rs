//! Frp 模块统一分发逻辑（frp_manager 的命令层实现）

mod auth_actions;
mod dispatcher;
mod provider_actions;
mod public_server_actions;
mod tunnel_actions;

pub use dispatcher::dispatch;
pub(crate) use dispatcher::{
    DetectPackageParams, EnsureFrpcParams, InstallProviderFromUrlParams, InstallProviderParams,
    ProviderIdParams, ReadLogParams, RunAuthAdapterParams, SaveApiKeyParams,
};
