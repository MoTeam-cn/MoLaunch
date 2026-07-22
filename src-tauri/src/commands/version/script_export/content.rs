//! .bat 脚本内容构建与文件写入

use crate::{log_error, log_warn};

/// 启动参数信息（供 build_script_content 使用，避免传递整个 LaunchArguments）
pub(super) struct ScriptLaunchInfo<'a> {
    pub version_id: &'a str,
    pub username: &'a str,
    pub java_str: &'a str,
    pub game_dir_display: &'a str,
    pub jvm_args: &'a [String],
    pub main_class: &'a str,
    pub game_args: &'a [String],
    pub pre_launch_cmd: Option<&'a String>,
}

/// 生成 .bat 脚本内容（绝对路径 Java + 版权信息 + 启动提示）
pub(super) fn build_script_content(info: &ScriptLaunchInfo<'_>) -> String {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut script = String::new();
    script.push_str("@echo off\n");
    // 切换控制台代码页到 GBK（936），与本文件编码一致，避免中文乱码
    // 较新的中文 Windows 11 可能默认 codepage 为 65001（UTF-8），会导致 GBK 编码的中文显示乱码
    script.push_str("chcp 936 >nul\n");
    script.push_str("@REM [!] 警告：此文件包含 Minecraft 访问令牌，请勿分享或上传到公共平台\n");
    script.push_str(&format!("title MoLaunch - {}\n", info.version_id));
    script.push('\n');
    // 版权信息头
    script.push_str("REM ============================================================\n");
    script.push_str("REM  MoLaunch 启动脚本\n");
    script.push_str(&format!("REM  版本: {}\n", info.version_id));
    script.push_str(&format!("REM  生成时间: {}\n", timestamp));
    script.push_str("REM  Copyright (c) MoLaunch. All rights reserved.\n");
    script.push_str("REM ============================================================\n");
    script.push('\n');
    // 启动提示信息
    script.push_str("echo.\n");
    script.push_str("echo  ============================================================\n");
    script.push_str("echo    MoLaunch 启动器 - 独立启动脚本\n");
    script.push_str(&format!("echo    版本: {}\n", info.version_id));
    script.push_str(&format!("echo    用户: {}\n", info.username));
    script.push_str(&format!("echo    Java: {}\n", info.java_str));
    script.push_str(&format!("echo    游戏目录: {}\n", info.game_dir_display));
    script.push_str("echo  ============================================================\n");
    script.push_str("echo.\n");
    script.push_str("echo  正在启动游戏，请稍候...\n");
    script.push_str("echo.\n");
    script.push('\n');
    // 切换到游戏目录
    script.push_str(&format!("cd /D \"{}\"\n", info.game_dir_display));
    script.push('\n');
    // 启动前命令（advance_run_cmd，语法与 cmd 一致）
    if let Some(ref cmd) = info.pre_launch_cmd {
        if !cmd.is_empty() {
            script.push_str("REM 启动前命令\n");
            script.push_str(cmd);
            script.push_str("\n\n");
        }
    }
    // Java 启动命令（使用绝对路径，不依赖系统 PATH）
    // 安全修复：对 game_args 中的敏感参数值脱敏，避免 access_token 明文写入 .bat 文件
    // 用户需手动填入 token，或在脚本中使用环境变量
    let sanitized_game_args = crate::minecraft::launch::sanitize_args_for_log(info.game_args);
    let has_redacted = sanitized_game_args.iter().any(|a| a == "***");
    if has_redacted {
        script.push_str("REM [!] 警告：以下参数中的 accessToken / uuid 等敏感信息已脱敏为 ***\n");
        script.push_str("REM [!] 请手动替换 *** 为你的实际 token，或通过环境变量传入\n");
    }
    script.push_str(&format!(
        "\"{}\" {} {} {}\n",
        info.java_str,
        info.jvm_args.join(" "),
        info.main_class,
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
    script.replace('\n', "\r\n")
}

/// 用 GBK 编码写入脚本文件（中文 Windows cmd 默认按 ANSI/GBK 解析批处理文件）
/// 若用 UTF-8 写入，中文字节会被错误拆分成命令，导致 "xxx 不是内部或外部命令" 错误
pub(super) fn write_script_file(script: &str, save_path: &str) -> Result<(), String> {
    let (bytes, _, had_errors) = encoding_rs::GBK.encode(script);
    if had_errors {
        log_warn!("[ExportScript] Some characters could not be encoded to GBK");
    }
    std::fs::write(save_path, &bytes).map_err(|e| {
        log_error!("Failed to write script: {}", e);
        e.to_string()
    })?;

    // 尝试限制文件权限为当前用户（防止 access_token 被其他用户读取）
    crate::minecraft::system::shell::restrict_file_permissions(std::path::Path::new(save_path));

    Ok(())
}
