//! 第三级低优先级匹配（参考 PCL2 AnalyzeCrit3）
//!
//! 在前两级未命中时，使用更宽松的关键字匹配。

use super::super::types::{CrashCategory, CrashInfo};
use super::util::make_crash_info;
use std::path::Path;

/// 第三级低优先级匹配（参考 PCL2 AnalyzeCrit3）
pub(super) fn analyze_crit3(
    log_mc: &str,
    log_crash: &str,
    error_lines: &[String],
    crash_report_path: Option<&Path>,
) -> Option<CrashInfo> {
    let log_mc_l = log_mc.to_lowercase();

    if !log_mc.is_empty() {
        // 极短的程序输出
        if log_mc.len() < 100 && !log_mc.contains("at net.") && !log_mc.contains("INFO]") {
            return Some(make_crash_info(
                "极短的程序输出",
                CrashCategory::Unknown,
                "游戏输出了极少的内容就退出了。\n这可能是 Java 路径错误或参数有误，请检查启动设置。",
                error_lines, crash_report_path,
            ));
        }
        if log_mc_l.contains("mod resolution failed") {
            return Some(make_crash_info(
                "Mod 加载器报错",
                CrashCategory::Fabric,
                "Fabric Mod 加载器报告了 Mod 解析错误。\n请检查 Mod 是否与当前 MC 版本和 Fabric 版本兼容。",
                error_lines, crash_report_path,
            ));
        }
        if log_mc_l.contains("failed to create mod instance") {
            return Some(make_crash_info(
                "Mod 初始化失败",
                CrashCategory::Mod,
                "某个 Mod 在初始化时失败。\n请检查 Mod 的前置要求，或尝试移除最近安装的 Mod。",
                error_lines, crash_report_path,
            ));
        }
        if log_mc_l.contains("an exception was thrown, the game will display an error screen and halt") {
            return Some(make_crash_info(
                "Forge 报错",
                CrashCategory::Forge,
                "Forge 抛出了异常并停止了游戏。\n请查看日志了解具体错误，或尝试重新安装 Forge。",
                error_lines, crash_report_path,
            ));
        }
        if log_mc_l.contains("a potential solution has been determined") {
            return Some(make_crash_info(
                "Fabric 报错并给出解决方案",
                CrashCategory::Fabric,
                "Fabric 报告了错误并给出了可能的解决方案。\n请查看日志中 Fabric 提供的建议。",
                error_lines, crash_report_path,
            ));
        }
    }

    if !log_crash.is_empty() {
        if log_crash.contains("Block location: World: ") {
            return Some(make_crash_info(
                "特定方块导致崩溃",
                CrashCategory::Mod,
                "游戏在处理某个方块时崩溃。\n请尝试移除可能引起问题的方块，或使用 MCEdit 等工具删除该方块。",
                error_lines, crash_report_path,
            ));
        }
        if log_crash.contains("Entity's Exact location: ") {
            return Some(make_crash_info(
                "特定实体导致崩溃",
                CrashCategory::Mod,
                "游戏在处理某个实体时崩溃。\n请尝试移除可能引起问题的实体。",
                error_lines, crash_report_path,
            ));
        }
    }

    None
}
