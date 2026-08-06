//! 实验性功能 SQLite 存储（连接由系统维护）
//!
//! - 数据库位置：`Storage::base_dir()/chat.db`（`.Molaunch/chat.db`）。
//! - 连接挂载：启动时若 `experimental_enabled` 已开启，或运行中在设置页开启时，
//!   调用 [`ensure_initialized`] 挂载全局连接（幂等）；未启用则不挂载。
//! - 表结构：本模块仅声明 schema（[`CHAT_TABLES`]），建表与自动迁移（补列/删列）
//!   统一由 `crate::utils::sqlite` 负责。
//! - 数据访问：全部经 `crate::utils::sqlite::Table` 的语义接口（表 + 列 + 条件）完成，
//!   本模块不出现任何 SQL 语句。
//! - 旧版数据库（`experimental/chat.db`）首次挂载时自动迁移。

use rusqlite::{types::Value, Connection, Row};

use super::types::{ConversationItem, MessageItem, ToolCallRecord};
use crate::storage::Storage;
use crate::utils::sqlite::{mount, with_conn, ColumnDef, Cond, Table, TableDef};
use crate::{log_debug, log_warn};

/// 聊天库声明式 schema（表 + 可迁移列 + 保留列）
///
/// 调整结构只需修改此定义：新库按 `create_sql` 建表；
/// 旧库挂载时自动补齐缺失列、删除多余列。
static CHAT_TABLES: &[TableDef] = &[
    TableDef {
        name: "conversations",
        create_sql: "CREATE TABLE IF NOT EXISTS conversations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL DEFAULT '新对话',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );",
        columns: &[],
        preserved: &["id", "title", "created_at", "updated_at"],
    },
    TableDef {
        name: "messages",
        create_sql: "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            conversation_id INTEGER NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            pair_id INTEGER,
            version_id TEXT,
            reasoning_content TEXT,
            model TEXT,
            retry_count INTEGER DEFAULT 1,
            prompt_tokens INTEGER DEFAULT 0,
            completion_tokens INTEGER DEFAULT 0,
            total_tokens INTEGER DEFAULT 0,
            duration_ms INTEGER DEFAULT 0,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id, id);",
        columns: &[
            ColumnDef {
                name: "pair_id",
                column_type: "INTEGER",
            },
            ColumnDef {
                name: "version_id",
                column_type: "TEXT",
            },
            ColumnDef {
                name: "reasoning_content",
                column_type: "TEXT",
            },
            ColumnDef {
                name: "model",
                column_type: "TEXT",
            },
            ColumnDef {
                name: "retry_count",
                column_type: "INTEGER",
            },
            ColumnDef {
                name: "prompt_tokens",
                column_type: "INTEGER",
            },
            ColumnDef {
                name: "completion_tokens",
                column_type: "INTEGER",
            },
            ColumnDef {
                name: "total_tokens",
                column_type: "INTEGER",
            },
            ColumnDef {
                name: "duration_ms",
                column_type: "INTEGER",
            },
        ],
        preserved: &["id", "conversation_id", "role", "content", "created_at", "pair_id", "version_id", "model", "retry_count", "prompt_tokens", "completion_tokens", "total_tokens", "duration_ms"],
    },
    TableDef {
        name: "tool_calls",
        create_sql: "CREATE TABLE IF NOT EXISTS tool_calls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            conversation_id INTEGER NOT NULL,
            message_id INTEGER NOT NULL,
            seq INTEGER NOT NULL,
            name TEXT NOT NULL,
            arguments TEXT NOT NULL,
            output TEXT,
            pre_content TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_tool_calls_message ON tool_calls(conversation_id, message_id, seq);",
        columns: &[ColumnDef {
            name: "pre_content",
            column_type: "TEXT",
        }],
        preserved: &[
            "id",
            "conversation_id",
            "message_id",
            "seq",
            "name",
            "arguments",
            "output",
            "pre_content",
            "created_at",
        ],
    },
];

/// 会话表访问句柄
const CONVERSATIONS: Table = Table::new("conversations");
/// 消息表访问句柄
const MESSAGES: Table = Table::new("messages");
/// 工具调用记录表访问句柄
const TOOL_CALLS: Table = Table::new("tool_calls");

const MSG_COLUMNS: &[&str] = &[
    "id",
    "role",
    "content",
    "created_at",
    "pair_id",
    "version_id",
    "reasoning_content",
    "model",
    "retry_count",
    "prompt_tokens",
    "completion_tokens",
    "total_tokens",
    "duration_ms",
];

fn db_path() -> std::path::PathBuf {
    Storage::instance().base_dir().join("chat.db")
}

/// 旧版数据库路径（v1：experimental/chat.db），首次挂载时迁移
fn legacy_db_path() -> std::path::PathBuf {
    Storage::instance()
        .base_dir()
        .join("experimental")
        .join("chat.db")
}

/// 挂载聊天库（幂等）：迁移旧库 + 建表/自动迁移列 + 建立全局连接
///
/// 由启动流程（配置已启用时）与 `apply_config`（运行中开启时）调用。
pub fn ensure_initialized() -> Result<(), String> {
    let path = db_path();
    migrate_legacy_db(&path)?;
    mount(&path, CHAT_TABLES)?;
    log_debug!("[Experimental] SQLite 聊天库已就绪: {}", path.display());
    Ok(())
}

/// 迁移旧版数据库（experimental/chat.db → .Molaunch/chat.db）
///
/// 仅在旧库存在且新库不存在时复制；完成后不删除旧库（保守策略，避免误删用户数据）。
/// 旧库消息缺失的 pair_id/version_id 列由声明式迁移自动补齐（补为 NULL）。
fn migrate_legacy_db(path: &std::path::Path) -> Result<(), String> {
    let legacy = legacy_db_path();
    if !legacy.exists() || path.exists() {
        return Ok(());
    }
    match std::fs::copy(&legacy, path) {
        Ok(_) => log_debug!(
            "[Experimental] 已迁移旧聊天库: {} -> {}",
            legacy.display(),
            path.display()
        ),
        Err(e) => log_warn!("[Experimental] 旧聊天库迁移失败（忽略，将新建空库）: {}", e),
    }
    Ok(())
}

fn now() -> i64 {
    chrono::Local::now().timestamp()
}

/// 行 → 会话映射
fn conv_mapper(row: &Row<'_>) -> Result<ConversationItem, rusqlite::Error> {
    Ok(ConversationItem {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
    })
}

/// 行 → 消息映射
fn msg_mapper(row: &Row<'_>) -> Result<MessageItem, rusqlite::Error> {
    Ok(MessageItem {
        id: row.get(0)?,
        role: row.get(1)?,
        content: row.get(2)?,
        created_at: row.get(3)?,
        pair_id: row.get(4)?,
        version_id: row.get(5)?,
        reasoning_content: row.get(6)?,
        model: row.get(7)?,
        retry_count: row.get(8)?,
        prompt_tokens: row.get(9)?,
        completion_tokens: row.get(10)?,
        total_tokens: row.get(11)?,
        duration_ms: row.get(12)?,
    })
}

/// 行 → 工具调用记录映射
fn tool_mapper(row: &Row<'_>) -> Result<ToolCallRecord, rusqlite::Error> {
    Ok(ToolCallRecord {
        message_id: row.get(0)?,
        seq: row.get(1)?,
        name: row.get(2)?,
        arguments: row.get(3)?,
        output: row.get(4)?,
        pre_content: row.get(5)?,
    })
}

/// 追加一批工具调用记录（绑定到 AI 回复消息；`seq` 从 0 起按数组顺序生成）
pub fn add_tool_calls(
    conversation_id: i64,
    message_id: i64,
    calls: &[ToolCallRecord],
) -> Result<(), String> {
    with_conn(|conn| {
        for (seq, call) in calls.iter().enumerate() {
            TOOL_CALLS.insert(
                conn,
                &[
                    ("conversation_id", Value::from(conversation_id)),
                    ("message_id", Value::from(message_id)),
                    ("seq", Value::from(seq as i64)),
                    ("name", Value::from(call.name.clone())),
                    ("arguments", Value::from(call.arguments.clone())),
                    ("output", Value::from(call.output.clone())),
                    ("pre_content", Value::from(call.pre_content.clone())),
                    ("created_at", Value::from(now())),
                ],
            )?;
        }
        Ok(())
    })
}

/// 读取会话内全部工具调用记录（按 seq 升序，前端按 message_id 分组展示工具链）
pub fn list_tool_calls(conversation_id: i64) -> Result<Vec<ToolCallRecord>, String> {
    let conds = [Cond::eq("conversation_id", Value::from(conversation_id))];
    with_conn(|conn| {
        TOOL_CALLS.query(
            conn,
            &["message_id", "seq", "name", "arguments", "output", "pre_content"],
            &conds,
            Some(("seq", true)),
            None,
            tool_mapper,
        )
    })
}

/// 删除指定消息的工具调用记录（级联配对删除时同步清理）
pub fn delete_tool_calls_for_message(conversation_id: i64, message_ids: &[i64]) -> Result<(), String> {
    with_conn(|conn| {
        for mid in message_ids {
            TOOL_CALLS.delete_where(
                conn,
                &[
                    Cond::eq("conversation_id", Value::from(conversation_id)),
                    Cond::eq("message_id", Value::from(*mid)),
                ],
            )?;
        }
        Ok(())
    })
}

/// 删除某条消息之后所有消息的工具调用记录（编辑/重新回复时清理旧链）
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

/// 删除会话的全部工具调用记录（清空/删除会话时同步清理）
pub fn clear_tool_calls(conversation_id: i64) -> Result<(), String> {
    with_conn(|conn| {
        TOOL_CALLS
            .delete_where(conn, &[Cond::eq("conversation_id", Value::from(conversation_id))])
            .map(|_| ())
    })
}

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
                &[("title", Value::from(title.to_string())), ("updated_at", Value::from(now()))],
            )
            .map(|_| ())
    })
}

/// 读取会话消息（按时间正序；`limit` 用于截取最近 N 条作为 AI 上下文）
pub fn list_messages(conversation_id: i64, limit: Option<i64>) -> Result<Vec<MessageItem>, String> {
    let conds = [Cond::eq("conversation_id", Value::from(conversation_id))];
    with_conn(|conn| match limit {
        Some(lim) => {
            // 取最近 N 条：按 id 倒序取上限，再反转成正序（与"子查询取尾再升序"等价）
            let mut items =
                MESSAGES.query(conn, MSG_COLUMNS, &conds, Some(("id", false)), Some(lim), msg_mapper)?;
            items.reverse();
            Ok(items)
        }
        None => MESSAGES.query(conn, MSG_COLUMNS, &conds, Some(("id", true)), None, msg_mapper),
    })
}

/// 追加消息，返回消息 id
///
/// - `pair_id`：与消息配对的另一条消息 id（用户↔AI 一一配对，删除时级联）
/// - `version_id`：该消息对应的游戏版本（AI 工具调用时记录）
/// - `reasoning_content`：思考模型的推理内容（仅 AI 回复携带），其余传 `None`
/// - `model`：生成该回复的模型名（仅 AI 回复携带，用于消息图标固定展示），其余传 `None`
/// - `retry_count`：该回复的生成序号（首次为 1，重新生成递增，用于「第 N 次重试」标识）
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

/// 更新会话最近活动时间（消息写入后调用）
pub fn touch_conversation(id: i64) -> Result<(), String> {
    with_conn(|conn| touch_conversation_with(conn, id))
}

/// 内部实现：在已持有的连接上更新会话活动时间（供同一连接内复用，避免嵌套加锁）
fn touch_conversation_with(conn: &Connection, id: i64) -> Result<(), String> {
    CONVERSATIONS
        .update_by_id(conn, id, &[("updated_at", Value::from(now()))])
        .map(|_| ())
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
