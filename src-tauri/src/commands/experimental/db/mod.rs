//! 实验性功能 SQLite 存储（连接由系统维护）
//!
//! - 数据库位置：`Storage::base_dir()/chat.db`（`.Molaunch/chat.db`）。
//! - 子模块：`schema`（表结构声明与表句柄）/ `init`（挂载与行映射）/
//!   `conversations` / `messages` / `tool_calls`（数据访问）。
//! - 建表与自动迁移由 `crate::utils::sqlite` 公共工具负责，本模块无裸 SQL。

mod conversations;
mod init;
mod messages;
mod schema;
mod summaries;
mod tool_calls;

pub use conversations::{
    clear_conversation, conversation_exists, create_conversation, delete_conversation,
    list_conversations, rename_conversation, touch_conversation,
};
pub use init::ensure_initialized;
pub use messages::{
    add_message, delete_message, delete_messages_after, get_message, get_message_pair_id,
    get_message_retry_count, last_user_message_id, list_messages, set_message_pair_id,
    update_message_content,
};
pub use summaries::{delete_summary, get_summary, upsert_summary, ConversationSummary};
pub use tool_calls::{
    add_tool_calls, clear_tool_calls, delete_tool_calls_after, delete_tool_calls_for_message,
    list_tool_calls,
};
