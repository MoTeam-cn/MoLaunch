//! 第一级高优先级精准匹配
//!
//! 按 log_crash → log_mc → log_hs 三个来源依次检查，命中即返回。

use super::super::types::{CrashCategory, CrashInfo};
use super::util::{extract_mod_from_keyword, make_crash_info};
use std::path::Path;

/// 第一级高优先级精准匹配
pub(super) fn analyze_crit1(
    log_mc: &str,
    log_crash: &str,
    log_hs: &str,
    error_lines: &[String],
    crash_report_path: Option<&Path>,
) -> Option<CrashInfo> {
    // --- 崩溃报告分析（高优先级）---
    if let Some(info) = analyze_crash_report(log_crash, error_lines, crash_report_path) {
        return Some(info);
    }

    // --- 游戏日志分析（高优先级）---
    if let Some(info) = analyze_game_log(log_mc, error_lines, crash_report_path) {
        return Some(info);
    }

    // --- hs_err 日志分析（JVM 崩溃）---
    if let Some(info) = analyze_hs_err(log_hs, error_lines, crash_report_path) {
        return Some(info);
    }

    None
}

/// 崩溃报告分析（高优先级）
fn analyze_crash_report(
    log_crash: &str,
    error_lines: &[String],
    crash_report_path: Option<&Path>,
) -> Option<CrashInfo> {
    if log_crash.is_empty() {
        return None;
    }

    if log_crash.contains("Unable to make protected final java.lang.Class java.lang.ClassLoader.defineClass") {
        return Some(make_crash_info(
            "Java 版本过高",
            CrashCategory::Java,
            "游戏似乎因为你所使用的 Java 版本过高而崩溃了。\n请在启动设置的 Java 选择一项中改用较低版本的 Java，然后再启动游戏。",
            error_lines, crash_report_path,
        ));
    }
    if log_crash.contains("maximum id range exceeded") {
        return Some(make_crash_info(
            "Mod 过多导致超出 ID 限制",
            CrashCategory::Mod,
            "安装的 Mod 过多，超出了游戏的 ID 限制。\n请尝试移除部分不常用的 Mod。",
            error_lines, crash_report_path,
        ));
    }
    if log_crash.contains("Pixel format not accelerated") {
        return Some(make_crash_info(
            "显卡驱动不支持导致无法设置像素格式",
            CrashCategory::Graphics,
            "显卡驱动不支持游戏的像素格式。\n请更新显卡驱动，或尝试在启动设置中关闭「使用高性能显卡」。",
            error_lines, crash_report_path,
        ));
    }
    if log_crash.contains("Manually triggered debug crash") {
        return Some(make_crash_info(
            "玩家手动触发调试崩溃",
            CrashCategory::Unknown,
            "事实上，你的游戏没有任何问题，这是你自己触发的崩溃（F3+C）。",
            error_lines, crash_report_path,
        ));
    }

    None
}

/// 游戏日志分析（高优先级）
fn analyze_game_log(
    log_mc: &str,
    error_lines: &[String],
    crash_report_path: Option<&Path>,
) -> Option<CrashInfo> {
    if log_mc.is_empty() {
        return None;
    }

    let log_mc_l = log_mc.to_lowercase();

    if log_mc_l.contains("unrecognized option:") {
        return Some(make_crash_info(
            "Java 虚拟机参数有误",
            CrashCategory::Java,
            "Java 无法识别启动参数中的某个选项。\n请检查启动设置中的 JVM 参数是否正确，或尝试清空自定义 JVM 参数。",
            error_lines, crash_report_path,
        ));
    }
    if log_mc_l.contains("could not create the java virtual machine") {
        return Some(make_crash_info(
            "无法创建 Java 虚拟机",
            CrashCategory::Java,
            "Java 虚拟机创建失败。\n请检查 JVM 参数是否正确，或尝试更换 Java 版本。",
            error_lines, crash_report_path,
        ));
    }
    if log_mc_l.contains("the driver does not appear to support opengl") {
        return Some(make_crash_info(
            "显卡不支持 OpenGL",
            CrashCategory::Graphics,
            "显卡驱动不支持 OpenGL，或驱动版本过低。\n请更新显卡驱动，若使用独显请确保 Java 使用的是高性能显卡。",
            error_lines, crash_report_path,
        ));
    }
    if log_mc_l.contains("couldn't set pixel format") {
        return Some(make_crash_info(
            "显卡驱动不支持导致无法设置像素格式",
            CrashCategory::Graphics,
            "显卡驱动无法设置像素格式。\n请更新显卡驱动，或尝试在启动设置中关闭「使用高性能显卡」。",
            error_lines, crash_report_path,
        ));
    }
    if log_mc_l.contains("open j9 is not supported")
        || log_mc_l.contains("openj9 is incompatible")
        || log_mc_l.contains(".j9vminternals.")
    {
        return Some(make_crash_info(
            "使用了不兼容的 OpenJ9 Java",
            CrashCategory::Java,
            "游戏不兼容 OpenJ9 虚拟机。\n请更换为 HotSpot 版本的 Java。",
            error_lines, crash_report_path,
        ));
    }
    if log_mc_l.contains("java.lang.outofmemoryerror")
        || log_mc_l.contains("an out of memory error")
    {
        return Some(make_crash_info(
            "内存不足",
            CrashCategory::Memory,
            "Minecraft 内存不足，导致其无法继续运行。\n请尝试在启动设置中增加为游戏分配的内存，并关闭其他占用内存的程序。",
            error_lines, crash_report_path,
        ));
    }
    if log_mc_l.contains("could not reserve enough space") {
        if log_mc_l.contains("for 1048576kb object heap") {
            return Some(make_crash_info(
                "使用 32 位 Java 导致 JVM 无法分配足够内存",
                CrashCategory::Java,
                "32 位 Java 无法分配足够的内存。\n请更换为 64 位 Java。",
                error_lines, crash_report_path,
            ));
        }
        return Some(make_crash_info(
            "内存不足",
            CrashCategory::Memory,
            "JVM 无法保留足够的内存空间。\n请尝试减少游戏内存分配，或关闭其他占用内存的程序。",
            error_lines, crash_report_path,
        ));
    }
    if log_mc_l.contains("1282: invalid operation") {
        return Some(make_crash_info(
            "光影或资源包导致 OpenGL 1282 错误",
            CrashCategory::Graphics,
            "光影或资源包导致了 OpenGL 错误。\n请尝试移除最近安装的光影或资源包。",
            error_lines, crash_report_path,
        ));
    }
    if log_mc_l.contains("duplicate mod") || log_mc_l.contains("duplicate mods found") {
        return Some(make_crash_info(
            "Mod 重复安装",
            CrashCategory::Mod,
            "检测到重复安装的 Mod。\n请检查 mods 文件夹，移除重复的 Mod 文件。",
            error_lines, crash_report_path,
        ));
    }
    if log_mc_l.contains("missing or unsupported mandatory dependencies") {
        return Some(make_crash_info(
            "Mod 缺少前置或 MC 版本错误",
            CrashCategory::Mod,
            "某个 Mod 缺少必要的前置 Mod，或与当前 MC 版本不兼容。\n请检查 Mod 的前置要求，并确保所有 Mod 与 MC 版本匹配。",
            error_lines, crash_report_path,
        ));
    }
    if log_mc_l.contains("java.lang.classcastexception: java.base/jdk") {
        return Some(make_crash_info(
            "使用了 JDK 而非 JRE",
            CrashCategory::Java,
            "游戏似乎因为使用了 JDK 而非 JRE 而崩溃。\n请更换为 JRE 版本的 Java。",
            error_lines, crash_report_path,
        ));
    }
    // ClassNotFoundException: 主类或关键类找不到（通常是 Fabric/Forge 加载器库缺失）
    if log_mc_l.contains("classnotfoundexception") {
        // 尝试提取缺失的类名
        let missing_class = extract_class_name(log_mc, "classnotfoundexception");
        let class_hint = missing_class.as_deref().unwrap_or("未知类");
        // 根据类名判断具体原因
        let (reason, suggestion) = if class_hint.contains("fabric") || class_hint.contains("knot") {
            (
                "Fabric 加载器库缺失",
                "Fabric Loader 的 jar 文件未正确加入 classpath。\n这通常是版本安装不完整导致的。\n请尝试重新安装该版本的 Fabric 加载器，或重新创建版本。".to_string(),
            )
        } else if class_hint.contains("forge") || class_hint.contains("fml") || class_hint.contains("modlauncher") {
            (
                "Forge 加载器库缺失",
                "Forge 的核心库未正确加入 classpath。\n这通常是版本安装不完整导致的。\n请尝试重新安装该版本的 Forge 加载器，或重新创建版本。".to_string(),
            )
        } else if class_hint.contains("neoforge") {
            (
                "NeoForge 加载器库缺失",
                "NeoForge 的核心库未正确加入 classpath。\n这通常是版本安装不完整导致的。\n请尝试重新安装该版本的 NeoForge 加载器，或重新创建版本。".to_string(),
            )
        } else {
            (
                "Java 类缺失",
                format!("Java 找不到类 {}，可能是版本安装不完整或 classpath 配置有误。\n请尝试重新安装该版本。", class_hint),
            )
        };
        return Some(make_crash_info(
            reason,
            CrashCategory::Unknown,
            &suggestion,
            error_lines, crash_report_path,
        ));
    }
    // 确定的 Mod 导致崩溃
    if let Some(mod_name) = extract_mod_from_keyword(log_mc, "caught exception from ") {
        return Some(CrashInfo {
            reason: format!("Mod '{}' 导致游戏崩溃", mod_name),
            category: CrashCategory::Mod,
            log_lines: error_lines.to_vec(),
            suggestion: format!("名为 {} 的 Mod 导致了游戏出错。\n你可以尝试禁用此 Mod，然后观察游戏是否还会崩溃。", mod_name),
            problematic_mod: Some(mod_name),
            crash_report_path: crash_report_path.map(|p| p.to_string_lossy().to_string()),
            log_tail: Vec::new(),
        });
    }

    None
}

/// 从日志中提取 ClassNotFoundException 中的类名
///
/// 匹配模式：`ClassNotFoundException: net.fabricmc.loader.impl.launch.knot.KnotClient`
/// 或 `java.lang.ClassNotFoundException: xxx`
fn extract_class_name(log: &str, keyword: &str) -> Option<String> {
    let log_l = log.to_lowercase();
    let keyword_l = keyword.to_lowercase();
    if let Some(pos) = log_l.find(&keyword_l) {
        // 从关键字后找类名（可能跨行）
        let rest = &log[pos + keyword_l.len()..];
        // 跳过冒号和空格
        let rest = rest.trim_start_matches(':').trim_start();
        // 类名格式：xxx.xxx.xxx（只含字母、数字、点、$、_）
        let class_end = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '.' || c == '$' || c == '_'))
            .unwrap_or(rest.len().min(200));
        let class_name = &rest[..class_end];
        if !class_name.is_empty() {
            return Some(class_name.to_string());
        }
    }
    None
}

/// hs_err 日志分析（JVM 崩溃）
fn analyze_hs_err(
    log_hs: &str,
    error_lines: &[String],
    crash_report_path: Option<&Path>,
) -> Option<CrashInfo> {
    if log_hs.is_empty() {
        return None;
    }

    let log_hs_l = log_hs.to_lowercase();

    if log_hs_l.contains("the system is out of physical ram or swap space")
        || log_hs_l.contains("out of memory error")
    {
        return Some(make_crash_info(
            "内存不足（JVM 崩溃）",
            CrashCategory::Memory,
            "JVM 因内存不足而崩溃。\n请尝试减少游戏内存分配，或关闭其他占用内存的程序。\n如果是 32 位 Java，请更换为 64 位。",
            error_lines, crash_report_path,
        ));
    }
    if log_hs_l.contains("exception_access_violation") {
        // Intel 驱动
        if log_hs_l.contains("# c  [ig") {
            return Some(make_crash_info(
                "Intel 显卡驱动不兼容导致 JVM 崩溃",
                CrashCategory::Graphics,
                "Intel 显卡驱动不兼容导致了 JVM 崩溃（EXCEPTION_ACCESS_VIOLATION）。\n请更新 Intel 显卡驱动到最新版本。\n参考: https://bugs.mojang.com/browse/MC-32606",
                error_lines, crash_report_path,
            ));
        }
        // AMD 驱动
        if log_hs_l.contains("# c  [atio") {
            return Some(make_crash_info(
                "AMD 显卡驱动不兼容导致 JVM 崩溃",
                CrashCategory::Graphics,
                "AMD 显卡驱动不兼容导致了 JVM 崩溃（EXCEPTION_ACCESS_VIOLATION）。\n请更新 AMD 显卡驱动到最新版本。\n参考: https://bugs.mojang.com/browse/MC-31618",
                error_lines, crash_report_path,
            ));
        }
        // Nvidia 驱动
        if log_hs_l.contains("# c  [nvoglv") {
            return Some(make_crash_info(
                "Nvidia 显卡驱动不兼容导致 JVM 崩溃",
                CrashCategory::Graphics,
                "Nvidia 显卡驱动不兼容导致了 JVM 崩溃（EXCEPTION_ACCESS_VIOLATION）。\n请更新 Nvidia 显卡驱动到最新版本。",
                error_lines, crash_report_path,
            ));
        }
        // 通用 EXCEPTION_ACCESS_VIOLATION
        return Some(make_crash_info(
            "JVM 崩溃（EXCEPTION_ACCESS_VIOLATION）",
            CrashCategory::Unknown,
            "Java 虚拟机遇到了访问违例崩溃。\n这通常是显卡驱动问题导致的，请尝试更新显卡驱动。\n如果使用独显，请确保 Java 使用的是高性能显卡。",
            error_lines, crash_report_path,
        ));
    }

    None
}
