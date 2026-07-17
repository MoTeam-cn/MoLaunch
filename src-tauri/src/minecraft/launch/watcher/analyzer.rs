//! 崩溃分析（运行时日志分析 + 崩溃报告文件解析）
//!
//! 原 `GameWatcher::analyze_crash` / `analyze_stack_for_mod` 静态方法 +
//! 模块级 `analyze_crash_report` / `parse_crash_report` 函数。

use super::types::{CrashCategory, CrashInfo, LogEntry, LogLevel};
use std::path::PathBuf;

/// 分析崩溃 (参考PCL2的ModCrash)
pub(crate) fn analyze_crash(exit_code: i32, logs: &[LogEntry]) -> Option<CrashInfo> {
    // 正常退出
    if exit_code == 0 {
        return None;
    }

    // 收集错误日志
    let error_lines: Vec<String> = logs
        .iter()
        .filter(|e| e.level == LogLevel::Error || e.level == LogLevel::Fatal)
        .map(|e| e.message.clone())
        .collect();

    // 检查常见崩溃模式
    for line in &error_lines {
        let line_lower = line.to_lowercase();

        // Java 虚拟机创建失败
        if line_lower.contains("could not create the java virtual machine") {
            return Some(CrashInfo {
                reason: "无法创建Java虚拟机".to_string(),
                category: CrashCategory::Java,
                log_lines: error_lines.clone(),
                suggestion: "请检查JVM参数是否正确，或尝试更换Java版本".to_string(),
                problematic_mod: None,
            });
        }

        // 内存不足
        if line_lower.contains("outofmemoryerror") || line_lower.contains("out of memory") {
            return Some(CrashInfo {
                reason: "内存不足".to_string(),
                category: CrashCategory::Memory,
                log_lines: error_lines.clone(),
                suggestion: "请增加最大内存分配，或关闭其他程序释放内存".to_string(),
                problematic_mod: None,
            });
        }

        // OpenGL 错误
        if line_lower.contains("opengl")
            && (line_lower.contains("error") || line_lower.contains("not supported"))
        {
            return Some(CrashInfo {
                reason: "OpenGL错误".to_string(),
                category: CrashCategory::Graphics,
                log_lines: error_lines.clone(),
                suggestion: "请更新显卡驱动，或尝试降低游戏设置".to_string(),
                problematic_mod: None,
            });
        }

        // Forge 错误
        if line_lower.contains("forge") && line_lower.contains("error") {
            return Some(CrashInfo {
                reason: "Forge加载错误".to_string(),
                category: CrashCategory::Forge,
                log_lines: error_lines.clone(),
                suggestion: "请尝试重新安装Forge，或检查Mod兼容性".to_string(),
                problematic_mod: None,
            });
        }

        // Fabric 错误
        if line_lower.contains("fabric") && line_lower.contains("error") {
            return Some(CrashInfo {
                reason: "Fabric加载错误".to_string(),
                category: CrashCategory::Fabric,
                log_lines: error_lines.clone(),
                suggestion: "请尝试重新安装Fabric，或检查Mod兼容性".to_string(),
                problematic_mod: None,
            });
        }
    }

    // 检查崩溃报告
    let has_crash_report = logs.iter().any(|e| {
        e.message.contains("Crash report saved to") || e.message.contains("crash-reports")
    });

    if has_crash_report {
        return Some(CrashInfo {
            reason: "游戏崩溃".to_string(),
            category: CrashCategory::Unknown,
            log_lines: error_lines,
            suggestion: "请查看崩溃报告获取详细信息".to_string(),
            problematic_mod: None,
        });
    }

    // 尝试从堆栈分析Mod
    let problematic_mod = analyze_stack_for_mod(&error_lines);

    // 通用崩溃
    if exit_code != 0 {
        let reason = if let Some(ref mod_id) = problematic_mod {
            format!("可能由Mod '{}' 导致的崩溃", mod_id)
        } else {
            format!("游戏异常退出 (代码: {})", exit_code)
        };

        let category = if problematic_mod.is_some() {
            CrashCategory::Mod
        } else {
            CrashCategory::Unknown
        };

        let suggestion = if let Some(ref mod_id) = problematic_mod {
            format!("请尝试移除Mod '{}' 或更新到兼容版本", mod_id)
        } else {
            "请查看日志获取详细信息".to_string()
        };

        return Some(CrashInfo {
            reason,
            category,
            log_lines: error_lines,
            suggestion,
            problematic_mod,
        });
    }

    None
}

/// 从堆栈分析可能的Mod
fn analyze_stack_for_mod(error_lines: &[String]) -> Option<String> {
    // 常见的非Mod包名
    let excluded_packages = [
        "java.",
        "javax.",
        "sun.",
        "com.sun.",
        "jdk.",
        "net.minecraft",
        "com.mojang",
        "net.minecraftforge",
        "net.fabricmc",
        "net.neoforged",
        "cpw.mods",
        "org.spongepowered",
        "org.apache",
        "com.google",
    ];

    for line in error_lines {
        // 查找 at 开头的堆栈行
        if line.trim().starts_with("at ") || line.contains("at ") {
            // 提取类名
            if let Some(at_pos) = line.find("at ") {
                let rest = &line[at_pos + 3..];
                if let Some(paren_pos) = rest.find('(') {
                    let class_path = &rest[..paren_pos];

                    // 检查是否是Mod包
                    let is_excluded =
                        excluded_packages.iter().any(|p| class_path.starts_with(p));
                    if !is_excluded && class_path.contains('.') {
                        // 可能是Mod包，尝试提取Mod ID
                        let parts: Vec<&str> = class_path.split('.').collect();
                        if parts.len() >= 3 {
                            // 通常格式: com.modid.xxx
                            let potential_mod_id = parts[1];
                            // 过滤掉常见的非Mod标识
                            if !["common", "core", "api", "util", "lib", "internal"]
                                .contains(&potential_mod_id)
                            {
                                return Some(potential_mod_id.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// 从日志文件分析崩溃
pub async fn analyze_crash_report(game_dir: &PathBuf, _version_id: &str) -> Option<CrashInfo> {
    // 查找最新的崩溃报告
    let crash_reports_dir = game_dir.join("crash-reports");
    if !crash_reports_dir.exists() {
        return None;
    }

    // 读取最新的崩溃报告
    let mut latest_report = None;
    let mut latest_time = std::time::SystemTime::UNIX_EPOCH;

    if let Ok(entries) = std::fs::read_dir(&crash_reports_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified > latest_time {
                            latest_time = modified;
                            latest_report = Some(path);
                        }
                    }
                }
            }
        }
    }

    if let Some(report_path) = latest_report {
        if let Ok(content) = std::fs::read_to_string(&report_path) {
            return parse_crash_report(&content);
        }
    }

    None
}

/// 解析崩溃报告
fn parse_crash_report(content: &str) -> Option<CrashInfo> {
    let mut reason = "未知崩溃".to_string();
    let mut category = CrashCategory::Unknown;
    let mut suggestion = "请查看崩溃报告获取详细信息".to_string();

    // 提取描述
    if let Some(desc_start) = content.find("---- Minecraft Crash Report ----") {
        let desc_section = &content[desc_start..];
        if let Some(desc_line) = desc_section.lines().find(|l| l.contains("Description:")) {
            reason = desc_line.replace("Description:", "").trim().to_string();
        }
    }

    // 检测类别
    let content_lower = content.to_lowercase();

    if content_lower.contains("optifine") {
        category = CrashCategory::OptiFine;
        suggestion = "请尝试移除OptiFine或更换兼容版本".to_string();
    } else if content_lower.contains("forge") || content_lower.contains("neoforge") {
        category = CrashCategory::Forge;
        suggestion = "请尝试重新安装Forge/NeoForge".to_string();
    } else if content_lower.contains("fabric") {
        category = CrashCategory::Fabric;
        suggestion = "请尝试重新安装Fabric".to_string();
    } else if content_lower.contains("outofmemoryerror") {
        category = CrashCategory::Memory;
        suggestion = "请增加最大内存分配".to_string();
    } else if content_lower.contains("opengl") || content_lower.contains("pixel format") {
        category = CrashCategory::Graphics;
        suggestion = "请更新显卡驱动".to_string();
    }

    Some(CrashInfo {
        reason,
        category,
        log_lines: content.lines().take(100).map(|l| l.to_string()).collect(),
        suggestion,
        problematic_mod: None,
    })
}
