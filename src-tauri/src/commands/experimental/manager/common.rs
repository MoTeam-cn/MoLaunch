//! manager 公共辅助：开关校验 / 上下文构建 / 压缩 turns / 配置摘要
//!
//! 供 dispatcher（ensure_enabled）、chat（build_context / build_chat_turns）、
//! 三入口复用。

use tauri::AppHandle;

use super::super::agent::AgentContext;
use super::super::db;
use super::super::types::MessageItem;
use super::compression;
use super::emit::emit_chat_status;
use crate::ai_core;
use crate::ai_core::client::ChatTurn;
use crate::state::AppState;

/// 作为 AI 上下文的历史消息条数
pub(super) const HISTORY_LIMIT: i64 = 30;

/// 校验实验性功能开关，未开启时返回错误
pub(super) async fn ensure_enabled(state: &AppState) -> Result<(), String> {
    let enabled = state.config.lock().await.experimental_enabled;
    if enabled {
        Ok(())
    } else {
        Err("实验性功能未开启，请先在「设置 → 进阶设置」中启用".to_string())
    }
}

/// 构造聊天上下文 turns（压缩管线统一入口）
///
/// 拉取历史窗口内的工具记录 → `compact_if_needed`（触发判定 / L1 / L3 / 重塑）→
/// 发生压缩时推送状态提示。供 chat_send / regenerate / edit 三入口复用。
pub(super) async fn build_chat_turns(
    app: &AppHandle,
    config: &ai_core::AiConfig,
    model: &str,
    conversation_id: i64,
    history: &[MessageItem],
) -> Result<Vec<ChatTurn>, String> {
    let tool_records = db::list_tool_calls(conversation_id)?;
    let (turns, info) =
        compression::compact_if_needed(config, model, conversation_id, history, &tool_records)
            .await?;
    if info.compacted {
        emit_chat_status(
            app,
            conversation_id,
            &format!(
                "上下文已自动压缩（原因：{}）{}",
                info.reason,
                if info.has_summary {
                    "，已生成摘要"
                } else {
                    ""
                }
            ),
        );
    }
    Ok(turns)
}

/// 构造 Agent 工具执行上下文（游戏目录 + 启动器版本 + 配置摘要 + 隔离模式 + 会话/app）
pub(super) async fn build_context(
    state: &AppState,
    app: &AppHandle,
    conversation_id: i64,
) -> AgentContext {
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
