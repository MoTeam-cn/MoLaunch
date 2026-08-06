//! 聊天库初始化与行映射
//!
//! `ensure_initialized` 幂等挂载（迁移旧库 + 建表/自动迁移列 + 建立全局连接），
//! 行映射供各表数据访问复用。

use rusqlite::types::Value;
use rusqlite::{Connection, Row};

use super::schema::{db_path, legacy_db_path, now, CHAT_TABLES, CONVERSATIONS};
use crate::commands::experimental::types::{ConversationItem, MessageItem, ToolCallRecord};
use crate::utils::sqlite::mount;
use crate::{log_debug, log_warn};

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

/// 行 → 会话映射
pub(crate) fn conv_mapper(row: &Row<'_>) -> Result<ConversationItem, rusqlite::Error> {
    Ok(ConversationItem {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
    })
}

/// 行 → 消息映射
pub(crate) fn msg_mapper(row: &Row<'_>) -> Result<MessageItem, rusqlite::Error> {
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
pub(crate) fn tool_mapper(row: &Row<'_>) -> Result<ToolCallRecord, rusqlite::Error> {
    Ok(ToolCallRecord {
        message_id: row.get(0)?,
        seq: row.get(1)?,
        name: row.get(2)?,
        arguments: row.get(3)?,
        output: row.get(4)?,
        pre_content: row.get(5)?,
    })
}

/// 在已持有的连接上更新会话活动时间（供同一连接内复用，避免嵌套加锁）
pub(crate) fn touch_conversation_with(conn: &Connection, id: i64) -> Result<(), String> {
    CONVERSATIONS
        .update_by_id(conn, id, &[("updated_at", Value::from(now()))])
        .map(|_| ())
}
