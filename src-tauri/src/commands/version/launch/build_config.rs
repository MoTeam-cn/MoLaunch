//! 启动配置构建
//!
//! 从全局配置 + 版本独立设置 + 前端入参解析出完整的 LaunchConfig。

mod assembly;
mod auth;
mod connection;
mod java;
mod memory;

use crate::minecraft::launch::LaunchConfig;
use crate::minecraft::version::setup::VersionSetup;
use crate::state::{resolve_game_dir, AppState};

/// 构建启动配置。
///
/// 保持原有参数、启动参数顺序、认证校验和错误兜底行为不变；具体职责由子模块处理。
#[allow(clippy::too_many_arguments)]
pub(super) async fn build_launch_config(
    state: &AppState,
    app_handle: &tauri::AppHandle,
    version_id: &str,
    java_path: Option<String>,
    username: String,
    uuid: String,
    login_type: Option<String>,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<String>,
    server_port: Option<u32>,
    extra_jvm_args_override: Option<Vec<String>>,
) -> LaunchConfig {
    let config = state.config.lock().await;
    let game_dir = resolve_game_dir(&config.game_dir);
    let version_dir = game_dir.join("versions").join(version_id);
    let setup = VersionSetup::load_or_create(&version_dir, version_id);

    let (java_path, java_mode, java_version_min, java_version_max) =
        java::resolve_java(&setup, java_path);
    let (server_address, server_port) =
        connection::resolve_server(&setup, server_address, server_port);
    let (extra_jvm_args, extra_game_args, pre_launch_cmd) =
        connection::resolve_extra_args(&setup, extra_jvm_args_override);
    let (min_memory, max_memory) = memory::resolve_memory(&setup, &config);
    let auth_info = auth::resolve_auth(state, username, uuid, login_type).await;
    let auth_info = auth::apply_offline_skin(state, auth_info, &game_dir, version_id).await;

    assembly::assemble_launch_config(
        app_handle,
        game_dir,
        version_id,
        &setup,
        &config,
        auth_info,
        min_memory,
        max_memory,
        window_width,
        window_height,
        server_address,
        server_port,
        java_path,
        java_mode,
        java_version_min,
        java_version_max,
        extra_jvm_args,
        extra_game_args,
        pre_launch_cmd,
    )
}
