//! 启动脚本导出逻辑实现（export_launch_script，原聚合入口 mod.rs 中的实现）
//!
//! 构建 .bat 脚本内容并写入文件：Java 路径解析、内存/隔离模式、认证信息构建、
//! 启动参数生成均在此编排，具体脚本内容与 Java 解析见 `content` / `resolve_java`。

use crate::minecraft::version::setup::VersionSetup;
use crate::state::AppState;
use crate::{log_error, log_info, log_warn};

use super::super::list::resolve_isolation_mode;
use super::super::sanitize_version_id;

/// 导出启动脚本（.bat 批处理文件，使用绝对路径 Java + 版权信息）
///
/// 安全修复：移除 access_token 参数，改为后端从 auth_storage 获取 token
/// 前端只传 username 和 uuid，避免 token 在 IPC 请求体中明文传输
///
/// 注：原为独立 `#[tauri::command]`，已聚合为 `version_launch_manager` IPC 入口，
/// 由 `super::super::launch::manager::dispatch` 反序列化参数后调用。
pub async fn export_launch_script(
    state: &AppState,
    version_id: String,
    username: String,
    uuid: String,
    login_type: Option<String>,
    java_path: Option<String>,
    save_path: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    // 按当前系统选择脚本格式（Windows .bat / macOS、Linux .sh）
    let is_windows = cfg!(target_os = "windows");
    log_info!(
        "Exporting {} launch script for version: {}",
        if is_windows { "Windows (.bat)" } else { "Unix (.sh)" },
        version_id
    );

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let config = state.config.lock().await;
    let global_isolation_mode = config.isolation_mode;

    // 读取版本独立设置（setup.ini）
    let version_dir = game_dir.join("versions").join(&version_id);
    let setup = VersionSetup::load_or_create(&version_dir, &version_id);

    // 内存：版本独立设置 > 全局
    let min_memory;
    let max_memory;
    match setup.java.memory_mode.as_deref().filter(|s| !s.is_empty()) {
        Some("auto") => {
            let (s_min, s_max) = crate::minecraft::system::suggest_memory();
            min_memory = s_min;
            max_memory = s_max;
        }
        Some("custom") => {
            max_memory = setup.java.max_memory.unwrap_or(config.memory.max);
            min_memory = setup.java.min_memory.unwrap_or(max_memory / 2);
        }
        _ => {
            min_memory = config.memory.min;
            max_memory = config.memory.max;
        }
    }
    drop(config);

    // 版本独立隔离设置覆盖全局
    let isolation_mode = resolve_isolation_mode(&game_dir, &version_id, global_isolation_mode);

    // Java 路径：前端传入 > custom 模式下的版本独立 > 自动检测
    // 注意：脚本导出仅支持 custom 模式的显式路径，auto_version/folder 模式按自动选择处理
    let resolved_java = java_path.or_else(|| {
        let mode = setup.java.java_mode.as_deref().unwrap_or("").trim();
        if mode.eq_ignore_ascii_case("custom") {
            setup.java.java_path.clone().filter(|s| !s.is_empty())
        } else {
            None
        }
    });

    // 解析 Java 路径：优先用户指定 → 否则按 MC 版本自动检测 → 都失败则报错
    let java_path_buf =
        super::resolve_java::resolve_java_path(&game_dir, &version_id, resolved_java.as_deref())
            .await
            .map_err(|e| {
                log_error!("Failed to resolve Java path for script: {}", e);
                e
            })?;
    let java_str = java_path_buf.to_string_lossy().to_string();
    log_info!("Script will use Java: {}", java_str);

    // 服务器：从版本独立 server_enter 解析（"IP:Port" 格式）
    let (server_addr, server_port) = setup
        .display
        .server_enter
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(super::super::launch::parse_server_enter)
        .unwrap_or((None, None));

    // 额外参数：按空白拆分
    let split_args = |s: &Option<String>| -> Vec<String> {
        s.as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    };
    let extra_jvm_args = split_args(&setup.advanced.jvm_args);
    let extra_game_args = split_args(&setup.advanced.game_args);

    // 构建认证信息
    // 从 auth_storage 获取当前用户的真实 token，写入脚本后可直接启动；
    // 安全提示已写入脚本头部，文件权限限制为当前用户
    let (real_access_token, real_client_token, real_server_url) = {
        match state.auth_storage.load().await {
            Ok(auth_state) => {
                if let Some(ref current) = auth_state.current_user {
                    if current.uuid == uuid {
                        (
                            current.access_token.clone(),
                            current.client_token.clone(),
                            current.server_url.clone(),
                        )
                    } else {
                        (String::new(), String::new(), None)
                    }
                } else {
                    (String::new(), String::new(), None)
                }
            }
            Err(_) => (String::new(), String::new(), None),
        }
    };
    let login_type_str = login_type.clone().unwrap_or_else(|| "Legacy".to_string());
    let is_legacy = login_type_str == "Legacy";
    let auth_info = crate::minecraft::launch::AuthInfo {
        username: username.clone(),
        uuid,
        access_token: real_access_token,
        client_token: real_client_token,
        login_type: login_type_str,
        server_url: real_server_url,
    };

    // 离线账号皮肤：与 launch.rs 一致，调整 UUID 匹配皮肤变体
    let auth_info = if is_legacy {
        match state.auth_storage.load().await {
            Ok(auth_state) => {
                if let Some(acc) = auth_state
                    .offline_accounts
                    .iter()
                    .find(|a| a.uuid == auth_info.uuid)
                {
                    if let Some(ref skin_name) = acc.skin {
                        let slim = matches!(
                            skin_name.as_str(),
                            "Alex" | "Ari" | "Efe" | "Makena" | "Noor" | "Sunny" | "Zuri"
                        );
                        let adjusted_uuid = crate::minecraft::auth::adjust_uuid_for_skin_variant(
                            &auth_info.uuid,
                            slim,
                        );
                        crate::minecraft::launch::AuthInfo {
                            uuid: adjusted_uuid,
                            ..auth_info
                        }
                    } else {
                        auth_info
                    }
                } else {
                    auth_info
                }
            }
            Err(_) => auth_info,
        }
    } else {
        auth_info
    };

    // 构建启动参数（注意：build_launch_arguments 会触发 set_game_language 副作用，此处可接受）
    let launch_args = crate::minecraft::launch::build_launch_arguments(
        &game_dir,
        &version_id,
        &java_path_buf,
        &auth_info,
        min_memory,
        max_memory,
        None,
        None,
        server_addr.as_deref(),
        server_port,
        isolation_mode,
        &extra_jvm_args,
        &extra_game_args,
        false, // 导出脚本时不启用 JLW
        false, // 导出脚本时不启用 LUA
        None,  // 导出脚本时不传 custom_info
        None,  // 导出脚本时不设置游戏语言
    )
    .map_err(|e| {
        log_error!("Failed to build launch arguments: {}", e);
        e.to_string()
    })?;

    // 生成启动脚本内容 + 写入文件
    // 补充扩展名：部分系统（如 Linux）文件对话框可能不自动追加，确保脚本格式正确
    let save_path = if is_windows {
        if save_path.to_ascii_lowercase().ends_with(".bat") {
            save_path
        } else {
            format!("{}.bat", save_path)
        }
    } else if save_path.ends_with(".sh") {
        save_path
    } else {
        format!("{}.sh", save_path)
    };
    log_warn!("Exporting launch script to: {}", save_path);

    let game_dir_display = launch_args.game_dir.clone();
    let script_info = super::content::ScriptLaunchInfo {
        version_id: &version_id,
        username: &username,
        java_str: &java_str,
        game_dir_display: &game_dir_display,
        jvm_args: &launch_args.jvm_args,
        main_class: &launch_args.main_class,
        game_args: &launch_args.game_args,
        pre_launch_cmd: setup.advanced.run_cmd.as_ref(),
    };
    let script = super::content::build_script_content(&script_info, is_windows);
    super::content::write_script_file(&script, &save_path, is_windows)?;

    log_info!("Launch script exported to: {}", save_path);
    Ok(())
}
