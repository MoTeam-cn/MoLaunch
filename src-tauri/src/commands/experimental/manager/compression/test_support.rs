//! 压缩模块测试公共夹具（仅测试构建）

use crate::commands::experimental::types::{MessageItem, ToolCallRecord};

/// 构造最小消息项
pub(super) fn msg(id: i64, role: &str, content: &str) -> MessageItem {
    MessageItem {
        id,
        role: role.to_string(),
        content: content.to_string(),
        created_at: 0,
        pair_id: None,
        version_id: None,
        reasoning_content: None,
        model: None,
        retry_count: None,
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: None,
        duration_ms: None,
    }
}

/// 构造最小工具调用记录
pub(super) fn tool(message_id: i64, name: &str, output: Option<&str>) -> ToolCallRecord {
    ToolCallRecord {
        message_id,
        seq: 0,
        name: name.to_string(),
        arguments: String::new(),
        output: output.map(|s| s.to_string()),
        pre_content: None,
    }
}
