//! AI 提示词构造（按场景提供 prompt）

use crate::utils::format::truncate_chars;

/// 提示词场景
pub enum PromptKind {
    /// 崩溃日志分析
    CrashLog,
    /// 文本摘要（保留给未来场景扩展）
    Summarize,
}

/// 构造系统提示词（场景固定的角色/任务描述）
pub fn system_prompt(kind: &PromptKind) -> String {
    match kind {
        PromptKind::CrashLog => {
            "你是 Minecraft 启动器的资深技术支持专家。\
             根据用户提供的崩溃报告与游戏日志，准确诊断崩溃原因并给出可操作的中文修复建议。\
             输出要求：使用 Markdown，结构为「崩溃原因」与「修复建议」两节，简洁明了。"
                .to_string()
        }
        PromptKind::Summarize => {
            "你是一个简洁的文本摘要助手，用中文输出要点列表。".to_string()
        }
    }
}

/// 构造用户消息（崩溃场景：拼接收集到的多源日志文本）
pub fn crash_user_prompt(
    runtime_log: &str,
    error_lines: &[String],
    crash_report: &str,
    hs_err: &str,
) -> String {
    let mut parts = Vec::new();
    if !crash_report.is_empty() {
        parts.push(format!("【崩溃报告】\n{}", truncate_chars(crash_report, 6000)));
    }
    if !hs_err.is_empty() {
        parts.push(format!("【JVM 崩溃日志】\n{}", truncate_chars(hs_err, 4000)));
    }
    let errs = error_lines.join("\n");
    if !errs.is_empty() {
        parts.push(format!("【错误级别日志】\n{}", truncate_chars(&errs, 4000)));
    }
    let tail = truncate_chars(runtime_log, 6000);
    if !tail.is_empty() {
        parts.push(format!("【游戏运行日志尾部】\n{}", tail));
    }
    if parts.is_empty() {
        return "（无可用日志）".to_string();
    }
    parts.join("\n\n---\n\n")
}
