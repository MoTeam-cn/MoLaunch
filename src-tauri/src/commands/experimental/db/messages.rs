//! 消息表数据访问（列表/增删改/配对/重试序号）

use rusqlite::types::Value;

use super::init::{msg_mapper, touch_conversation_with};
use super::schema::{now, MESSAGES, MSG_COLUMNS, TOOL_CALLS};
use crate::commands::experimental::types::MessageItem;
use crate::utils::sqlite::{with_conn, Cond};

/// 读取会话消息（按时间正序；`limit` 用于截取最近 N 条作为 AI 上下文）
pub fn list_messages(conversation_id: i64, limit: Option<i64>) -> Result<Vec<MessageItem>, String> {
    let conds = [Cond::eq("conversation_id", Value::from(conversation_id))];
    with_conn(|conn| match limit {
        Some(lim) => {
            // 取最近 N 条：按 id 倒序取上限，再反转成正序（与"子查询取尾再升序"等价）
            let mut items = MESSAGES.query(
                conn,
                MSG_COLUMNS,
                &conds,
                Some(("id", false)),
                Some(lim),
                msg_mapper,
            )?;
            items.reverse();
            Ok(items)
        }
        None => MESSAGES.query(
            conn,
            MSG_COLUMNS,
            &conds,
            Some(("id", true)),
            None,
            msg_mapper,
        ),
    })
}

/// 追加消息，返回消息 id
///
/// - `pair_id`：与消息配对的另一条消息 id（用户↔AI 一一配对，删除时级联）
/// - `version_id`：该消息对应的游戏版本（AI 工具调用时记录）
/// - `reasoning_content`：思考模型的推理内容（仅 AI 回复携带），其余传 `None`
/// - `model`：生成该回复的模型名（仅 AI 回复携带，用于消息图标固定展示），其余传 `None`
/// - `retry_count`：该回复的生成序号（首次为 1，重新生成递增，用于「第 N 次重试」标识）
#[allow(clippy::too_many_arguments)]
pub fn add_message(
    conversation_id: i64,
    role: &str,
    content: &str,
    pair_id: Option<i64>,
    version_id: Option<String>,
    reasoning_content: Option<String>,
    model: Option<String>,
    retry_count: i64,
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
    duration_ms: u64,
) -> Result<i64, String> {
    with_conn(|conn| {
        MESSAGES.insert(
            conn,
            &[
                ("conversation_id", Value::from(conversation_id)),
                ("role", Value::from(role.to_string())),
                ("content", Value::from(content.to_string())),
                ("created_at", Value::from(now())),
                ("pair_id", Value::from(pair_id)),
                ("version_id", Value::from(version_id)),
                ("reasoning_content", Value::from(reasoning_content)),
                ("model", Value::from(model)),
                ("retry_count", Value::from(retry_count)),
                ("prompt_tokens", Value::from(prompt_tokens as i64)),
                ("completion_tokens", Value::from(completion_tokens as i64)),
                ("total_tokens", Value::from(total_tokens as i64)),
                ("duration_ms", Value::from(duration_ms as i64)),
            ],
        )
    })
}

/// 更新消息内容（编辑用户消息）
pub fn update_message_content(id: i64, content: &str) -> Result<(), String> {
    with_conn(|conn| {
        MESSAGES
            .update_by_id(conn, id, &[("content", Value::from(content.to_string()))])
            .map(|_| ())
    })
}

/// 回填配对 id（AI 回复保存后，将其 id 写入对应用户消息的 pair_id，删除时级联）
pub fn set_message_pair_id(message_id: i64, pair_id: i64) -> Result<(), String> {
    with_conn(|conn| {
        MESSAGES
            .update_by_id(conn, message_id, &[("pair_id", Value::from(pair_id))])
            .map(|_| ())
    })
}

/// 读取单条消息（id / conversation_id 均匹配）；不存在时返回 None
pub fn get_message(conversation_id: i64, id: i64) -> Result<Option<(i64, String, String)>, String> {
    with_conn(|conn| {
        MESSAGES.query_first(
            conn,
            &["id", "role", "content"],
            &[
                Cond::eq("id", Value::from(id)),
                Cond::eq("conversation_id", Value::from(conversation_id)),
            ],
            None,
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
    })
}

/// 读取单条消息的 pair_id（消息不存在或为空时返回 None）
pub fn get_message_pair_id(conversation_id: i64, id: i64) -> Result<Option<i64>, String> {
    with_conn(|conn| {
        let pair: Option<Option<i64>> = MESSAGES.query_first(
            conn,
            &["pair_id"],
            &[
                Cond::eq("id", Value::from(id)),
                Cond::eq("conversation_id", Value::from(conversation_id)),
            ],
            None,
            |row| row.get(0),
        )?;
        Ok(pair.flatten())
    })
}

/// 读取单条消息的生成序号（retry_count）；消息不存在或旧数据无该列时返回 None
pub fn get_message_retry_count(conversation_id: i64, id: i64) -> Result<Option<i64>, String> {
    with_conn(|conn| {
        MESSAGES.query_first(
            conn,
            &["retry_count"],
            &[
                Cond::eq("id", Value::from(id)),
                Cond::eq("conversation_id", Value::from(conversation_id)),
            ],
            None,
            |row| row.get(0),
        )
    })
}

/// 删除单条消息及配对消息（级联配对）
///
/// 若删除 AI 消息，同时删除其对应的用户消息（pair_id 指向）；反之亦然。
pub fn delete_message(conversation_id: i64, message_id: i64) -> Result<Vec<i64>, String> {
    with_conn(|conn| {
        let found: Option<(i64, String, String)> = MESSAGES.query_first(
            conn,
            &["id", "role", "content"],
            &[
                Cond::eq("id", Value::from(message_id)),
                Cond::eq("conversation_id", Value::from(conversation_id)),
            ],
            None,
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        let Some((_, role, _)) = found else {
            return Err("消息不存在".to_string());
        };

        // 配对规则：AI 消息的 pair_id = 对应用户消息 id；用户消息通过查 pair_id 反向找 AI
        let mut ids_to_delete = vec![message_id];
        let pair: Option<Option<i64>> = MESSAGES.query_first(
            conn,
            &["pair_id"],
            &[Cond::eq("id", Value::from(message_id))],
            None,
            |row| row.get(0),
        )?;
        if let Some(pid) = pair.flatten() {
            ids_to_delete.push(pid);
        } else if role == "user" {
            // 用户消息：删除其后的 AI 回复（pair_id 指向本条用户消息）
            let ai_id: Option<i64> = MESSAGES.query_first(
                conn,
                &["id"],
                &[
                    Cond::eq("conversation_id", Value::from(conversation_id)),
                    Cond::eq("pair_id", Value::from(message_id)),
                ],
                Some(("id", true)),
                |row| row.get(0),
            )?;
            if let Some(aid) = ai_id {
                ids_to_delete.push(aid);
            }
        }

        for id in &ids_to_delete {
            MESSAGES.delete_by_id(conn, *id)?;
        }

        // 级联清理这些消息绑定的工具调用记录
        for id in &ids_to_delete {
            TOOL_CALLS.delete_where(
                conn,
                &[
                    Cond::eq("conversation_id", Value::from(conversation_id)),
                    Cond::eq("message_id", Value::from(*id)),
                ],
            )?;
        }

        touch_conversation_with(conn, conversation_id)?;
        Ok(ids_to_delete)
    })
}

/// 删除某条消息之后的所有消息（编辑用户消息后，删除其后的 AI 回复链）
pub fn delete_messages_after(conversation_id: i64, message_id: i64) -> Result<usize, String> {
    with_conn(|conn| {
        let count = MESSAGES.delete_where(
            conn,
            &[
                Cond::eq("conversation_id", Value::from(conversation_id)),
                Cond::gt("id", Value::from(message_id)),
            ],
        )?;
        // 同步清理被删消息之后绑定的工具调用记录
        TOOL_CALLS.delete_where(
            conn,
            &[
                Cond::eq("conversation_id", Value::from(conversation_id)),
                Cond::gt("message_id", Value::from(message_id)),
            ],
        )?;
        touch_conversation_with(conn, conversation_id)?;
        Ok(count)
    })
}

/// 查找最后一条用户消息（用于"仅最近一条可编辑"校验）
pub fn last_user_message_id(conversation_id: i64) -> Result<Option<i64>, String> {
    with_conn(|conn| {
        MESSAGES.query_first(
            conn,
            &["id"],
            &[
                Cond::eq("conversation_id", Value::from(conversation_id)),
                Cond::eq("role", Value::from("user".to_string())),
            ],
            Some(("id", false)),
            |row| row.get(0),
        )
    })
}
