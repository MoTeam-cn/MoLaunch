//! 崩溃分析（运行时日志 + 崩溃报告文件 + hs_err + latest.log）
//!
//! 严格参考 PCL2 ModCrash.vb 的 CrashAnalyzer 四步流程：
//! 1. Collect  — 收集 crash-reports/*.txt、hs_err_pid*.log、logs/latest.log、运行时日志
//! 2. Prepare  — 提取各源文本（头N行 + 尾M行）
//! 3. Analyze  — 三级关键字匹配（高优先级精准 → 堆栈分析 → 低优先级）
//! 4. Output   — 返回 CrashInfo（含 reason/suggestion/crash_report_path/log_tail）

use super::types::{CrashCategory, CrashInfo, LogEntry, LogLevel};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 分析崩溃（主入口）
///
/// 参考 PCL2 ModCrash.vb CrashAnalyzer.Analyze
/// 综合 exit_code、运行时日志、crash-reports 文件、hs_err 文件、latest.log 判断崩溃原因
pub(crate) async fn analyze_crash(
    exit_code: i32,
    logs: &[LogEntry],
    game_dir: &Path,
) -> Option<CrashInfo> {
    // 正常退出不分析
    if exit_code == 0 {
        return None;
    }

    crate::log_info!("[CrashAnalyzer] 开始崩溃分析（exit_code={}）", exit_code);

    // ===== 步骤1: Collect — 收集各源文本 =====
    let runtime_log: String = logs
        .iter()
        .map(|e| format!("[{:?}] {}", e.level, e.message))
        .collect::<Vec<_>>()
        .join("\n");

    // 收集错误/致命级别日志行
    let error_lines: Vec<String> = logs
        .iter()
        .filter(|e| e.level == LogLevel::Error || e.level == LogLevel::Fatal)
        .map(|e| e.message.clone())
        .collect();

    // 读取 crash-reports 目录中最新的崩溃报告（3分钟内）
    let crash_report = read_latest_crash_report(game_dir);
    let crash_report_text: String = crash_report
        .as_ref()
        .and_then(|(path, content)| {
            crate::log_info!("[CrashAnalyzer] 找到崩溃报告: {}", path.display());
            Some(content.clone())
        })
        .unwrap_or_default();

    // 读取 hs_err_pid*.log（JVM 崩溃报告，3分钟内）
    let hs_err_text = read_latest_hs_err(game_dir);
    if !hs_err_text.is_empty() {
        crate::log_info!("[CrashAnalyzer] 找到 hs_err_pid 日志（{}字符）", hs_err_text.len());
    }

    // 读取 logs/latest.log 尾部（500行）
    let latest_log_tail = read_latest_log_tail(game_dir, 500);
    if !latest_log_tail.is_empty() {
        crate::log_info!("[CrashAnalyzer] 读取 latest.log 尾部（{}行）", latest_log_tail.len());
    }

    // ===== 步骤2: Analyze — 三级关键字匹配（参考 PCL2 AnalyzeCrit1/2/3）=====

    // 第一级：高优先级精准匹配
    if let Some(info) = analyze_crit1(
        &runtime_log,
        &crash_report_text,
        &hs_err_text,
        &error_lines,
        crash_report.as_ref().map(|(p, _)| p.clone()),
    ) {
        crate::log_info!("[CrashAnalyzer] 一级匹配命中: {}", info.reason);
        return Some(info);
    }

    // 第二级：堆栈分析（仅当存在 Mod 加载器时）
    let has_mod_loader = runtime_log.contains("orge")
        || runtime_log.contains("abric")
        || runtime_log.contains("uilt")
        || runtime_log.contains("iteloader")
        || runtime_log.contains("ModLauncher")
        || runtime_log.contains("fmlloader");
    if has_mod_loader {
        if let Some(info) = analyze_stack(
            &runtime_log,
            &crash_report_text,
            &hs_err_text,
            &error_lines,
            crash_report.as_ref().map(|(p, _)| p.clone()),
        ) {
            crate::log_info!("[CrashAnalyzer] 堆栈分析命中: {}", info.reason);
            return Some(info);
        }
    }

    // 第三级：低优先级匹配
    if let Some(info) = analyze_crit3(
        &runtime_log,
        &crash_report_text,
        &error_lines,
        crash_report.as_ref().map(|(p, _)| p.clone()),
    ) {
        crate::log_info!("[CrashAnalyzer] 三级匹配命中: {}", info.reason);
        return Some(info);
    }

    // ===== 兜底：未识别的崩溃 =====
    crate::log_info!("[CrashAnalyzer] 未匹配到已知崩溃模式，返回通用崩溃信息");
    let log_tail: Vec<String> = if !latest_log_tail.is_empty() {
        latest_log_tail
    } else {
        logs.iter().rev().take(30).rev().map(|e| e.message.clone()).collect()
    };

    Some(CrashInfo {
        reason: format!("游戏异常退出（退出码 {}）", exit_code),
        category: CrashCategory::Unknown,
        log_lines: error_lines,
        suggestion: "未识别到已知的崩溃模式。请查看日志文件获取更多信息，或尝试将崩溃报告发送给他人寻求帮助。".to_string(),
        problematic_mod: None,
        crash_report_path: crash_report.as_ref().map(|(p, _)| p.to_string_lossy().to_string()),
        log_tail,
    })
}

// ============================================================================
// 步骤3: Analyze — 第一级高优先级精准匹配（参考 PCL2 AnalyzeCrit1）
// ============================================================================

fn analyze_crit1(
    log_mc: &str,
    log_crash: &str,
    log_hs: &str,
    error_lines: &[String],
    crash_report_path: Option<PathBuf>,
) -> Option<CrashInfo> {
    let log_mc_l = log_mc.to_lowercase();
    let _log_crash_l = log_crash.to_lowercase();
    let log_hs_l = log_hs.to_lowercase();

    // --- 崩溃报告分析（高优先级）---
    if !log_crash.is_empty() {
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
    }

    // --- 游戏日志分析（高优先级）---
    if !log_mc.is_empty() {
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
    }

    // --- hs_err 日志分析（JVM 崩溃）---
    if !log_hs.is_empty() {
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
    }

    None
}

// ============================================================================
// 步骤3: Analyze — 第二级堆栈分析（参考 PCL2 AnalyzeStackKeyword）
// ============================================================================

fn analyze_stack(
    log_mc: &str,
    log_crash: &str,
    log_hs: &str,
    error_lines: &[String],
    crash_report_path: Option<PathBuf>,
) -> Option<CrashInfo> {
    // 从各源提取堆栈关键字
    let mut keywords = Vec::new();

    // 从崩溃报告提取
    if !log_crash.is_empty() {
        if let Some(details_end) = log_crash.find("System Details") {
            keywords.extend(extract_stack_keywords(&log_crash[..details_end]));
        }
    }

    // 从运行时日志的 FATAL 行提取
    if !log_mc.is_empty() {
        keywords.extend(extract_stack_keywords(log_mc));
    }

    // 从 hs_err 的 THREAD 段提取
    if !log_hs.is_empty() {
        if let Some(thread_start) = log_hs.find("T H R E A D") {
            let thread_section = if let Some(reg_start) = log_hs[thread_start..].find("Registers:") {
                &log_hs[thread_start..thread_start + reg_start]
            } else {
                &log_hs[thread_start..]
            };
            keywords.extend(extract_stack_keywords(thread_section));
        }
    }

    if keywords.is_empty() {
        return None;
    }

    // 过滤并提取 Mod 名称
    let mod_names = analyze_mod_name(&keywords);
    if let Some(ref names) = mod_names {
        let names_str = names.join(", ");
        return Some(CrashInfo {
            reason: format!("堆栈分析发现可能的 Mod: {}", names_str),
            category: CrashCategory::Mod,
            log_lines: error_lines.to_vec(),
            suggestion: format!("以下 Mod 可能导致了游戏崩溃：{}\n你可以尝试依次禁用上述 Mod，然后观察游戏是否还会崩溃。", names_str),
            problematic_mod: Some(names_str),
            crash_report_path: crash_report_path.map(|p| p.to_string_lossy().to_string()),
            log_tail: Vec::new(),
        });
    }

    None
}

/// 从堆栈文本提取关键字（参考 PCL2 AnalyzeStackKeyword）
fn extract_stack_keywords(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let excluded_packages = [
        "java.", "javax.", "sun.", "com.sun.", "jdk.", "oolloo.",
        "org.lwjgl", "net.minecraftforge", "paulscode.sound", "com.mojang",
        "net.minecraft", "cpw.mods", "com.google", "org.apache",
        "org.spongepowered", "net.fabricmc", "com.mumfrey",
        "com.electronwill.nightconfig", "it.unimi.dsi",
        "MojangTricksIntelDriversForPerformance",
    ];

    // 匹配 "at xxx.yyy.Zzz(" 格式的堆栈行
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("at ") {
            continue;
        }
        let rest = &trimmed[3..];
        if let Some(paren) = rest.find('(') {
            let class_path = &rest[..paren];
            // 排除非 Mod 包名
            let is_excluded = excluded_packages.iter().any(|p| class_path.starts_with(p));
            if !is_excluded && class_path.contains('.') {
                // 提取前 4 节作为关键字
                let parts: Vec<&str> = class_path.split('.').collect();
                for part in parts.iter().take(4.min(parts.len())) {
                    let word = *part;
                    if word.len() <= 2 || word.starts_with("func_") {
                        continue;
                    }
                    let word_l = word.to_lowercase();
                    // 排除通用词
                    if matches!(word_l.as_str(),
                        "com" | "org" | "net" | "asm" | "fml" | "mod" | "forge" |
                        "fabric" | "minecraft" | "optifine" | "internal" | "common" |
                        "core" | "api" | "util" | "lib" | "client" | "server" |
                        "event" | "config" | "block" | "item" | "entity" | "render" |
                        "world" | "game" | "player" | "tile" | "gui" | "screen" |
                        "packet" | "network" | "registry" | "loader" | "mixin" |
                        "concurrent"
                    ) {
                        continue;
                    }
                    results.push(word.to_string());
                }
            }
        }
    }

    results
}

/// 从关键字列表分析 Mod 名称（参考 PCL2 AnalyzeModName）
fn analyze_mod_name(keywords: &[String]) -> Option<Vec<String>> {
    if keywords.is_empty() {
        return None;
    }
    if keywords.len() > 10 {
        // 关键词过多，可能是匹配出错
        return None;
    }
    let unique: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        keywords.iter().filter(|k| seen.insert((*k).clone())).cloned().collect()
    };
    if unique.is_empty() {
        None
    } else {
        Some(unique)
    }
}

// ============================================================================
// 步骤3: Analyze — 第三级低优先级匹配（参考 PCL2 AnalyzeCrit3）
// ============================================================================

fn analyze_crit3(
    log_mc: &str,
    log_crash: &str,
    error_lines: &[String],
    crash_report_path: Option<PathBuf>,
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

// ============================================================================
// 文件读取辅助函数
// ============================================================================

/// 读取 crash-reports 目录中最新的崩溃报告（3分钟内修改过）
/// 参考 PCL2 ModCrash.vb Collect 方法
fn read_latest_crash_report(game_dir: &Path) -> Option<(PathBuf, String)> {
    let crash_dir = game_dir.join("crash-reports");
    if !crash_dir.exists() {
        return None;
    }

    let now = SystemTime::now();
    let mut latest: Option<(PathBuf, SystemTime)> = None;

    if let Ok(entries) = std::fs::read_dir(&crash_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // 只看 crash-*.txt 文件
            let name = path.file_name()?.to_string_lossy();
            if !name.starts_with("crash-") || path.extension().map_or(true, |e| e != "txt") {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    // 只看 3 分钟内修改过的文件（参考 PCL2）
                    if let Ok(age) = now.duration_since(modified) {
                        if age.as_secs() < 180 {
                            if latest.as_ref().map_or(true, |(_, t)| modified > *t) {
                                latest = Some((path, modified));
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some((path, _)) = latest {
        if let Ok(content) = std::fs::read_to_string(&path) {
            return Some((path, content));
        }
    }
    None
}

/// 读取最新的 hs_err_pid*.log 文件（3分钟内）
/// 参考 PCL2 ModCrash.vb Collect 中收集 .minecraft 根目录下 .log 文件的逻辑
fn read_latest_hs_err(game_dir: &Path) -> String {
    let now = SystemTime::now();
    let mut latest: Option<(PathBuf, SystemTime)> = None;

    // hs_err_pid*.log 可能在游戏根目录或版本目录
    let search_dirs = [game_dir.to_path_buf()];

    for dir in &search_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = match path.file_name() {
                    Some(n) => n.to_string_lossy().to_string(),
                    None => continue,
                };
                if !name.starts_with("hs_err_pid") {
                    continue;
                }
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age.as_secs() < 180 {
                                if latest.as_ref().map_or(true, |(_, t)| modified > *t) {
                                    latest = Some((path, modified));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some((path, _)) = latest {
        if let Ok(content) = std::fs::read_to_string(&path) {
            // 截取头 200 行 + 尾 100 行（参考 PCL2 GetHeadTailLines）
            return truncate_head_tail(&content, 200, 100);
        }
    }
    String::new()
}

/// 读取 logs/latest.log 的尾部 N 行
fn read_latest_log_tail(game_dir: &Path, tail_lines: usize) -> Vec<String> {
    let log_path = game_dir.join("logs").join("latest.log");
    if !log_path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&log_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let lines: Vec<&str> = content.lines().collect();
    let start = if lines.len() > tail_lines {
        lines.len() - tail_lines
    } else {
        0
    };
    lines[start..].iter().map(|s| s.to_string()).collect()
}

// ============================================================================
// 工具函数
// ============================================================================

/// 从日志文本中提取 "Caught exception from {ModName}" 格式的 Mod 名称
fn extract_mod_from_keyword(text: &str, prefix: &str) -> Option<String> {
    let text_l = text.to_lowercase();
    let prefix_l = prefix.to_lowercase();
    if let Some(pos) = text_l.find(&prefix_l) {
        let rest = &text[pos + prefix_l.len()..];
        // 取到行尾或下一个空格
        let end = rest.find(|c: char| c == '\n' || c == '\r').unwrap_or(rest.len());
        let mod_name = rest[..end].trim();
        if !mod_name.is_empty() {
            return Some(mod_name.to_string());
        }
    }
    None
}

/// 截取头 N 行 + 尾 M 行（参考 PCL2 GetHeadTailLines）
fn truncate_head_tail(content: &str, head: usize, tail: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= head + tail {
        return content.to_string();
    }
    let mut result = String::new();
    for line in &lines[..head] {
        result.push_str(line);
        result.push('\n');
    }
    result.push_str("...（省略中间部分）...\n");
    for line in &lines[lines.len() - tail..] {
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// 构造 CrashInfo 的快捷函数
fn make_crash_info(
    reason: &str,
    category: CrashCategory,
    suggestion: &str,
    error_lines: &[String],
    crash_report_path: Option<PathBuf>,
) -> CrashInfo {
    CrashInfo {
        reason: reason.to_string(),
        category,
        log_lines: error_lines.to_vec(),
        suggestion: suggestion.to_string(),
        problematic_mod: None,
        crash_report_path: crash_report_path.map(|p| p.to_string_lossy().to_string()),
        log_tail: Vec::new(),
    }
}
