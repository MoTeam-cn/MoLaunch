//! OpenAI 兼容 API 客户端（复用 `crate::http::get_client`，支持 Bearer 认证）

use serde::{Deserialize, Serialize};

use super::config::AiConfig;
use super::prompt::{system_prompt, PromptKind};
use crate::utils::format::truncate_chars;

/// 聊天请求体（OpenAI 兼容）
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

/// 聊天响应（非流式）
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

/// 模型列表响应（OpenAI 兼容 `GET /models`）
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelsData>,
}

#[derive(Debug, Deserialize)]
struct ModelsData {
    id: Option<String>,
}

/// 通用超时包装：限制外部 future 在 `timeout_secs`（下限 5s）内完成
///
/// `crate::http` 全局客户端自带 30s 超时，此处提供可配置的请求级超时。
async fn send_with_timeout<F>(timeout_secs: u64, fut: F) -> anyhow::Result<(u16, String)>
where
    F: std::future::Future<Output = anyhow::Result<(u16, String)>> + Send,
{
    let timeout = std::time::Duration::from_secs(timeout_secs.max(5));
    tokio::time::timeout(timeout, fut)
        .await
        .map_err(|_| anyhow::anyhow!("AI 请求超时（{}s）", timeout.as_secs()))?
}

/// 构造带可选 `Authorization: Bearer <api_key>` 的请求构建器
fn authorized_builder(config: &AiConfig, method: reqwest::Method, url: String) -> reqwest::RequestBuilder {
    let mut builder = crate::http::get_client()
        .request(method, url)
        .header("Accept-Language", "zh-CN");
    if !config.api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {}", config.api_key));
    }
    builder
}

/// 调用本地 OpenAI 兼容服务，返回模型回复
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
            truncate_chars(&text, 200)
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
        let resp = authorized_builder(config, reqwest::Method::GET, url).send().await?;
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        Ok::<(u16, String), anyhow::Error>((status, text))
    })
    .await?;

    if !(200..300).contains(&status) {
        return Err(anyhow::anyhow!(
            "AI 服务返回 HTTP {}: {}",
            status,
            truncate_chars(&text, 200)
        ));
    }

    let parsed: ModelsResponse =
        serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("解析模型列表失败: {}", e))?;
    let mut ids: Vec<String> = parsed.data.into_iter().filter_map(|m| m.id).collect();
    ids.sort();
    ids.dedup();
    Ok(ids)
}
