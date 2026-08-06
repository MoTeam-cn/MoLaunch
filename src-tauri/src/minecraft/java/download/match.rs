//! Java 版本与 Mojang component 匹配
//!
//! 平台 key 由编译期 target_os / target_arch 推导（不再硬编码 Windows）；
//! 组件 key 使用显式映射表对齐官方 all.json 真值（ground truth 见
//! `docs/java-runtime-download-bugs-and-fix.md`），不依赖 version.name 模糊匹配与
//! HashMap 遍历顺序。

use serde_json::Value;

use super::types::JavaRuntimeEntry;

/// target_major → 官方组件 key 显式映射
///
/// 官方当前仅提供 5 档（8/16/17/21/25），其余版本（9/10/11/15/18/19/20/22/23/24 等）
/// 未在索引中分发，返回 None 表示无官方下载源。
pub(super) fn component_key_for_major(target_major: u32) -> Option<&'static str> {
    match target_major {
        8 => Some("jre-legacy"),
        16 => Some("java-runtime-alpha"),
        17 => Some("java-runtime-gamma"),
        21 => Some("java-runtime-delta"),
        25 => Some("java-runtime-epsilon"),
        _ => None,
    }
}

/// 当前编译平台 → 官方 all.json 平台 key
///
/// 找不到对应平台节点时返回清晰错误，不默认回退到 Windows。
pub fn platform_key() -> Result<&'static str, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Ok("windows-x64"),
        ("windows", "aarch64") => Ok("windows-arm64"),
        ("windows", "x86") | ("windows", "i686") => Ok("windows-x86"),
        ("macos", "x86_64") => Ok("mac-os"),
        ("macos", "aarch64") => Ok("mac-os-arm64"),
        ("linux", "x86_64") => Ok("linux"),
        ("linux", "x86") | ("linux", "i686") => Ok("linux-i386"),
        (os, arch) => Err(format!("不支持该平台: {}-{}", os, arch)),
    }
}

/// 根据 Java 大版本号匹配 Mojang component
///
/// 返回 `(component_key, JavaRuntimeEntry)`；平台不支持、组件缺失或解析失败时返回清晰错误。
pub fn match_component(
    all_json: &Value,
    target_major: u32,
) -> Result<(String, JavaRuntimeEntry), String> {
    let platform = platform_key()?;
    let platform_node = all_json
        .get(platform)
        .ok_or_else(|| format!("索引中不存在平台 {} 的数据", platform))?;
    let components = platform_node
        .as_object()
        .ok_or_else(|| format!("平台 {} 数据格式异常", platform))?;

    let key = component_key_for_major(target_major)
        .ok_or_else(|| format!("官方 Runtime 不提供 Java {} 的下载", target_major))?;

    let arr = components
        .get(key)
        .ok_or_else(|| format!("平台 {} 未提供组件 {}", platform, key))?;
    let arr = arr
        .as_array()
        .ok_or_else(|| format!("组件 {} 数据格式异常", key))?;
    let first = arr
        .first()
        .ok_or_else(|| format!("组件 {} 数据为空", key))?;
    let entry = serde_json::from_value::<JavaRuntimeEntry>(first.clone())
        .map_err(|e| format!("组件 {} 解析失败: {}", key, e))?;

    Ok((key.to_string(), entry))
}
