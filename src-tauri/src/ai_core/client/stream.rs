//! 流式多轮 + 工具调用聊天（SSE 逐块解析）
//!
//! 请求 `stream: true`，逐行解析 SSE `data:` 块，每个增量通过 [`StreamCallbacks`]
//! 即时回调（打字机效果），调用方负责累积文本与聚合工具调用增量。
//! 返回该轮完成的 [`ChatResult`]（含完整 tool_calls）。

use super::transport::{authorized_stream_builder, send_stream_with_timeout};
use super::types::{
    ChatCompletionsRequest, ChatResult, ChatTurn, StreamCallbacks, StreamToolDelta, StreamUsage,
    ToolCall, ToolCallFunction, ToolDef,
};
use crate::ai_core::config::AiConfig;
use std::sync::atomic::Ordering;

/// 取消信号检查（未提供取消信号时视为未取消）
fn is_cancelled(cancelled: Option<&std::sync::atomic::AtomicBool>) -> bool {
    cancelled
        .map(|f| f.load(Ordering::Relaxed))
        .unwrap_or(false)
}

pub async fn chat_completions_stream(
    config: &AiConfig,
    messages: Vec<ChatTurn>,
    tools: Option<&[ToolDef]>,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
    callbacks: &StreamCallbacks,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> anyhow::Result<ChatResult> {
    let model = config.resolve_model(model);
    if model.is_empty() {
        return Err(anyhow::anyhow!("未选择 AI 模型"));
    }

    // 请求发出前检查取消信号，已取消则直接返回空结果（由调用方判断是否继续）
    if is_cancelled(cancelled) {
        return Ok(ChatResult {
            content: None,
            reasoning_content: None,
            tool_calls: Vec::new(),
        });
    }

    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let req = ChatCompletionsRequest {
        model: model.clone(),
        messages,
        tools: tools.map(|t| t.to_vec()),
        stream: true,
        max_tokens: Some(config.max_output_tokens),
        reasoning_effort: reasoning_effort.map(|s| s.to_string()),
    };

    // 响应头（首字节）等待超时：思考型模型首 token 可能数十秒~数分钟，这里按
    // 配置超时与 180s 下限取较大值，避免还在思考就被误杀；正文读取无整体超时。
    let header_timeout = config.timeout_secs.max(180);
    let (_status, mut stream) = send_stream_with_timeout(header_timeout, async {
        let resp = authorized_stream_builder(config, reqwest::Method::POST, url)
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&req)
            .send()
            .await?;
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

    let mut content = String::new();
    let mut reasoning_content = String::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut usage = StreamUsage::default();

    // —— SSE 行缓冲解析 ——
    // reqwest 的 chunk() 返回任意字节块，SSE 的 `data:` 行可能被 chunk 边界切断。
    // 若直接逐 chunk 按行处理：不完整 JSON 行解析失败被丢、续行（无 data: 前缀）
    // 被跳过，导致增量内容丢字符（如 `[::game]` 缺 `]`、`MoLaunch` 丢 "Mo"）。
    // 因此维护行缓冲：只处理换行完整的行，不完整尾部留在缓冲等待下一个 chunk。
    let mut buf = String::new();

    // 单行处理：返回 Some(result) 表示流已结束（[DONE] / finish_reason），None 表示继续
    let handle_line = |line: &str,
                       content: &mut String,
                       reasoning_content: &mut String,
                       tool_calls: &mut Vec<ToolCall>,
                       usage: &mut StreamUsage|
     -> Option<ChatResult> {
        let line = line.trim();
        if !line.starts_with("data:") {
            return None;
        }
        let data = line[5..].trim();
        if data == "[DONE]" {
            (callbacks.on_done)(usage);
            return Some(finish_result(content, reasoning_content, tool_calls, true));
        }
        let parsed: serde_json::Value = match serde_json::from_str(data) {
            Ok(v) => v,
            Err(_) => return None,
        };
        let choice = parsed
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())?;

        // 流末 usage（部分服务在 final chunk 携带）
        if let Some(u) = parsed.get("usage") {
            *usage = StreamUsage {
                prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                completion_tokens: u
                    .get("completion_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            };
        }

        let finish = choice.get("finish_reason").and_then(|v| v.as_str());

        // 内容增量
        if let Some(delta) = choice
            .get("delta")
            .and_then(|d| d.get("content"))
            .and_then(|v| v.as_str())
        {
            if !delta.is_empty() {
                content.push_str(delta);
                (callbacks.on_delta)(delta);
            }
        }

        // 思考内容增量（思考模型，如 DeepSeek-R1；走 `delta.reasoning_content`）
        if let Some(delta) = choice
            .get("delta")
            .and_then(|d| d.get("reasoning_content"))
            .and_then(|v| v.as_str())
        {
            if !delta.is_empty() {
                reasoning_content.push_str(delta);
                (callbacks.on_reasoning_delta)(delta);
            }
        }

        // 工具调用增量（按 index 聚合）
        if let Some(calls) = choice
            .get("delta")
            .and_then(|d| d.get("tool_calls"))
            .and_then(|v| v.as_array())
        {
            for c in calls {
                let index = c.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let id = c.get("id").and_then(|v| v.as_str()).map(String::from);
                let name = c
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let arguments = c
                    .get("function")
                    .and_then(|f| f.get("arguments"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                // 追加/新建对应 index 的工具调用
                while tool_calls.len() <= index {
                    tool_calls.push(ToolCall {
                        id: String::new(),
                        ty: "function".to_string(),
                        function: ToolCallFunction {
                            name: String::new(),
                            arguments: String::new(),
                        },
                    });
                }
                let target = &mut tool_calls[index];
                if let Some(id) = id {
                    target.id = id;
                }
                if let Some(name) = name {
                    target.function.name = name;
                }
                target.function.arguments.push_str(&arguments);
                if !arguments.is_empty() {
                    (callbacks.on_tool_delta)(&StreamToolDelta {
                        index,
                        id: None,
                        name: None,
                        arguments,
                    });
                }
            }
        }

        if finish == Some("tool_calls") || finish == Some("stop") {
            super::chat::finalize_tool_calls(tool_calls);
            (callbacks.on_done)(usage);
            return Some(finish_result(content, reasoning_content, tool_calls, true));
        }
        None
    };

    'stream: while let Some(chunk) = stream.chunk().await? {
        // 流式过程中检测到取消信号：立即中断，返回已生成的部分内容（丢弃不完整的工具调用）
        if is_cancelled(cancelled) {
            break 'stream;
        }
        buf.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(nl) = buf.find('\n') {
            let line = buf[..nl].to_string();
            buf.drain(..=nl);
            if let Some(result) = handle_line(
                &line,
                &mut content,
                &mut reasoning_content,
                &mut tool_calls,
                &mut usage,
            ) {
                return Ok(result);
            }
        }
    }

    // 流自然结束：处理缓冲中残留的最后一行（无换行尾）
    if !buf.trim().is_empty() {
        if let Some(result) = handle_line(
            &buf,
            &mut content,
            &mut reasoning_content,
            &mut tool_calls,
            &mut usage,
        ) {
            return Ok(result);
        }
    }

    // 取消中断：不触发 finalize/on_done，丢弃不完整的工具调用
    if is_cancelled(cancelled) {
        return Ok(finish_result(
            &mut content,
            &mut reasoning_content,
            &mut tool_calls,
            false,
        ));
    }

    super::chat::finalize_tool_calls(&mut tool_calls);
    (callbacks.on_done)(&usage);
    Ok(finish_result(
        &mut content,
        &mut reasoning_content,
        &mut tool_calls,
        true,
    ))
}

/// 从累积内容构造最终结果（take 移动字段，释放内存）
fn finish_result(
    content: &mut String,
    reasoning_content: &mut String,
    tool_calls: &mut Vec<ToolCall>,
    keep_tool_calls: bool,
) -> ChatResult {
    ChatResult {
        content: if content.is_empty() {
            None
        } else {
            Some(std::mem::take(content))
        },
        reasoning_content: if reasoning_content.is_empty() {
            None
        } else {
            Some(std::mem::take(reasoning_content))
        },
        tool_calls: if keep_tool_calls {
            std::mem::take(tool_calls)
        } else {
            Vec::new()
        },
    }
}
