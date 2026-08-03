//! JVM 参数构建
//!
//! `build_jvm_args` 编排参数拼装（build）与版本规则（rules）。

mod build;
mod rules;

use std::path::Path;

use super::AuthInfo;
use build::{add_authlib_args, add_json_jvm_args, add_jlw_args, add_lua_args};
use rules::add_gc_args;

#[allow(clippy::too_many_arguments)]
/// Build JVM arguments
pub(super) fn build_jvm_args(
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
    add_gc_args(&mut args, java_major);

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