//! 一级规则判定
//!
//! 崩溃报告与 hs_err 文本的关键字规则表，命中即返回 CrashInfo。

use super::super::super::types::{CrashCategory, CrashInfo};
use super::super::util::make_crash_info;
use std::path::Path;

/// 崩溃报告分析（高优先级）
pub(super) fn analyze_crash_report(
    log_crash: &str,
    error_lines: &[String],
    crash_report_path: Option<&Path>,
) -> Option<CrashInfo> {
    if log_crash.is_empty() {
        return None;
    }

    if log_crash.contains(
        "Unable to make protected final java.lang.Class java.lang.ClassLoader.defineClass",
    ) {
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
            error_lines,
            crash_report_path,
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
            error_lines,
            crash_report_path,
        ));
    }

    None
}

/// hs_err 日志分析（JVM 崩溃）
pub(super) fn analyze_hs_err(
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
