//! 安装器 processors 的参数解析
//!
//! processors 的命令行写在 install_profile.json 里，混着两种引用：占位符
//! （`{MINECRAFT_JAR}` 这类全局路径）和 data 表键（`[MC_SRG]`）。解析错一个参数，
//! processor 就会静默失败——它失败不影响安装器退出码，产物却不会再生成。

use std::collections::HashMap;
use std::path::Path;

/// 解析一个 processor 参数时能拿到的全部上下文
pub(crate) struct ProcessorContext<'a> {
    /// 游戏根目录（.minecraft）
    pub game_dir: &'a Path,
    /// 原版 client jar
    pub minecraft_jar: &'a Path,
    /// 安装器自身
    pub installer: &'a Path,
    /// binpatches 包，只有老版本 Forge 才有
    pub binpatch: Option<&'a Path>,
    /// 目标 MC 版本
    pub version: &'a str,
    /// install_profile 的 data 表，值已按 client 侧取好
    pub data: &'a HashMap<String, String>,
}

/// 解析单个参数
pub(crate) fn resolve_argument(argument: &str, context: &ProcessorContext<'_>) -> String {
    match argument {
        "{MINECRAFT_JAR}" => context.minecraft_jar.to_string_lossy().to_string(),
        "{MINECRAFT_VERSION}" => context.version.to_string(),
        "{ROOT}" => context.game_dir.to_string_lossy().to_string(),
        "{INSTALLER}" => context.installer.to_string_lossy().to_string(),
        "{LIBRARY_DIR}" => context
            .game_dir
            .join("libraries")
            .to_string_lossy()
            .to_string(),
        "{SIDE}" => "client".to_string(),
        "{BINPATCH}" => context
            .binpatch
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
        _ => match argument
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            Some(key) => context
                .data
                .get(key)
                .map(|value| normalize_data_value(value, context))
                .unwrap_or_default(),
            None => argument.to_string(),
        },
    }
}

/// 把 data 表里的值变成能直接当参数用的字符串
///
/// 三种形态：字面量（`'1.21.1'`）、绝对路径、Maven 坐标。
/// 坐标要转成本地路径，否则 processor 找不到文件。
fn normalize_data_value(value: &str, context: &ProcessorContext<'_>) -> String {
    let trimmed = value.trim();
    if let Some(literal) = trimmed
        .strip_prefix("'")
        .and_then(|rest| rest.strip_suffix("'"))
    {
        return literal.to_string();
    }
    if trimmed.contains(':') && !trimmed.contains('/') && !trimmed.contains('\\') {
        return super::shared::maven_path_to_local(trimmed, context.game_dir)
            .to_string_lossy()
            .to_string();
    }
    trimmed.to_string()
}

/// 解析一整条命令行
pub(crate) fn resolve_arguments(
    arguments: &[String],
    context: &ProcessorContext<'_>,
) -> Vec<String> {
    arguments
        .iter()
        .map(|argument| resolve_argument(argument, context))
        .collect()
}

#[cfg(test)]
#[path = "processor_args_test.rs"]
mod processor_args_test;
