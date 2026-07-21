//! JVM 参数构建
//!
//! 构建逻辑：
//! - LUA：仅当版本库列表包含 org.lwjgl:lwjgl:3.4.1 时注入 -javaagent
//! - JLW：仅当非 GBK 编码、路径非纯 ASCII、且无自定义 -javaagent 时启用
//!   - Java 9+ 添加 --add-exports cpw.mods.bootstraplauncher
//!   - 添加 -Doolloo.jlw.tmpdir={pure_directory}
//!   - 末尾添加 -jar java-wrapper.jar（覆盖 mainClass 作为入口）
//!
//! `build_jvm_args` 按关注点拆分为多个 helper：
//! - `add_lua_args`:       LUA（LWJGL Unsafe Agent）注入
//! - `add_gc_args`:        根据 Java 主版本号选择 GC 策略
//! - `add_json_jvm_args`:  解析版本 JSON 的 arguments.jvm
//! - `add_jlw_args`:       JLW（Java Launch Wrapper）注入

use std::path::Path;

use super::embedded::{has_library, resolve_embedded_jar};

/// Build JVM arguments
pub(super) fn build_jvm_args(
    game_dir: &Path,
    version_id: &str,
    classpath: &str,
    min_memory: u32,
    max_memory: u32,
    java_path: &Path,
    extra_jvm_args: &[String],
    json: &serde_json::Value,
    disable_jlw: bool,
    disable_lua: bool,
) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();

    // 检测 Java 主版本号（用于决定 GC 策略和 JLW 的 --add-exports）
    let java_major = crate::minecraft::java::detect_java_version(&java_path.to_string_lossy());

    // ===== LUA（LWJGL Unsafe Agent）=====
    add_lua_args(&mut args, json, disable_lua);

    args.push(format!("-Xms{}M", min_memory));
    args.push(format!("-Xmx{}M", max_memory));

    // ===== GC 策略 =====
    add_gc_args(&mut args, java_major);

    // ===== 版本 JSON 的 arguments.jvm（必需 JVM 参数）=====
    add_json_jvm_args(&mut args, json, game_dir, version_id);

    // 用户额外 JVM 参数（版本独立 > 全局）
    args.extend(extra_jvm_args.iter().cloned());

    args.push("-cp".to_string());
    args.push(classpath.to_string());

    let natives_dir = game_dir
        .join("versions")
        .join(version_id)
        .join(format!("{}-natives", version_id));
    args.push(format!(
        "-Djava.library.path={}",
        natives_dir.to_string_lossy()
    ));

    // ===== JLW（Java Launch Wrapper）=====
    add_jlw_args(&mut args, game_dir, java_major, extra_jvm_args, disable_jlw);

    Ok(args)
}

/// LUA（LWJGL Unsafe Agent）
/// 仅当库列表包含 org.lwjgl:lwjgl:3.4.1 且未禁用时注入
fn add_lua_args(args: &mut Vec<String>, json: &serde_json::Value, disable_lua: bool) {
    let use_lua = !disable_lua && has_library(json, "org.lwjgl:lwjgl:3.4.1");
    if use_lua {
        if let Some(agent_path) =
            resolve_embedded_jar("lwjgl-unsafe-agent.jar", "launch/lwjgl-unsafe-agent.jar")
        {
            args.push(format!("-javaagent:{}", agent_path.to_string_lossy()));
        } else {
            crate::log_warn!("[Launch] lwjgl-unsafe-agent.jar 释放失败，跳过 LUA");
        }
    }
    crate::log_info!("[Launch] 使用 LUA：{}", use_lua);
}

/// GC 策略：Java 21+ 使用 ZGC + ZGenerational，Java 15+ 使用 ZGC，否则 G1GC
fn add_gc_args(args: &mut Vec<String>, java_major: Option<u32>) {
    if let Some(version) = java_major {
        if version >= 21 {
            args.push("-XX:+UseZGC".to_string());
            args.push("-XX:+ZGenerational".to_string());
        } else if version >= 15 {
            args.push("-XX:+UseZGC".to_string());
        } else {
            args.push("-XX:+UseG1GC".to_string());
        }
    }
}

/// 解析版本 JSON 的 arguments.jvm（必需 JVM 参数）
/// 跳过 ${classpath} 和 ${natives_directory}（由调用方单独处理）
fn add_json_jvm_args(
    args: &mut Vec<String>,
    json: &serde_json::Value,
    game_dir: &Path,
    version_id: &str,
) {
    let libraries_dir = game_dir.join("libraries");
    let libraries_dir_str = libraries_dir.to_string_lossy().replace('/', "\\");
    if let Some(jvm_args_json) = json["arguments"]["jvm"].as_array() {
        for arg in jvm_args_json {
            let (value, rules) = if let Some(s) = arg.as_str() {
                (s.to_string(), None)
            } else if let Some(obj) = arg.as_object() {
                let value = obj.get("value").and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(arr) = v.as_array() {
                        Some(
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect::<Vec<_>>()
                                .join(" "),
                        )
                    } else {
                        None
                    }
                });
                let rules = obj
                    .get("rules")
                    .and_then(|r| r.as_array())
                    .map(|a| a.clone());
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

            if value.contains("${classpath}") || value.contains("${natives_directory}") {
                continue;
            }

            let value = value
                .replace("${library_directory}", &libraries_dir_str)
                .replace("${classpath_separator}", ";")
                .replace("${version_name}", version_id);

            args.push(value);
        }
    }
}

/// JLW（Java Launch Wrapper）
/// - 仅当未禁用、非 GBK 编码、路径非纯 ASCII 时触发（仅在该环境下才会触发 JDK-8272352 Bug）
/// - 若用户自定义参数含 -javaagent 则禁用 JLW（冲突会导致崩溃）
/// - Java 9+ 添加 --add-exports cpw.mods.bootstraplauncher
/// - 添加 -Doolloo.jlw.tmpdir={pure_directory}（不以 \ 结尾）
/// - 末尾添加 -jar java-wrapper.jar（作为 JVM 入口，接收原 mainClass 作为参数）
fn add_jlw_args(
    args: &mut Vec<String>,
    game_dir: &Path,
    java_major: Option<u32>,
    extra_jvm_args: &[String],
    disable_jlw: bool,
) {
    let is_gbk = is_gbk_encoding();
    let game_dir_str = game_dir.to_string_lossy();
    let is_ascii_only = game_dir_str.chars().all(|c| c.is_ascii());
    let has_custom_javaagent = extra_jvm_args.iter().any(|a| a.contains("-javaagent"));

    let use_jlw = !disable_jlw && !is_gbk && !is_ascii_only && !has_custom_javaagent;

    if use_jlw {
        if let Some(major) = java_major {
            if major >= 9 {
                args.push("--add-exports".to_string());
                args.push(
                    "cpw.mods.bootstraplauncher/cpw.mods.bootstraplauncher=ALL-UNNAMED".to_string(),
                );
            }
        }
        // pure_directory：游戏目录路径（不以 \ 结尾）
        let pure_dir = game_dir_str.trim_end_matches('\\').trim_end_matches('/');
        args.push(format!("-Doolloo.jlw.tmpdir={}", pure_dir));
        // -jar java-wrapper.jar 必须放在 JVM args 末尾，作为入口接管 mainClass
        if let Some(wrapper_path) =
            resolve_embedded_jar("java-wrapper.jar", "launch/java-wrapper.jar")
        {
            args.push("-jar".to_string());
            args.push(wrapper_path.to_string_lossy().to_string());
        } else {
            crate::log_warn!("[Launch] java-wrapper.jar 释放失败，JLW 降级为直接启动");
        }
    } else if has_custom_javaagent && !disable_jlw {
        crate::log_warn!("[Launch] 检测到自定义 -javaagent，已禁用 JLW 以避免冲突");
    }
    crate::log_info!("[Launch] 使用 JLW：{}", use_jlw);
}

/// 检测系统是否使用 GBK 编码（Windows ANSI 代码页 936）
#[cfg(target_os = "windows")]
fn is_gbk_encoding() -> bool {
    // 通过注册表读取系统 ANSI 代码页：HKLM\SYSTEM\CurrentControlSet\Control\Nls\CodePage::ACP
    // 936 = GBK，返回 true；其他值返回 false
    use winreg::enums::*;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\Nls\\CodePage") {
        if let Ok(acp) = key.get_value::<String, _>("ACP") {
            return acp == "936";
        }
    }
    // 读取失败时默认非 GBK（避免误触发 JLW）
    false
}

#[cfg(not(target_os = "windows"))]
fn is_gbk_encoding() -> bool {
    false
}
