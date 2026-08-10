//! 启动脚本内容构建与文件写入（Windows .bat / macOS、Linux .sh）

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

/// 生成启动脚本内容（按当前系统选择格式：Windows .bat，macOS/Linux .sh）
pub(super) fn build_script_content(info: &ScriptLaunchInfo<'_>, is_windows: bool) -> String {
    if is_windows {
        build_bat_content(info)
    } else {
        build_sh_content(info)
    }
}

/// 生成 Windows .bat 脚本内容（绝对路径 Java + 版权信息 + 启动提示）
fn build_bat_content(info: &ScriptLaunchInfo<'_>) -> String {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    // Windows 批处理内路径统一反斜杠
    let java_str = info.java_str.replace('/', "\\");
    let game_dir = info.game_dir_display.replace('/', "\\");

    let mut script = String::new();
    script.push_str("@echo off\n");
    // 切换控制台代码页到 GBK（936），与本文件编码一致，避免中文乱码
    // 较新的中文 Windows 11 可能默认 codepage 为 65001（UTF-8），会导致 GBK 编码的中文显示乱码
    script.push_str("chcp 936 >nul\n");
    script.push_str("@REM [!] 警告：此文件包含启动必需的真实 Minecraft 访问令牌与 UUID，请勿分享或上传到公共平台，失效后请重新导出\n");
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
    script.push_str(&format!("echo    Java: {}\n", java_str));
    script.push_str(&format!("echo    游戏目录: {}\n", game_dir));
    script.push_str("echo  ============================================================\n");
    script.push_str("echo.\n");
    script.push_str("echo  正在启动游戏，请稍候...\n");
    script.push_str("echo.\n");
    script.push('\n');
    // 切换到游戏目录
    script.push_str(&format!("cd /D \"{}\"\n", game_dir));
    script.push('\n');
    // 启动前命令（advance_run_cmd，语法与 cmd 一致）
    if let Some(cmd) = info.pre_launch_cmd {
        if !cmd.is_empty() {
            script.push_str("REM 启动前命令\n");
            script.push_str(cmd);
            script.push_str("\n\n");
        }
    }
    // Java 启动命令（使用绝对路径，不依赖系统 PATH）
    // 注意：game_args 中包含真实 access_token / uuid，脚本可直接启动；
    // 文件权限已限制为当前用户，且文件头部有分享警告，勿手动脱敏
    script.push_str(&format!(
        "\"{}\" {} {} {}\n",
        java_str,
        info.jvm_args.join(" "),
        info.main_class,
        info.game_args.join(" ")
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

/// 生成 macOS / Linux .sh 脚本内容（UTF-8 + shebang + 可直接执行）
fn build_sh_content(info: &ScriptLaunchInfo<'_>) -> String {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut script = String::new();
    script.push_str("#!/usr/bin/env bash\n");
    script.push_str("# [!] 警告：此文件包含启动必需的真实 Minecraft 访问令牌与 UUID，请勿分享或上传到公共平台，失效后请重新导出\n");
    script.push_str(&format!("# title MoLaunch - {}\n", info.version_id));
    script.push('\n');
    // 版权信息头
    script.push_str("# ============================================================\n");
    script.push_str("#  MoLaunch 启动脚本\n");
    script.push_str(&format!("#  版本: {}\n", info.version_id));
    script.push_str(&format!("#  生成时间: {}\n", timestamp));
    script.push_str("#  Copyright (c) MoLaunch. All rights reserved.\n");
    script.push_str("# ============================================================\n");
    script.push('\n');
    // 启动提示信息
    script.push_str("echo '============================================================'\n");
    script.push_str("echo '  MoLaunch 启动器 - 独立启动脚本'\n");
    script.push_str(&format!("echo '   版本: {}'\n", info.version_id));
    script.push_str(&format!("echo '   用户: {}'\n", info.username));
    script.push_str(&format!("echo '   Java: {}'\n", info.java_str));
    script.push_str(&format!("echo '   游戏目录: {}'\n", info.game_dir_display));
    script.push_str("echo '============================================================'\n");
    script.push_str("echo\n");
    script.push_str("echo '正在启动游戏，请稍候...'\n");
    script.push_str("echo\n");
    script.push('\n');
    // 切换到游戏目录
    script.push_str(&format!("cd \"{}\" || exit 1\n", info.game_dir_display));
    script.push('\n');
    // 启动前命令（advance_run_cmd，语法与 shell 一致）
    if let Some(cmd) = info.pre_launch_cmd {
        if !cmd.is_empty() {
            script.push_str("# 启动前命令\n");
            script.push_str(cmd);
            script.push_str("\n\n");
        }
    }
    // Java 启动命令（使用绝对路径，不依赖系统 PATH）
    // 注意：game_args 中包含真实 access_token / uuid，脚本可直接启动；
    // 文件权限已设为当前用户可执行，且文件头部有分享警告，勿手动脱敏
    script.push_str(&format!(
        "\"{}\" {} {} {}\n",
        info.java_str,
        info.jvm_args.join(" "),
        info.main_class,
        info.game_args.join(" ")
    ));
    script.push('\n');
    // 退出提示
    script.push_str("echo '============================================================'\n");
    script.push_str("echo '   游戏已退出'\n");
    script.push_str("echo '============================================================'\n");
    script.push_str("read -r -p '按回车键退出...' _\n");

    script
}

/// 写入脚本文件：Windows 用 GBK 编码（中文 Windows cmd 默认按 ANSI/GBK 解析批处理文件），
/// macOS/Linux 用 UTF-8 并添加执行权限
pub(super) fn write_script_file(
    script: &str,
    save_path: &str,
    is_windows: bool,
) -> Result<(), String> {
    if is_windows {
        // 若用 UTF-8 写入，中文字节会被错误拆分成命令，导致 "xxx 不是内部或外部命令" 错误
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
    } else {
        std::fs::write(save_path, script.as_bytes()).map_err(|e| {
            log_error!("Failed to write script: {}", e);
            e.to_string()
        })?;
        // 添加执行权限，支持 ./Run_xxx.sh 直接运行
        crate::minecraft::system::shell::make_executable(std::path::Path::new(save_path));
    }
    Ok(())
}
