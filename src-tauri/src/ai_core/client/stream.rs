//! 流式多轮 + 工具调用聊天（SSE 逐块解析）

use super::accumulator::StreamAccumulator;
use super::sse::{SseEvent, SseLineBuffer};
use super::transport::{authorized_stream_builder, send_stream_with_timeout};
use super::types::{ChatCompletionsRequest, ChatResult, ChatTurn, StreamCallbacks, ToolDef};
use crate::ai_core::config::AiConfig;
use std::sync::atomic::{AtomicBool, Ordering};

fn is_cancelled(cancelled: Option<&AtomicBool>) -> bool {
    cancelled
        .map(|flag| flag.load(Ordering::Relaxed))
        .unwrap_or(false)
}

pub async fn chat_completions_stream(
    config: &AiConfig,
    messages: Vec<ChatTurn>,
    tools: Option<&[ToolDef]>,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
    callbacks: &StreamCallbacks,
    cancelled: Option<&AtomicBool>,
) -> anyhow::Result<ChatResult> {
    let model = config.resolve_model(model);
    if model.is_empty() {
        return Err(anyhow::anyhow!("未选择 AI 模型"));
    }
    if is_cancelled(cancelled) {
        return Ok(ChatResult {
            content: None,
            reasoning_content: None,
            tool_calls: Vec::new(),
        });
    }

    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let req = ChatCompletionsRequest {
        model,
        messages,
        tools: tools.map(|items| items.to_vec()),
        stream: true,
        max_tokens: Some(config.max_output_tokens),
        reasoning_effort: reasoning_effort.map(str::to_string),
    };
    let header_timeout = config.timeout_secs.max(180);
    let (_status, mut stream) = send_stream_with_timeout(header_timeout, async {
        let resp = authorized_stream_builder(config, reqwest::Method::POST, url)
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!(crate::http::request_error_msg(&e)))?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "AI 服务返回 HTTP {}: {}",
                status,
                text.trim()
            ));
        }
        Ok::<(u16, reqwest::Response), anyhow::Error>((status, resp))
    })
    .await?;

    let mut lines = SseLineBuffer::new();
    let mut result = StreamAccumulator::new(callbacks);
    'stream: while let Some(chunk) = stream.chunk().await? {
        if is_cancelled(cancelled) {
            break 'stream;
        }
        for event in lines.push(&chunk) {
            match event {
                SseEvent::Done => {
                    (callbacks.on_done)(&result.usage);
                    return Ok(result.finish(true));
                }
                SseEvent::Json(parsed) => {
                    if let Some(done) = result.apply(&parsed) {
                        return Ok(done);
                    }
                }
            }
        }
    }

    if let Some(event) = lines.finish() {
        match event {
            SseEvent::Done => {
                (callbacks.on_done)(&result.usage);
                return Ok(result.finish(true));
            }
            SseEvent::Json(parsed) => {
                if let Some(done) = result.apply(&parsed) {
                    return Ok(done);
                }
            }
        }
    }
    if is_cancelled(cancelled) {
        return Ok(result.finish(false));
    }
    super::chat::finalize_tool_calls(&mut result.tool_calls);
    (callbacks.on_done)(&result.usage);
    Ok(result.finish(true))
}
