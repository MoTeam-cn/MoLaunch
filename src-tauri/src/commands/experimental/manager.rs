//! 实验性功能 action 分发
//!
//! action：`create_conversation` / `list_conversations` / `delete_conversation` /
//! `rename_conversation` / `list_messages` / `clear_conversation` / `chat_send` /
//! `collect_context` / `delete_message` / `regenerate_reply` / `edit_message` /
//! `reply_ask_user` / `list_installed_versions` / AI（analyze_crash / check_status /
//! save_config / load_config / list_models）。
//!
//! 所有 action 先校验 `experimental_enabled`：未开启时返回错误，前端据此提示
//! 用户前往「设置 → 进阶设置」开启实验性功能（同时惰性初始化 SQLite 聊天库）。
//!
//! 聊天为流式（SSE）：后端逐块解析增量并通过 `app.emit("ai-chat-stream", ...)`
//! 推送到前端，前端每个增量即时追加到气泡，实现打字机效果。

use once_cell::sync::Lazy;
use serde_json::{json, Value};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter};

use super::agent::{self, AgentContext};
use super::db;
use super::types::{
    AiAnalyzeLogParams, ChatSendParams, CollectContextParams, CollectContextResult,
    ConversationIdParams, ConversationItem, CreateConversationParams, DeleteMessageParams,
    EditMessageParams, ListMessagesParams, ListToolCallsParams, MessageItem,
    RegenerateReplyParams, RenameConversationParams, ReplyAskUserParams, ToolCallRecord,
};
use crate::ai_core;
use crate::ai_core::client::{estimate_tokens, ChatTurn, StreamCallbacks, StreamUsage};
use crate::commands::ai::types::{AiProbeParams, AnalyzeCrashParams};
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};
use crate::{log_info, log_warn};

/// 单次聊天允许的最大工具调用轮次（防止模型陷入工具死循环）
const MAX_TOOL_ITERATIONS: usize = 4;
/// 作为 AI 上下文的历史消息条数
const HISTORY_LIMIT: i64 = 30;
/// 上下文窗口使用率上限（超过此比例开始压缩）
const CONTEXT_COMPRESS_THRESHOLD: f64 = 0.8;
/// 上下文压缩目标使用率
const CONTEXT_COMPRESS_TARGET: f64 = 0.6;

/// 日志分析环节标记（支持全角【】与半角[]，捕获环节序号）
static STEP_MARKER_RE: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"[【\[]STEP:(\d+)/5[】\]]").expect("STEP_MARKER_RE 编译失败")
});

/// 解析模型返回的工具调用参数 JSON
///
/// 模型输出的 `arguments` 不保证是严格合法 JSON（可能带换行、围栏或夹杂说明文字，
/// 例如 `ask_user` 曾出现“缺少 question 参数”的误报）。逐级容错：
/// 1. 直接解析；
/// 2. 取首个 `{` 到末个 `}` 之间的子串再解析（剥离前后夹杂文本）；
/// 3. 若解析结果本身是字符串，再解析一次（防双重编码）。
/// 全部失败返回 `Value::Null`（工具将按缺参处理）。
fn parse_tool_arguments(raw: &str) -> Value {
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
            // 双重编码：`"{...}"` → 解析为字符串，再解析一次得到对象
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

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "create_conversation",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let p: CreateConversationParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let id = db::create_conversation(p.title.as_deref().unwrap_or(""))?;
            let item = ConversationItem {
                id,
                title: p.title.filter(|t| !t.trim().is_empty()).unwrap_or_else(|| "新对话".to_string()),
                created_at: chrono::Local::now().timestamp(),
                updated_at: chrono::Local::now().timestamp(),
            };
            serde_json::to_value(item).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_conversations",
        handler!(state, _app, _params, {
            ensure_enabled(&state).await?;
            let items = db::list_conversations()?;
            serde_json::to_value(items).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "delete_conversation",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let p: ConversationIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            db::delete_conversation(p.conversation_id)?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "rename_conversation",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let p: RenameConversationParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            db::rename_conversation(p.conversation_id, &p.title)?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_messages",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let p: ListMessagesParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let items: Vec<MessageItem> = db::list_messages(p.conversation_id, None)?;
            serde_json::to_value(items).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_tool_calls",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let p: ListToolCallsParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let items: Vec<ToolCallRecord> = db::list_tool_calls(p.conversation_id)?;
            serde_json::to_value(items).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "clear_conversation",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let p: ConversationIdParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            db::clear_conversation(p.conversation_id)?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "chat_send",
        handler!(state, app, params, {
            ensure_enabled(&state).await?;
            let p: ChatSendParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            chat_send(&state, &app, p).await
        }),
    );

    d.register(
        "delete_message",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let p: DeleteMessageParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let deleted = db::delete_message(p.conversation_id, p.message_id)?;
            serde_json::to_value(json!({ "deletedIds": deleted })).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "regenerate_reply",
        handler!(state, app, params, {
            ensure_enabled(&state).await?;
            let p: RegenerateReplyParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            regenerate_reply(&state, &app, p).await
        }),
    );

    d.register(
        "edit_message",
        handler!(state, app, params, {
            ensure_enabled(&state).await?;
            let p: EditMessageParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            edit_message(&state, &app, p).await
        }),
    );

    d.register(
        "reply_ask_user",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let p: ReplyAskUserParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            reply_ask_user(p).await
        }),
    );

    d.register(
        "cancel_chat",
        handler!(state, _app, _params, {
            ensure_enabled(&state).await?;
            // 置位聊天取消信号：正在进行的流式回复在下一个数据块到达前尽快中断
            state.chat_cancel_flag.store(true, Ordering::Relaxed);
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "collect_context",
        handler!(state, app, params, {
            ensure_enabled(&state).await?;
            let p: CollectContextParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let version_id = p.version_id.clone().unwrap_or_default();
            let ctx = build_context(&state, &app, p.conversation_id.unwrap_or(0)).await;
            let text = agent::collect_context(&p.kind, &version_id, &ctx)?;
            let result = CollectContextResult {
                kind: p.kind,
                text,
            };
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_installed_versions",
        handler!(state, app, _params, {
            ensure_enabled(&state).await?;
            let ctx = build_context(&state, &app, 0).await;
            let ids = crate::minecraft::version::scan::scan_installed_versions(&ctx.game_dir)
                .into_iter()
                .map(|i| i.id)
                .collect::<Vec<_>>();
            serde_json::to_value(ids).map_err(|e| e.to_string())
        }),
    );

    // ===== AI（本地 OpenAI 兼容服务）=====
    // 原独立 `ai_manager` 已并入实验性统一分发；未开启实验性功能时不可用。
    d.register(
        "analyze_crash",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let p: AnalyzeCrashParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::ai::manager::analyze_crash(p).await
        }),
    );

    d.register(
        "ai_analyze_log",
        handler!(state, app, params, {
            ensure_enabled(&state).await?;
            let p: AiAnalyzeLogParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            ai_analyze_log(&app, p).await
        }),
    );

    d.register(
        "check_status",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let probe = if params.is_null() {
                None
            } else {
                Some(
                    serde_json::from_value::<AiProbeParams>(params)
                        .map_err(|e| format!("参数解析失败: {}", e))?,
                )
            };
            crate::commands::ai::manager::check_status(probe).await
        }),
    );

    d.register(
        "save_config",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let cfg: ai_core::AiConfig = serde_json::from_value(params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            ai_core::save_config(&state.sdk, &cfg).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "load_config",
        handler!(state, _app, _params, {
            ensure_enabled(&state).await?;
            let cfg = ai_core::load_config_async().await;
            serde_json::to_value(cfg).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_models",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let p: AiProbeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let config = ai_core::AiConfig {
                base_url: p.base_url,
                api_key: p.api_key,
                timeout_secs: p.timeout_secs,
                models: Vec::new(),
                default_model: String::new(),
                max_input_tokens: 184_000,
                max_output_tokens: 16_000,
                icon_color_mode: "color".to_string(),
            };
            let models =
                ai_core::list_models(&config).await.map_err(|e| e.to_string())?;
            serde_json::to_value(models).map_err(|e| e.to_string())
        }),
    );

    d
});

/// action 分发入口（由 `super::experimental_manager` 调用）
pub async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}

/// 校验实验性功能开关，未开启时返回错误
async fn ensure_enabled(state: &AppState) -> Result<(), String> {
    let enabled = state.config.lock().await.experimental_enabled;
    if enabled {
        Ok(())
    } else {
        Err("实验性功能未开启，请先在「设置 → 进阶设置」中启用".to_string())
    }
}

/// 构造 Agent 工具执行上下文（游戏目录 + 启动器版本 + 配置摘要 + 隔离模式 + 会话/app）
async fn build_context(state: &AppState, app: &AppHandle, conversation_id: i64) -> AgentContext {
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let (config_summary, isolation_mode) = {
        let config = state.config.lock().await;
        (
            build_config_summary(&config),
            crate::minecraft::isolation::IsolationMode::from_u32(config.isolation_mode),
        )
    };
    AgentContext {
        game_dir,
        version: env!("CARGO_PKG_VERSION").to_string(),
        config_summary,
        isolation_mode,
        conversation_id,
        app: app.clone(),
    }
}

fn build_config_summary(config: &crate::state::AppConfig) -> String {
    let log_level_name = match config.log_level {
        0 | 1 => "ERROR",
        2 => "WARN",
        3 => "INFO",
        4 => "DEBUG",
        _ => "TRACE",
    };
    format!(
        "界面语言: {} | 日志级别: {} | 主题主色: {} | 版本隔离模式: {}",
        config.game_language, log_level_name, config.primary_color, config.isolation_mode
    )
}

/// 加载模型配置并解析本次对话使用的模型
fn resolve_chat_model(params: &ChatSendParams, config: &ai_core::AiConfig) -> Result<String, String> {
    if config.base_url.trim().is_empty() {
        return Err("未配置 AI 服务地址，请先在「实验性 → AI 设置」中配置本地 OpenAI 兼容服务".to_string());
    }
    let model = if let Some(m) = params.model.as_deref() {
        if m.trim().is_empty() { "" } else { m.trim() }
    } else {
        ""
    };
    let model = if model.is_empty() {
        config.resolve_model(None)
    } else {
        model.to_string()
    };
    if model.is_empty() {
        return Err("未启用任何模型，请先在「实验性 → AI 设置」中加载并启用模型".to_string());
    }
    Ok(model)
}

/// 从数据库历史构造对话 turns（含系统提示词）
fn build_turns(history: &[MessageItem], with_system: bool) -> Vec<ChatTurn> {
    let mut turns: Vec<ChatTurn> = history
        .iter()
        .map(|m| ChatTurn {
            role: m.role.clone(),
            content: Some(m.content.clone()),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        })
        .collect();
    if with_system {
        turns.insert(
            0,
            ChatTurn {
                role: "system".to_string(),
                content: Some(ai_core::prompt::chat_system_prompt()),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        );
    }
    turns
}

/// 估算当前上下文真实占用（与前端 `tokenEstimate` 口径一致）：
/// 倒序查找最新一条带真实 usage（`prompt_tokens`）的 AI 消息，
/// 以该值（即上次请求实际发送给模型的完整输入，含系统提示词/工具定义/全部历史/格式开销）
/// 为基准，再加上其后新增消息的字符估算；无真实 usage 时退化为全量字符估算（计入思考内容）。
fn estimate_context_usage(history: &[MessageItem]) -> u64 {
    let mut extra: u64 = 0;
    for m in history.iter().rev() {
        if m.role == "assistant" {
            if let Some(t) = m.prompt_tokens {
                if t > 0 {
                    return t as u64 + extra;
                }
            }
        }
        extra += estimate_tokens(m.content.as_str());
        extra += estimate_tokens(m.reasoning_content.as_deref().unwrap_or(""));
    }
    extra
}

/// 上下文压缩：按上下文真实占用（真实 usage 或退化估算）丢弃最旧消息，直到低于阈值
fn compress_context(
    turns: &mut Vec<ChatTurn>,
    max_input_tokens: u32,
    real_usage: u64,
) -> (bool, usize) {
    // 占用基准优先用真实 usage（字符估算会显著低估系统提示词/工具定义/格式开销，
    // 导致压缩过晚、请求超长被服务拒绝）；无 usage（新会话/旧数据）退化为估算
    let mut total: u64 = if real_usage > 0 {
        real_usage
    } else {
        turns.iter().map(|t| estimate_tokens(t.content.as_deref().unwrap_or(""))).sum()
    };
    if total <= (max_input_tokens as f64 * CONTEXT_COMPRESS_THRESHOLD) as u64 {
        return (false, 0);
    }
    let target = (max_input_tokens as f64 * CONTEXT_COMPRESS_TARGET) as u64;
    let mut removed = 0usize;
    // 保留系统提示词与最近的对话，从最旧非 system 消息开始丢
    let mut keep_from = 1usize;
    while keep_from < turns.len() && total > target {
        total = total.saturating_sub(estimate_tokens(turns[keep_from].content.as_deref().unwrap_or("")));
        keep_from += 1;
        removed += 1;
    }
    if removed > 0 {
        // 保留系统提示词
        let system = turns[0].clone();
        let rest: Vec<ChatTurn> = turns[keep_from..].to_vec();
        let mut new_turns = vec![system];
        new_turns.extend(rest);
        *turns = new_turns;
    }
    (removed > 0, removed)
}

/// 执行多轮「模型请求 → 工具调用 → 回填 tool turns」循环（最多 `MAX_TOOL_ITERATIONS` 轮）
///
/// 与旧实现的三处内联重复循环相比，此抽象统一了：
/// - `reply` 保留最后输出的文本（工具调用过程由前端工具链展示，消息只保留最终答复；
///   旧实现每轮覆盖导致工具前文本丢失，或累积导致最终消息混入过渡语句）
/// - 工具调用以 `toolCall` 事件推送（旧实现 regenerate/edit 不推送，前端看不到调用过程）
/// - 工具调用记录返回给调用方持久化（SQLite `tool_calls` 表，刷新后工具链仍保留）
/// - `done` 事件不在循环内触发：每轮完成仅累积 usage，
///   由调用方在全部轮次结束后统一推送一次，避免前端每轮收到 `done` 提前清空流式状态
///
/// 返回 `(最终回复, 思考内容, 工具执行日志, 工具调用记录, 各轮 usage 合计, 总生成耗时 ms)`。
async fn run_tool_loop(
    config: &ai_core::AiConfig,
    app: &AppHandle,
    conv_id: i64,
    model: &str,
    reasoning_effort: Option<&str>,
    tools: &[crate::ai_core::client::ToolDef],
    ctx: &AgentContext,
    turns: &mut Vec<ChatTurn>,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> Result<(String, Option<String>, Vec<String>, Vec<ToolCallRecord>, StreamUsage, u64), String> {
    let loop_started = std::time::Instant::now();
    let mut reply = String::new();
    let mut reasoning = String::new();
    let mut tool_log: Vec<String> = Vec::new();
    let mut tool_records: Vec<ToolCallRecord> = Vec::new();
    let usage_cell = std::sync::Arc::new(std::sync::Mutex::new(StreamUsage::default()));

    let app_delta = app.clone();
    let app_reasoning = app.clone();
    let usage_capture = usage_cell.clone();
    let stream_callbacks = StreamCallbacks {
        on_delta: Box::new(move |delta: &str| {
            let _ = app_delta.emit(
                "ai-chat-stream",
                json!({
                    "conversationId": conv_id,
                    "delta": delta,
                    "done": false
                }),
            );
        }),
        // 思考内容增量（思考模型，如 DeepSeek-R1）：前端在 AI 消息内实时渲染「深度思考」区块
        on_reasoning_delta: Box::new(move |delta: &str| {
            let _ = app_reasoning.emit(
                "ai-chat-stream",
                json!({
                    "conversationId": conv_id,
                    "reasoning": delta
                }),
            );
        }),
        on_tool_delta: Box::new(|_delta: &crate::ai_core::client::StreamToolDelta| {
            // 工具调用增量由调用方聚合，前端仅关心 done 时的工具列表
        }),
        // 每轮流结束都会回调（`finish_reason` 为 stop 或 tool_calls）：
        // 仅累积 usage，`done` 事件由调用方在循环结束后统一推送一次
        on_done: Box::new(move |u: &StreamUsage| {
            if let Ok(mut acc) = usage_capture.lock() {
                acc.prompt_tokens += u.prompt_tokens;
                acc.completion_tokens += u.completion_tokens;
                acc.total_tokens += u.total_tokens;
            }
        }),
    };

    for round in 0..MAX_TOOL_ITERATIONS {
        let result =
            ai_core::chat_completions_stream(config, turns.clone(), Some(tools), Some(model), reasoning_effort, &stream_callbacks, cancelled)
                .await
                .map_err(|e| {
                    log_warn!("[Experimental] 聊天请求失败: {}", e);
                    format!("AI 请求失败: {}", e)
                })?;

        // 累计思考内容（供最终消息「深度思考」区块展示）
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
                // 仅保留最后一次输出：工具调用前的过渡语句不混入最终消息，
                // 工具调用过程由前端工具链（toolCall 事件）完整展示
                reply = content.to_string();
            }
        }
        if result.tool_calls.is_empty() {
            break;
        }

        // 该轮模型在调用工具前输出的过渡文本（同一轮内多个工具共享，展示在工具链节点上方）
        let pre_content = result
            .content
            .as_deref()
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());

        // 执行工具调用并回填 tool 角色消息
        // 思考模型注意：涉及工具调用的轮次必须完整回传 reasoning_content，否则服务返回 400
        let assistant_turn = ChatTurn {
            role: "assistant".to_string(),
            content: result.content.clone(),
            reasoning_content: result.reasoning_content.clone(),
            tool_calls: Some(result.tool_calls.clone()),
            tool_call_id: None,
            name: None,
        };
        let mut tool_turns: Vec<ChatTurn> = Vec::new();
        for (tool_idx, call) in result.tool_calls.iter().enumerate() {
            let args: Value = parse_tool_arguments(&call.function.arguments);
            tool_log.push(format!("{} {}", call.function.name, args));
            // 工具开始/结束状态推送（前端在对话流中展示可点击详情）
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
            // 记录工具调用（message_id 在 AI 回复消息落库后由调用方回填持久化）
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
            tool_turns.push(ChatTurn {
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

    let usage = usage_cell
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default();
    let reasoning = if reasoning.is_empty() {
        None
    } else {
        Some(reasoning)
    };
    // 总生成耗时：从首个请求发出到全部工具轮次结束（含工具调用与 ask_user 等待）
    let duration_ms = loop_started.elapsed().as_millis() as u64;
    Ok((reply, reasoning, tool_log, tool_records, usage, duration_ms))
}

/// 推送一次流式完成事件（全部工具轮次结束后由调用方统一调用）
fn emit_chat_done(app: &AppHandle, conv_id: i64, usage: &StreamUsage, duration_ms: u64) {
    let _ = app.emit(
        "ai-chat-stream",
        json!({
            "conversationId": conv_id,
            "done": true,
            "usage": usage,
            "durationMs": duration_ms
        }),
    );
}

/// 聊天发送（流式）：保存用户消息 → 携带历史 + 工具发起流式对话 → 保存回复
async fn chat_send(state: &AppState, app: &AppHandle, params: ChatSendParams) -> Result<Value, String> {
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
            user_content = format!("【用户附带的上下文】\n{}\n\n---\n\n{}", attach, user_content);
        }
    }

    // 记录用户消息（先查是否首条，用于自动生成标题）
    let existing = db::list_messages(params.conversation_id, None)?;
    let is_first_message = existing.is_empty();

    // 保存用户消息（pair_id 在生成 AI 回复后回填；model 由 AI 回复记录，用户消息不填）
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
    let mut turns = build_turns(&history, true);

    // 上下文压缩（按真实 usage 估算当前上下文占用，超出阈值时丢弃最旧消息）
    let (compressed, removed_count) =
        compress_context(&mut turns, config.max_input_tokens, estimate_context_usage(&history));
    if compressed {
        emit_chat_status(app, params.conversation_id, &format!("上下文已自动压缩，丢弃了最旧的 {} 条消息", removed_count));
    }

    let tools = agent::tool_definitions();

    let started = std::time::Instant::now();
    let (mut reply, reasoning, tool_log, tool_records, usage, duration_ms) =
        run_tool_loop(&config, app, params.conversation_id, &model, params.reasoning_effort.as_deref(), &tools, &ctx, &mut turns, Some(&state.chat_cancel_flag)).await?;
    emit_chat_done(app, params.conversation_id, &usage, duration_ms);

    if reply.trim().is_empty() {
        reply = if state.chat_cancel_flag.load(Ordering::Relaxed) {
            "（已停止生成）".to_string()
        } else {
            "（模型未生成有效回复，请检查服务状态或重试）".to_string()
        };
    }

    // 保存 AI 回复，回填配对 id；记录实际生成该回复的模型名（切换全局模型后历史消息图标仍固定）
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
        generate_title(app, params.conversation_id, &config, &model, &user_content, &reply);
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

/// 重新回复：找到该 AI 消息对应的用户消息，删除其后消息并重新生成
async fn regenerate_reply(
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
    let user_msg_id = db::get_message(params.conversation_id, params.message_id)?
        .and_then(|(_, role, _)| {
            if role == "assistant" {
                db::get_message_pair_id(params.conversation_id, params.message_id).ok().flatten()
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
    let model = if let Some(m) = params.model.as_deref() {
        if m.trim().is_empty() { config.resolve_model(None) } else { m.trim().to_string() }
    } else {
        config.resolve_model(None)
    };
    if model.is_empty() {
        return Err("未启用任何模型".to_string());
    }

    let ctx = build_context(state, app, params.conversation_id).await;
    let history = db::list_messages(params.conversation_id, Some(HISTORY_LIMIT))?;
    let mut turns = build_turns(&history, true);
    let _ = compress_context(&mut turns, config.max_input_tokens, estimate_context_usage(&history));

    let tools = agent::tool_definitions();

    let (mut reply, reasoning, tool_log, tool_records, usage, duration_ms) =
        run_tool_loop(&config, app, params.conversation_id, &model, params.reasoning_effort.as_deref(), &tools, &ctx, &mut turns, Some(&state.chat_cancel_flag)).await?;
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
async fn edit_message(
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

    // 自动重新生成回复（复用 regenerate 逻辑：以该用户消息为起点）
    let config = ai_core::load_config_async().await;
    let model = if let Some(m) = params.model.as_deref() {
        if m.trim().is_empty() { config.resolve_model(None) } else { m.trim().to_string() }
    } else {
        config.resolve_model(None)
    };
    if model.is_empty() {
        return Err("未启用任何模型".to_string());
    }

    let ctx = build_context(state, app, params.conversation_id).await;
    let history = db::list_messages(params.conversation_id, Some(HISTORY_LIMIT))?;
    let mut turns = build_turns(&history, true);
    let _ = compress_context(&mut turns, config.max_input_tokens, estimate_context_usage(&history));

    let tools = agent::tool_definitions();

    let (mut reply, reasoning, tool_log, tool_records, usage, duration_ms) =
        run_tool_loop(&config, app, params.conversation_id, &model, params.reasoning_effort.as_deref(), &tools, &ctx, &mut turns, Some(&state.chat_cancel_flag)).await?;
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
async fn reply_ask_user(params: ReplyAskUserParams) -> Result<Value, String> {
    agent::reply_ask_user(params.conversation_id, params.reply).await?;
    serde_json::to_value(()).map_err(|e| e.to_string())
}

/// 生成会话标题（模型生成，≤20 字；非阻塞）
fn generate_title(
    app: &AppHandle,
    conversation_id: i64,
    config: &ai_core::AiConfig,
    model: &str,
    user_content: &str,
    reply: &str,
) {
    let app = app.clone();
    let config = config.clone();
    let model = model.to_string();
    let user_content = user_content.to_string();
    let reply = reply.to_string();
    tauri::async_runtime::spawn(async move {
        let title_prompt = ai_core::prompt::system_prompt(&ai_core::PromptKind::Title);
        let user_msg = format!(
            "用户消息：{}\n\nAI 回复（开头）：{}",
            crate::utils::format::truncate_chars(&user_content, 200),
            crate::utils::format::truncate_chars(&reply, 200)
        );
        let turns = vec![
            ChatTurn {
                role: "system".to_string(),
                content: Some(title_prompt),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            ChatTurn {
                role: "user".to_string(),
                content: Some(user_msg.clone()),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        ];
        let _ = turns; // 保留多轮形式（当前只用首轮），后续如需扩展标题上下文可直接复用
        match ai_core::chat(&config, ai_core::PromptKind::Title, user_msg.clone(), Some(&model)).await {
            Ok(title) => {
                let title = title.trim().trim_matches('"').chars().take(20).collect::<String>();
                if !title.is_empty() {
                    let _ = db::rename_conversation(conversation_id, &title);
                    let _ = app.emit(
                        "conversation-title-updated",
                        json!({ "conversationId": conversation_id, "title": title }),
                    );
                }
            }
            Err(e) => log_warn!("[Experimental] 生成会话标题失败: {}", e),
        }
        let _ = turns;
    });
}

/// 推送聊天状态事件（如上下文压缩提示）
fn emit_chat_status(app: &AppHandle, conversation_id: i64, message: &str) {
    let _ = app.emit(
        "ai-chat-stream",
        json!({
            "conversationId": conversation_id,
            "status": message
        }),
    );
}

/// AI 日志分析（5 环节流式）：读配置 → 系统提示词 → 用户消息（截断保护）→
/// `chat_completions_stream` 流式 → 每段 delta 即时 emit；检测到 `【STEP:N/5】`
/// 标记时 emit step 事件且该标记行不进展示文本；流结束后 emit 剔除标记后的全文。
async fn ai_analyze_log(app: &AppHandle, params: AiAnalyzeLogParams) -> Result<Value, String> {
    let log_text = params.log_text.trim();
    if log_text.is_empty() {
        return Err("日志内容不能为空".to_string());
    }

    let config = ai_core::load_config_async().await;
    if config.base_url.trim().is_empty() {
        return Err("未配置 AI 服务地址，请先在「实验性 → AI 设置」中配置本地 OpenAI 兼容服务".to_string());
    }
    let model = if let Some(m) = params.model.as_deref() {
        if m.trim().is_empty() { config.resolve_model(None) } else { m.trim().to_string() }
    } else {
        config.resolve_model(None)
    };
    if model.is_empty() {
        return Err("未启用任何模型，请先在「实验性 → AI 设置」中加载并启用模型".to_string());
    }

    let system = ai_core::prompt::system_prompt(&ai_core::PromptKind::LogAnalyzeSteps);

    // 本地预检：用规则引擎先收敛问题范围，把「本地初检结果摘要」作为上下文注入 AI，
    // 避免把超长全文直接发给模型（用户原始日志可能是数千行）
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
    let _ = app.emit("ai-analyze-stream", json!({ "done": true, "content": content }));
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