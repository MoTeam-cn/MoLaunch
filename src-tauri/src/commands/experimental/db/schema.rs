//! 聊天库 schema 声明与表句柄
//!
//! 调整结构只需修改此定义：新库按 `create_sql` 建表；旧库挂载时自动补齐缺失列、删除多余列。

use crate::storage::Storage;
use crate::utils::sqlite::{ColumnDef, Table, TableDef};

/// 聊天库声明式 schema（表 + 可迁移列 + 保留列）
pub(super) static CHAT_TABLES: &[TableDef] = &[
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
    TableDef {
        name: "conversation_summaries",
        create_sql: "CREATE TABLE IF NOT EXISTS conversation_summaries (
            conversation_id INTEGER PRIMARY KEY,
            summary TEXT NOT NULL,
            summary_version INTEGER DEFAULT 1,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );",
        columns: &[ColumnDef {
            name: "summary_version",
            column_type: "INTEGER",
        }],
        preserved: &["conversation_id", "summary", "summary_version", "created_at", "updated_at"],
    },
];

/// 会话表访问句柄
pub(super) const CONVERSATIONS: Table = Table::new("conversations");
/// 消息表访问句柄
pub(super) const MESSAGES: Table = Table::new("messages");
/// 工具调用记录表访问句柄
pub(super) const TOOL_CALLS: Table = Table::new("tool_calls");
/// 会话压缩摘要表访问句柄
pub(super) const CONVERSATION_SUMMARIES: Table = Table::new("conversation_summaries");

/// 消息表查询列（与 `msg_mapper` 顺序一致）
pub(super) const MSG_COLUMNS: &[&str] = &[
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

/// 当前时间戳（秒）
pub(super) fn now() -> i64 {
    chrono::Local::now().timestamp()
}

/// 聊天库路径（`.Molaunch/chat.db`）
pub(super) fn db_path() -> std::path::PathBuf {
    Storage::instance().base_dir().join("chat.db")
}

/// 旧版数据库路径（v1：experimental/chat.db），首次挂载时迁移
pub(super) fn legacy_db_path() -> std::path::PathBuf {
    Storage::instance()
        .base_dir()
        .join("experimental")
        .join("chat.db")
}
