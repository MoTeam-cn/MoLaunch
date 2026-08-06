//! JVM 参数拼装
//!
//! authlib/lua/json_jvm/jlw 各段参数构造。

use super::super::embedded::{has_library, resolve_embedded_jar};
use super::super::AuthInfo;
use super::rules::is_gbk_encoding;
use std::path::Path;

/// authlib-injector.jar 在缓存目录的相对路径
///
/// 由 `ensure_authlib_injector_jar`（阶段 3.3 实现）异步下载到此处。
/// `add_authlib_args` 同步读取，不存在则跳过注入并打印警告。
const AUTHLIB_INJECTOR_JAR_REL: &str = "launch/authlib-injector.jar";

/// Authlib-injector（外置登录，yggdrasil 协议）
///
/// 仅当 `auth_info.server_url` 有值时注入。
/// jar 由 `ensure_authlib_injector_jar` 预下载到缓存，不存在则跳过并警告。
/// 预取元数据按 host 缓存，避免切换服务器时复用错误元数据。
pub(super) fn add_authlib_args(args: &mut Vec<String>, auth_info: &AuthInfo) {
    let server_url = match auth_info.server_url.as_ref() {
        Some(url) if !url.is_empty() => url,
        _ => return, // 非外置登录，跳过
    };

    if !crate::utils::cache::exists(AUTHLIB_INJECTOR_JAR_REL) {
        crate::log_warn!(
            "[Launch] authlib-injector.jar 不存在于缓存，跳过 authlib 注入（外置登录将不可用）"
        );
        return;
    }
    let jar_path = crate::utils::cache::path(AUTHLIB_INJECTOR_JAR_REL);

    args.push(format!(
        "-javaagent:{}={}",
        jar_path.to_string_lossy(),
        server_url
    ));

    if let Some(prefetched) = read_prefetched_metadata(server_url) {
        args.push(format!(
            "-Dauthlibinjector.yggdrasil.prefetched={}",
            prefetched
        ));
    }

    crate::log_info!("[Launch] 注入 authlib-injector: server={}", server_url);
}

/// 读取已缓存的预取服务器元数据（base64 编码）
///
/// 缓存路径：`launch/authlib-prefetched-<host>.txt`
/// 返回 None 表示无缓存（authlib-injector 将在游戏启动时自行拉取）
fn read_prefetched_metadata(server_url: &str) -> Option<String> {
    let host = extract_host(server_url)?;
    let rel = format!("launch/authlib-prefetched-{}.txt", host);
    crate::utils::cache::read(&rel)
        .ok()
        .filter(|s| !s.is_empty())
}

/// 从 server_url 提取 host 部分，用作缓存文件名的安全标识
///
/// 仅保留字母、数字、点、连字符，其他字符替换为 `_`，避免文件名非法字符。
fn extract_host(server_url: &str) -> Option<String> {
    let after_scheme = server_url
        .strip_prefix("https://")
        .or_else(|| server_url.strip_prefix("http://"))
        .unwrap_or(server_url);
    let host_part = after_scheme.split('/').next()?;
    let sanitized: String = host_part
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

/// LUA（LWJGL Unsafe Agent）
/// 仅当库列表包含 org.lwjgl:lwjgl:3.4.1 且未禁用时注入
///
/// 来源：`lwjgl-unsafe-agent.jar` 为第三方开源项目 lwjgl-unsafe-agent
/// （https://github.com/HMCL-dev/lwjgl-unsafe-agent，Apache-2.0，作者 Glavo）。
/// 该项目通过 javaagent 修改 LWJGL 3.4.1 的字节码以修复其 FFM API 内联不佳导致的性能问题，
/// 本项目以外部依赖方式引入，许可证声明见 `src-tauri/resources/about/licenses.txt`。
pub(super) fn add_lua_args(args: &mut Vec<String>, json: &serde_json::Value, disable_lua: bool) {
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

/// 解析版本 JSON 的 arguments.jvm（必需 JVM 参数）
/// 跳过 ${classpath} 和 ${natives_directory}（由调用方单独处理）
pub(super) fn add_json_jvm_args(
    args: &mut Vec<String>,
    json: &serde_json::Value,
    game_dir: &Path,
    version_id: &str,
) {
    let libraries_dir = game_dir.join("libraries");
    // 保留 PathBuf 原生分隔符：Windows 上是 `\`，Unix 上是 `/`
    // JVM 在所有平台上都接受原生分隔符，无需强制替换
    // （原代码 `.replace('/', "\\")` 在 macOS/Linux 上会生成带反斜杠的非法路径）
    let libraries_dir_str = libraries_dir.to_string_lossy().to_string();
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
                let rules = obj.get("rules").and_then(|r| r.as_array()).cloned();
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

            // classpath 分隔符：Windows 用 `;`，Unix 系（macOS/Linux）用 `:`
            // 原代码硬编码 `;` 会导致 macOS/Linux 上 JVM 无法解析 classpath
            let classpath_separator = if cfg!(target_os = "windows") {
                ";"
            } else {
                ":"
            };
            let value = value
                .replace("${library_directory}", &libraries_dir_str)
                .replace("${classpath_separator}", classpath_separator)
                .replace("${version_name}", version_id);

            args.push(value);
        }
    }
}

/// JLW（Java Launch Wrapper）
/// - 仅当未禁用、非 GBK 编码、路径非纯 ASCII 时触发（仅在该环境下才会触发 JDK-8272352 Bug）
/// - 若用户自定义参数含 -javaagent 则禁用 JLW（冲突会导致崩溃）
/// - Java 9+ 添加 --add-exports cpw.mods.bootstraplauncher
/// - 添加 -Doolloo.jlw.tmpdir={pure_directory}（不以 \ 结尾，属性名由二进制内部约定）
/// - 末尾添加 -jar java-wrapper.jar（作为 JVM 入口，接收原 mainClass 作为参数）
///
/// 来源：嵌入的 `java-wrapper.jar` 为第三方开源项目 Java Launch Wrapper
/// （https://github.com/00ll00/java_launch_wrapper，MIT License，包名 oolloo.jlw）。
/// 该工具用于规避 JDK-8272352（非 ASCII 路径下 JVM 命令行参数乱码）导致的启动失败，
/// 本项目以外部依赖方式引入，许可证声明见 `src-tauri/resources/about/licenses.txt`。
pub(super) fn add_jlw_args(
    args: &mut Vec<String>,
    game_dir: &Path,
    java_major: Option<u32>,
    extra_jvm_args: &[String],
    disable_jlw: bool,
) {
    let is_gbk = is_gbk_encoding();
    let game_dir_str = game_dir.to_string_lossy();
    let is_ascii_only = game_dir_str.is_ascii();
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

/// Build JVM arguments
///
/// 编排 build 各段参数与 rules 的 GC 策略，返回完整 JVM 参数列表。
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_jvm_args(
    game_dir: &Path,
    version_id: &str,
    classpath: &str,
    min_memory: u32,
    max_memory: u32,
    java_path: &Path,
    auth_info: &AuthInfo,
    extra_jvm_args: &[String],
    json: &serde_json::Value,
    disable_jlw: bool,
    disable_lua: bool,
) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();

    // 检测 Java 主版本号（用于决定 GC 策略和 JLW 的 --add-exports）
    let java_major = crate::minecraft::java::detect_java_version(&java_path.to_string_lossy());

    // Authlib-injector 必须在 LUA/JLW 之前注入，让 -javaagent 出现在 args 首位便于排查
    add_authlib_args(&mut args, auth_info);

    // LUA（LWJGL Unsafe Agent）
    add_lua_args(&mut args, json, disable_lua);

    args.push(format!("-Xms{}M", min_memory));
    args.push(format!("-Xmx{}M", max_memory));

    // GC 策略
    super::rules::add_gc_args(&mut args, java_major);

    // 版本 JSON 的 arguments.jvm（必需 JVM 参数）
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

    // JLW（Java Launch Wrapper）
    add_jlw_args(&mut args, game_dir, java_major, extra_jvm_args, disable_jlw);

    Ok(args)
}
