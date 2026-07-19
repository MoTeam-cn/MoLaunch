//! 游戏参数构建
//!
//! 解析版本 JSON 的 arguments.game（或旧版 minecraftArguments），
//! 替换占位符并补充窗口、服务器、用户额外参数。

use std::path::Path;

use super::AuthInfo;

/// Build game arguments
pub(super) fn build_game_args(
    json: &serde_json::Value,
    game_dir: &Path,
    version_id: &str,
    assets_dir: &str,
    asset_index: &str,
    auth_info: &AuthInfo,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<&str>,
    server_port: Option<u32>,
    extra_game_args: &[String],
) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();

    if let Some(game_args) = json["arguments"]["game"].as_array() {
        for arg in game_args {
            let (value, rules) = if let Some(s) = arg.as_str() {
                (s.to_string(), None)
            } else if let Some(obj) = arg.as_object() {
                let value = obj.get("value").and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(arr) = v.as_array() {
                        Some(arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>().join(" "))
                    } else {
                        None
                    }
                });
                let rules = obj.get("rules").and_then(|r| r.as_array()).map(|a| a.clone());
                match value {
                    Some(v) => (v, rules),
                    None => continue,
                }
            } else {
                continue;
            };

            if !crate::minecraft::version::libraries::check_rules(&rules) {
                continue;
            }

            args.push(value);
        }
    } else if let Some(mc_args) = json["minecraftArguments"].as_str() {
        for arg in mc_args.split(' ') {
            args.push(arg.to_string());
        }
    }

    // 如果 arguments.game 未提供标准 Minecraft 客户端参数（如 Forge 26.2 自包含 JSON），
    // 自动补充必需参数（参考 Mojang 原版 JSON 的 arguments.game 模板）
    if !args.iter().any(|a| a == "--accessToken") {
        let mut std_args = vec![
            "--username".to_string(),
            "${auth_player_name}".to_string(),
            "--version".to_string(),
            "${version_name}".to_string(),
            "--gameDir".to_string(),
            "${game_directory}".to_string(),
            "--assetsDir".to_string(),
            "${assets_root}".to_string(),
            "--assetIndex".to_string(),
            "${assets_index_name}".to_string(),
            "--uuid".to_string(),
            "${auth_uuid}".to_string(),
            "--accessToken".to_string(),
            "${auth_access_token}".to_string(),
            "--userType".to_string(),
            "${user_type}".to_string(),
            "--versionType".to_string(),
            "${version_type}".to_string(),
        ];
        std_args.extend(args);
        args = std_args;
    }

    let mut final_args = Vec::new();
    for arg in args {
        let replaced = arg
            .replace("${auth_player_name}", &auth_info.username)
            .replace("${auth_session}", &auth_info.access_token)
            .replace("${auth_uuid}", &auth_info.uuid)
            .replace("${auth_access_token}", &auth_info.access_token)
            .replace("${auth_client_token}", &auth_info.client_token)
            .replace("${user_type}", &auth_info.login_type)
            .replace("${version_name}", version_id)
            .replace("${game_directory}", &game_dir.to_string_lossy())
            .replace("${game_assets}", assets_dir)
            .replace("${assets_root}", assets_dir)
            .replace("${assets_index_name}", asset_index)
            .replace("${user_properties}", "{}")
            .replace("${version_type}", "MoLaunch");
        final_args.push(replaced);
    }

    if let (Some(width), Some(height)) = (window_width, window_height) {
        final_args.push("--width".to_string());
        final_args.push(width.to_string());
        final_args.push("--height".to_string());
        final_args.push(height.to_string());
    }

    if let Some(server) = server_address {
        final_args.push("--server".to_string());
        final_args.push(server.to_string());
        if let Some(port) = server_port {
            final_args.push("--port".to_string());
            final_args.push(port.to_string());
        }
    }

    // 用户额外游戏参数（参考 PCL2 的 AdvanceGame）
    final_args.extend(extra_game_args.iter().cloned());

    Ok(final_args)
}
