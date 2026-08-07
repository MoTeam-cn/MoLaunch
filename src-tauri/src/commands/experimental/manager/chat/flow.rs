use serde_json::{json, Value};
use std::sync::atomic::Ordering;
use tauri::AppHandle;

use super::super::super::agent;
use super::super::super::db;
use super::super::super::types::MessageItem;
use super::super::common::build_context;
use super::super::context::resolve_chat_model;
use super::super::emit::emit_chat_done;
use super::super::tool_loop::run_tool_loop;
use crate::ai_core;
use crate::state::AppState;

pub(super) fn resolve_model_override(
    params_model: &Option<String>,
    config: &ai_core::AiConfig,
) -> Result<String, String> {
    let model = if let Some(m) = params_model.as_deref() {
        if m.trim().is_empty() {
            config.resolve_model(None)
        } else {
            m.trim().to_string()
        }
    } else {
        config.resolve_model(None)
    };
    if model.is_empty() {
        return Err("未启用任何模型".to_string());
    }
    Ok(model)
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn generate_reply(
    state: &AppState,
    app: &AppHandle,
    conversation_id: i64,
    config: &ai_core::AiConfig,
    model: &str,
    reasoning_effort: Option<&str>,
    ctx: &super::super::super::agent::AgentContext,
    turns: &mut Vec<crate::ai_core::client::ChatTurn>,
) -> Result<
    (
        String,
        Option<String>,
        Vec<String>,
        Vec<crate::commands::experimental::types::ToolCallRecord>,
        crate::ai_core::client::StreamUsage,
        u64,
    ),
    String,
> {
    let tools = agent::tool_definitions();
    let result = run_tool_loop(
        config,
        app,
        conversation_id,
        model,
        reasoning_effort,
        &tools,
        ctx,
        turns,
        Some(&state.chat_cancel_flag),
    )
    .await?;
    emit_chat_done(app, conversation_id, &result.4, result.5);
    Ok(result)
}

pub(super) fn empty_reply(state: &AppState, mut reply: String) -> String {
    if reply.trim().is_empty() {
        reply = if state.chat_cancel_flag.load(Ordering::Relaxed) {
            "（已停止生成）".to_string()
        } else {
            "（模型未生成有效回复，请检查服务状态或重试）".to_string()
        };
    }
    reply
}

pub(super) fn result(
    conversation_id: i64,
    message_id: i64,
    tool_log: Vec<String>,
) -> Result<Value, String> {
    serde_json::to_value(json!({"conversationId": conversation_id, "messageId": message_id, "toolCallsLog": tool_log})).map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn touch_and_save(
    conversation_id: i64,
    user_id: i64,
    reply: &str,
    reasoning: Option<String>,
    model: String,
    retry_count: i64,
    usage: &crate::ai_core::client::StreamUsage,
    duration_ms: u64,
    tool_records: &[crate::commands::experimental::types::ToolCallRecord],
) -> Result<i64, String> {
    let ai_id = db::add_message(
        conversation_id,
        "assistant",
        reply,
        Some(user_id),
        None,
        reasoning,
        Some(model),
        retry_count,
        usage.prompt_tokens,
        usage.completion_tokens,
        usage.total_tokens,
        duration_ms,
    )?;
    db::add_tool_calls(conversation_id, ai_id, tool_records)?;
    db::set_message_pair_id(user_id, ai_id)?;
    db::touch_conversation(conversation_id)?;
    Ok(ai_id)
}

pub(super) async fn context(
    state: &AppState,
    app: &AppHandle,
    conversation_id: i64,
) -> super::super::super::agent::AgentContext {
    build_context(state, app, conversation_id).await
}

pub(super) fn model_for_send(
    params: &super::super::super::types::ChatSendParams,
    config: &ai_core::AiConfig,
) -> Result<String, String> {
    resolve_chat_model(params, config)
}

pub(super) fn _message_content(message: &MessageItem) -> &str {
    &message.content
}
