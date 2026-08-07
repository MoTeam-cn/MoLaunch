//! 启动配置中的隔离模式与最终配置组装。

use crate::minecraft::launch::LaunchConfig;
use crate::minecraft::version::setup::VersionSetup;

pub(super) fn resolve_isolation_mode(
    game_dir: &std::path::Path,
    version_id: &str,
    global_mode: u32,
) -> u32 {
    super::super::super::list::resolve_isolation_mode(game_dir, version_id, global_mode)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn assemble_launch_config(
    app_handle: &tauri::AppHandle,
    game_dir: std::path::PathBuf,
    version_id: &str,
    setup: &VersionSetup,
    config: &crate::state::AppConfig,
    auth_info: crate::minecraft::launch::AuthInfo,
    min_memory: u32,
    max_memory: u32,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<String>,
    server_port: Option<u32>,
    java_path: Option<String>,
    java_mode: Option<String>,
    java_version_min: u32,
    java_version_max: u32,
    extra_jvm_args: Vec<String>,
    extra_game_args: Vec<String>,
    pre_launch_cmd: Option<String>,
) -> LaunchConfig {
    LaunchConfig {
        game_dir: game_dir.clone(),
        version_id: version_id.to_string(),
        auth_info,
        min_memory,
        max_memory,
        window_width,
        window_height,
        server_address,
        server_port,
        isolation_mode: resolve_isolation_mode(&game_dir, version_id, config.isolation_mode),
        java_path,
        java_mode,
        java_version_min,
        java_version_max,
        download_source: config.download.source.clone(),
        mirror_url: config.download.mirror_url.clone(),
        max_threads: config.download.max_threads,
        chunk_count: config.download.chunk_count,
        speed_limit: config.download.max_speed,
        extra_jvm_args,
        extra_game_args,
        pre_launch_cmd,
        disable_jlw: config.launch_advanced.disable_jlw
            || setup.advanced.disable_jlw.unwrap_or(false),
        disable_lua: config.launch_advanced.disable_lua
            || setup.advanced.disable_lua.unwrap_or(false),
        ignore_java_warning: setup.advanced.ignore_java_warning.unwrap_or(false),
        disable_assets_verify: setup.advanced.disable_assets_verify.unwrap_or(false),
        use_dedicated_gpu: config.launch_advanced.use_dedicated_gpu,
        custom_info: setup.display.custom_info.clone(),
        window_title: setup.display.window_title.clone(),
        game_language: super::super::resolve_game_language(&config.game_language, &config.language),
        app_handle: Some(app_handle.clone()),
    }
}
