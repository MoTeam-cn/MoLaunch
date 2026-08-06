//! 非流式聊天接口：单轮 `chat`、模型列表 `list_models`、多轮 `chat_completions`
//!
//! 配置热重载：本模块每个入口都接收调用方传入的 [`AiConfig`]（由上层每次
//! `load_config_async()` 重新读取），HTTP 传输复用 `crate::http` 全局客户端
//! （代理/IP/TLS 变更后由 `apply_config` 重建），修改配置无需重启应用。

use super::transport::{authorized_builder, send_with_timeout};
use super::types::{
    ChatCompletionsRequest, ChatCompletionsResponse, ChatMessage, ChatRequest, ChatResponse,
    ChatResult, ChatTurn, ModelsResponse, ToolCall, ToolDef,
};
use crate::ai_core::config::AiConfig;
use crate::ai_core::prompt::{system_prompt, PromptKind};

/// 调用 OpenAI 兼容服务（单轮，无工具），返回模型回复
///
/// `model` 为可选显式指定；为空时按「默认模型 > 已启用模型首个」解析。
pub async fn chat(
    config: &AiConfig,
    kind: PromptKind,
    user_content: String,
    model: Option<&str>,
) -> anyhow::Result<String> {
    let model = config.resolve_model(model);
    if model.is_empty() {
        return Err(anyhow::anyhow!("未选择 AI 模型"));
    }

    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let req = ChatRequest {
        model: model.clone(),
        messages: vec![
            ChatMessage {
                role: "system",
                content: system_prompt(&kind),
            },
            ChatMessage {
                role: "user",
                content: user_content,
            },
        ],
        stream: false,
        max_tokens: Some(config.max_output_tokens),
    };

    let (status, text) = send_with_timeout(config.timeout_secs, async {
        let resp = authorized_builder(config, reqwest::Method::POST, url)
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&req)
            .send()
            .await?;
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        Ok::<(u16, String), anyhow::Error>((status, text))
    })
    .await?;

    if !(200..300).contains(&status) {
        return Err(anyhow::anyhow!(
            "AI 服务返回 HTTP {}: {}",
            status,
            text.trim()
        ));
    }

    let parsed: ChatResponse =
        serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("解析 AI 响应失败: {}", e))?;
    parsed
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .ok_or_else(|| anyhow::anyhow!("AI 响应为空"))
}

/// 拉取服务端模型列表（`GET /models`），返回按 id 排序的模型名
pub async fn list_models(config: &AiConfig) -> anyhow::Result<Vec<String>> {
    if config.base_url.trim().is_empty() {
        return Err(anyhow::anyhow!("未配置 AI 服务地址"));
    }

    let url = format!("{}/models", config.base_url.trim_end_matches('/'));
    let (status, text) = send_with_timeout(config.timeout_secs, async {
        let resp = authorized_builder(config, reqwest::Method::GET, url)
            .send()
            .await?;
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        Ok::<(u16, String), anyhow::Error>((status, text))
    })
    .await?;

    if !(200..300).contains(&status) {
        return Err(anyhow::anyhow!(
            "AI 服务返回 HTTP {}: {}",
            status,
            text.trim()
        ));
    }

    let parsed: ModelsResponse =
        serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("解析模型列表失败: {}", e))?;
    let mut ids: Vec<String> = parsed.data.into_iter().filter_map(|m| m.id).collect();
    ids.sort();
    ids.dedup();
    Ok(ids)
}

/// 非流式多轮聊天（可带工具定义），返回最终结果（内容或工具调用）
pub async fn chat_completions(
    config: &AiConfig,
    turns: Vec<ChatTurn>,
    tools: Option<&[ToolDef]>,
    model_override: Option<&str>,
) -> anyhow::Result<ChatResult> {
    let model = config.resolve_model(model_override);
    if model.is_empty() {
        return Err(anyhow::anyhow!("未选择 AI 模型"));
    }

    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let req = ChatCompletionsRequest {
        model: model.clone(),
        messages: turns,
        tools: tools.map(|t| t.to_vec()),
        stream: false,
        max_tokens: Some(config.max_output_tokens),
        reasoning_effort: None,
    };

    let (status, text) = send_with_timeout(config.timeout_secs, async {
        let resp = authorized_builder(config, reqwest::Method::POST, url)
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&req)
            .send()
            .await?;
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        Ok::<(u16, String), anyhow::Error>((status, text))
    })
    .await?;

    if !(200..300).contains(&status) {
        return Err(anyhow::anyhow!(
            "AI 服务返回 HTTP {}: {}",
            status,
            text.trim()
        ));
    }

    let parsed: ChatCompletionsResponse =
        serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("解析 AI 响应失败: {}", e))?;
    let mut result = parsed
        .choices
        .into_iter()
        .next()
        .map(|c| ChatResult {
            content: c.message.content,
            reasoning_content: c.message.reasoning_content,
            tool_calls: c.message.tool_calls,
        })
        .ok_or_else(|| anyhow::anyhow!("AI 响应为空"))?;
    finalize_tool_calls(&mut result.tool_calls);
    Ok(result)
}

/// 校验并兜底工具调用聚合结果：
/// - `id` 为空（部分提供商流式增量不返回 id）时生成 `call_{index}`
/// - `function.name` 为空的调用视为无效，过滤掉（无法执行）
pub(super) fn finalize_tool_calls(tool_calls: &mut Vec<ToolCall>) {
    tool_calls.retain(|tc| !tc.function.name.trim().is_empty());
    for (i, tc) in tool_calls.iter_mut().enumerate() {
        if tc.id.trim().is_empty() {
            tc.id = format!("call_{}", i);
        }
    }
}
