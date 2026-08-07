use serde_json::Value;
use std::sync::atomic::Ordering;
use tauri::AppHandle;

use super::super::super::db;
use super::super::super::types::RegenerateReplyParams;
use super::super::common::{build_chat_turns, HISTORY_LIMIT};
use super::flow::{context, empty_reply, generate_reply, result, touch_and_save};
use crate::ai_core;
use crate::state::AppState;

pub(crate) async fn regenerate_reply(
    state: &AppState,
    app: &AppHandle,
    params: RegenerateReplyParams,
) -> Result<Value, String> {
    if !db::conversation_exists(params.conversation_id)? {
        return Err("会话不存在".to_string());
    }
    state.chat_cancel_flag.store(false, Ordering::Relaxed);
    let Some((_, role, _)) = db::get_message(params.conversation_id, params.message_id)? else {
        return Err("消息不存在".to_string());
    };
    if role != "assistant" {
        return Err("只能对 AI 消息重新回复".to_string());
    }
    let Some(user_id) = db::get_message_pair_id(params.conversation_id, params.message_id)? else {
        return Err("未找到该回复对应的用户消息".to_string());
    };
    let retry =
        db::get_message_retry_count(params.conversation_id, params.message_id)?.unwrap_or(1) + 1;
    db::delete_messages_after(params.conversation_id, user_id)?;
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
        user_id,
        &empty_reply(state, reply),
        reasoning,
        model,
        retry,
        &usage,
        duration,
        &records,
    )?;
    result(params.conversation_id, ai_id, tool_log)
}
