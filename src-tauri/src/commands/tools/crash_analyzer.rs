//! 崩溃日志分析
//!
//! - `analyze`：对游戏崩溃日志文本做大小写不敏感的模式匹配，识别常见崩溃原因
//!   （Java 版本 / 缺失 mod / 内存 / 驱动 / Mod 冲突 / 其他），返回结构化的分析条目供前端展示。
//!
//! 本模块为纯文本分析，不读取文件系统，不使用 state 参数（签名保持统一）。

use crate::log_info;
use crate::state::AppState;

use super::types::{CrashAnalyzeParams, CrashAnalyzeResult, CrashAnalysisItem};

/// 分析崩溃日志文本，识别常见崩溃模式
///
/// 匹配规则（大小写不敏感）：
/// - Java 版本不匹配：`UnsupportedClassVersionError` / `Unsupported class file major version`
///   / `has been compiled by a more recent version`
/// - 缺失 mod：`NoClassDefFoundError` / `NoSuchMethodError` / `FileNotFoundException` 且路径含 mods
/// - 内存不足：`OutOfMemoryError` / `OutOfMemory`
/// - 显卡驱动：`GLFW` / `OpenGL` / `Pixel format` / `driver`
/// - Mod 冲突：`MixinApplyError` / `Duplicate` / `Conflicting`
/// - 其他：含 `Exception` / `Error` / `Crash` 的通用条目（仅当未命中上述具体分类时）
///
/// 空文本返回空 analyses。
pub async fn analyze(
    state: &AppState,
    params: CrashAnalyzeParams,
) -> Result<serde_json::Value, String> {
    let _ = state; // 纯文本分析，不使用 state
    let log_text = params.log_text;

    if log_text.trim().is_empty() {
        let result = CrashAnalyzeResult {
            analyses: Vec::new(),
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    log_info!(
        "[CrashAnalyzer] 开始分析日志，长度 {} 字节",
        log_text.len()
    );

    let lower = log_text.to_lowercase();
    let lines: Vec<&str> = log_text.lines().collect();
    let mut analyses: Vec<CrashAnalysisItem> = Vec::new();

    // 1. Java 版本不匹配
    if lower.contains("unsupported class file major version")
        || lower.contains("has been compiled by a more recent version")
        || lower.contains("java.lang.unsupportedclassversionerror")
    {
        analyses.push(CrashAnalysisItem {
            category: "java_version".to_string(),
            severity: "error".to_string(),
            title: "Java 版本不匹配".to_string(),
            detail: find_relevant_line(&lines, "UnsupportedClassVersionError"),
            suggestion: "当前 Java 版本与 mod / 游戏所需的版本不一致。请在启动器设置中切换到更高（或更低）的 Java 版本，通常较新的 mod 需要更新版本的 Java。".to_string(),
        });
    }

    // 2. 缺失 mod
    if lower.contains("noclassdeffounderror")
        || lower.contains("nosuchmethoderror")
        || (lower.contains("filenotfoundexception") && lower.contains("mods"))
    {
        let detail = if lower.contains("noclassdeffounderror") {
            find_relevant_line(&lines, "NoClassDefFoundError")
        } else if lower.contains("nosuchmethoderror") {
            find_relevant_line(&lines, "NoSuchMethodError")
        } else {
            find_relevant_line(&lines, "FileNotFoundException")
        };
        analyses.push(CrashAnalysisItem {
            category: "missing_mod".to_string(),
            severity: "error".to_string(),
            title: "缺失 Mod 或 Mod 文件损坏".to_string(),
            detail,
            suggestion: "游戏引用的类 / 方法找不到，通常是缺少前置 mod 或 mod 文件损坏。请检查是否漏装了所需的前置 mod，或重新下载相关 mod。".to_string(),
        });
    }

    // 3. 内存不足
    if lower.contains("outofmemoryerror") || lower.contains("outofmemory") {
        analyses.push(CrashAnalysisItem {
            category: "memory".to_string(),
            severity: "error".to_string(),
            title: "内存不足".to_string(),
            detail: find_relevant_line(&lines, "OutOfMemory"),
            suggestion: "JVM 堆内存不足导致崩溃。请在启动器版本设置中增大 Java 内存分配（如 4096MB 或更高），并确保电脑物理内存充足。".to_string(),
        });
    }

    // 4. 显卡驱动
    if lower.contains("glfw")
        || lower.contains("opengl")
        || lower.contains("pixel format")
        || lower.contains("driver")
    {
        analyses.push(CrashAnalysisItem {
            category: "driver".to_string(),
            severity: "warning".to_string(),
            title: "显卡驱动 / 图形库问题".to_string(),
            detail: find_relevant_line(&lines, "GLFW"),
            suggestion: "图形相关错误，可能是显卡驱动过旧或不兼容。请更新显卡驱动到最新版本，或在启动器中尝试切换使用的显卡（集成显卡 / 独立显卡）。".to_string(),
        });
    }

    // 5. Mod 冲突
    if lower.contains("mixinapplyerror")
        || lower.contains("duplicate")
        || lower.contains("conflicting")
    {
        let detail = if lower.contains("mixinapplyerror") {
            find_relevant_line(&lines, "MixinApplyError")
        } else if lower.contains("duplicate") {
            find_relevant_line(&lines, "Duplicate")
        } else {
            find_relevant_line(&lines, "Conflicting")
        };
        analyses.push(CrashAnalysisItem {
            category: "mod_conflict".to_string(),
            severity: "warning".to_string(),
            title: "Mod 冲突".to_string(),
            detail,
            suggestion: "检测到 mod 之间存在冲突（如重复注册、Mixin 冲突）。请尝试逐个排查最近安装的 mod，移除冲突项或查看 mod 兼容性说明。".to_string(),
        });
    }

    // 6. 其他通用错误（仅当未命中具体分类时）
    if analyses.is_empty()
        && (lower.contains("exception") || lower.contains("error") || lower.contains("crash"))
    {
        let detail = if lower.contains("exception") {
            find_relevant_line(&lines, "Exception")
        } else if lower.contains("error") {
            find_relevant_line(&lines, "Error")
        } else {
            find_relevant_line(&lines, "Crash")
        };
        analyses.push(CrashAnalysisItem {
            category: "other".to_string(),
            severity: "info".to_string(),
            title: "检测到通用错误信息".to_string(),
            detail,
            suggestion: "日志中包含异常信息但未识别出具体模式。建议查看完整崩溃日志，或在社区反馈时附上完整日志以便进一步诊断。".to_string(),
        });
    }

    log_info!(
        "[CrashAnalyzer] 分析完成，识别 {} 条问题",
        analyses.len()
    );

    let result = CrashAnalyzeResult { analyses };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 在日志行中查找首个包含 needle（大小写不敏感）的非空行，返回其截断片段
///
/// 找不到时回退到首个非空行；都没有则返回空字符串。
fn find_relevant_line(lines: &[&str], needle: &str) -> String {
    let needle_l = needle.to_lowercase();
    for line in lines {
        if line.to_lowercase().contains(&needle_l) {
            let t = line.trim();
            if !t.is_empty() {
                return truncate(t, 300);
            }
        }
    }
    // 回退：首个非空行
    for line in lines {
        let t = line.trim();
        if !t.is_empty() {
            return truncate(t, 300);
        }
    }
    String::new()
}

/// 将字符串按字符数截断，超长时追加省略号
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push('…');
        out
    }
}
