//! 聊天上下文构建（模型解析 / turns 构造 / 占用估算）
//!
//! 压缩管线见 `super::compression`（触发判定 → L1/L3 → 重塑器）。

use crate::ai_core;
use crate::ai_core::client::{estimate_tokens, ChatTurn};
use crate::commands::experimental::types::{ChatSendParams, MessageItem};

/// 加载模型配置并解析本次对话使用的模型
pub(super) fn resolve_chat_model(
    params: &ChatSendParams,
    config: &ai_core::AiConfig,
) -> Result<String, String> {
    if config.base_url.trim().is_empty() {
        return Err(
            "未配置 AI 服务地址，请先在「实验性 → AI 设置」中配置本地 OpenAI 兼容服务".to_string(),
        );
    }
    let model = if let Some(m) = params.model.as_deref() {
        if m.trim().is_empty() {
            ""
        } else {
            m.trim()
        }
    } else {
        ""
    };
    let model = if model.is_empty() {
        config.resolve_model(None)
    } else {
        model.to_string()
    };
    if model.is_empty() {
        return Err("未启用任何模型，请先在「实验性 → AI 设置」中加载并启用模型".to_string());
    }
    Ok(model)
}

/// 从数据库历史构造对话 turns（含系统提示词）
pub fn build_turns(history: &[MessageItem], with_system: bool) -> Vec<ChatTurn> {
    let mut turns: Vec<ChatTurn> = history
        .iter()
        .map(|m| ChatTurn::plain(m.role.clone(), m.content.clone()))
        .collect();
    if with_system {
        turns.insert(
            0,
            ChatTurn::plain("system", ai_core::prompt::chat_system_prompt()),
        );
    }
    turns
}

/// 估算当前上下文真实占用：倒序查找最新一条带真实 usage 的 AI 消息，
/// 以其 prompt_tokens 为基准加上其后新增消息的字符估算；无 usage 时退化为全量字符估算。
pub fn estimate_context_usage(history: &[MessageItem]) -> u64 {
    let mut extra: u64 = 0;
    for m in history.iter().rev() {
        if m.role == "assistant" {
            if let Some(t) = m.prompt_tokens {
                if t > 0 {
                    return t as u64 + extra;
                }
            }
        }
        extra += estimate_tokens(m.content.as_str());
        extra += estimate_tokens(m.reasoning_content.as_deref().unwrap_or(""));
    }
    extra
}
