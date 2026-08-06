//! AI 提示词构造（按场景提供 prompt）
//!
//! 提示词文本外置在 `resources/prompts/*.md`，经 `resources::embedded_text`
//! 编译期内嵌（include_str!），改文案后重新编译即可，无需硬编码在此。

use crate::resources;
use crate::utils::format::truncate_chars;

/// 提示词场景
pub enum PromptKind {
    /// 崩溃日志分析
    CrashLog,
    /// 文本摘要（上下文压缩）
    Summarize,
    /// 实验性 AI 聊天助手（Agent 对话）
    Chat,
    /// 会话标题生成
    Title,
    /// 日志分析（通用）
    LogAnalysis,
    /// 日志分析（AI 模式，5 环节逐步输出）
    LogAnalyzeSteps,
}

impl PromptKind {
    /// 对应 resources/prompts/ 下的模板文件名
    fn template_path(&self) -> &'static str {
        match self {
            PromptKind::CrashLog => "prompts/crash_log.md",
            PromptKind::Summarize => "prompts/summarize.md",
            PromptKind::Chat => "prompts/chat.md",
            PromptKind::Title => "prompts/title.md",
            PromptKind::LogAnalysis => "prompts/log_analysis.md",
            PromptKind::LogAnalyzeSteps => "prompts/log_analyze_steps.md",
        }
    }
}

/// 构造系统提示词（场景固定的角色/任务描述，来自外置模板）
pub fn system_prompt(kind: &PromptKind) -> String {
    resources::read_resource(kind.template_path()).unwrap_or_else(|e| {
        crate::log_warn!("[AI] 读取提示词模板失败（{}），使用内置兜底", e);
        fallback_prompt(kind)
    })
}

/// 构造聊天系统提示词（会话目录直接取用户消息文本，无需模型生成摘要标签）
pub fn chat_system_prompt() -> String {
    system_prompt(&PromptKind::Chat)
}

/// 内置兜底提示词（模板缺失时使用，避免功能完全不可用）
fn fallback_prompt(kind: &PromptKind) -> String {
    match kind {
        PromptKind::CrashLog => {
            "你是 Minecraft 启动器的资深技术支持专家。根据用户提供的崩溃报告与游戏日志，\
             准确诊断崩溃原因并给出可操作的中文修复建议。输出要求：使用 Markdown，结构为\
             「崩溃原因」与「修复建议」两节，简洁明了。禁止使用 Emoji 表情。"
                .to_string()
        }
        PromptKind::Summarize => {
            "你是一个简洁的文本摘要助手，用中文输出要点列表，不要使用 Emoji 表情。".to_string()
        }
        PromptKind::Chat => {
            "你是 MoLaunch 启动器的智能助手，可以回答用户关于 Minecraft、启动器使用、\
             崩溃日志、Mod 安装、联机、下载源等问题。当用户需要分析日志、崩溃报告或\
             排查问题时，请优先使用提供的工具获取真实数据，再给出判断。\
             回答使用简体中文，结构清晰，可使用 Markdown 排版。禁止使用 Emoji 表情。"
                .to_string()
        }
        PromptKind::Title => {
            "为对话生成一个简洁的中文标题，不超过 20 字，只输出标题本身，不要使用 Emoji。"
                .to_string()
        }
        PromptKind::LogAnalysis => {
            "你是 Minecraft 日志分析专家，请分析用户提供的日志，定位问题并给出解决建议。\
             使用 Markdown 输出，不要使用 Emoji 表情。".to_string()
        }
        PromptKind::LogAnalyzeSteps => {
            "你是 Minecraft 日志分析专家，擅长从游戏日志、崩溃报告、启动器日志中定位问题。\
             请按 5 个环节依次分析：读取整理日志、环境依赖检查、异常链定位、根因判断、修复建议。\
             每完成一个环节，先独占一行输出环节标记 【STEP:序号/5】，再输出该环节的简要分析；\
             全部完成后输出完整 Markdown 结论，包含「问题定位」与「解决建议」分节。\
             禁止使用 Emoji 表情，不要臆造不存在的错误。".to_string()
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
