//! 启动脚本导出（.bat 批处理文件）

use crate::minecraft::version::setup::VersionSetup;
use crate::state::AppState;
use crate::{log_error, log_info, log_warn};
use tauri::State;

use super::sanitize_version_id;
use super::list::resolve_isolation_mode;

/// 导出启动脚本（.bat 批处理文件，使用绝对路径 Java + 版权信息）
///
/// 安全修复：移除 access_token 参数，改为后端从 auth_storage 获取 token
/// 前端只传 username 和 uuid，避免 token 在 IPC 请求体中明文传输
#[tauri::command]
pub async fn export_launch_script(
    state: State<'_, AppState>,
    version_id: String,
    username: String,
    uuid: String,
    login_type: Option<String>,
    java_path: Option<String>,
    save_path: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Exporting launch script for version: {}", version_id);
    log_warn!("Exporting launch script to: {}", save_path);

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let global_isolation_mode = config.isolation_mode;

    // 读取版本独立设置（setup.ini）
    let version_dir = game_dir.join("versions").join(&version_id);
    let setup = VersionSetup::load_or_create(&version_dir, &version_id);

    // 内存：版本独立设置 > 全局
    let min_memory;
    let max_memory;
    match setup.memory_mode.as_deref().filter(|s| !s.is_empty()) {
        Some("auto") => {
            let (s_min, s_max) = crate::minecraft::system::suggest_memory();
            min_memory = s_min;
            max_memory = s_max;
        }
        Some("custom") => {
            max_memory = setup.max_memory.unwrap_or(config.max_memory);
            min_memory = setup.min_memory.unwrap_or_else(|| max_memory / 2);
        }
        _ => {
            min_memory = config.min_memory;
            max_memory = config.max_memory;
        }
    }
    drop(config);

    // 版本独立隔离设置覆盖全局
    let isolation_mode = resolve_isolation_mode(&game_dir, &version_id, global_isolation_mode);

    // Java 路径：前端传入 > custom 模式下的版本独立 > 自动检测
    // 注意：脚本导出仅支持 custom 模式的显式路径，auto_version/folder 模式按自动选择处理
    let resolved_java = java_path.or_else(|| {
        let mode = setup.java_mode.as_deref().unwrap_or("").trim();
        if mode.eq_ignore_ascii_case("custom") {
            setup.java_path.clone().filter(|s| !s.is_empty())
        } else {
            None
        }
    });

    // 解析 Java 路径：优先用户指定 → 否则按 MC 版本自动检测 → 都失败则报错
    let java_path_buf = resolve_java_path(&game_dir, &version_id, resolved_java.as_deref())
        .await
        .map_err(|e| {
            log_error!("Failed to resolve Java path for script: {}", e);
            e
        })?;
    let java_str = java_path_buf.to_string_lossy().replace('/', "\\");
    log_info!("Script will use Java: {}", java_str);

    // 服务器：从版本独立 server_enter 解析（"IP:Port" 格式）
    let (server_addr, server_port) = setup
        .server_enter
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(super::launch::parse_server_enter)
        .unwrap_or((None, None));

    // 额外参数：按空白拆分
    let split_args = |s: &Option<String>| -> Vec<String> {
        s.as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    };
    let extra_jvm_args = split_args(&setup.advance_jvm_args);
    let extra_game_args = split_args(&setup.advance_game_args);

    // 构建认证信息
    // 安全修复：导出脚本时不使用真实 token（game_args 已脱敏为 ***）
    // 后端从 auth_storage 获取 token 仅用于构建参数结构，实际 token 不会写入脚本
    let (real_access_token, real_client_token) = {
        match state.auth_storage.load().await {
            Ok(auth_state) => {
                if let Some(ref current) = auth_state.current_user {
                    if current.uuid == uuid {
                        (current.access_token.clone(), current.client_token.clone())
                    } else {
                        (String::new(), String::new())
                    }
                } else {
                    (String::new(), String::new())
                }
            }
            Err(_) => (String::new(), String::new()),
        }
    };
    let auth_info = crate::minecraft::launch::AuthInfo {
        username: username.clone(),
        uuid,
        access_token: real_access_token,
        client_token: real_client_token,
        login_type: login_type.unwrap_or_else(|| "Legacy".to_string()),
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
    )
    .map_err(|e| {
        log_error!("Failed to build launch arguments: {}", e);
        e.to_string()
    })?;

    // 生成 .bat 脚本内容（绝对路径 Java + 版权信息 + 启动提示）
    let game_dir_display = launch_args.game_dir.replace('/', "\\");
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut script = String::new();
    script.push_str("@echo off\n");
    // 切换控制台代码页到 GBK（936），与本文件编码一致，避免中文乱码
    // 较新的中文 Windows 11 可能默认 codepage 为 65001（UTF-8），会导致 GBK 编码的中文显示乱码
    script.push_str("chcp 936 >nul\n");
    script.push_str("@REM [!] 警告：此文件包含 Minecraft 访问令牌，请勿分享或上传到公共平台\n");
    script.push_str(&format!("title MoLaunch - {}\n", version_id));
    script.push('\n');
    // 版权信息头
    script.push_str("REM ============================================================\n");
    script.push_str("REM  MoLaunch 启动脚本\n");
    script.push_str(&format!("REM  版本: {}\n", version_id));
    script.push_str(&format!("REM  生成时间: {}\n", timestamp));
    script.push_str("REM  Copyright (c) MoLaunch. All rights reserved.\n");
    script.push_str("REM ============================================================\n");
    script.push('\n');
    // 启动提示信息
    script.push_str("echo.\n");
    script.push_str("echo  ============================================================\n");
    script.push_str("echo    MoLaunch 启动器 - 独立启动脚本\n");
    script.push_str(&format!("echo    版本: {}\n", version_id));
    script.push_str(&format!("echo    用户: {}\n", username));
    script.push_str(&format!("echo    Java: {}\n", java_str));
    script.push_str(&format!("echo    游戏目录: {}\n", game_dir_display));
    script.push_str("echo  ============================================================\n");
    script.push_str("echo.\n");
    script.push_str("echo  正在启动游戏，请稍候...\n");
    script.push_str("echo.\n");
    script.push('\n');
    // 切换到游戏目录
    script.push_str(&format!("cd /D \"{}\"\n", game_dir_display));
    script.push('\n');
    // 启动前命令（advance_run_cmd，语法与 cmd 一致）
    if let Some(ref cmd) = setup.advance_run_cmd {
        if !cmd.is_empty() {
            script.push_str("REM 启动前命令\n");
            script.push_str(cmd);
            script.push_str("\n\n");
        }
    }
    // Java 启动命令（使用绝对路径，不依赖系统 PATH）
    // 安全修复：对 game_args 中的敏感参数值脱敏，避免 access_token 明文写入 .bat 文件
    // 用户需手动填入 token，或在脚本中使用环境变量
    let sanitized_game_args = crate::minecraft::launch::sanitize_args_for_log(&launch_args.game_args);
    let has_redacted = sanitized_game_args.iter().any(|a| a == "***");
    if has_redacted {
        script.push_str("REM [!] 警告：以下参数中的 accessToken / uuid 等敏感信息已脱敏为 ***\n");
        script.push_str("REM [!] 请手动替换 *** 为你的实际 token，或通过环境变量传入\n");
    }
    script.push_str(&format!(
        "\"{}\" {} {} {}\n",
        java_str,
        launch_args.jvm_args.join(" "),
        launch_args.main_class,
        sanitized_game_args.join(" ")
    ));
    script.push('\n');
    // 退出提示
    script.push_str("echo.\n");
    script.push_str("echo  ============================================================\n");
    script.push_str("echo    游戏已退出\n");
    script.push_str("echo  ============================================================\n");
    script.push_str("pause\n");

    // Windows 批处理文件必须使用 CRLF 换行符（cmd.exe 按 CRLF 识别行边界）
    // 若用 LF，cmd 会把多行内容拼成一行，导致行中间的单词被当成命令执行
    let script = script.replace('\n', "\r\n");

    // 用 GBK 编码写入文件（中文 Windows cmd 默认按 ANSI/GBK 解析批处理文件）
    // 若用 UTF-8 写入，中文字节会被错误拆分成命令，导致 "xxx 不是内部或外部命令" 错误
    let (bytes, _, had_errors) = encoding_rs::GBK.encode(&script);
    if had_errors {
        log_warn!("[ExportScript] Some characters could not be encoded to GBK");
    }
    std::fs::write(&save_path, &bytes).map_err(|e| {
        log_error!("Failed to write script: {}", e);
        e.to_string()
    })?;

    // 尝试限制文件权限为当前用户（防止 access_token 被其他用户读取）
    crate::minecraft::system::shell::restrict_file_permissions(std::path::Path::new(&save_path));

    log_info!("Launch script exported to: {}", save_path);
    Ok(())
}

/// 解析脚本使用的 Java 路径（优先用户指定 → 否则按 MC 版本自动检测）
/// 用户指定路径会校验版本兼容性，不兼容时返回错误
async fn resolve_java_path(
    game_dir: &std::path::Path,
    version_id: &str,
    user_java_path: Option<&str>,
) -> Result<std::path::PathBuf, String> {
    // 获取版本目录和 MC 版本号 + 加载器
    let version_dir = game_dir.join("versions").join(version_id);
    let (mc_version, loader) = crate::minecraft::version::setup::detect_version_and_loader(
        &version_dir,
        version_id,
    );

    // 1. 优先使用用户指定的 Java 路径（校验版本兼容性）
    if let Some(path) = user_java_path {
        if !path.is_empty() {
            let p = std::path::PathBuf::from(path);
            if p.exists() {
                // 校验版本兼容性
                if let Some(java_ver) = crate::minecraft::java::detect_java_version(path) {
                    if let Err((_cur, cur_min, cur_max)) =
                        crate::minecraft::java_selector::check_java_compatible(
                            java_ver,
                            &mc_version,
                            loader.as_deref(),
                        )
                    {
                        let req_desc = crate::minecraft::java_selector::describe_java_requirement(cur_min, cur_max);
                        return Err(format!(
                            "Java 版本不兼容：当前版本{}，{}。\n请前往 版本设置 → 游戏 Java 重新选择，或切换为「自动选择」",
                            java_ver, req_desc
                        ));
                    }
                }
                return Ok(p);
            }
            log_error!("User-specified Java not found: {}", path);
        }
    }

    log_info!(
        "[ExportScript] Auto-detecting Java for MC {} (loader: {:?})...",
        mc_version,
        loader
    );

    // 2. 搜索系统 Java
    let java_list = tokio::task::spawn_blocking(crate::minecraft::java::search_java)
        .await
        .map_err(|e| format!("Java 搜索失败: {}", e))?;

    if java_list.is_empty() {
        return Err("未找到任何已安装的 Java，请先安装 Java 或在设置中指定 Java 路径".to_string());
    }

    // 3. 按版本号选择最佳 Java（支持加载器约束）
    let selected = crate::minecraft::java_selector::select_best_java_with_loader(
        &mc_version,
        loader.as_deref(),
        &java_list,
        None,
    )
    .ok_or_else(|| {
        let (min_req, max_req) = crate::minecraft::java_selector::get_java_version_range(
            &mc_version,
            loader.as_deref(),
        );
        format!(
            "未找到满足 MC {} 要求的 Java (需要 Java {}-{})",
            mc_version,
            min_req.unwrap_or(0),
            max_req.map(|m| m.to_string()).unwrap_or("∞".to_string())
        )
    })?;

    Ok(std::path::PathBuf::from(&selected))
}
