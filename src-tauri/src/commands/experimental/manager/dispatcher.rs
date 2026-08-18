//! action 注册表与分发入口
//!
//! 所有 action 先校验 `experimental_enabled`，未开启时返回错误并提示
//! 用户前往「设置 → 进阶设置」开启实验性功能。

use once_cell::sync::Lazy;
use serde_json::{json, Value};
use std::sync::atomic::Ordering;
use tauri::AppHandle;

use super::super::agent;
use super::super::db;
use super::super::types::{
    AiAnalyzeLogParams, ChatSendParams, CollectContextParams, CollectContextResult,
    ConversationIdParams, ConversationItem, CreateConversationParams, DeleteMessageParams,
    EditMessageParams, ListMessagesParams, ListToolCallsParams, MessageItem, RegenerateReplyParams,
    RenameConversationParams, ReplyAskUserParams,
};
use super::analyze::ai_analyze_log;
use super::chat::{chat_send, edit_message, regenerate_reply, reply_ask_user};
use super::common::ensure_enabled;
use crate::ai_core;
use crate::commands::ai::types::{AiProbeParams, AnalyzeCrashParams};
use crate::handler;
use crate::mod_translation;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

/// 全局 action 注册表（进程启动时初始化一次）
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
                title: p
                    .title
                    .filter(|t| !t.trim().is_empty())
                    .unwrap_or_else(|| "新对话".to_string()),
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
            // 清理会话压缩的防抖记录与持久化摘要，避免内存/数据残留
            db::delete_summary(p.conversation_id)?;
            super::compression::clear_cooldown(p.conversation_id);
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
            let items: Vec<crate::commands::experimental::types::ToolCallRecord> =
                db::list_tool_calls(p.conversation_id)?;
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
            let ctx =
                super::common::build_context(&state, &app, p.conversation_id.unwrap_or(0)).await;
            let text = agent::collect_context(&p.kind, &version_id, &ctx)?;
            let result = CollectContextResult { kind: p.kind, text };
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_installed_versions",
        handler!(state, app, _params, {
            ensure_enabled(&state).await?;
            let ctx = super::common::build_context(&state, &app, 0).await;
            let ids = crate::minecraft::version::scan::scan_installed_versions(&ctx.game_dir)
                .into_iter()
                .map(|i| i.id)
                .collect::<Vec<_>>();
            serde_json::to_value(ids).map_err(|e| e.to_string())
        }),
    );

    // ===== AI（本地 OpenAI 兼容服务）=====
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
            let cfg: ai_core::AiConfig =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
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
            let models = ai_core::list_models(&config)
                .await
                .map_err(|e| e.to_string())?;
            serde_json::to_value(models).map_err(|e| e.to_string())
        }),
    );

    // ===== 模组翻译（AI 批量翻译 JAR 语言文件）=====
    d.register(
        "mod_translation_analyze",
        handler!(state, _app, params, {
            ensure_enabled(&state).await?;
            let p: mod_translation::types::AnalyzeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let result = mod_translation::analyze_jar(p).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "mod_translation_start",
        handler!(state, app, params, {
            ensure_enabled(&state).await?;
            let p: mod_translation::types::StartParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let result = mod_translation::start_task(app, p).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "mod_translation_cancel",
        handler!(state, _app, _params, {
            ensure_enabled(&state).await?;
            mod_translation::cancel_task()?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "mod_translation_status",
        handler!(state, _app, _params, {
            ensure_enabled(&state).await?;
            serde_json::to_value(mod_translation::current_status()).map_err(|e| e.to_string())
        }),
    );

    d
});

/// action 分发入口（由 experimental 命令层调用）
pub async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}
