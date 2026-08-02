//! Frp 内网穿透命令模块（编排层，子模块由 `utils::frp_manager::dispatch` 统一调用）

pub mod api_spec;
pub mod auth;
pub mod binary;
pub mod install;
pub mod log_redact;
pub mod paths;
pub mod process;
pub mod provider;
pub mod sandbox;
pub mod tunnel;
pub mod types;

use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::{AppHandle, State};

// 共享类型与路径函数 re-export（外部调用方通过 `crate::commands::frp::Tunnel` 等访问）
pub use paths::{
    ensure_dir, frp_config_dir, frp_data_dir, frp_logs_dir, providers_root, providers_state_path,
    tunnels_path, validate_provider_id,
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
/// `crate::utils::frp_manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn frp_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::frp_manager::dispatch(state, app, req).await
}
