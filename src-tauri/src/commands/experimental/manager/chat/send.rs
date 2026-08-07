use serde_json::Value;
use std::sync::atomic::Ordering;
use tauri::AppHandle;

use super::super::super::db;
use super::super::super::types::ChatSendParams;
use super::super::common::build_chat_turns;
use super::flow::{context, empty_reply, generate_reply, model_for_send, result, touch_and_save};
use crate::ai_core;
use crate::log_info;
use crate::state::AppState;

pub(crate) async fn chat_send(
    state: &AppState,
    app: &AppHandle,
    params: ChatSendParams,
) -> Result<Value, String> {
    if !db::conversation_exists(params.conversation_id)? {
        return Err("会话不存在，可能已被删除".to_string());
    }
    state.chat_cancel_flag.store(false, Ordering::Relaxed);
    let config = ai_core::load_config_async().await;
    let model = model_for_send(&params, &config)?;
    let ctx = context(state, app, params.conversation_id).await;
    let mut user_content = params.content.trim().to_string();
    if user_content.is_empty() {
        return Err("消息内容不能为空".to_string());
    }
    if let Some(attach) = params.attach_context.as_deref() {
        let attach = attach.trim();
        if !attach.is_empty() {
            user_content = format!(
                "【用户附带的上下文】\n{}\n\n---\n\n{}",
                attach, user_content
            );
        }
    }
    let is_first = db::list_messages(params.conversation_id, None)?.is_empty();
    let user_id = db::add_message(
        params.conversation_id,
        "user",
        &user_content,
        None,
        params.version_id.clone(),
        None,
        None,
        1,
        0,
        0,
        0,
        0,
    )?;
    let history = db::list_messages(
        params.conversation_id,
        Some(super::super::common::HISTORY_LIMIT),
    )?;
    let mut turns =
        build_chat_turns(app, &config, &model, params.conversation_id, &history).await?;
    let started = std::time::Instant::now();
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
    let reply = empty_reply(state, reply);
    let ai_id = touch_and_save(
        params.conversation_id,
        user_id,
        &reply,
        reasoning,
        model.clone(),
        1,
        &usage,
        duration,
        &records,
    )?;
    if is_first {
        super::super::emit::generate_title(
            app,
            params.conversation_id,
            &config,
            &model,
            &user_content,
            &reply,
        );
    }
    log_info!(
        "[Experimental] 聊天完成，耗时 {}ms，工具调用 {} 次",
        started.elapsed().as_millis(),
        tool_log.len()
    );
    result(params.conversation_id, ai_id, tool_log)
}
