//! 会话表数据访问（创建/列表/删除/重命名/活动时间）

use rusqlite::types::Value;

use super::init::{conv_mapper, touch_conversation_with};
use super::schema::{now, CONVERSATIONS, MESSAGES, TOOL_CALLS};
use crate::commands::experimental::types::ConversationItem;
use crate::utils::sqlite::{with_conn, Cond};

/// 创建会话，返回会话 id；`title` 为空时使用默认标题「新对话」
pub fn create_conversation(title: &str) -> Result<i64, String> {
    let effective_title = if title.trim().is_empty() {
        "新对话".to_string()
    } else {
        title.trim().to_string()
    };
    let ts = now();
    with_conn(|conn| {
        CONVERSATIONS.insert(
            conn,
            &[
                ("title", Value::from(effective_title)),
                ("created_at", Value::from(ts)),
                ("updated_at", Value::from(ts)),
            ],
        )
    })
}

/// 会话列表（按最近更新时间倒序）
pub fn list_conversations() -> Result<Vec<ConversationItem>, String> {
    with_conn(|conn| {
        CONVERSATIONS.query(
            conn,
            &["id", "title", "created_at", "updated_at"],
            &[],
            Some(("updated_at", false)),
            None,
            conv_mapper,
        )
    })
}

/// 删除会话（级联删除其消息与工具调用记录）
pub fn delete_conversation(id: i64) -> Result<(), String> {
    with_conn(|conn| {
        TOOL_CALLS.delete_where(conn, &[Cond::eq("conversation_id", Value::from(id))])?;
        CONVERSATIONS.delete_by_id(conn, id).map(|_| ())
    })
}

/// 重命名会话
pub fn rename_conversation(id: i64, title: &str) -> Result<(), String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("会话标题不能为空".to_string());
    }
    with_conn(|conn| {
        CONVERSATIONS
            .update_by_id(
                conn,
                id,
                &[
                    ("title", Value::from(title.to_string())),
                    ("updated_at", Value::from(now())),
                ],
            )
            .map(|_| ())
    })
}

/// 更新会话最近活动时间（消息写入后调用）
pub fn touch_conversation(id: i64) -> Result<(), String> {
    with_conn(|conn| touch_conversation_with(conn, id))
}

/// 清空会话消息（保留会话本身）
pub fn clear_conversation(id: i64) -> Result<(), String> {
    with_conn(|conn| {
        MESSAGES.delete_where(conn, &[Cond::eq("conversation_id", Value::from(id))])?;
        TOOL_CALLS
            .delete_where(conn, &[Cond::eq("conversation_id", Value::from(id))])
            .map(|_| ())
    })
}

/// 校验会话是否存在
pub fn conversation_exists(id: i64) -> Result<bool, String> {
    with_conn(|conn| {
        let count = CONVERSATIONS.count(conn, &[Cond::eq("id", Value::from(id))])?;
        Ok(count > 0)
    })
}
