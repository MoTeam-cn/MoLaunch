//! Frp 内网穿透命令模块（编排层，子模块由 `manager::dispatch` 统一调用）

pub mod api_spec;
pub mod auth;
pub mod binary;
pub mod install;
pub mod log_redact;
pub mod manager;
pub mod paths;
pub mod process;
pub mod provider;
pub mod sandbox;
pub mod tunnel;
pub mod types;

// 共享类型与路径函数 re-export（外部调用方通过 `crate::commands::frp::Tunnel` 等访问）
pub use crate::utils::fs::ensure_dir;
pub use paths::{
    frp_config_dir, frp_data_dir, frp_logs_dir, providers_root, providers_state_path, tunnels_path,
    validate_provider_id,
};
pub use types::{
    ApiKeyConfig, ApiRef, ApiSpec, AuthConfig, AuthFile, AuthFileApiKey, AuthFileDeviceCode,
    AuthFileOAuth2, AuthFlows, AuthHeader, BinaryConfig, ConfigMode, DeviceCodeConfig,
    DownloadConfig, EndpointDef, EndpointsDef, Envelope, FieldExtractor, FieldMapping, FlowRequest,
    LogFileContent, LogFileInfo, NetworkPermissions, OAuth2Config, ProcessPermissions,
    ProviderInfo, ProviderManifest, RemoteLoginFlow, ResponseDef, Tunnel, TunnelStatus, TunnelType,
    TunnelWithStatus, TunnelsDef,
};

/// 统一 Frp 管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `manager::dispatch` 进行 action 分发。
/// 注：该函数为 Tauri `generate_handler!` 所需的命令注册点，必须定义在本模块
/// （`#[tauri::command]` 生成的 `__cmd__*` 宏仅在本模块作用域可见，无法经 `pub use` 重导出）。
#[tauri::command]
pub async fn frp_manager(
    state: tauri::State<'_, crate::state::AppState>,
    app: tauri::AppHandle,
    req: crate::utils::dispatcher::ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    manager::dispatch(state, app, req).await
}
