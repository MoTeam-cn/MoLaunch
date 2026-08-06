//! 工具调用记录表数据访问（批量增/查/清）

use rusqlite::types::Value;

use super::init::tool_mapper;
use super::schema::{now, TOOL_CALLS};
use crate::commands::experimental::types::ToolCallRecord;
use crate::utils::sqlite::{with_conn, Cond};

/// 批量追加工具调用记录（绑定到 AI 回复消息，刷新/重启后工具链仍保留）
///
/// `message_id` 为 AI 回复消息 id；`records` 中各条 `message_id` 被忽略，
/// 统一落库为传入的 `message_id`（调用方构造记录时未知最终消息 id，先以 0 占位）。
pub fn add_tool_calls(
    conversation_id: i64,
    message_id: i64,
    records: &[ToolCallRecord],
) -> Result<(), String> {
    with_conn(|conn| {
        for rec in records {
            TOOL_CALLS.insert(
                conn,
                &[
                    ("conversation_id", Value::from(conversation_id)),
                    ("message_id", Value::from(message_id)),
                    ("seq", Value::from(rec.seq)),
                    ("name", Value::from(rec.name.clone())),
                    ("arguments", Value::from(rec.arguments.clone())),
                    (
                        "output",
                        Value::from(rec.output.clone().unwrap_or_default()),
                    ),
                    (
                        "pre_content",
                        Value::from(rec.pre_content.clone().unwrap_or_default()),
                    ),
                    ("created_at", Value::from(now())),
                ],
            )?;
        }
        Ok(())
    })
}

/// 读取某会话的全部工具调用记录（按 seq 升序；前端按 message_id 分组展示）
pub fn list_tool_calls(conversation_id: i64) -> Result<Vec<ToolCallRecord>, String> {
    with_conn(|conn| {
        TOOL_CALLS.query(
            conn,
            &[
                "message_id",
                "seq",
                "name",
                "arguments",
                "output",
                "pre_content",
            ],
            &[Cond::eq("conversation_id", Value::from(conversation_id))],
            Some(("seq", true)),
            None,
            tool_mapper,
        )
    })
}

/// 删除某条消息的所有工具调用记录
pub fn delete_tool_calls_for_message(conversation_id: i64, message_id: i64) -> Result<(), String> {
    with_conn(|conn| {
        TOOL_CALLS
            .delete_where(
                conn,
                &[
                    Cond::eq("conversation_id", Value::from(conversation_id)),
                    Cond::eq("message_id", Value::from(message_id)),
                ],
            )
            .map(|_| ())
    })
}

/// 删除某条消息之后的所有工具调用记录
pub fn delete_tool_calls_after(conversation_id: i64, message_id: i64) -> Result<(), String> {
    with_conn(|conn| {
        TOOL_CALLS
            .delete_where(
                conn,
                &[
                    Cond::eq("conversation_id", Value::from(conversation_id)),
                    Cond::gt("message_id", Value::from(message_id)),
                ],
            )
            .map(|_| ())
    })
}

/// 清空会话的全部工具调用记录
pub fn clear_tool_calls(conversation_id: i64) -> Result<(), String> {
    with_conn(|conn| {
        TOOL_CALLS
            .delete_where(
                conn,
                &[Cond::eq("conversation_id", Value::from(conversation_id))],
            )
            .map(|_| ())
    })
}
