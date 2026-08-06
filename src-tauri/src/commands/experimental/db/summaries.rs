//! 会话压缩摘要表数据访问（增/查/清）
//!
//! 摘要以独立表持久化：压缩后的「远距离记忆」，供后续对话注入，前端零影响。

use rusqlite::types::Value;

use super::schema::{now, CONVERSATION_SUMMARIES};
use crate::utils::sqlite::{with_conn, Cond};

/// 会话摘要行
pub struct ConversationSummary {
    /// 摘要文本
    pub summary: String,
    /// 摘要版本（每次压缩递增，供前端判断新旧）
    pub summary_version: i64,
    /// 最近更新时间
    pub updated_at: i64,
}

/// 写入会话摘要（存在则覆盖，版本 +1）
///
/// 摘要表主键为 `conversation_id`，无自增 `id`，故 upsert 采用
/// 「先按会话删除、再插入」的方式实现（`update_by_id` 固定按 `id` 列，不适用于本表）。
pub fn upsert_summary(conversation_id: i64, summary: &str) -> Result<(), String> {
    with_conn(|conn| {
        let existing: i64 = CONVERSATION_SUMMARIES.count(
            conn,
            &[Cond::eq("conversation_id", Value::from(conversation_id))],
        )?;
        let next_version = existing + 1;
        let ts = now();
        CONVERSATION_SUMMARIES.delete_where(
            conn,
            &[Cond::eq("conversation_id", Value::from(conversation_id))],
        )?;
        CONVERSATION_SUMMARIES.insert(
            conn,
            &[
                ("conversation_id", Value::from(conversation_id)),
                ("summary", Value::from(summary.to_string())),
                ("summary_version", Value::from(next_version)),
                ("created_at", Value::from(ts)),
                ("updated_at", Value::from(ts)),
            ],
        )?;
        Ok(())
    })
}

/// 读取会话摘要
pub fn get_summary(conversation_id: i64) -> Result<Option<ConversationSummary>, String> {
    with_conn(|conn| {
        CONVERSATION_SUMMARIES.query_first(
            conn,
            &["summary", "summary_version", "updated_at"],
            &[Cond::eq("conversation_id", Value::from(conversation_id))],
            None,
            |row| {
                Ok(ConversationSummary {
                    summary: row.get(0)?,
                    summary_version: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            },
        )
    })
}

/// 删除会话摘要
pub fn delete_summary(conversation_id: i64) -> Result<(), String> {
    with_conn(|conn| {
        CONVERSATION_SUMMARIES
            .delete_where(
                conn,
                &[Cond::eq("conversation_id", Value::from(conversation_id))],
            )
            .map(|_| ())
    })
}
