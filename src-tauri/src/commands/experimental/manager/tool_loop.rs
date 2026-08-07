//! 工具调用循环（多轮「模型请求 → 工具执行 → 回填 tool turns」）
//!
//! 统一了 chat_send / regenerate / edit 三处的流式 + 工具循环逻辑，
//! 边流式边推送增量，最多 `MAX_TOOL_ITERATIONS` 轮防死循环。

use serde_json::{json, Value};
use std::sync::atomic::AtomicBool;
use tauri::{AppHandle, Emitter};

use super::super::agent::{self, AgentContext};
use crate::ai_core;
use crate::ai_core::client::{ChatResult, StreamCallbacks, StreamUsage, ToolDef};
use crate::commands::experimental::types::ToolCallRecord;
use crate::{log_info, log_warn};

/// 单次对话工具循环最大迭代轮数（防止工具死循环）
const MAX_TOOL_ITERATIONS: usize = 4;

/// 解析模型返回的工具调用参数 JSON（逐步容错）
pub(super) fn parse_tool_arguments(raw: &str) -> Value {
    let trimmed = raw.trim();
    let mut candidates: Vec<&str> = Vec::new();
    candidates.push(trimmed);
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if start < end {
            candidates.push(&trimmed[start..=end]);
        }
    }
    for text in &candidates {
        if let Ok(v) = serde_json::from_str::<Value>(text) {
            if let Some(inner) = v.as_str() {
                if let Ok(v2) = serde_json::from_str::<Value>(inner) {
                    return v2;
                }
            }
            return v;
        }
    }
    Value::Null
}

/// 执行多轮「模型请求 → 工具调用 → 回填 tool turns」循环（最多 `MAX_TOOL_ITERATIONS` 轮）
///
/// 返回 `(最终回复, 思考内容, 工具执行日志, 工具调用记录, 累计 usage, 总生成耗时 ms)`。
#[allow(clippy::too_many_arguments)]
pub(super) async fn run_tool_loop(
    config: &ai_core::AiConfig,
    app: &AppHandle,
    conv_id: i64,
    model: &str,
    reasoning_effort: Option<&str>,
    tools: &[ToolDef],
    ctx: &AgentContext,
    turns: &mut Vec<crate::ai_core::client::ChatTurn>,
    cancelled: Option<&AtomicBool>,
) -> Result<
    (
        String,
        Option<String>,
        Vec<String>,
        Vec<ToolCallRecord>,
        StreamUsage,
        u64,
    ),
    String,
> {
    let loop_started = std::time::Instant::now();
    let mut reply = String::new();
    let mut reasoning = String::new();
    let mut tool_log: Vec<String> = Vec::new();
    let mut tool_records: Vec<ToolCallRecord> = Vec::new();
    let mut empty_followup_retried = false;
    let usage_cell = std::sync::Arc::new(std::sync::Mutex::new(StreamUsage::default()));

    let app_delta = app.clone();
    let app_reasoning = app.clone();
    let usage_capture = usage_cell.clone();
    let stream_callbacks = StreamCallbacks {
        on_delta: Box::new(move |delta: &str| {
            let _ = app_delta.emit(
                "ai-chat-stream",
                json!({ "conversationId": conv_id, "delta": delta, "done": false }),
            );
        }),
        on_reasoning_delta: Box::new(move |delta: &str| {
            let _ = app_reasoning.emit(
                "ai-chat-stream",
                json!({ "conversationId": conv_id, "reasoning": delta }),
            );
        }),
        on_tool_delta: Box::new(|_delta: &crate::ai_core::client::StreamToolDelta| {}),
        on_done: Box::new(move |u: &StreamUsage| {
            if let Ok(mut acc) = usage_capture.lock() {
                acc.prompt_tokens += u.prompt_tokens;
                acc.completion_tokens += u.completion_tokens;
                acc.total_tokens += u.total_tokens;
            }
        }),
    };

    for round in 0..MAX_TOOL_ITERATIONS {
        let result: ChatResult = ai_core::chat_completions_stream(
            config,
            turns.clone(),
            Some(tools),
            Some(model),
            reasoning_effort,
            &stream_callbacks,
            cancelled,
        )
        .await
        .map_err(|e| {
            log_warn!("[Experimental] 聊天请求失败: {}", e);
            format!("AI 请求失败: {}", e)
        })?;

        if let Some(r) = result.reasoning_content.as_deref() {
            let r = r.trim();
            if !r.is_empty() {
                if !reasoning.is_empty() {
                    reasoning.push('\n');
                }
                reasoning.push_str(r);
            }
        }
        if let Some(content) = result.content.as_deref() {
            if !content.trim().is_empty() {
                reply = content.to_string();
            }
        }
        if result.tool_calls.is_empty() {
            let empty_reply = result
                .content
                .as_deref()
                .map(str::trim)
                .map(|content| content.is_empty())
                .unwrap_or(true);
            if empty_reply && !tool_records.is_empty() && !empty_followup_retried {
                empty_followup_retried = true;
                log_warn!("[Experimental] 工具执行后模型返回空回复，重试一次最终回答");
                continue;
            }
            break;
        }

        let pre_content = result
            .content
            .as_deref()
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());

        // 工具调用的轮次必须完整回传 reasoning_content，否则服务返回 400
        let assistant_turn = crate::ai_core::client::ChatTurn {
            role: "assistant".to_string(),
            content: result.content.clone(),
            reasoning_content: result.reasoning_content.clone(),
            tool_calls: Some(result.tool_calls.clone()),
            tool_call_id: None,
            name: None,
        };
        let mut tool_turns: Vec<crate::ai_core::client::ChatTurn> = Vec::new();
        for (tool_idx, call) in result.tool_calls.iter().enumerate() {
            let args: Value = parse_tool_arguments(&call.function.arguments);
            tool_log.push(format!("{} {}", call.function.name, args));
            let tool_seq = format!("r{}-{}", round, tool_idx);
            let mut running_event = json!({
                "conversationId": conv_id,
                "toolCall": {
                    "name": call.function.name,
                    "status": "running",
                    "index": tool_seq,
                    "arguments": call.function.arguments
                }
            });
            if let Some(pc) = &pre_content {
                running_event["toolCall"]["preContent"] = json!(pc);
            }
            let _ = app.emit("ai-chat-stream", running_event);
            let output = match agent::execute_tool(&call.function.name, &args, ctx).await {
                Ok(text) => text,
                Err(e) => format!("（工具执行失败: {}）", e),
            };
            tool_records.push(ToolCallRecord {
                message_id: 0,
                seq: tool_records.len() as i64,
                name: call.function.name.clone(),
                arguments: call.function.arguments.clone(),
                output: Some(output.clone()),
                pre_content: pre_content.clone(),
            });
            let _ = app.emit(
                "ai-chat-stream",
                json!({
                    "conversationId": conv_id,
                    "toolCall": {
                        "name": call.function.name,
                        "status": "done",
                        "index": tool_seq,
                        "output": output
                    }
                }),
            );
            tool_turns.push(crate::ai_core::client::ChatTurn {
                role: "tool".to_string(),
                content: Some(output),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: Some(call.id.clone()),
                name: Some(call.function.name.clone()),
            });
        }
        turns.push(assistant_turn);
        turns.extend(tool_turns);
        log_info!(
            "[Experimental] 第 {} 轮工具调用，已执行 {} 个工具",
            round + 1,
            result.tool_calls.len()
        );
    }

    let usage = usage_cell.lock().map(|g| g.clone()).unwrap_or_default();
    let reasoning = if reasoning.is_empty() {
        None
    } else {
        Some(reasoning)
    };
    let duration_ms = loop_started.elapsed().as_millis() as u64;
    Ok((reply, reasoning, tool_log, tool_records, usage, duration_ms))
}
