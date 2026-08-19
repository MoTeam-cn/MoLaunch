//! 启动参数预览（组装 JVM 参数但不启动）

use serde::Serialize;

use crate::log_info;
use crate::minecraft::launch::LaunchPipeline;
use crate::state::AppState;

use super::super::sanitize_version_id;
use super::build_config::build_launch_config;

/// 启动参数预览结果（token 脱敏，不返回 access_token / client_token）
#[derive(Debug, Serialize)]
pub struct LaunchArgsPreview {
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub main_class: String,
    pub classpath: String,
    pub version_id: String,
    pub game_dir: String,
    pub assets_dir: String,
    pub asset_index: String,
    pub username: String,
    pub uuid: String,
    pub login_type: String,
    pub server_url: Option<String>,
    pub xuid: String,
    /// 实际使用的 Java 路径
    pub java_path: String,
}

/// 预览启动参数：复用 build_launch_config + 流水线 detect_java / build_arguments，
/// 仅组装参数不启动游戏。token 不返回（脱敏），避免泄露到前端。
#[allow(clippy::too_many_arguments)]
pub async fn preview_launch_args(
    state: &AppState,
    app_handle: &tauri::AppHandle,
    version_id: String,
    java_path: Option<String>,
    username: String,
    uuid: String,
    login_type: Option<String>,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<String>,
    server_port: Option<u32>,
    extra_jvm_args: Option<Vec<String>>,
) -> Result<LaunchArgsPreview, String> {
    sanitize_version_id(&version_id)?;
    log_info!("Previewing launch args for version: {}", version_id);

    let launch_config = build_launch_config(
        state,
        app_handle,
        &version_id,
        java_path,
        username,
        uuid,
        login_type,
        window_width,
        window_height,
        server_address,
        server_port,
        extra_jvm_args,
    )
    .await;

    let pipeline = LaunchPipeline::new(launch_config);
    let java_path = pipeline.detect_java().await.map_err(|e| e.message)?;
    let args = pipeline
        .build_arguments(&java_path)
        .await
        .map_err(|e| e.message)?;

    Ok(LaunchArgsPreview {
        jvm_args: args.jvm_args,
        game_args: args.game_args,
        main_class: args.main_class,
        classpath: args.classpath,
        version_id: args.version_id,
        game_dir: args.game_dir,
        assets_dir: args.assets_dir,
        asset_index: args.asset_index,
        username: args.auth_info.username,
        uuid: args.auth_info.uuid,
        login_type: args.auth_info.login_type,
        server_url: args.auth_info.server_url,
        xuid: args.auth_info.xuid,
        java_path: java_path.to_string_lossy().to_string(),
    })
}
