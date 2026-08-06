//! 聊天动作：发送 / 重新回复 / 编辑消息 / ask_user 回填
//!
//! 三个动作共用同一套「历史构造 → 上下文压缩 → 工具循环 → 落库」流程，
//! 差异仅在起点：chat_send 新建用户消息，regenerate/edit 复用既有消息。

use serde_json::{json, Value};
use std::sync::atomic::Ordering;
use tauri::AppHandle;

use super::super::agent;
use super::super::db;
use super::super::types::{
    ChatSendParams, EditMessageParams, RegenerateReplyParams, ReplyAskUserParams,
};
use super::common::{build_chat_turns, build_context, HISTORY_LIMIT};
use super::context::resolve_chat_model;
use super::emit::{emit_chat_done, generate_title};
use super::tool_loop::run_tool_loop;
use crate::ai_core;
use crate::log_info;
use crate::state::AppState;

/// 聊天发送（流式）：保存用户消息 → 携带历史 + 工具发起流式对话 → 保存回复
pub(super) async fn chat_send(
    state: &AppState,
    app: &AppHandle,
    params: ChatSendParams,
) -> Result<Value, String> {
    if !db::conversation_exists(params.conversation_id)? {
        return Err("会话不存在，可能已被删除".to_string());
    }
    // 每次发起新对话前重置取消信号（避免上一次暂停状态残留）
    state.chat_cancel_flag.store(false, Ordering::Relaxed);

    let config = ai_core::load_config_async().await;
    let model = resolve_chat_model(&params, &config)?;
    let ctx = build_context(state, app, params.conversation_id).await;

    // 拼接手动附加的上下文（模型不支持工具调用时的兜底）
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

    // 记录用户消息（先查是否首条，用于自动生成标题）
    let existing = db::list_messages(params.conversation_id, None)?;
    let is_first_message = existing.is_empty();

    // 保存用户消息（pair_id 在生成 AI 回复后回填）
    let user_msg_id = db::add_message(
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

    // 构造消息历史（含刚写入的用户消息）
    let history = db::list_messages(params.conversation_id, Some(HISTORY_LIMIT))?;

    // 上下文压缩管线（触发判定 → L1/L3 → 重塑），未压缩时直构历史并注入工具轮次
    let mut turns =
        build_chat_turns(app, &config, &model, params.conversation_id, &history).await?;

    let tools = agent::tool_definitions();

    let started = std::time::Instant::now();
    let (mut reply, reasoning, tool_log, tool_records, usage, duration_ms) = run_tool_loop(
        &config,
        app,
        params.conversation_id,
        &model,
        params.reasoning_effort.as_deref(),
        &tools,
        &ctx,
        &mut turns,
        Some(&state.chat_cancel_flag),
    )
    .await?;
    emit_chat_done(app, params.conversation_id, &usage, duration_ms);

    if reply.trim().is_empty() {
        reply = if state.chat_cancel_flag.load(Ordering::Relaxed) {
            "（已停止生成）".to_string()
        } else {
            "（模型未生成有效回复，请检查服务状态或重试）".to_string()
        };
    }

    // 保存 AI 回复，回填配对 id；记录实际生成该回复的模型名
    let ai_msg_id = db::add_message(
        params.conversation_id,
        "assistant",
        &reply,
        Some(user_msg_id),
        None,
        reasoning,
        Some(model.clone()),
        1,
        usage.prompt_tokens,
        usage.completion_tokens,
        usage.total_tokens,
        duration_ms,
    )?;
    // 持久化工具调用记录（绑定到 AI 回复消息，刷新/重启后工具链仍保留）
    db::add_tool_calls(params.conversation_id, ai_msg_id, &tool_records)?;
    // 回填用户消息的 pair_id
    db::set_message_pair_id(user_msg_id, ai_msg_id)?;
    db::touch_conversation(params.conversation_id)?;

    // 首条消息：模型生成标题（非阻塞）
    if is_first_message {
        generate_title(
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

    serde_json::to_value(json!({
        "conversationId": params.conversation_id,
        "messageId": ai_msg_id,
        "toolCallsLog": tool_log
    }))
    .map_err(|e| e.to_string())
}

/// 解析本次重新生成/编辑使用的模型（未指定时用默认模型）
fn resolve_model_override(
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

/// 重新回复：找到该 AI 消息对应的用户消息，删除其后消息并重新生成
pub(super) async fn regenerate_reply(
    state: &AppState,
    app: &AppHandle,
    params: RegenerateReplyParams,
) -> Result<Value, String> {
    if !db::conversation_exists(params.conversation_id)? {
        return Err("会话不存在".to_string());
    }
    // 每次重新生成前重置取消信号（避免上一次暂停状态残留）
    state.chat_cancel_flag.store(false, Ordering::Relaxed);
    let Some((_, role, _)) = db::get_message(params.conversation_id, params.message_id)? else {
        return Err("消息不存在".to_string());
    };
    if role != "assistant" {
        return Err("只能对 AI 消息重新回复".to_string());
    }

    // 找到该 AI 消息对应用户消息（AI 消息的 pair_id 指向用户消息）
    let user_msg_id =
        db::get_message(params.conversation_id, params.message_id)?.and_then(|(_, role, _)| {
            if role == "assistant" {
                db::get_message_pair_id(params.conversation_id, params.message_id)
                    .ok()
                    .flatten()
            } else {
                None
            }
        });

    let Some(user_id) = user_msg_id else {
        return Err("未找到该回复对应的用户消息".to_string());
    };

    // 读取旧 AI 回复的生成序号，重新生成时递增（「第 N 次重试」标识）
    let old_retry_count =
        db::get_message_retry_count(params.conversation_id, params.message_id)?.unwrap_or(1);
    let new_retry_count = old_retry_count + 1;

    // 删除该用户消息之后的所有消息（含该 AI 回复）
    db::delete_messages_after(params.conversation_id, user_id)?;

    // 重新生成（复用 chat_send 的流式核心，但用户消息已存在）
    let config = ai_core::load_config_async().await;
    let model = resolve_model_override(&params.model, &config)?;

    let ctx = build_context(state, app, params.conversation_id).await;
    let history = db::list_messages(params.conversation_id, Some(HISTORY_LIMIT))?;
    let mut turns =
        build_chat_turns(app, &config, &model, params.conversation_id, &history).await?;

    let tools = agent::tool_definitions();

    let (mut reply, reasoning, tool_log, tool_records, usage, duration_ms) = run_tool_loop(
        &config,
        app,
        params.conversation_id,
        &model,
        params.reasoning_effort.as_deref(),
        &tools,
        &ctx,
        &mut turns,
        Some(&state.chat_cancel_flag),
    )
    .await?;
    emit_chat_done(app, params.conversation_id, &usage, duration_ms);

    if reply.trim().is_empty() {
        reply = if state.chat_cancel_flag.load(Ordering::Relaxed) {
            "（已停止生成）".to_string()
        } else {
            "（模型未生成有效回复，请检查服务状态或重试）".to_string()
        };
    }

    // 保存 AI 回复，回填配对 id（重新生成后绑定到原用户消息）
    let ai_msg_id = db::add_message(
        params.conversation_id,
        "assistant",
        &reply,
        Some(user_id),
        None,
        reasoning,
        Some(model.clone()),
        new_retry_count,
        usage.prompt_tokens,
        usage.completion_tokens,
        usage.total_tokens,
        duration_ms,
    )?;
    // 持久化工具调用记录（绑定到新回复，编辑/重新回复会先清理旧链）
    db::add_tool_calls(params.conversation_id, ai_msg_id, &tool_records)?;
    db::set_message_pair_id(user_id, ai_msg_id)?;
    db::touch_conversation(params.conversation_id)?;

    serde_json::to_value(json!({
        "conversationId": params.conversation_id,
        "messageId": ai_msg_id,
        "toolCallsLog": tool_log
    }))
    .map_err(|e| e.to_string())
}

/// 编辑消息：仅最近一条用户消息可编辑；编辑后删除其后消息并重新生成
pub(super) async fn edit_message(
    state: &AppState,
    app: &AppHandle,
    params: EditMessageParams,
) -> Result<Value, String> {
    if !db::conversation_exists(params.conversation_id)? {
        return Err("会话不存在".to_string());
    }
    // 每次编辑重新生成前重置取消信号（避免上一次暂停状态残留）
    state.chat_cancel_flag.store(false, Ordering::Relaxed);
    let Some((_, role, _)) = db::get_message(params.conversation_id, params.message_id)? else {
        return Err("消息不存在".to_string());
    };
    if role != "user" {
        return Err("只能编辑用户发送的消息".to_string());
    }
    // 仅最近一条用户消息可编辑（除非前面已删除）
    let last_user = db::last_user_message_id(params.conversation_id)?;
    if last_user != Some(params.message_id) {
        return Err("只能编辑最近一条消息，更早的消息请先删除到该条".to_string());
    }
    let content = params.content.trim().to_string();
    if content.is_empty() {
        return Err("消息内容不能为空".to_string());
    }

    // 更新内容并删除其后消息
    db::update_message_content(params.message_id, &content)?;
    db::delete_messages_after(params.conversation_id, params.message_id)?;

    // 自动重新生成回复（以该用户消息为起点）
    let config = ai_core::load_config_async().await;
    let model = resolve_model_override(&params.model, &config)?;

    let ctx = build_context(state, app, params.conversation_id).await;
    let history = db::list_messages(params.conversation_id, Some(HISTORY_LIMIT))?;
    let mut turns =
        build_chat_turns(app, &config, &model, params.conversation_id, &history).await?;

    let tools = agent::tool_definitions();

    let (mut reply, reasoning, tool_log, tool_records, usage, duration_ms) = run_tool_loop(
        &config,
        app,
        params.conversation_id,
        &model,
        params.reasoning_effort.as_deref(),
        &tools,
        &ctx,
        &mut turns,
        Some(&state.chat_cancel_flag),
    )
    .await?;
    emit_chat_done(app, params.conversation_id, &usage, duration_ms);

    if reply.trim().is_empty() {
        reply = if state.chat_cancel_flag.load(Ordering::Relaxed) {
            "（已停止生成）".to_string()
        } else {
            "（模型未生成有效回复，请检查服务状态或重试）".to_string()
        };
    }

    // 保存 AI 回复，回填配对 id（编辑后绑定到编辑的用户消息）
    let ai_msg_id = db::add_message(
        params.conversation_id,
        "assistant",
        &reply,
        Some(params.message_id),
        None,
        reasoning,
        Some(model.clone()),
        1,
        usage.prompt_tokens,
        usage.completion_tokens,
        usage.total_tokens,
        duration_ms,
    )?;
    // 持久化工具调用记录（绑定到新回复，编辑后 delete_messages_after 已清理旧链）
    db::add_tool_calls(params.conversation_id, ai_msg_id, &tool_records)?;
    db::set_message_pair_id(params.message_id, ai_msg_id)?;
    db::touch_conversation(params.conversation_id)?;

    serde_json::to_value(json!({
        "conversationId": params.conversation_id,
        "messageId": ai_msg_id,
        "toolCallsLog": tool_log
    }))
    .map_err(|e| e.to_string())
}

/// 回填 ask_user 提问结果（委托 agent::reply_ask_user 处理等待队列）
pub(super) async fn reply_ask_user(params: ReplyAskUserParams) -> Result<Value, String> {
    agent::reply_ask_user(params.conversation_id, params.reply).await?;
    serde_json::to_value(()).map_err(|e| e.to_string())
}
