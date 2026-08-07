use serde_json::Value;
use std::sync::atomic::Ordering;
use tauri::AppHandle;

use super::super::super::agent;
use super::super::super::db;
use super::super::super::types::EditMessageParams;
use super::super::common::{build_chat_turns, HISTORY_LIMIT};
use super::flow::{context, empty_reply, generate_reply, result, touch_and_save};
use crate::ai_core;
use crate::state::AppState;

pub(crate) async fn edit_message(
    state: &AppState,
    app: &AppHandle,
    params: EditMessageParams,
) -> Result<Value, String> {
    if !db::conversation_exists(params.conversation_id)? {
        return Err("会话不存在".to_string());
    }
    state.chat_cancel_flag.store(false, Ordering::Relaxed);
    let Some((_, role, _)) = db::get_message(params.conversation_id, params.message_id)? else {
        return Err("消息不存在".to_string());
    };
    if role != "user" {
        return Err("只能编辑用户发送的消息".to_string());
    }
    if db::last_user_message_id(params.conversation_id)? != Some(params.message_id) {
        return Err("只能编辑最近一条消息，更早的消息请先删除到该条".to_string());
    }
    let content = params.content.trim().to_string();
    if content.is_empty() {
        return Err("消息内容不能为空".to_string());
    }
    db::update_message_content(params.message_id, &content)?;
    db::delete_messages_after(params.conversation_id, params.message_id)?;
    let config = ai_core::load_config_async().await;
    let model = super::flow::resolve_model_override(&params.model, &config)?;
    let ctx = context(state, app, params.conversation_id).await;
    let history = db::list_messages(params.conversation_id, Some(HISTORY_LIMIT))?;
    let mut turns =
        build_chat_turns(app, &config, &model, params.conversation_id, &history).await?;
    let (reply, reasoning, tool_log, records, usage, duration) = generate_reply(
        state,
        app,
        params.conversation_id,
        &config,
        &model,
        params.reasoning_effort.as_deref(),
        &ctx,
        &mut turns,
    )
    .await?;
    let ai_id = touch_and_save(
        params.conversation_id,
        params.message_id,
        &empty_reply(state, reply),
        reasoning,
        model,
        1,
        &usage,
        duration,
        &records,
    )?;
    result(params.conversation_id, ai_id, tool_log)
}

#[allow(dead_code)]
fn _keep_agent_module_visible(_: &agent::AgentContext) {}
