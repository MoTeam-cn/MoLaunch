//! 流式事件与标题生成
//!
//! `emit_chat_done` 在全部工具轮次结束后统一推送一次完成事件，
//! `emit_chat_status` 推送聊天状态（如上下文压缩提示），
//! `generate_title` 首条消息后非阻塞调用模型生成会话标题。

use serde_json::json;
use tauri::{AppHandle, Emitter};

use super::super::db;
use crate::ai_core;
use crate::ai_core::client::{ChatTurn, StreamUsage};
use crate::log_warn;

/// 推送一次流式完成事件（全部工具轮次结束后由调用方统一调用）
pub(super) fn emit_chat_done(app: &AppHandle, conv_id: i64, usage: &StreamUsage, duration_ms: u64) {
    let _ = app.emit(
        "ai-chat-stream",
        json!({
            "conversationId": conv_id,
            "done": true,
            "usage": usage,
            "durationMs": duration_ms
        }),
    );
}

/// 推送聊天状态事件（如上下文压缩提示）
pub(super) fn emit_chat_status(app: &AppHandle, conversation_id: i64, message: &str) {
    let _ = app.emit(
        "ai-chat-stream",
        json!({
            "conversationId": conversation_id,
            "status": message
        }),
    );
}

/// 生成会话标题（模型生成，≤20 字；非阻塞）
pub(super) fn generate_title(
    app: &AppHandle,
    conversation_id: i64,
    config: &ai_core::AiConfig,
    model: &str,
    user_content: &str,
    reply: &str,
) {
    let app = app.clone();
    let config = config.clone();
    let model = model.to_string();
    let user_content = user_content.to_string();
    let reply = reply.to_string();
    tauri::async_runtime::spawn(async move {
        let title_prompt = ai_core::prompt::system_prompt(&ai_core::PromptKind::Title);
        let user_msg = format!(
            "用户消息：{}\n\nAI 回复（开头）：{}",
            crate::utils::format::truncate_chars(&user_content, 200),
            crate::utils::format::truncate_chars(&reply, 200)
        );
        let turns = vec![
            ChatTurn {
                role: "system".to_string(),
                content: Some(title_prompt),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            ChatTurn {
                role: "user".to_string(),
                content: Some(user_msg.clone()),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        ];
        let _ = turns;
        match ai_core::chat(
            &config,
            ai_core::PromptKind::Title,
            user_msg.clone(),
            Some(&model),
        )
        .await
        {
            Ok(title) => {
                let title = title
                    .trim()
                    .trim_matches('"')
                    .chars()
                    .take(20)
                    .collect::<String>();
                if !title.is_empty() {
                    let _ = db::rename_conversation(conversation_id, &title);
                    let _ = app.emit(
                        "conversation-title-updated",
                        json!({ "conversationId": conversation_id, "title": title }),
                    );
                }
            }
            Err(e) => log_warn!("[Experimental] 生成会话标题失败: {}", e),
        }
        let _ = turns;
    });
}
