//! AI 日志分析（5 环节流式）
//!
//! 读配置 → 系统提示词 → 用户消息（截断保护）→ `chat_completions_stream` 流式 →
//! 每段 delta 即时 emit；检测到 `【STEP:N/5】` 标记时 emit step 事件且标记行不进展示文本。

use once_cell::sync::Lazy;
use serde_json::json;
use tauri::{AppHandle, Emitter};

use super::super::types::AiAnalyzeLogParams;
use crate::ai_core;
use crate::ai_core::client::{ChatTurn, StreamCallbacks, StreamUsage};
use crate::log_warn;

/// 日志分析环节标记（支持全角【】与半角[]，捕获环节序号）
static STEP_MARKER_RE: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"[【\[]STEP:(\d+)/5[】\]]").expect("STEP_MARKER_RE 编译失败"));

/// AI 日志分析（5 环节流式）：流结束后 emit 剔除标记后的全文
pub(super) async fn ai_analyze_log(
    app: &AppHandle,
    params: AiAnalyzeLogParams,
) -> Result<serde_json::Value, String> {
    let log_text = params.log_text.trim();
    if log_text.is_empty() {
        return Err("日志内容不能为空".to_string());
    }

    let config = ai_core::load_config_async().await;
    if config.base_url.trim().is_empty() {
        return Err(
            "未配置 AI 服务地址，请先在「实验性 → AI 设置」中配置本地 OpenAI 兼容服务".to_string(),
        );
    }
    let model = if let Some(m) = params.model.as_deref() {
        if m.trim().is_empty() {
            config.resolve_model(None)
        } else {
            m.trim().to_string()
        }
    } else {
        config.resolve_model(None)
    };
    if model.is_empty() {
        return Err("未启用任何模型，请先在「实验性 → AI 设置」中加载并启用模型".to_string());
    }

    let system = ai_core::prompt::system_prompt(&ai_core::PromptKind::LogAnalyzeSteps);

    // 本地预检：先收敛问题范围再交给 AI，避免超长全文直接发送
    let user_content = if params.local_analyze {
        let items = crate::commands::tools::crash_analyzer::analyze_log_text(log_text);
        if items.is_empty() {
            format!(
                "用户提供的日志本地初检未命中已知崩溃模式，以下是日志原文（截断）：\n{}",
                crate::utils::format::truncate_chars(log_text, config.max_input_tokens as usize)
            )
        } else {
            let mut summary = format!(
                "用户提供的日志经过本地规则引擎初检，共识别到 {} 个可能问题，请围绕以下范围深入分析：\n",
                items.len()
            );
            for (i, it) in items.iter().enumerate() {
                summary.push_str(&format!(
                    "{}. 分类[{}] 级别[{}] 标题: {}\n",
                    i + 1,
                    it.category,
                    it.severity,
                    it.title
                ));
                if !it.detail.is_empty() {
                    summary.push_str(&format!("   关键行: {}\n", it.detail));
                }
                if !it.suggestion.is_empty() {
                    summary.push_str(&format!("   建议: {}\n", it.suggestion));
                }
            }
            summary.push_str(&format!(
                "\n以下是日志原文（截断，供核实细节）：\n{}",
                crate::utils::format::truncate_chars(log_text, config.max_input_tokens as usize)
            ));
            summary
        }
    } else {
        crate::utils::format::truncate_chars(log_text, config.max_input_tokens as usize)
    };
    let turns = vec![
        ChatTurn {
            role: "system".to_string(),
            content: Some(system),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        ChatTurn {
            role: "user".to_string(),
            content: Some(user_content),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ];

    // 流式增量共享状态：剔除标记行后的展示文本、跨 delta 行缓冲、已发射的最大环节号
    let content_cell = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let line_buf_cell = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let step_seen_cell = std::sync::Arc::new(std::sync::Mutex::new(0u32));

    let app_delta = app.clone();
    let content_capture = content_cell.clone();
    let line_buf_capture = line_buf_cell.clone();
    let step_seen_capture = step_seen_cell.clone();
    let stream_callbacks = StreamCallbacks {
        // 逐 delta 累积，按完整行扫描 `【STEP:N/5】` 标记（标记行剔除，其余行作为 delta 即时推送）
        on_delta: Box::new(move |delta: &str| {
            let mut line_buf = line_buf_capture.lock().unwrap();
            line_buf.push_str(delta);
            while let Some(nl) = line_buf.find('\n') {
                let line = line_buf[..nl].to_string();
                line_buf.drain(..=nl);
                if process_analyze_line(&app_delta, &content_capture, &step_seen_capture, &line) {
                    let _ = app_delta.emit(
                        "ai-analyze-stream",
                        json!({ "delta": format!("{}\n", line) }),
                    );
                }
            }
        }),
        on_reasoning_delta: Box::new(|_delta: &str| {}),
        on_tool_delta: Box::new(|_delta: &crate::ai_core::client::StreamToolDelta| {}),
        on_done: Box::new(|_usage: &StreamUsage| {}),
    };

    ai_core::chat_completions_stream(
        &config,
        turns,
        None,
        Some(&model),
        params.reasoning_effort.as_deref(),
        &stream_callbacks,
        None,
    )
    .await
    .map_err(|e| {
        log_warn!("[Experimental] AI 日志分析失败: {}", e);
        format!("AI 日志分析失败: {}", e)
    })?;

    // 处理流结束残留的未换行尾部（标记行同样剔除）
    let tail = std::mem::take(&mut *line_buf_cell.lock().unwrap());
    if !tail.is_empty() && process_analyze_line(app, &content_cell, &step_seen_cell, &tail) {
        let _ = app.emit("ai-analyze-stream", json!({ "delta": tail }));
    }

    let content = content_cell.lock().unwrap().clone();
    let _ = app.emit(
        "ai-analyze-stream",
        json!({ "done": true, "content": content }),
    );
    serde_json::to_value(()).map_err(|e| e.to_string())
}

/// 处理日志分析的一行流式内容：
/// - 若含 `【STEP:N/5】`（或半角）标记 → 递增发射 step 事件，标记行不进展示文本，返回 false
/// - 否则 → 追加到剔除后的展示文本，返回 true（调用方作为 delta 事件发射）
fn process_analyze_line(
    app: &AppHandle,
    content: &std::sync::Mutex<String>,
    step_seen: &std::sync::Mutex<u32>,
    line: &str,
) -> bool {
    if let Some(caps) = STEP_MARKER_RE.captures(line) {
        if let Some(n) = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok()) {
            let mut seen = step_seen.lock().unwrap();
            if n > *seen {
                *seen = n;
                let _ = app.emit("ai-analyze-stream", json!({ "step": n }));
            }
        }
        return false;
    }
    content.lock().unwrap().push_str(line);
    content.lock().unwrap().push('\n');
    true
}
